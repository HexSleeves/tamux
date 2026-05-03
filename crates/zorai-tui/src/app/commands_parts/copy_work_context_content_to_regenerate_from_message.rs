impl TuiModel {
    pub(super) fn copy_work_context_content(&mut self) {
        let Some(thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };

        let text = match self.sidebar.active_tab() {
            sidebar::SidebarTab::Files => {
                let Some(path) = self.selected_sidebar_file_path() else {
                    return;
                };
                let Some(entry) = self
                    .tasks
                    .work_context_for_thread(&thread_id)
                    .and_then(|context| context.entries.iter().find(|entry| entry.path == path))
                else {
                    return;
                };
                if let Some(repo_root) = entry.repo_root.as_deref() {
                    self.tasks
                        .diff_for_path(repo_root, &entry.path)
                        .map(str::to_string)
                        .filter(|value| !value.trim().is_empty())
                } else {
                    self.tasks
                        .preview_for_path(&entry.path)
                        .filter(|preview| preview.is_text)
                        .map(|preview| preview.content.clone())
                        .filter(|value| !value.trim().is_empty())
                }
            }
            sidebar::SidebarTab::Todos => self
                .tasks
                .todos_for_thread(&thread_id)
                .get(self.sidebar.selected_item())
                .map(|todo| todo.content.clone())
                .filter(|value| !value.trim().is_empty()),
            sidebar::SidebarTab::Spawned => None,
            sidebar::SidebarTab::Pinned => self
                .selected_sidebar_pinned_message()
                .map(|message| message.content)
                .filter(|value| !value.trim().is_empty()),
        };

        if let Some(text) = text {
            conversion::copy_to_clipboard(&text);
            self.status_line = "Copied to clipboard".to_string();
        }
    }

    pub(super) fn resend_message(&mut self, index: usize) {
        let content = self
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.get(index))
            .map(|message| message.content.clone());
        if let Some(content) = content.filter(|value| !value.trim().is_empty()) {
            self.submit_prompt(content);
        }
    }

    pub(super) fn pin_message_for_compaction(&mut self, index: usize) {
        let (thread_id, message_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            let Some(message) = thread.messages.get(index) else {
                return;
            };
            let Some(message_id) = message.id.clone().filter(|id| !id.is_empty()) else {
                self.status_line = "Cannot pin message without a daemon id".to_string();
                return;
            };
            (thread.id.clone(), message_id)
        };

        self.send_daemon_command(DaemonCommand::PinThreadMessageForCompaction {
            thread_id,
            message_id,
        });
    }

    pub(super) fn unpin_message_for_compaction(&mut self, index: usize) {
        let (thread_id, message_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            let Some(message) = thread.messages.get(index) else {
                return;
            };
            let Some(message_id) = message.id.clone().filter(|id| !id.is_empty()) else {
                self.status_line = "Cannot unpin message without a daemon id".to_string();
                return;
            };
            (thread.id.clone(), message_id)
        };

        let absolute_index = self
            .chat
            .active_thread()
            .map(|thread| thread.loaded_message_start.saturating_add(index));
        self.unpin_message_for_compaction_by_id(thread_id, message_id, absolute_index);
    }

    fn unpin_message_for_compaction_by_id(
        &mut self,
        thread_id: String,
        message_id: String,
        absolute_index: Option<usize>,
    ) {
        self.send_daemon_command(DaemonCommand::UnpinThreadMessageForCompaction {
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        });
        self.chat
            .reduce(chat::ChatAction::UnpinMessageForCompaction {
                thread_id,
                message_id,
                absolute_index,
            });
        if self.sidebar.active_tab() == sidebar::SidebarTab::Pinned
            && !self.chat.active_thread_has_pinned_messages()
        {
            self.sidebar.reduce(sidebar::SidebarAction::SwitchTab(
                sidebar::SidebarTab::Todos,
            ));
        }
    }

    pub(super) fn unpin_selected_sidebar_message(&mut self) {
        let Some(pinned_message) = self.selected_sidebar_pinned_message() else {
            return;
        };
        let Some(thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };
        self.unpin_message_for_compaction_by_id(
            thread_id,
            pinned_message.message_id,
            Some(pinned_message.absolute_index),
        );
    }

    pub(super) fn delete_message(&mut self, index: usize) {
        let (thread_id, msg_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            if index >= thread.messages.len() {
                return;
            }
            let mid = thread.messages[index]
                .id
                .clone()
                .unwrap_or_else(|| format!("{}:{}", thread.id, index));
            (thread.id.clone(), mid)
        };

        self.send_daemon_command(DaemonCommand::DeleteMessages {
            thread_id,
            message_ids: vec![msg_id],
        });

        // Remove locally.
        self.chat.delete_active_message(index);
        self.status_line = format!("Deleted message {}", index + 1);
    }

    pub(super) fn regenerate_from_message(&mut self, index: usize) {
        let prompt = self.chat.active_thread().and_then(|thread| {
            thread
                .messages
                .iter()
                .take(index)
                .rev()
                .find(|message| {
                    message.role == chat::MessageRole::User && !message.content.trim().is_empty()
                })
                .map(|message| message.content.clone())
        });
        if let Some(prompt) = prompt {
            self.submit_prompt(prompt);
        }
    }
}
