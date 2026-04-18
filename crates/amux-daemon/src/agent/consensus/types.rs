use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ConsensusBidPrior {
    pub role: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub prior_score: f64,
    pub last_updated_ms: u64,
}
