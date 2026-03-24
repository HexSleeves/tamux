pub mod loader;
pub mod manifest;
pub mod persistence;
pub mod schema;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

pub use loader::LoadedPlugin;
pub use persistence::{PluginPersistence, PluginRecord};

/// Manages plugin lifecycle: loading, validation, persistence, and queries.
/// Initialized once in server.rs, shared via Arc.
pub struct PluginManager {
    plugins: RwLock<HashMap<String, LoadedPlugin>>,
    persistence: PluginPersistence,
    plugins_dir: PathBuf,
    schema_validator: jsonschema::Validator,
}

impl PluginManager {
    /// Create a new PluginManager. Does NOT load plugins yet -- call load_all_from_disk().
    pub fn new(history: Arc<crate::history::HistoryStore>, plugins_dir: PathBuf) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            persistence: PluginPersistence::new(history),
            plugins_dir,
            schema_validator: schema::compile_schema_v1(),
        }
    }

    /// Load all plugins from disk, validate, persist to SQLite, reconcile stale records.
    /// Per D-09: skip and warn on failures.
    /// Returns (loaded_count, skipped_count).
    pub async fn load_all_from_disk(&self) -> (usize, usize) {
        // Create plugins dir if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&self.plugins_dir) {
            tracing::warn!(
                path = %self.plugins_dir.display(),
                error = %e,
                "failed to create plugins directory"
            );
            return (0, 0);
        }

        let scan = loader::scan_plugins_dir(&self.plugins_dir, &self.schema_validator);
        let loaded_count = scan.loaded.len();
        let skipped_count = scan.skipped.len();

        let now = chrono::Utc::now().to_rfc3339();
        let mut active_names = Vec::with_capacity(loaded_count);
        let mut plugins_map = HashMap::with_capacity(loaded_count);

        for plugin in scan.loaded {
            let record = PluginRecord {
                name: plugin.manifest.name.clone(),
                version: plugin.manifest.version.clone(),
                description: plugin.manifest.description.clone(),
                author: plugin.manifest.author.clone(),
                manifest_json: plugin.manifest_json.clone(),
                install_source: "local".to_string(),
                enabled: true,
                installed_at: now.clone(),
                updated_at: now.clone(),
            };

            if let Err(e) = self.persistence.upsert_plugin(&record).await {
                tracing::warn!(
                    plugin = %record.name,
                    error = %e,
                    "failed to persist plugin record"
                );
                continue;
            }

            active_names.push(plugin.manifest.name.clone());
            plugins_map.insert(plugin.manifest.name.clone(), plugin);
        }

        // Reconcile stale records (Pitfall 6)
        if let Err(e) = self.persistence.remove_stale_plugins(&active_names).await {
            tracing::warn!(error = %e, "failed to reconcile stale plugin records");
        }

        *self.plugins.write().await = plugins_map;

        tracing::info!(
            loaded = loaded_count,
            skipped = skipped_count,
            "plugin loader: loaded {} plugins ({} skipped)",
            loaded_count,
            skipped_count
        );

        (loaded_count, skipped_count)
    }

    /// List all plugins (from SQLite for accurate enabled state).
    pub async fn list_plugins(&self) -> Vec<amux_protocol::PluginInfo> {
        let plugins = self.plugins.read().await;
        let records = match self.persistence.list_plugins().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(error = %e, "failed to list plugins from database");
                return Vec::new();
            }
        };

        records
            .iter()
            .map(|rec| {
                let loaded = plugins.get(&rec.name);
                to_plugin_info_from_record(rec, loaded)
            })
            .collect()
    }

    /// Get a single plugin by name.
    /// Returns (PluginInfo, settings_schema_json).
    pub async fn get_plugin(
        &self,
        name: &str,
    ) -> Option<(amux_protocol::PluginInfo, Option<String>)> {
        let record = match self.persistence.get_plugin(name).await {
            Ok(Some(r)) => r,
            Ok(None) => return None,
            Err(e) => {
                tracing::warn!(plugin = %name, error = %e, "failed to get plugin from database");
                return None;
            }
        };

        let plugins = self.plugins.read().await;
        let loaded = plugins.get(name);
        let info = to_plugin_info_from_record(&record, loaded);

        // Extract settings schema from manifest JSON for dynamic form rendering
        let settings_schema = extract_settings_schema(&record.manifest_json);

        Some((info, settings_schema))
    }

    /// Enable or disable a plugin.
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> Result<()> {
        self.persistence.set_enabled(name, enabled).await
    }
}

/// Convert a PluginRecord + optional LoadedPlugin to PluginInfo.
fn to_plugin_info_from_record(
    record: &PluginRecord,
    loaded: Option<&LoadedPlugin>,
) -> amux_protocol::PluginInfo {
    if let Some(plugin) = loaded {
        to_plugin_info(
            plugin,
            record.enabled,
            &record.install_source,
            &record.installed_at,
            &record.updated_at,
        )
    } else {
        // Fallback: reconstruct from manifest_json in record
        amux_protocol::PluginInfo {
            name: record.name.clone(),
            version: record.version.clone(),
            description: record.description.clone(),
            author: record.author.clone(),
            enabled: record.enabled,
            install_source: record.install_source.clone(),
            has_api: false,
            has_auth: false,
            has_commands: false,
            has_skills: false,
            endpoint_count: 0,
            settings_count: 0,
            installed_at: record.installed_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

fn to_plugin_info(
    plugin: &LoadedPlugin,
    enabled: bool,
    install_source: &str,
    installed_at: &str,
    updated_at: &str,
) -> amux_protocol::PluginInfo {
    amux_protocol::PluginInfo {
        name: plugin.manifest.name.clone(),
        version: plugin.manifest.version.clone(),
        description: plugin.manifest.description.clone(),
        author: plugin.manifest.author.clone(),
        enabled,
        install_source: install_source.to_string(),
        has_api: plugin.manifest.api.is_some(),
        has_auth: plugin.manifest.auth.is_some(),
        has_commands: plugin.manifest.commands.is_some(),
        has_skills: plugin.manifest.skills.is_some(),
        endpoint_count: plugin
            .manifest
            .api
            .as_ref()
            .map(|a| a.endpoints.len() as u32)
            .unwrap_or(0),
        settings_count: plugin
            .manifest
            .settings
            .as_ref()
            .map(|s| s.len() as u32)
            .unwrap_or(0),
        installed_at: installed_at.to_string(),
        updated_at: updated_at.to_string(),
    }
}

/// Extract the "settings" section from manifest JSON as a standalone JSON string.
fn extract_settings_schema(manifest_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(manifest_json).ok()?;
    let settings = value.get("settings")?;
    Some(serde_json::to_string(settings).ok()?)
}
