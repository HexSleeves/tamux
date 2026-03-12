use anyhow::{Context, Result};
use amux_protocol::HistorySearchHit;
use rusqlite::{params, Connection};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use sysinfo::System;

/// Result of verifying a single WORM ledger file's hash-chain integrity.
pub struct WormIntegrityResult {
    pub kind: String,
    pub total_entries: usize,
    pub valid: bool,
    pub first_invalid_seq: Option<usize>,
    pub message: String,
}

#[derive(Clone)]
pub struct HistoryStore {
    db_path: PathBuf,
    skill_dir: PathBuf,
    telemetry_dir: PathBuf,
    worm_dir: PathBuf,
}

pub struct ManagedHistoryRecord {
    pub execution_id: String,
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub command: String,
    pub rationale: String,
    pub source: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub snapshot_path: Option<String>,
}

impl HistoryStore {
    pub fn new() -> Result<Self> {
        let base = amux_protocol::ensure_amux_data_dir()?;
        let history_dir = base.join("history");
        let skill_dir = base.join("skills").join("generated");
        let telemetry_dir = base.join("semantic-logs");
        let worm_dir = telemetry_dir.join("worm");

        std::fs::create_dir_all(&history_dir)?;
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::create_dir_all(&telemetry_dir)?;
        std::fs::create_dir_all(&worm_dir)?;

        let store = Self {
            db_path: history_dir.join("command-history.db"),
            skill_dir,
            telemetry_dir,
            worm_dir,
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn record_managed_finish(&self, record: &ManagedHistoryRecord) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        let timestamp = now_ts() as i64;
        let excerpt = format!(
            "exit={:?} duration_ms={:?} snapshot={} rationale={}",
            record.exit_code,
            record.duration_ms,
            record.snapshot_path.as_deref().unwrap_or("none"),
            record.rationale
        );

        connection.execute(
            "INSERT OR REPLACE INTO history_entries (id, kind, title, excerpt, content, path, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.execution_id,
                "managed-command",
                record.command,
                excerpt,
                format!("{}\n{}", record.command, record.rationale),
                record.snapshot_path,
                timestamp,
            ],
        )?;
        connection.execute(
            "INSERT OR REPLACE INTO history_fts (id, title, excerpt, content) VALUES (?1, ?2, ?3, ?4)",
            params![record.execution_id, record.command, excerpt, record.rationale],
        )?;

        self.append_telemetry("operational", json!({
            "timestamp": timestamp,
            "execution_id": record.execution_id,
            "session_id": record.session_id,
            "workspace_id": record.workspace_id,
            "command": record.command,
            "exit_code": record.exit_code,
            "duration_ms": record.duration_ms,
            "snapshot": record.snapshot_path,
        }))?;
        self.append_telemetry("cognitive", json!({
            "timestamp": timestamp,
            "execution_id": record.execution_id,
            "source": record.source,
            "rationale": record.rationale,
        }))?;

        let mut system = System::new_all();
        system.refresh_memory();
        system.refresh_cpu();
        self.append_telemetry("contextual", json!({
            "timestamp": timestamp,
            "execution_id": record.execution_id,
            "total_memory": system.total_memory(),
            "used_memory": system.used_memory(),
            "cpu_usage": system.global_cpu_info().cpu_usage(),
        }))?;

        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<(String, Vec<HistorySearchHit>)> {
        let connection = Connection::open(&self.db_path)?;
        let mut stmt = connection.prepare(
            "SELECT history_entries.id, kind, title, excerpt, path, timestamp, bm25(history_fts) \
             FROM history_fts JOIN history_entries ON history_entries.id = history_fts.id \
             WHERE history_fts MATCH ?1 ORDER BY bm25(history_fts) LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(HistorySearchHit {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                excerpt: row.get(3)?,
                path: row.get(4)?,
                timestamp: row.get::<_, i64>(5)? as u64,
                score: row.get(6)?,
            })
        })?;

        let hits = rows.filter_map(|row| row.ok()).collect::<Vec<_>>();
        let summary = if hits.is_empty() {
            format!("No prior runs matched '{query}'.")
        } else {
            format!("Found {} historical matches for '{query}'.", hits.len())
        };
        Ok((summary, hits))
    }

    pub fn generate_skill(&self, query: Option<&str>, title: Option<&str>) -> Result<(String, String)> {
        let title = title.unwrap_or("Recovered Workflow").trim();
        let (summary, hits) = self.search(query.unwrap_or("*"), 8).unwrap_or_else(|_| ("No history available.".to_string(), Vec::new()));
        let safe_name = title
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_ascii_lowercase();
        let path = self.skill_dir.join(format!("{}.md", if safe_name.is_empty() { "recovered-workflow" } else { &safe_name }));
        let mut body = format!("# {}\n\n## Summary\n{}\n\n## Retrieved Steps\n", title, summary);
        for hit in &hits {
            body.push_str(&format!("- {}\n", hit.title));
            body.push_str(&format!("  {}\n", hit.excerpt));
        }
        if hits.is_empty() {
            body.push_str("- No matching executions were available.\n");
        }
        std::fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
        Ok((title.to_string(), path.to_string_lossy().into_owned()))
    }

    fn init_schema(&self) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history_entries (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                excerpt TEXT NOT NULL,
                content TEXT NOT NULL,
                path TEXT,
                timestamp INTEGER NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
                id UNINDEXED,
                title,
                excerpt,
                content
            );
            ",
        )?;
        Ok(())
    }

    fn append_telemetry(&self, kind: &str, payload: serde_json::Value) -> Result<()> {
        let line = serde_json::to_string(&payload)?;
        let log_path = self.telemetry_dir.join(format!("{}.jsonl", kind));
        let worm_path = self.worm_dir.join(format!("{}-ledger.jsonl", kind));

        append_line(&log_path, &line)?;

        // Read the last entry to obtain prev_hash and seq for hash-chain.
        let (prev_hash, seq) = read_last_worm_entry(&worm_path);

        let payload_json = serde_json::to_string(&payload)?;
        let hash = hex_hash(&format!("{}{}", prev_hash, payload_json));
        let worm_line = serde_json::to_string(&json!({
            "seq": seq,
            "prev_hash": prev_hash,
            "hash": hash,
            "payload": payload,
        }))?;
        append_line(&worm_path, &worm_line)?;
        Ok(())
    }

    /// Detect sequences of 3+ consecutive successful managed commands
    /// that completed within a 5-minute window.
    pub fn detect_skill_candidates(&self) -> Result<Vec<(String, Vec<HistorySearchHit>)>> {
        let connection = Connection::open(&self.db_path)?;
        let mut stmt = connection.prepare(
            "SELECT id, kind, title, excerpt, path, timestamp FROM history_entries \
             WHERE kind = 'managed-command' \
             ORDER BY timestamp DESC LIMIT 20"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HistorySearchHit {
                id: row.get(0)?,
                kind: row.get(1)?,
                title: row.get(2)?,
                excerpt: row.get(3)?,
                path: row.get(4)?,
                timestamp: row.get::<_, i64>(5)? as u64,
                score: 0.0,
            })
        })?;

        let hits: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        let mut candidates = Vec::new();

        // Find runs of 3+ successful commands within 5-minute windows
        let mut run: Vec<HistorySearchHit> = Vec::new();
        for hit in &hits {
            // Check if excerpt indicates success (exit=Some(0))
            if hit.excerpt.contains("exit=Some(0)") {
                if run.is_empty() || (run.last().unwrap().timestamp.abs_diff(hit.timestamp) < 300) {
                    run.push(hit.clone());
                } else {
                    if run.len() >= 3 {
                        let title = format!("Workflow: {}", run.first().unwrap().title);
                        candidates.push((title, run.clone()));
                    }
                    run = vec![hit.clone()];
                }
            } else {
                if run.len() >= 3 {
                    let title = format!("Workflow: {}", run.first().unwrap().title);
                    candidates.push((title, run.clone()));
                }
                run.clear();
            }
        }
        if run.len() >= 3 {
            let title = format!("Workflow: {}", run.first().unwrap().title);
            candidates.push((title, run));
        }

        Ok(candidates)
    }

    /// Verify the hash-chain integrity of all WORM telemetry ledger files.
    pub fn verify_worm_integrity(&self) -> Result<Vec<WormIntegrityResult>> {
        let ledger_kinds = ["operational", "cognitive", "contextual"];
        let mut results = Vec::with_capacity(ledger_kinds.len());

        for kind in &ledger_kinds {
            let worm_path = self.worm_dir.join(format!("{}-ledger.jsonl", kind));
            results.push(verify_ledger_file(kind, &worm_path));
        }

        Ok(results)
    }
}

fn append_line(path: &PathBuf, line: &str) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn hex_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Read the last line of a WORM ledger file and extract (prev_hash, next_seq).
/// Returns ("genesis", 0) if the file does not exist or is empty.
fn read_last_worm_entry(path: &PathBuf) -> (String, usize) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ("genesis".to_string(), 0),
    };

    let reader = std::io::BufReader::new(file);
    let mut last_line: Option<String> = None;
    for line in reader.lines() {
        if let Ok(l) = line {
            if !l.trim().is_empty() {
                last_line = Some(l);
            }
        }
    }

    match last_line {
        None => ("genesis".to_string(), 0),
        Some(line) => {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                let hash = entry
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("genesis")
                    .to_string();
                let seq = entry
                    .get("seq")
                    .and_then(|v| v.as_u64())
                    .map(|s| s as usize + 1)
                    .unwrap_or(0);
                (hash, seq)
            } else {
                // Could not parse last line (possibly old format); start fresh chain.
                ("genesis".to_string(), 0)
            }
        }
    }
}

/// Verify an individual WORM ledger file's hash-chain integrity.
fn verify_ledger_file(kind: &str, path: &PathBuf) -> WormIntegrityResult {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            return WormIntegrityResult {
                kind: kind.to_string(),
                total_entries: 0,
                valid: true,
                first_invalid_seq: None,
                message: "Ledger file not found; no entries to verify.".to_string(),
            };
        }
    };

    let reader = std::io::BufReader::new(file);
    let mut prev_hash = "genesis".to_string();
    let mut total: usize = 0;
    let mut expected_seq: usize = 0;
    let mut first_invalid_seq: Option<usize> = None;
    let mut failure_message: Option<String> = None;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                if first_invalid_seq.is_none() {
                    first_invalid_seq = Some(expected_seq);
                    failure_message = Some(format!("IO error reading line at seq {}: {}", expected_seq, e));
                }
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        total += 1;

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                if first_invalid_seq.is_none() {
                    first_invalid_seq = Some(expected_seq);
                    failure_message = Some(format!("JSON parse error at seq {}: {}", expected_seq, e));
                }
                break;
            }
        };

        // Detect old-format entries (no seq/prev_hash fields) and handle gracefully.
        let has_seq = entry.get("seq").is_some();
        let has_prev_hash = entry.get("prev_hash").is_some();

        if !has_seq || !has_prev_hash {
            // Old-format entry: verify standalone hash only.
            let payload = &entry["payload"];
            let recorded_hash = entry
                .get("hash")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let payload_json = serde_json::to_string(payload).unwrap_or_default();
            let computed = hex_hash(&payload_json);

            if recorded_hash != computed {
                if first_invalid_seq.is_none() {
                    first_invalid_seq = Some(expected_seq);
                    failure_message = Some(format!(
                        "Old-format entry at position {} has invalid standalone hash.",
                        expected_seq
                    ));
                }
                break;
            }

            // For chain continuity, treat old entries' hash as the prev_hash for the next entry.
            prev_hash = recorded_hash.to_string();
            expected_seq += 1;
            continue;
        }

        // New-format entry: full hash-chain verification.
        let entry_seq = entry["seq"].as_u64().unwrap_or(0) as usize;
        let entry_prev_hash = entry
            .get("prev_hash")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let recorded_hash = entry
            .get("hash")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let payload = &entry["payload"];
        let payload_json = serde_json::to_string(payload).unwrap_or_default();

        // Verify sequence number.
        if entry_seq != expected_seq {
            if first_invalid_seq.is_none() {
                first_invalid_seq = Some(entry_seq);
                failure_message = Some(format!(
                    "Sequence number mismatch: expected {}, found {} at entry {}.",
                    expected_seq, entry_seq, total
                ));
            }
            break;
        }

        // Verify prev_hash matches previous entry's hash.
        if entry_prev_hash != prev_hash {
            if first_invalid_seq.is_none() {
                first_invalid_seq = Some(entry_seq);
                failure_message = Some(format!(
                    "prev_hash mismatch at seq {}: expected '{}', found '{}'.",
                    entry_seq,
                    &prev_hash[..prev_hash.len().min(16)],
                    &entry_prev_hash[..entry_prev_hash.len().min(16)]
                ));
            }
            break;
        }

        // Verify hash = sha256(prev_hash + payload_json).
        let computed_hash = hex_hash(&format!("{}{}", entry_prev_hash, payload_json));
        if recorded_hash != computed_hash {
            if first_invalid_seq.is_none() {
                first_invalid_seq = Some(entry_seq);
                failure_message = Some(format!(
                    "Hash mismatch at seq {}: recorded '{}...', computed '{}...'.",
                    entry_seq,
                    &recorded_hash[..recorded_hash.len().min(16)],
                    &computed_hash[..computed_hash.len().min(16)]
                ));
            }
            break;
        }

        prev_hash = recorded_hash.to_string();
        expected_seq += 1;
    }

    let valid = first_invalid_seq.is_none();
    let message = if valid {
        format!("{} ledger: all {} entries verified successfully.", kind, total)
    } else {
        failure_message.unwrap_or_else(|| format!("{} ledger: integrity check failed.", kind))
    };

    WormIntegrityResult {
        kind: kind.to_string(),
        total_entries: total,
        valid,
        first_invalid_seq,
        message,
    }
}

fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}