//! Trajectory-Informed Self-Reflection Loop (Spec 02) — Svarog "Forge"
//!
//! Analyzes execution traces to detect recurring patterns (fallback loops,
//! revision triggers, timeout patterns, approval friction) and generates
//! strategy hints. High-priority hints are appended to MEMORY.md with a
//! `[forge]` provenance prefix.

use serde::{Deserialize, Serialize};

/// An execution pattern detected by the forge pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPattern {
    pub pattern_type: PatternType,
    pub frequency: usize,
    pub affected_tools: Vec<String>,
    pub operator_impact: String, // e.g., "caused 3 revisions"
    pub confidence: f64,
}

/// Types of execution patterns the forge detects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// bash_command → read_file happens repeatedly
    ToolFallbackLoop,
    /// Agent output consistently revised by operator
    RevisionTrigger,
    /// Certain commands consistently time out
    TimeoutProne,
    /// Tools that trigger approval but are always denied
    ApprovalFriction,
    /// Tasks spawned but never completed
    StaleTaskAccumulation,
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolFallbackLoop => write!(f, "tool_fallback_loop"),
            Self::RevisionTrigger => write!(f, "revision_trigger"),
            Self::TimeoutProne => write!(f, "timeout_prone"),
            Self::ApprovalFriction => write!(f, "approval_friction"),
            Self::StaleTaskAccumulation => write!(f, "stale_task_accumulation"),
        }
    }
}

/// A strategy hint generated from detected patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyHint {
    pub for_agent: String,
    pub target: String,            // tool name, behavior, or workflow
    pub hint: String,              // actionable suggestion
    pub priority: u8,              // 1-5, 5 being highest
    pub source_pattern: String,    // pattern_type that generated this
}

/// Configuration for the forge pass.
#[derive(Debug, Clone)]
pub struct ForgeConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub lookback_hours: u64,
    pub min_pattern_frequency: usize,  // patterns must occur ≥N times
    pub min_auto_apply_priority: u8,   // hints with priority ≥ N auto-applied
    pub agent_id: String,
    pub max_forge_entries_per_pass: usize,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 6,
            lookback_hours: 48,
            min_pattern_frequency: 3,
            min_auto_apply_priority: 3,
            agent_id: "svarog".into(),
            max_forge_entries_per_pass: 10,
        }
    }
}

/// Result of a forge analysis pass.
#[derive(Debug)]
pub struct ForgeAnalysis {
    pub agent_id: String,
    pub period_start_ms: u64,
    pub period_end_ms: u64,
    pub traces_analyzed: usize,
    pub patterns: Vec<ExecutionPattern>,
    pub strategy_hints: Vec<StrategyHint>,
}

/// Result summary returned after applying forge hints.
#[derive(Debug, Default)]
pub struct ForgeResult {
    pub traces_analyzed: usize,
    pub patterns_detected: usize,
    pub hints_generated: usize,
    pub hints_auto_applied: usize,
    pub hints_logged_only: usize,
}

/// Run a forge pass over execution traces for the given agent.
pub async fn run_forge_pass(
    _db: &crate::history::HistoryStore,
    _config: &ForgeConfig,
    _memory_dir: &std::path::Path,
) -> anyhow::Result<ForgeResult> {
    // TODO: implement
    // 1. Query execution_traces for the agent within the lookback window
    // 2. Group by tool call patterns, outcome types, operator feedback signals
    // 3. Identify recurring patterns (frequency > threshold)
    // 4. Generate strategy hints from patterns
    // 5. Apply high-priority hints (priority ≥ min_auto_apply_priority) to MEMORY.md
    // 6. Log lower-priority hints without writing
    // 7. Record pass in forge_pass_log
    Ok(ForgeResult::default())
}

/// Apply forge hints to MEMORY.md (append with [forge] prefix).
pub async fn apply_forge_hints(
    _hints: &[StrategyHint],
    _min_priority: u8,
    _memory_dir: &std::path::Path,
    _max_entries: usize,
) -> anyhow::Result<Vec<String>> {
    // TODO: implement
    // 1. Filter hints by min_priority
    // 2. For each hint, append to MEMORY.md as "- [forge] {timestamp}: {hint}"
    // 3. Enforce file size limits
    // 4. Return list of applied hint texts
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_type_display() {
        assert_eq!(PatternType::ToolFallbackLoop.to_string(), "tool_fallback_loop");
        assert_eq!(PatternType::TimeoutProne.to_string(), "timeout_prone");
    }

    #[test]
    fn default_config_sane() {
        let cfg = ForgeConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_hours, 6);
        assert_eq!(cfg.min_pattern_frequency, 3);
        assert_eq!(cfg.agent_id, "svarog");
    }

    #[test]
    fn strategy_hint_serialization() {
        let hint = StrategyHint {
            for_agent: "svarog".into(),
            target: "bash_command".into(),
            hint: "prefer read_file over bash_command for file reads".into(),
            priority: 4,
            source_pattern: "tool_fallback_loop".into(),
        };
        let json = serde_json::to_string(&hint).unwrap();
        assert!(json.contains("tool_fallback_loop"));
        assert!(json.contains("prefer read_file"));
    }
}
