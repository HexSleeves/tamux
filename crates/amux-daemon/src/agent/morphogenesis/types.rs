use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct MorphogenesisAffinity {
    pub agent_id: String,
    pub domain: String,
    pub affinity_score: f64,
    pub task_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_updated_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MorphogenesisOutcome {
    Success,
    Partial,
    Failure,
}
