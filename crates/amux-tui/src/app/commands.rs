use super::*;

impl TuiModel {
    pub(super) fn execute_command(&mut self, command: &str) {
        tracing::info!("execute_command: {:?}", command);
        match command {
            "provider" => {
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
                self.modal.set_picker_item_count(providers::PROVIDERS.len());
            }
            "model" => {
                let models = providers::known_models_for_provider(&self.config.provider);
                if !models.is_empty() {
                    self.config
                        .reduce(config::ConfigAction::ModelsFetched(models));
                }
                self.send_daemon_command(DaemonCommand::FetchModels {
                    provider_id: self.config.provider.clone(),
                    base_url: self.config.base_url.clone(),
                    api_key: self.config.api_key.clone(),
                });
                let count = self.config.fetched_models().len().max(1);
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
                self.modal.set_picker_item_count(count);
            }
            "tools" => {
                self.status_line = "Tools config: use /settings -> Tools tab".to_string();
            }
            "effort" => {
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::EffortPicker));
                self.modal.set_picker_item_count(5);
            }
            "thread" => self
                .modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker)),
            "new" => self.chat.reduce(chat::ChatAction::NewThread),
            "settings" => self
                .modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::Settings)),
            "view" => {
                let next = match self.chat.transcript_mode() {
                    chat::TranscriptMode::Compact => chat::TranscriptMode::Tools,
                    chat::TranscriptMode::Tools => chat::TranscriptMode::Full,
                    chat::TranscriptMode::Full => chat::TranscriptMode::Compact,
                };
                self.chat.reduce(chat::ChatAction::SetTranscriptMode(next));
                self.status_line = format!("View: {:?}", next);
            }
            "quit" => self.pending_quit = true,
            "prompt" => {
                self.status_line = "System prompt: use /settings -> Agent tab".to_string();
            }
            "goal" => {
                self.status_line = "Goal runs: type your goal as a message".to_string();
            }
            "attach" => {
                self.status_line =
                    "Usage: /attach <path>  — attach a file to the next message".to_string();
            }
            "help" => {
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::Help));
                self.modal.set_picker_item_count(100);
            }
            _ => self.status_line = format!("Unknown command: {}", command),
        }
    }

    pub(super) fn submit_prompt(&mut self, prompt: String) {
        if !self.connected {
            self.status_line = "Not connected to daemon".to_string();
            return;
        }
        if self.assistant_busy() {
            self.queued_prompts.push(prompt);
            self.status_line = format!("QUEUED ({})", self.queued_prompts.len());
            return;
        }

        let final_content = if self.attachments.is_empty() {
            prompt.clone()
        } else {
            let mut parts: Vec<String> = self
                .attachments
                .drain(..)
                .map(|att| {
                    format!(
                        "<attached_file name=\"{}\">\n{}\n</attached_file>",
                        att.filename, att.content
                    )
                })
                .collect();
            parts.push(prompt.clone());
            parts.join("\n\n")
        };

        let thread_id = self.chat.active_thread_id().map(String::from);
        if thread_id.is_none() {
            self.chat.reduce(chat::ChatAction::ThreadCreated {
                thread_id: format!("local-{}", self.tick_counter),
                title: if prompt.len() > 40 {
                    format!("{}...", &prompt[..40])
                } else {
                    prompt.clone()
                },
            });
        }

        if let Some(thread) = self.chat.active_thread_mut() {
            thread.messages.push(chat::AgentMessage {
                role: chat::MessageRole::User,
                content: final_content.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                ..Default::default()
            });
        }

        self.send_daemon_command(DaemonCommand::SendMessage {
            thread_id,
            content: final_content,
            session_id: self.default_session_id.clone(),
        });

        self.focus = FocusArea::Chat;
        self.input.set_mode(input::InputMode::Insert);
        self.status_line = "Prompt sent".to_string();
        self.agent_activity = Some("thinking".to_string());
        self.error_active = false;
    }

    pub(super) fn focus_next(&mut self) {
        self.focus = match self.focus {
            FocusArea::Chat => FocusArea::Sidebar,
            FocusArea::Sidebar => FocusArea::Input,
            FocusArea::Input => FocusArea::Chat,
        };
        self.input.set_mode(input::InputMode::Insert);
    }

    pub(super) fn focus_prev(&mut self) {
        self.focus = match self.focus {
            FocusArea::Chat => FocusArea::Input,
            FocusArea::Sidebar => FocusArea::Chat,
            FocusArea::Input => FocusArea::Sidebar,
        };
        self.input.set_mode(input::InputMode::Insert);
    }

    pub(super) fn handle_sidebar_enter(&mut self) {
        let selected = self.sidebar.selected_item();
        let mut flat_items: Vec<SidebarFlatItem> = Vec::new();

        for run in self.tasks.goal_runs() {
            flat_items.push(SidebarFlatItem {
                thread_id: None,
                goal_run_id: Some(run.id.clone()),
                title: run.title.clone(),
            });
            if self.sidebar.is_expanded(&run.id) {
                for step in &run.steps {
                    flat_items.push(SidebarFlatItem {
                        thread_id: None,
                        goal_run_id: Some(run.id.clone()),
                        title: step.title.clone(),
                    });
                }
            }
        }

        for task in self.tasks.tasks() {
            if task.goal_run_id.is_none() {
                flat_items.push(SidebarFlatItem {
                    thread_id: None,
                    goal_run_id: None,
                    title: task.title.clone(),
                });
            }
        }

        if let Some(item) = flat_items.get(selected) {
            if let Some(goal_run_id) = &item.goal_run_id {
                self.send_daemon_command(DaemonCommand::RequestGoalRunDetail(goal_run_id.clone()));
                self.status_line = format!("Goal: {}", item.title);
            } else {
                self.status_line = format!("Task: {}", item.title);
            }
        }
    }
}
