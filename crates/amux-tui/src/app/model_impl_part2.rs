impl TuiModel {
    fn cleanup_concierge_on_navigate(&mut self) {
        if !self.concierge.auto_cleanup_on_navigate {
            return;
        }

        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.ignore_pending_concierge_welcome = true;
        self.concierge
            .reduce(crate::state::ConciergeAction::WelcomeDismissed);
        self.chat.reduce(chat::ChatAction::DismissConciergeWelcome);
        self.send_daemon_command(DaemonCommand::DismissConciergeWelcome);

        if self.chat.active_thread_id() == Some("concierge") && self.assistant_busy() {
            let thread_id = "concierge".to_string();
            self.cancelled_thread_id = Some(thread_id.clone());
            self.chat.reduce(chat::ChatAction::ForceStopStreaming);
            self.agent_activity = None;
            self.send_daemon_command(DaemonCommand::StopStream { thread_id });
        }

        self.clear_pending_stop();
    }

    fn open_thread_conversation(&mut self, thread_id: String) {
        self.cleanup_concierge_on_navigate();
        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.chat
            .reduce(chat::ChatAction::SelectThread(thread_id.clone()));
        self.send_daemon_command(DaemonCommand::RequestThread(thread_id));
        self.main_pane_view = MainPaneView::Conversation;
        self.focus = FocusArea::Chat;
    }

    fn start_new_thread_view(&mut self) {
        self.cleanup_concierge_on_navigate();
        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.chat.reduce(chat::ChatAction::NewThread);
        self.main_pane_view = MainPaneView::Conversation;
        self.focus = FocusArea::Input;
        self.concierge
            .reduce(crate::state::ConciergeAction::WelcomeLoading(false));
    }

    fn request_concierge_welcome(&mut self) {
        self.ignore_pending_concierge_welcome = false;
        self.concierge
            .reduce(crate::state::ConciergeAction::WelcomeLoading(true));
        self.send_daemon_command(DaemonCommand::RequestConciergeWelcome);
    }

    fn set_main_pane_conversation(&mut self, focus: FocusArea) {
        self.main_pane_view = MainPaneView::Conversation;
        self.task_view_scroll = 0;
        self.focus = focus;
    }

    fn dismiss_active_main_pane(&mut self, focus: FocusArea) -> bool {
        match &self.main_pane_view {
            MainPaneView::Task(target) => {
                if let Some(thread_id) = self.target_thread_id(target) {
                    if self.tasks.selected_work_path(&thread_id).is_some() {
                        self.tasks.reduce(task::TaskAction::SelectWorkPath {
                            thread_id,
                            path: None,
                        });
                        self.focus = focus;
                        return true;
                    }
                }
                self.set_main_pane_conversation(focus);
                true
            }
            MainPaneView::WorkContext | MainPaneView::FilePreview(_) | MainPaneView::GoalComposer => {
                self.set_main_pane_conversation(focus);
                true
            }
            MainPaneView::Conversation => false,
        }
    }

    fn should_toggle_work_context_from_sidebar(&self, thread_id: &str) -> bool {
        if !matches!(self.main_pane_view, MainPaneView::WorkContext) {
            return false;
        }

        match self.sidebar.active_tab() {
            SidebarTab::Files => self
                .tasks
                .work_context_for_thread(thread_id)
                .and_then(|context| context.entries.get(self.sidebar.selected_item()))
                .is_some_and(|entry| {
                    self.tasks.selected_work_path(thread_id) == Some(entry.path.as_str())
                }),
            SidebarTab::Todos => self
                .tasks
                .todos_for_thread(thread_id)
                .get(self.sidebar.selected_item())
                .is_some(),
        }
    }

    fn visible_concierge_action_count(&self) -> usize {
        let active_actions = self.chat.active_actions();
        if !active_actions.is_empty() {
            active_actions.len()
        } else {
            self.concierge.welcome_actions.len()
        }
    }

    fn select_visible_concierge_action(&mut self, action_index: usize) {
        let action_count = self.visible_concierge_action_count();
        self.concierge.selected_action = if action_count == 0 {
            0
        } else {
            action_index.min(action_count - 1)
        };
    }

    fn navigate_visible_concierge_action(&mut self, delta: i32) {
        let action_count = self.visible_concierge_action_count();
        if action_count == 0 {
            self.concierge.selected_action = 0;
        } else if delta > 0 {
            self.concierge.selected_action =
                (self.concierge.selected_action + delta as usize).min(action_count - 1);
        } else {
            self.concierge.selected_action = self
                .concierge
                .selected_action
                .saturating_sub((-delta) as usize);
        }
    }

    fn resolve_visible_concierge_action(
        &self,
        action_index: usize,
    ) -> Option<crate::state::ConciergeActionVm> {
        self.chat
            .active_actions()
            .get(action_index)
            .map(|action| crate::state::ConciergeActionVm {
                label: action.label.clone(),
                action_type: action.action_type.clone(),
                thread_id: action.thread_id.clone(),
            })
            .or_else(|| self.concierge.welcome_actions.get(action_index).cloned())
    }

    fn execute_concierge_action(&mut self, action_index: usize) {
        let Some(action) = self.resolve_visible_concierge_action(action_index) else {
            return;
        };
        self.run_concierge_action(action);
    }

    fn selected_inline_message_action_count(&self) -> usize {
        let Some(selected_message) = self.chat.selected_message() else {
            return 0;
        };
        let Some(message) = self
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.get(selected_message))
        else {
            return 0;
        };
        let is_last_actionable = !message.actions.is_empty()
            && self
                .chat
                .active_actions()
                .first()
                .map(|action| &action.label)
                == message.actions.first().map(|action| &action.label);
        if is_last_actionable {
            0
        } else {
            widgets::chat::message_action_targets(
                &self.chat,
                selected_message,
                message,
                self.tick_counter,
            )
            .len()
        }
    }

    fn execute_concierge_message_action(&mut self, message_index: usize, action_index: usize) {
        let Some(action) = self
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.get(message_index))
            .and_then(|message| message.actions.get(action_index))
            .cloned()
        else {
            return;
        };
        self.run_concierge_action(crate::state::ConciergeActionVm {
            label: action.label,
            action_type: action.action_type,
            thread_id: action.thread_id,
        });
    }

    fn run_concierge_action(&mut self, action: crate::state::ConciergeActionVm) {
        match action.action_type.as_str() {
            "continue_session" => {
                if let Some(thread_id) = action.thread_id {
                    self.open_thread_conversation(thread_id);
                }
            }
            "start_new" => {
                self.start_new_thread_view();
            }
            "search" => {
                self.ignore_pending_concierge_welcome = true;
                self.concierge
                    .reduce(crate::state::ConciergeAction::WelcomeDismissed);
                self.send_daemon_command(DaemonCommand::DismissConciergeWelcome);
                self.chat
                    .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
                self.send_daemon_command(DaemonCommand::RequestThread("concierge".to_string()));
                self.main_pane_view = MainPaneView::Conversation;
                self.focus = FocusArea::Input;
                self.set_input_text("Search history for: ");
                self.status_line = "Describe what you want to search and press Enter".to_string();
            }
            "dismiss" | "dismiss_welcome" => {
                self.cleanup_concierge_on_navigate();
                if self.chat.active_thread_id() == Some("concierge") {
                    self.chat.reduce(chat::ChatAction::NewThread);
                    self.main_pane_view = MainPaneView::Conversation;
                    self.focus = FocusArea::Input;
                }
            }
            "start_goal_run" => {
                self.cleanup_concierge_on_navigate();
                self.chat
                    .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
                self.send_daemon_command(DaemonCommand::RequestThread("concierge".to_string()));
                self.main_pane_view = MainPaneView::Conversation;
                self.focus = FocusArea::Input;
                self.set_input_text("/goal ");
                self.status_line = "Describe your goal and press Enter".to_string();
            }
            "focus_chat" => {
                self.cleanup_concierge_on_navigate();
                self.chat
                    .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
                self.send_daemon_command(DaemonCommand::RequestThread("concierge".to_string()));
                self.main_pane_view = MainPaneView::Conversation;
                self.focus = FocusArea::Input;
            }
            "open_settings" => {
                self.cleanup_concierge_on_navigate();
                self.open_settings_tab(SettingsTab::Auth);
            }
            "operator_profile_skip" => {
                let _ = self.skip_operator_profile_question();
            }
            "operator_profile_defer" => {
                let _ = self.defer_operator_profile_question();
            }
            "operator_profile_retry" => {
                self.send_daemon_command(DaemonCommand::RetryOperatorProfile);
                self.status_line = "Retrying operator profile operation…".to_string();
                self.show_input_notice(
                    "Retrying operator profile operation…",
                    InputNoticeKind::Success,
                    40,
                    true,
                );
            }
            _ => {}
        }
    }

    fn open_chat_action_confirm(&mut self, message_index: usize, action: PendingChatActionKind) {
        self.pending_chat_action_confirm = Some(PendingChatActionConfirm {
            message_index,
            action,
        });
        self.chat_action_confirm_accept_selected = true;
        if self.modal.top() != Some(modal::ModalKind::ChatActionConfirm) {
            self.modal.reduce(modal::ModalAction::Push(
                modal::ModalKind::ChatActionConfirm,
            ));
        }
    }

    fn close_chat_action_confirm(&mut self) {
        self.pending_chat_action_confirm = None;
        self.chat_action_confirm_accept_selected = true;
        if self.modal.top() == Some(modal::ModalKind::ChatActionConfirm) {
            self.close_top_modal();
        }
    }

    fn request_regenerate_message(&mut self, index: usize) {
        self.open_chat_action_confirm(index, PendingChatActionKind::Regenerate);
    }

    fn request_delete_message(&mut self, index: usize) {
        self.open_chat_action_confirm(index, PendingChatActionKind::Delete);
    }

    fn confirm_pending_chat_action(&mut self) {
        let Some(pending) = self.pending_chat_action_confirm.take() else {
            return;
        };
        if self.modal.top() == Some(modal::ModalKind::ChatActionConfirm) {
            self.close_top_modal();
        }
        self.chat_action_confirm_accept_selected = true;
        match pending.action {
            PendingChatActionKind::Regenerate => {
                self.regenerate_from_message(pending.message_index)
            }
            PendingChatActionKind::Delete => self.delete_message(pending.message_index),
        }
    }

    fn execute_selected_inline_message_action(&mut self) -> bool {
        let Some(message_index) = self.chat.selected_message() else {
            return false;
        };
        let Some(message) = self
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.get(message_index))
        else {
            return false;
        };

        let action_index = self.chat.selected_message_action();
        let Some((_, target)) = widgets::chat::message_action_targets(
            &self.chat,
            message_index,
            message,
            self.tick_counter,
        )
        .into_iter()
        .nth(action_index) else {
            return false;
        };

        match target {
            chat::ChatHitTarget::MessageAction {
                message_index,
                action_index,
            } => {
                self.chat.select_message(Some(message_index));
                self.chat.select_message_action(action_index);
                self.execute_concierge_message_action(message_index, action_index);
                true
            }
            chat::ChatHitTarget::CopyMessage(index) => {
                self.chat.select_message(Some(index));
                self.copy_message(index);
                true
            }
            chat::ChatHitTarget::ResendMessage(index) => {
                self.chat.select_message(Some(index));
                self.resend_message(index);
                true
            }
            chat::ChatHitTarget::RegenerateMessage(index) => {
                self.chat.select_message(Some(index));
                self.request_regenerate_message(index);
                true
            }
            chat::ChatHitTarget::DeleteMessage(index) => {
                self.chat.select_message(Some(index));
                self.request_delete_message(index);
                true
            }
            chat::ChatHitTarget::ToolFilePath { message_index } => {
                self.chat.select_message(Some(message_index));
                false
            }
            _ => false,
        }
    }

    fn update_held_modifier(&mut self, code: KeyCode, pressed: bool) {
        let modifier = match code {
            KeyCode::Modifier(
                ModifierKeyCode::LeftShift
                | ModifierKeyCode::RightShift
                | ModifierKeyCode::IsoLevel3Shift
                | ModifierKeyCode::IsoLevel5Shift,
            ) => Some(KeyModifiers::SHIFT),
            KeyCode::Modifier(ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl) => {
                Some(KeyModifiers::CONTROL)
            }
            KeyCode::Modifier(ModifierKeyCode::LeftAlt | ModifierKeyCode::RightAlt) => {
                Some(KeyModifiers::ALT)
            }
            _ => None,
        };

        if let Some(modifier) = modifier {
            if pressed {
                self.held_key_modifiers.insert(modifier);
            } else {
                self.held_key_modifiers.remove(modifier);
            }
        }
    }

    fn input_notice_style(&self) -> Option<(&str, Style)> {
        self.input_notice.as_ref().map(|notice| {
            let style = match notice.kind {
                InputNoticeKind::Warning => Style::default().fg(Color::Indexed(214)),
                InputNoticeKind::Success => Style::default().fg(Color::Indexed(114)),
            };
            (notice.text.as_str(), style)
        })
    }

}
