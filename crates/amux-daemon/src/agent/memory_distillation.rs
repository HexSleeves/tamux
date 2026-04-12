//! Episodic → Semantic Memory Distillation (Spec 01)
//!
//! Analyzes old, undistilled thread transcripts and extracts actionable
//! memory entries (preferences, conventions, patterns, corrections, lessons).
//! Candidates above the auto-apply confidence threshold are written to
//! MEMORY.md/USER.md with a `[distilled]` provenance prefix.

use super::*;
use crate::history::HistoryStore;
use amux_protocol::{AgentDbMessage, InboxNotification};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap, HashSet};

const MIN_FACT_CHARS: usize = 24;
const MAX_FACT_CHARS: usize = 220;
const THREAD_BATCH_LIMIT: usize = 24;
const OLD_THREAD_AGE_MS: u64 = 60 * 60 * 1000;

/// Categories for distilled memory entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCategory {
    /// Operator habits, risk tolerance, session patterns.
    Preference,
    /// Workspace facts, coding conventions, tool patterns.
    Convention,
    /// Repeated behaviors, workflow habits.
    Pattern,
    /// Things the operator corrected (high value).
    Correction,
    /// Generalizable insights from task outcomes.
    Lesson,
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preference => write!(f, "preference"),
            Self::Convention => write!(f, "convention"),
            Self::Pattern => write!(f, "pattern"),
            Self::Correction => write!(f, "correction"),
            Self::Lesson => write!(f, "lesson"),
        }
    }
}

/// A candidate memory entry distilled from thread content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationCandidate {
    pub source_thread_id: String,
    pub source_message_range: Option<String>,
    pub distilled_fact: String,
    pub target_file: String, // "MEMORY.md" or "USER.md"
    pub category: MemoryCategory,
    pub confidence: f64,   // 0.0–1.0
    pub reasoning: String, // why this should be saved
}

/// Configuration for the distillation pass.
#[derive(Debug, Clone)]
pub struct DistillationConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub confidence_auto_apply: f64,   // default: 0.7
    pub confidence_review_queue: f64, // default: 0.5
    pub max_entries_per_file: usize,  // default: 50
    pub agent_id: String,
    pub review_notification: bool,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 4,
            confidence_auto_apply: 0.7,
            confidence_review_queue: 0.5,
            max_entries_per_file: 50,
            agent_id: "rarog".into(),
            review_notification: true,
        }
    }
}

/// Result of a distillation pass.
#[derive(Debug, Default)]
pub struct DistillationResult {
    pub threads_analyzed: usize,
    pub candidates_generated: usize,
    pub auto_applied: usize,
    pub queued_for_review: usize,
    pub discarded: usize,
    pub review_notifications_emitted: usize,
}

/// Run a distillation pass over old, undistilled threads.
pub async fn run_distillation_pass(
    db: &HistoryStore,
    config: &DistillationConfig,
    agent_data_dir: &std::path::Path,
) -> anyhow::Result<DistillationResult> {
    if !config.enabled {
        return Ok(DistillationResult::default());
    }

    let now = now_millis();
    let cutoff = now.saturating_sub(OLD_THREAD_AGE_MS);
    let thread_ids = list_undistilled_threads(db, cutoff, THREAD_BATCH_LIMIT).await?;
    let mut result = DistillationResult::default();
    let mut applied_per_target: HashMap<String, usize> = HashMap::new();
    let mut queued_per_target: HashMap<String, usize> = HashMap::new();
    let mut review_candidates = Vec::new();

    for thread_id in thread_ids {
        result.threads_analyzed += 1;
        let messages = db.list_recent_messages(&thread_id, 40).await?;
        let candidates = extract_candidates_from_messages(&thread_id, &messages);

        for candidate in candidates {
            result.candidates_generated += 1;
            let target_key = candidate.target_file.clone();
            let applied_count = *applied_per_target.get(&target_key).unwrap_or(&0);
            let queued_count = *queued_per_target.get(&target_key).unwrap_or(&0);
            let remaining_budget = config
                .max_entries_per_file
                .saturating_sub(applied_count + queued_count);

            if remaining_budget == 0 {
                result.discarded += 1;
                log_distillation_candidate(db, &candidate, false, &config.agent_id).await?;
                continue;
            }

            if candidate.confidence >= config.confidence_auto_apply {
                if apply_distilled_candidate(db, agent_data_dir, config, &candidate).await? {
                    *applied_per_target.entry(target_key).or_insert(0) += 1;
                    result.auto_applied += 1;
                    log_distillation_candidate(db, &candidate, true, &config.agent_id).await?;
                } else {
                    result.discarded += 1;
                    log_distillation_candidate(db, &candidate, false, &config.agent_id).await?;
                }
            } else if candidate.confidence >= config.confidence_review_queue {
                *queued_per_target.entry(target_key).or_insert(0) += 1;
                result.queued_for_review += 1;
                review_candidates.push(candidate.clone());
                log_distillation_candidate(db, &candidate, false, &config.agent_id).await?;
            } else {
                result.discarded += 1;
                log_distillation_candidate(db, &candidate, false, &config.agent_id).await?;
            }
        }
    }

    if config.review_notification && !review_candidates.is_empty() {
        emit_review_notification(db, &review_candidates, &config.agent_id).await?;
        result.review_notifications_emitted = 1;
    }

    Ok(result)
}

async fn list_undistilled_threads(
    db: &HistoryStore,
    cutoff_ms: u64,
    limit: usize,
) -> anyhow::Result<Vec<String>> {
    db.conn
        .call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id
                 FROM agent_threads
                 WHERE updated_at < ?1
                   AND id NOT IN (SELECT source_thread_id FROM memory_distillation_log)
                 ORDER BY updated_at ASC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![cutoff_ms as i64, limit as i64], |row| row.get(0))?;
            Ok(rows.collect::<std::result::Result<Vec<String>, _>>()?)
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

fn extract_candidates_from_messages(
    thread_id: &str,
    messages: &[AgentDbMessage],
) -> Vec<DistillationCandidate> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for (index, message) in messages.iter().enumerate() {
        if !message.role.eq_ignore_ascii_case("user") {
            continue;
        }

        for line in message.content.lines() {
            let Some(candidate) = candidate_from_line(thread_id, index, line) else {
                continue;
            };
            let dedupe_key = format!(
                "{}::{}",
                candidate.target_file,
                candidate.distilled_fact.to_ascii_lowercase()
            );
            if seen.insert(dedupe_key) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

fn candidate_from_line(
    thread_id: &str,
    message_index: usize,
    raw_line: &str,
) -> Option<DistillationCandidate> {
    let cleaned = sanitize_line(raw_line)?;
    if looks_ephemeral(&cleaned) {
        return None;
    }

    let lower = cleaned.to_ascii_lowercase();
    let workspace_scoped = has_workspace_markers(&lower);

    let (category, target_file, confidence, reasoning) = if lower.contains("package name")
        || lower.contains("cargo -p")
        || lower.contains("crate path")
        || lower.contains("workspace")
        || lower.contains("daemon package")
        || lower.contains("use `")
        || lower.contains("path is `")
    {
        (
            MemoryCategory::Convention,
            "MEMORY.md",
            0.86,
            "explicit workspace convention or implementation correction",
        )
    } else if lower.contains("i prefer")
        || lower.contains("prefer ")
        || lower.contains("summary-first")
        || lower.contains("be concise")
        || lower.contains("be direct")
        || lower.contains("do not ask")
        || lower.contains("don't ask")
        || lower.contains("verbose")
    {
        (
            MemoryCategory::Preference,
            "USER.md",
            0.78,
            "explicit operator preference phrasing",
        )
    } else if lower.contains("actually")
        || lower.contains("instead")
        || lower.contains("correction")
        || lower.contains("wrong")
        || lower.contains("use the cargo package name")
    {
        (
            MemoryCategory::Correction,
            if workspace_scoped {
                "MEMORY.md"
            } else {
                "USER.md"
            },
            if workspace_scoped { 0.84 } else { 0.72 },
            "high-signal correction language",
        )
    } else if lower.contains("usually")
        || lower.contains("often")
        || lower.contains("tend to")
        || lower.contains("responds well")
    {
        (
            MemoryCategory::Pattern,
            if workspace_scoped {
                "MEMORY.md"
            } else {
                "USER.md"
            },
            0.58,
            "stable-looking behavioral pattern",
        )
    } else {
        return None;
    };

    Some(DistillationCandidate {
        source_thread_id: thread_id.to_string(),
        source_message_range: Some(format!("msg#{message_index}")),
        distilled_fact: cleaned,
        target_file: target_file.to_string(),
        category,
        confidence,
        reasoning: reasoning.to_string(),
    })
}

fn sanitize_line(raw_line: &str) -> Option<String> {
    let cleaned = raw_line
        .trim()
        .trim_start_matches(['-', '*', '>', '•', ' '])
        .trim();
    if cleaned.is_empty() || cleaned.ends_with('?') {
        return None;
    }

    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() < MIN_FACT_CHARS || collapsed.len() > MAX_FACT_CHARS {
        return None;
    }
    Some(collapsed)
}

fn looks_ephemeral(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("begin implementation")
        || lower.starts_with("please proceed")
        || lower.starts_with("thanks")
        || lower.starts_with("ok")
        || lower.starts_with("okay")
        || lower.contains("current task")
        || lower.contains("right now")
        || lower.contains("today")
        || lower.contains("immediately")
}

fn has_workspace_markers(lower: &str) -> bool {
    [
        "cargo",
        "crate",
        "workspace",
        "repo",
        "repository",
        "daemon",
        "sqlite",
        "rust",
        "package",
        "memory.md",
        "user.md",
        "soul.md",
        "thread",
        "agent",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

async fn apply_distilled_candidate(
    db: &HistoryStore,
    agent_data_dir: &std::path::Path,
    config: &DistillationConfig,
    candidate: &DistillationCandidate,
) -> anyhow::Result<bool> {
    let target = match candidate.target_file.as_str() {
        "USER.md" => MemoryTarget::User,
        _ => MemoryTarget::Memory,
    };
    let scope_id = current_agent_scope_id();
    let paths = memory_paths_for_scope(agent_data_dir, &scope_id);
    let path = match target {
        MemoryTarget::Soul => paths.soul_path,
        MemoryTarget::Memory => paths.memory_path,
        MemoryTarget::User => paths.user_path,
    };
    let note = format!("- [distilled] {}", candidate.distilled_fact);
    let existing = tokio::fs::read_to_string(&path).await.unwrap_or_default();
    if existing.contains(&note) {
        return Ok(false);
    }

    let applied = apply_memory_update(
        agent_data_dir,
        db,
        target,
        MemoryUpdateMode::Append,
        &note,
        MemoryWriteContext {
            source_kind: "memory_distillation",
            thread_id: Some(candidate.source_thread_id.as_str()),
            task_id: None,
            goal_run_id: None,
        },
    )
    .await;

    match applied {
        Ok(_) => {
            if target != MemoryTarget::User {
                trim_distilled_entries_to_limit(&path, config.max_entries_per_file).await?;
            }
            Ok(true)
        }
        Err(error) => {
            tracing::warn!(
                thread_id = %candidate.source_thread_id,
                target = %candidate.target_file,
                "failed to apply distilled candidate: {error}"
            );
            Ok(false)
        }
    }
}

async fn log_distillation_candidate(
    db: &HistoryStore,
    candidate: &DistillationCandidate,
    applied_to_memory: bool,
    agent_id: &str,
) -> anyhow::Result<()> {
    let source_thread_id = candidate.source_thread_id.clone();
    let source_message_range = candidate.source_message_range.clone();
    let distilled_fact = candidate.distilled_fact.clone();
    let target_file = candidate.target_file.clone();
    let category = candidate.category.to_string();
    let confidence = candidate.confidence;
    let created_at_ms = now_millis() as i64;
    let applied_flag = if applied_to_memory { 1_i64 } else { 0_i64 };
    let agent_id = agent_id.to_string();

    db.conn
        .call(move |conn| {
            conn.execute(
                "INSERT INTO memory_distillation_log \
                 (source_thread_id, source_message_range, distilled_fact, target_file, category, confidence, created_at_ms, applied_to_memory, agent_id) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    source_thread_id,
                    source_message_range,
                    distilled_fact,
                    target_file,
                    category,
                    confidence,
                    created_at_ms,
                    applied_flag,
                    agent_id,
                ],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

async fn emit_review_notification(
    db: &HistoryStore,
    candidates: &[DistillationCandidate],
    agent_id: &str,
) -> anyhow::Result<()> {
    let count = candidates.len();
    let preview = candidates
        .iter()
        .take(3)
        .map(|candidate| {
            format!(
                "- {} [{} → {}]",
                candidate.distilled_fact, candidate.category, candidate.target_file
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut digest = Sha256::new();
    for candidate in candidates {
        digest.update(candidate.source_thread_id.as_bytes());
        digest.update([0u8]);
        digest.update(candidate.distilled_fact.as_bytes());
        digest.update([0xffu8]);
    }
    let digest_hex = format!("{:x}", digest.finalize());
    let now = now_millis() as i64;

    db.upsert_notification(&InboxNotification {
        id: format!("distillation-review:{}:{}", agent_id, &digest_hex[..16]),
        source: "memory_distillation".to_string(),
        kind: "memory_distillation_review".to_string(),
        title: format!("Memory distillation queued {} review item(s)", count),
        body: if count > 3 {
            format!("{}\n- … and {} more", preview, count - 3)
        } else {
            preview
        },
        subtitle: Some(agent_id.to_string()),
        severity: "info".to_string(),
        created_at: now,
        updated_at: now,
        read_at: None,
        archived_at: None,
        deleted_at: None,
        actions: Vec::new(),
        metadata_json: Some(
            serde_json::json!({
                "candidate_count": count,
                "agent_id": agent_id,
                "target_files": candidates.iter().map(|c| c.target_file.as_str()).collect::<Vec<_>>(),
                "categories": candidates.iter().map(|c| c.category.to_string()).collect::<Vec<_>>()
            })
            .to_string(),
        ),
    })
    .await
}

async fn trim_distilled_entries_to_limit(
    path: &std::path::Path,
    max_entries: usize,
) -> anyhow::Result<()> {
    if max_entries == 0 {
        return Ok(());
    }

    let existing = tokio::fs::read_to_string(path).await.unwrap_or_default();
    let trimmed = trim_distilled_entries_in_content(&existing, max_entries);
    if trimmed != existing {
        tokio::fs::write(path, trimmed).await?;
    }
    Ok(())
}

fn trim_distilled_entries_in_content(content: &str, max_entries: usize) -> String {
    if max_entries == 0 {
        return content.to_string();
    }

    let lines = content.lines().collect::<Vec<_>>();
    let distilled_indices = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| is_distilled_entry(line).then_some(idx))
        .collect::<Vec<_>>();
    if distilled_indices.len() <= max_entries {
        return content.to_string();
    }

    let remove_count = distilled_indices.len() - max_entries;
    let to_remove = distilled_indices
        .into_iter()
        .take(remove_count)
        .collect::<HashSet<_>>();
    let kept = lines
        .into_iter()
        .enumerate()
        .filter_map(|(idx, line)| (!to_remove.contains(&idx)).then_some(line))
        .collect::<Vec<_>>()
        .join("\n");

    if content.ends_with('\n') && !kept.is_empty() {
        format!("{kept}\n")
    } else {
        kept
    }
}

fn is_distilled_entry(line: &str) -> bool {
    line.trim_start().starts_with("- [distilled]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn memory_category_display() {
        assert_eq!(MemoryCategory::Preference.to_string(), "preference");
        assert_eq!(MemoryCategory::Correction.to_string(), "correction");
        assert_eq!(MemoryCategory::Lesson.to_string(), "lesson");
    }

    #[test]
    fn default_config_sane() {
        let cfg = DistillationConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.confidence_auto_apply, 0.7);
        assert_eq!(cfg.agent_id, "rarog");
    }

    #[test]
    fn extracts_workspace_convention_candidate() {
        let candidate = candidate_from_line(
            "thread-1",
            0,
            "Use the cargo package name `tamux-daemon`, not the crate path, when invoking `cargo -p`.",
        )
        .expect("candidate");
        assert_eq!(candidate.category, MemoryCategory::Convention);
        assert_eq!(candidate.target_file, "MEMORY.md");
        assert!(candidate.confidence >= 0.8);
    }

    #[test]
    fn extracts_operator_preference_candidate() {
        let candidate = candidate_from_line(
            "thread-1",
            0,
            "I prefer summary-first answers that still include the hard details below.",
        )
        .expect("candidate");
        assert_eq!(candidate.category, MemoryCategory::Preference);
        assert_eq!(candidate.target_file, "USER.md");
    }

    #[test]
    fn filters_ephemeral_lines() {
        assert!(candidate_from_line("thread-1", 0, "begin implementation").is_none());
        assert!(candidate_from_line("thread-1", 0, "Can you continue?").is_none());
    }

    #[test]
    fn trims_oldest_distilled_entries_only() {
        let content = "# Memory\n\n- durable fact\n- [distilled] oldest\n- [distilled] middle\n- [distilled] newest\n";
        let trimmed = trim_distilled_entries_in_content(content, 2);
        assert!(trimmed.contains("- durable fact"));
        assert!(!trimmed.contains("- [distilled] oldest"));
        assert!(trimmed.contains("- [distilled] middle"));
        assert!(trimmed.contains("- [distilled] newest"));
    }

    #[tokio::test]
    async fn review_notification_is_persisted() -> anyhow::Result<()> {
        let root = std::env::temp_dir().join(format!("tamux-distill-test-{}", Uuid::new_v4()));
        let history = HistoryStore::new_test_store(&root).await?;
        let candidates = vec![
            DistillationCandidate {
                source_thread_id: "thread-1".into(),
                source_message_range: Some("msg#1".into()),
                distilled_fact: "Use the cargo package name `tamux-daemon` for `cargo -p`.".into(),
                target_file: "MEMORY.md".into(),
                category: MemoryCategory::Convention,
                confidence: 0.62,
                reasoning: "explicit correction".into(),
            },
            DistillationCandidate {
                source_thread_id: "thread-2".into(),
                source_message_range: Some("msg#2".into()),
                distilled_fact: "Prefer summary-first answers with hard details below.".into(),
                target_file: "USER.md".into(),
                category: MemoryCategory::Preference,
                confidence: 0.58,
                reasoning: "explicit operator preference".into(),
            },
        ];

        emit_review_notification(&history, &candidates, "rarog").await?;

        let notifications = history.list_notifications(false, Some(10)).await?;
        let notification = notifications
            .into_iter()
            .find(|item| item.kind == "memory_distillation_review")
            .expect("memory distillation review notification should exist");
        assert!(notification.title.contains("2 review item"));
        assert!(notification.body.contains("tamux-daemon"));
        assert!(notification.body.contains("summary-first"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
