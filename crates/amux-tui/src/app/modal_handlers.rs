use super::*;

impl TuiModel {
    pub(super) fn handle_key_modal(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        kind: modal::ModalKind,
    ) -> bool {
        if kind == modal::ModalKind::Settings {
            if self.settings.is_editing() {
                if self.settings.is_textarea() {
                    match code {
                        KeyCode::Enter if modifiers.contains(KeyModifiers::CONTROL) => {}
                        KeyCode::Enter => {
                            self.settings.reduce(SettingsAction::InsertChar('\n'));
                            return false;
                        }
                        KeyCode::Esc => {
                            self.settings.reduce(SettingsAction::CancelEdit);
                            return false;
                        }
                        KeyCode::Backspace => {
                            self.settings.reduce(SettingsAction::Backspace);
                            return false;
                        }
                        KeyCode::Char(c) => {
                            self.settings.reduce(SettingsAction::InsertChar(c));
                            return false;
                        }
                        _ => return false,
                    }
                }

                match code {
                    KeyCode::Enter => {
                        let field = self.settings.editing_field().unwrap_or("").to_string();
                        let value = self.settings.edit_buffer().to_string();
                        match field.as_str() {
                            "base_url" => self.config.base_url = value,
                            "api_key" => self.config.api_key = value,
                            "gateway_prefix" => self.config.gateway_prefix = value,
                            "slack_token" => self.config.slack_token = value,
                            "slack_channel_filter" => self.config.slack_channel_filter = value,
                            "telegram_token" => self.config.telegram_token = value,
                            "telegram_allowed_chats" => self.config.telegram_allowed_chats = value,
                            "discord_token" => self.config.discord_token = value,
                            "discord_channel_filter" => self.config.discord_channel_filter = value,
                            "discord_allowed_users" => self.config.discord_allowed_users = value,
                            "whatsapp_allowed_contacts" => {
                                self.config.whatsapp_allowed_contacts = value
                            }
                            "whatsapp_token" => self.config.whatsapp_token = value,
                            "whatsapp_phone_id" => self.config.whatsapp_phone_id = value,
                            "firecrawl_api_key" => self.config.firecrawl_api_key = value,
                            "exa_api_key" => self.config.exa_api_key = value,
                            "tavily_api_key" => self.config.tavily_api_key = value,
                            "search_max_results" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.search_max_results = n.clamp(1, 20);
                                }
                            }
                            "search_timeout" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.search_timeout_secs = n.clamp(3, 120);
                                }
                            }
                            "honcho_api_key" => self.config.honcho_api_key = value,
                            "honcho_base_url" => self.config.honcho_base_url = value,
                            "honcho_workspace_id" => self.config.honcho_workspace_id = value,
                            "max_context_messages" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.max_context_messages = n.clamp(10, 500);
                                }
                            }
                            "max_tool_loops" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.max_tool_loops = n.clamp(0, 1000);
                                }
                            }
                            "max_retries" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.max_retries = n.clamp(0, 10);
                                }
                            }
                            "retry_delay_ms" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.retry_delay_ms = n.clamp(100, 60000);
                                }
                            }
                            "context_budget_tokens" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.context_budget_tokens = n.clamp(10000, 500000);
                                }
                            }
                            "compact_threshold_pct" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.compact_threshold_pct = n.clamp(50, 95);
                                }
                            }
                            "keep_recent_on_compact" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.keep_recent_on_compact = n.clamp(1, 50);
                                }
                            }
                            "bash_timeout_secs" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.bash_timeout_secs = n.clamp(5, 300);
                                }
                            }
                            "snapshot_max_count" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.snapshot_max_count = n.clamp(1, 100);
                                }
                            }
                            "snapshot_max_size_mb" => {
                                if let Ok(n) = value.parse::<u32>() {
                                    self.config.snapshot_max_size_mb = n.clamp(100, 500_000);
                                }
                            }
                            "agent_name" => {
                                if let Some(ref mut raw) = self.config.agent_config_raw {
                                    raw["agent_name"] = serde_json::Value::String(value);
                                }
                            }
                            "system_prompt" => {
                                if let Some(ref mut raw) = self.config.agent_config_raw {
                                    raw["system_prompt"] = serde_json::Value::String(value);
                                } else {
                                    let mut raw = serde_json::json!({});
                                    raw["system_prompt"] = serde_json::Value::String(value);
                                    self.config.agent_config_raw = Some(raw);
                                }
                            }
                            _ => {}
                        }
                        self.settings.reduce(SettingsAction::ConfirmEdit);
                        self.sync_config_to_daemon();
                    }
                    KeyCode::Esc => self.settings.reduce(SettingsAction::CancelEdit),
                    KeyCode::Backspace => self.settings.reduce(SettingsAction::Backspace),
                    KeyCode::Char(c) => self.settings.reduce(SettingsAction::InsertChar(c)),
                    _ => {}
                }
                return false;
            }

            match code {
                KeyCode::Tab => {
                    let all = SettingsTab::all();
                    let current = self.settings.active_tab();
                    let next_idx = all
                        .iter()
                        .position(|&tab| tab == current)
                        .map(|i| (i + 1) % all.len())
                        .unwrap_or(0);
                    self.settings
                        .reduce(SettingsAction::SwitchTab(all[next_idx]));
                    return false;
                }
                KeyCode::BackTab => {
                    let all = SettingsTab::all();
                    let current = self.settings.active_tab();
                    let prev_idx = all
                        .iter()
                        .position(|&tab| tab == current)
                        .map(|i| if i == 0 { all.len() - 1 } else { i - 1 })
                        .unwrap_or(0);
                    self.settings
                        .reduce(SettingsAction::SwitchTab(all[prev_idx]));
                    return false;
                }
                KeyCode::Down => {
                    self.settings.reduce(SettingsAction::NavigateField(1));
                    return false;
                }
                KeyCode::Up => {
                    self.settings.reduce(SettingsAction::NavigateField(-1));
                    return false;
                }
                KeyCode::Enter => {
                    self.activate_settings_field();
                    return false;
                }
                KeyCode::Char(' ') => {
                    self.toggle_settings_field();
                    return false;
                }
                _ => {}
            }
        }

        if kind == modal::ModalKind::ApprovalOverlay {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(ap) = self.approval.current_approval() {
                        self.send_daemon_command(DaemonCommand::ResolveTaskApproval {
                            approval_id: ap.approval_id.clone(),
                            decision: "allow_once".to_string(),
                        });
                    }
                    self.modal.reduce(modal::ModalAction::Pop);
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    if let Some(ap) = self.approval.current_approval() {
                        self.send_daemon_command(DaemonCommand::ResolveTaskApproval {
                            approval_id: ap.approval_id.clone(),
                            decision: "allow_session".to_string(),
                        });
                    }
                    self.modal.reduce(modal::ModalAction::Pop);
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if let Some(ap) = self.approval.current_approval() {
                        self.send_daemon_command(DaemonCommand::ResolveTaskApproval {
                            approval_id: ap.approval_id.clone(),
                            decision: "reject".to_string(),
                        });
                    }
                    self.modal.reduce(modal::ModalAction::Pop);
                }
                _ => {}
            }
            return false;
        }

        let is_searchable = matches!(
            kind,
            modal::ModalKind::CommandPalette | modal::ModalKind::ThreadPicker
        );

        match code {
            KeyCode::Esc => {
                self.modal.reduce(modal::ModalAction::Pop);
                self.input.reduce(input::InputAction::Clear);
            }
            KeyCode::Down => self.modal.reduce(modal::ModalAction::Navigate(1)),
            KeyCode::Up => self.modal.reduce(modal::ModalAction::Navigate(-1)),
            KeyCode::Enter => {
                self.handle_modal_enter(kind);
                if self.pending_quit {
                    self.pending_quit = false;
                    return true;
                }
            }
            KeyCode::Backspace if is_searchable => {
                self.input.reduce(input::InputAction::Backspace);
                self.modal.reduce(modal::ModalAction::SetQuery(
                    self.input.buffer().to_string(),
                ));
            }
            KeyCode::Char(c) if is_searchable => {
                self.input.reduce(input::InputAction::InsertChar(c));
                self.modal.reduce(modal::ModalAction::SetQuery(
                    self.input.buffer().to_string(),
                ));
            }
            _ => {}
        }

        false
    }

    pub(super) fn handle_modal_enter(&mut self, kind: modal::ModalKind) {
        tracing::info!("handle_modal_enter: {:?}", kind);
        match kind {
            modal::ModalKind::CommandPalette => {
                let cmd_name = self.modal.selected_command().map(|cmd| cmd.command.clone());
                tracing::info!(
                    "selected_command: {:?}, cursor: {}, filtered: {:?}",
                    cmd_name,
                    self.modal.picker_cursor(),
                    self.modal.filtered_items()
                );
                self.modal.reduce(modal::ModalAction::Pop);
                self.input.reduce(input::InputAction::Clear);
                if let Some(command) = cmd_name {
                    self.execute_command(&command);
                }
            }
            modal::ModalKind::ThreadPicker => {
                let cursor = self.modal.picker_cursor();
                self.modal.reduce(modal::ModalAction::Pop);
                self.input.reduce(input::InputAction::Clear);
                if cursor == 0 {
                    self.chat.reduce(chat::ChatAction::NewThread);
                    self.status_line = "New conversation".to_string();
                } else if let Some(thread) = self.chat.threads().get(cursor - 1) {
                    let tid = thread.id.clone();
                    let title = thread.title.clone();
                    self.chat
                        .reduce(chat::ChatAction::SelectThread(tid.clone()));
                    self.send_daemon_command(DaemonCommand::RequestThread(tid));
                    self.status_line = format!("Thread: {}", title);
                }
            }
            modal::ModalKind::ProviderPicker => {
                let cursor = self.modal.picker_cursor();
                if let Some(def) = providers::PROVIDERS.get(cursor) {
                    let old_base_url = self.config.base_url.clone();
                    self.config.provider = def.id.to_string();
                    if old_base_url.is_empty() || providers::is_known_default_url(&old_base_url) {
                        self.config.base_url = def.default_base_url.to_string();
                    }
                    self.config.model = def.default_model.to_string();
                    self.config.api_transport = def.default_transport.to_string();
                    self.config.assistant_id.clear();

                    if let Some(raw) = &self.config.agent_config_raw {
                        if let Some(provider_config) = raw.get(def.id) {
                            if let Some(key) = provider_config
                                .get("apiKey")
                                .and_then(|value| value.as_str())
                            {
                                if !key.is_empty() {
                                    self.config.api_key = key.to_string();
                                }
                            }
                            if let Some(saved_model) = provider_config
                                .get("model")
                                .and_then(|value| value.as_str())
                            {
                                if !saved_model.is_empty() {
                                    self.config.model = saved_model.to_string();
                                }
                            }
                            if let Some(saved_transport) = provider_config
                                .get("apiTransport")
                                .and_then(|value| value.as_str())
                            {
                                self.config.api_transport = if def
                                    .supported_transports
                                    .contains(&saved_transport)
                                {
                                    saved_transport.to_string()
                                } else {
                                    def.default_transport.to_string()
                                };
                            }
                            if let Some(saved_assistant_id) = provider_config
                                .get("assistantId")
                                .and_then(|value| value.as_str())
                            {
                                self.config.assistant_id = saved_assistant_id.to_string();
                            }
                        }
                    }

                    let models = providers::known_models_for_provider(def.id);
                    self.config
                        .reduce(config::ConfigAction::ModelsFetched(models));
                    self.status_line = format!("Provider: {}", def.name);
                    self.sync_config_to_daemon();
                }
                self.modal.reduce(modal::ModalAction::Pop);
            }
            modal::ModalKind::ModelPicker => {
                let models = self.config.fetched_models();
                if models.is_empty() {
                    self.status_line = "No models available. Set model in /settings".to_string();
                } else {
                    let cursor = self.modal.picker_cursor();
                    if let Some(model) = models.get(cursor) {
                        let model_id = model.id.clone();
                        self.config
                            .reduce(config::ConfigAction::SetModel(model_id.clone()));
                        self.status_line = format!("Model: {}", model_id);
                        if let Ok(json) = serde_json::to_string(&serde_json::json!({
                            "model": model_id,
                        })) {
                            self.send_daemon_command(DaemonCommand::SetConfigJson(json));
                        }
                        self.save_settings();
                    }
                }
                self.modal.reduce(modal::ModalAction::Pop);
            }
            modal::ModalKind::EffortPicker => {
                let efforts = ["", "low", "medium", "high", "xhigh"];
                let cursor = self.modal.picker_cursor();
                if let Some(&effort) = efforts.get(cursor) {
                    self.config
                        .reduce(config::ConfigAction::SetReasoningEffort(effort.to_string()));
                    if let Ok(json) = serde_json::to_string(&serde_json::json!({
                        "reasoning_effort": effort,
                    })) {
                        self.send_daemon_command(DaemonCommand::SetConfigJson(json));
                    }
                    self.status_line = if effort.is_empty() {
                        "Effort: off".to_string()
                    } else {
                        format!("Effort: {}", effort)
                    };
                    self.save_settings();
                }
                self.modal.reduce(modal::ModalAction::Pop);
            }
            _ => {
                self.modal.reduce(modal::ModalAction::Pop);
                self.input.reduce(input::InputAction::Clear);
            }
        }
    }
}
