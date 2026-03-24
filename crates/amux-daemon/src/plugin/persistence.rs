use anyhow::Result;
use rusqlite::params;
use std::sync::Arc;

/// Internal plugin record (not exposed to protocol crate).
#[derive(Debug, Clone)]
pub struct PluginRecord {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub manifest_json: String,
    pub install_source: String,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
}

/// SQLite persistence layer for plugin metadata.
pub struct PluginPersistence {
    history: Arc<crate::history::HistoryStore>,
}

impl PluginPersistence {
    pub fn new(history: Arc<crate::history::HistoryStore>) -> Self {
        Self { history }
    }

    /// List all plugins. Per PLUG-09.
    pub async fn list_plugins(&self) -> Result<Vec<PluginRecord>> {
        self.history
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT name, version, description, author, manifest_json, install_source, enabled, installed_at, updated_at FROM plugins ORDER BY name ASC",
                )?;
                let rows = stmt.query_map([], |row| {
                    Ok(PluginRecord {
                        name: row.get(0)?,
                        version: row.get(1)?,
                        description: row.get(2)?,
                        author: row.get(3)?,
                        manifest_json: row.get(4)?,
                        install_source: row.get(5)?,
                        enabled: row.get::<_, i64>(6)? != 0,
                        installed_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })?;
                let mut records = Vec::new();
                for row in rows {
                    records.push(row?);
                }
                Ok(records)
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    /// Get a single plugin by name. Per PLUG-09.
    pub async fn get_plugin(&self, name: &str) -> Result<Option<PluginRecord>> {
        let name = name.to_string();
        self.history
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT name, version, description, author, manifest_json, install_source, enabled, installed_at, updated_at FROM plugins WHERE name = ?1",
                )?;
                let record = stmt
                    .query_row(params![name], |row| {
                        Ok(PluginRecord {
                            name: row.get(0)?,
                            version: row.get(1)?,
                            description: row.get(2)?,
                            author: row.get(3)?,
                            manifest_json: row.get(4)?,
                            install_source: row.get(5)?,
                            enabled: row.get::<_, i64>(6)? != 0,
                            installed_at: row.get(7)?,
                            updated_at: row.get(8)?,
                        })
                    })
                    .optional()?;
                Ok(record)
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    /// Insert or update a plugin record. Per PLUG-09.
    pub async fn upsert_plugin(&self, record: &PluginRecord) -> Result<()> {
        let record = record.clone();
        self.history
            .conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO plugins (name, version, description, author, manifest_json, install_source, enabled, installed_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        record.name,
                        record.version,
                        record.description,
                        record.author,
                        record.manifest_json,
                        record.install_source,
                        record.enabled as i64,
                        record.installed_at,
                        record.updated_at,
                    ],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    /// Set enabled/disabled status. Per PLUG-09.
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> Result<()> {
        let name = name.to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.history
            .conn
            .call(move |conn| {
                let rows = conn.execute(
                    "UPDATE plugins SET enabled = ?1, updated_at = ?2 WHERE name = ?3",
                    params![enabled as i64, now, name],
                )?;
                if rows == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows.into());
                }
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    /// Remove plugins not in the provided set (stale record reconciliation per Pitfall 6).
    /// Returns the number of rows deleted.
    pub async fn remove_stale_plugins(&self, active_names: &[String]) -> Result<u64> {
        let active_names = active_names.to_vec();
        self.history
            .conn
            .call(move |conn| {
                if active_names.is_empty() {
                    // Delete all plugins if no active names
                    let deleted = conn.execute("DELETE FROM plugins", [])?;
                    return Ok(deleted as u64);
                }
                // Build dynamic SQL: DELETE FROM plugins WHERE name NOT IN (?,?,...)
                let placeholders: Vec<&str> = active_names.iter().map(|_| "?").collect();
                let sql = format!(
                    "DELETE FROM plugins WHERE name NOT IN ({})",
                    placeholders.join(",")
                );
                let params: Vec<&dyn rusqlite::types::ToSql> = active_names
                    .iter()
                    .map(|s| s as &dyn rusqlite::types::ToSql)
                    .collect();
                let deleted = conn.execute(&sql, params.as_slice())?;
                Ok(deleted as u64)
            })
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_test_history() -> Arc<crate::history::HistoryStore> {
        let root = std::env::temp_dir().join(format!(
            "tamux-plugin-persist-test-{}",
            uuid::Uuid::new_v4()
        ));
        let store = crate::history::HistoryStore::new_test_store(&root)
            .await
            .unwrap();
        Arc::new(store)
    }

    fn sample_record(name: &str) -> PluginRecord {
        PluginRecord {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test plugin".to_string()),
            author: Some("Test Author".to_string()),
            manifest_json: r#"{"name":"test","version":"1.0.0","schema_version":1}"#.to_string(),
            install_source: "local".to_string(),
            enabled: true,
            installed_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn list_plugins_returns_empty_on_fresh_db() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);
        let plugins = persistence.list_plugins().await.unwrap();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn upsert_then_list_returns_record() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);

        let record = sample_record("test-plugin");
        persistence.upsert_plugin(&record).await.unwrap();

        let plugins = persistence.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "test-plugin");
        assert_eq!(plugins[0].version, "1.0.0");
        assert!(plugins[0].enabled);
    }

    #[tokio::test]
    async fn set_enabled_toggles_flag() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);

        let record = sample_record("test-plugin");
        persistence.upsert_plugin(&record).await.unwrap();

        // Disable
        persistence
            .set_enabled("test-plugin", false)
            .await
            .unwrap();
        let plugin = persistence.get_plugin("test-plugin").await.unwrap().unwrap();
        assert!(!plugin.enabled);

        // Re-enable
        persistence
            .set_enabled("test-plugin", true)
            .await
            .unwrap();
        let plugin = persistence.get_plugin("test-plugin").await.unwrap().unwrap();
        assert!(plugin.enabled);
    }

    #[tokio::test]
    async fn remove_stale_plugins_removes_absent_names() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);

        persistence
            .upsert_plugin(&sample_record("keep-me"))
            .await
            .unwrap();
        persistence
            .upsert_plugin(&sample_record("remove-me"))
            .await
            .unwrap();
        persistence
            .upsert_plugin(&sample_record("also-remove"))
            .await
            .unwrap();

        let deleted = persistence
            .remove_stale_plugins(&["keep-me".to_string()])
            .await
            .unwrap();
        assert_eq!(deleted, 2);

        let plugins = persistence.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "keep-me");
    }

    #[tokio::test]
    async fn get_plugin_returns_none_for_missing() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);
        let result = persistence.get_plugin("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn upsert_updates_existing_record() {
        let history = make_test_history().await;
        let persistence = PluginPersistence::new(history);

        let mut record = sample_record("test-plugin");
        persistence.upsert_plugin(&record).await.unwrap();

        record.version = "2.0.0".to_string();
        record.updated_at = "2026-06-01T00:00:00Z".to_string();
        persistence.upsert_plugin(&record).await.unwrap();

        let plugins = persistence.list_plugins().await.unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].version, "2.0.0");
        assert_eq!(plugins[0].updated_at, "2026-06-01T00:00:00Z");
    }
}
