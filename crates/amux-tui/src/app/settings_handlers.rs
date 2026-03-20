use super::*;

impl TuiModel {
    pub(super) fn activate_settings_field(&mut self) {
        let field = self.settings.current_field_name().to_string();
        match field.as_str() {
            "provider" => self.execute_command("provider"),
            "model" => self.execute_command("model"),
            "reasoning_effort" => self.execute_command("effort"),
            "base_url" => self
                .settings
                .start_editing("base_url", &self.config.base_url.clone()),
            "api_key" => self
                .settings
                .start_editing("api_key", &self.config.api_key.clone()),
            "gateway_prefix" => {
                self.settings
                    .start_editing("gateway_prefix", &self.config.gateway_prefix.clone());
            }
            "slack_token" => {
                self.settings
                    .start_editing("slack_token", &self.config.slack_token.clone());
            }
            "slack_channel_filter" => {
                self.settings.start_editing(
                    "slack_channel_filter",
                    &self.config.slack_channel_filter.clone(),
                );
            }
            "telegram_token" => {
                self.settings
                    .start_editing("telegram_token", &self.config.telegram_token.clone());
            }
            "telegram_allowed_chats" => {
                self.settings.start_editing(
                    "telegram_allowed_chats",
                    &self.config.telegram_allowed_chats.clone(),
                );
            }
            "discord_token" => {
                self.settings
                    .start_editing("discord_token", &self.config.discord_token.clone());
            }
            "discord_channel_filter" => {
                self.settings.start_editing(
                    "discord_channel_filter",
                    &self.config.discord_channel_filter.clone(),
                );
            }
            "discord_allowed_users" => {
                self.settings.start_editing(
                    "discord_allowed_users",
                    &self.config.discord_allowed_users.clone(),
                );
            }
            "whatsapp_allowed_contacts" => {
                self.settings.start_editing(
                    "whatsapp_allowed_contacts",
                    &self.config.whatsapp_allowed_contacts.clone(),
                );
            }
            "whatsapp_token" => {
                self.settings
                    .start_editing("whatsapp_token", &self.config.whatsapp_token.clone());
            }
            "whatsapp_phone_id" => {
                self.settings
                    .start_editing("whatsapp_phone_id", &self.config.whatsapp_phone_id.clone());
            }
            "search_provider" => {
                let next = match self.config.search_provider.as_str() {
                    "none" | "" => "firecrawl",
                    "firecrawl" => "exa",
                    "exa" => "tavily",
                    _ => "none",
                };
                self.config.search_provider = next.to_string();
                self.sync_config_to_daemon();
            }
            "firecrawl_api_key" => self
                .settings
                .start_editing("firecrawl_api_key", &self.config.firecrawl_api_key.clone()),
            "exa_api_key" => self
                .settings
                .start_editing("exa_api_key", &self.config.exa_api_key.clone()),
            "tavily_api_key" => self
                .settings
                .start_editing("tavily_api_key", &self.config.tavily_api_key.clone()),
            "search_max_results" => self.settings.start_editing(
                "search_max_results",
                &self.config.search_max_results.to_string(),
            ),
            "search_timeout" => self.settings.start_editing(
                "search_timeout",
                &self.config.search_timeout_secs.to_string(),
            ),
            "honcho_api_key" => self
                .settings
                .start_editing("honcho_api_key", &self.config.honcho_api_key.clone()),
            "honcho_base_url" => self
                .settings
                .start_editing("honcho_base_url", &self.config.honcho_base_url.clone()),
            "honcho_workspace_id" => self.settings.start_editing(
                "honcho_workspace_id",
                &self.config.honcho_workspace_id.clone(),
            ),
            "max_context_messages" => self.settings.start_editing(
                "max_context_messages",
                &self.config.max_context_messages.to_string(),
            ),
            "max_tool_loops" => self
                .settings
                .start_editing("max_tool_loops", &self.config.max_tool_loops.to_string()),
            "max_retries" => self
                .settings
                .start_editing("max_retries", &self.config.max_retries.to_string()),
            "retry_delay_ms" => self
                .settings
                .start_editing("retry_delay_ms", &self.config.retry_delay_ms.to_string()),
            "context_budget_tokens" => self.settings.start_editing(
                "context_budget_tokens",
                &self.config.context_budget_tokens.to_string(),
            ),
            "compact_threshold_pct" => self.settings.start_editing(
                "compact_threshold_pct",
                &self.config.compact_threshold_pct.to_string(),
            ),
            "keep_recent_on_compact" => self.settings.start_editing(
                "keep_recent_on_compact",
                &self.config.keep_recent_on_compact.to_string(),
            ),
            "bash_timeout_secs" => self.settings.start_editing(
                "bash_timeout_secs",
                &self.config.bash_timeout_secs.to_string(),
            ),
            "snapshot_max_count" => self.settings.start_editing(
                "snapshot_max_count",
                &self.config.snapshot_max_count.to_string(),
            ),
            "snapshot_max_size_mb" => self.settings.start_editing(
                "snapshot_max_size_mb",
                &self.config.snapshot_max_size_mb.to_string(),
            ),
            "snapshot_stats" => {
                // Read-only field, no-op on Enter
            }
            "agent_name" => {
                let current = self
                    .config
                    .agent_config_raw
                    .as_ref()
                    .and_then(|raw| raw.get("agent_name"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("Sisyphus")
                    .to_string();
                self.settings.start_editing("agent_name", &current);
            }
            "system_prompt" => {
                let current = self
                    .config
                    .agent_config_raw
                    .as_ref()
                    .and_then(|raw| raw.get("system_prompt"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string();
                self.settings.start_editing("system_prompt", &current);
            }
            _ => {}
        }
    }

    pub(super) fn toggle_settings_field(&mut self) {
        let field = self.settings.current_field_name().to_string();
        match field.as_str() {
            "gateway_enabled" => {
                self.config.gateway_enabled = !self.config.gateway_enabled;
                self.sync_config_to_daemon();
            }
            "web_search_enabled" => {
                self.config.tool_web_search = !self.config.tool_web_search;
                self.sync_config_to_daemon();
            }
            "enable_streaming" => {
                self.config.enable_streaming = !self.config.enable_streaming;
                self.sync_config_to_daemon();
            }
            "enable_conversation_memory" => {
                self.config.enable_conversation_memory = !self.config.enable_conversation_memory;
                self.sync_config_to_daemon();
            }
            "enable_honcho_memory" => {
                self.config.enable_honcho_memory = !self.config.enable_honcho_memory;
                self.sync_config_to_daemon();
            }
            "auto_compact_context" => {
                self.config.auto_compact_context = !self.config.auto_compact_context;
                self.sync_config_to_daemon();
            }
            "snapshot_auto_cleanup" => {
                self.config.snapshot_auto_cleanup = !self.config.snapshot_auto_cleanup;
                self.sync_config_to_daemon();
            }
            field if field.starts_with("tool_") => {
                let tool_name = field.strip_prefix("tool_").unwrap_or(field).to_string();
                self.config
                    .reduce(config::ConfigAction::ToggleTool(tool_name));
                self.sync_config_to_daemon();
            }
            _ => self.settings.reduce(SettingsAction::ToggleCheckbox),
        }
    }

    pub(super) fn settings_field_click_uses_toggle(&self) -> bool {
        matches!(
            self.settings.current_field_name(),
            "gateway_enabled"
                | "web_search_enabled"
                | "enable_streaming"
                | "enable_conversation_memory"
                | "enable_honcho_memory"
                | "auto_compact_context"
                | "snapshot_auto_cleanup"
        ) || self.settings.current_field_name().starts_with("tool_")
    }
}
