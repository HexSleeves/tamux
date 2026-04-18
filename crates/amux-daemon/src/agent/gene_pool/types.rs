use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct GenePoolCandidate {
    pub trace_id: String,
    pub proposed_skill_name: String,
    pub task_type: String,
    pub context_tags: Vec<String>,
    pub quality_score: f64,
    pub tool_sequence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct GenePoolArenaScore {
    pub variant_id: String,
    pub skill_name: String,
    pub variant_name: String,
    pub status: String,
    pub arena_score: f64,
    pub success_rate: f64,
    pub fitness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct GenePoolLifecycleAction {
    pub action: String,
    pub variant_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct GenePoolRuntimeSnapshot {
    pub generated_at_ms: u64,
    pub candidates: Vec<GenePoolCandidate>,
    pub arena_scores: Vec<GenePoolArenaScore>,
    pub lifecycle_actions: Vec<GenePoolLifecycleAction>,
    pub summary: String,
}
