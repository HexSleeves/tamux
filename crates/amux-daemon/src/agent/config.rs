//! Agent configuration get/set.

use super::*;

impl AgentEngine {
    pub async fn get_config(&self) -> AgentConfig {
        self.config.read().await.clone()
    }

    pub async fn set_config(&self, config: AgentConfig) {
        *self.config.write().await = config;
        self.persist_config().await;
    }
}
