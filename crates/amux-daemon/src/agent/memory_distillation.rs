//! Episodic → Semantic Memory Distillation (Spec 01)
//!
//! Analyzes old, undistilled thread transcripts and extracts actionable
//! memory entries (preferences, conventions, patterns, corrections, lessons).
//! Candidates above the auto-apply confidence threshold are written to
//! MEMORY.md/USER.md with a `[distilled]` provenance prefix.

use crate::history::HistoryStore;
use serde::{Deserialize, Serialize};

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
    pub target_file: String,      // "MEMORY.md" or "USER.md"
    pub category: MemoryCategory,
    pub confidence: f64,           // 0.0–1.0
    pub reasoning: String,         // why this should be saved
}

/// Configuration for the distillation pass.
#[derive(Debug, Clone)]
pub struct DistillationConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub confidence_auto_apply: f64,    // default: 0.7
    pub confidence_review_queue: f64,  // default: 0.5
    pub max_entries_per_file: usize,   // default: 50
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
}

/// Run a distillation pass over old, undistilled threads.
pub async fn run_distillation_pass(
    _db: &HistoryStore,
    _config: &DistillationConfig,
    _memory_dir: &std::path::Path,
) -> anyhow::Result<DistillationResult> {
    // TODO: implement
    // 1. Query threads older than 1h that haven't been distilled
    // 2. For each thread, extract operator corrections, patterns, facts
    // 3. Score candidates by confidence
    // 4. Apply high-confidence candidates to MEMORY.md/USER.md
    // 5. Queue medium-confidence for operator review
    // 6. Log all decisions to memory_distillation_log
    Ok(DistillationResult::default())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
