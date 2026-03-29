//! TuiModel compositor -- delegates to decomposed state modules.
//!
//! This replaces the old monolithic 3,500-line app.rs with a clean
//! compositor that owns the 8 state sub-modules and bridges between
//! the daemon client events and the UI state.

mod commands;
mod config_io;
mod conversion;
mod events;
mod input_ops;
mod keyboard;
mod modal_handlers;
mod mouse;
mod render_helpers;
mod rendering;
mod settings_handlers;

use std::sync::mpsc::Receiver;

use crossterm::event::{
    KeyCode, KeyModifiers, ModifierKeyCode, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use tokio::sync::mpsc::UnboundedSender;

use crate::client::ClientEvent;
use crate::providers;
use crate::state::*;
use crate::theme::ThemeTokens;
use crate::widgets;

/// A file attached to the next outgoing message.
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content: String,
    pub size_bytes: usize,
}

/// A recent autonomous action displayed in the sidebar.
#[derive(Debug, Clone)]
pub struct RecentActionVm {
    pub action_type: String,
    pub summary: String,
    pub timestamp: u64,
}

/// Flat representation of a sidebar item for matching selected index to data.
struct SidebarFlatItem {
    target: Option<sidebar::SidebarItemTarget>,
    title: String,
}

#[derive(Clone, Copy, Debug)]
struct PaneLayout {
    chat: Rect,
    sidebar: Option<Rect>,
    concierge: Rect,
    input: Rect,
}

#[derive(Clone, Debug)]
enum MainPaneView {
    Conversation,
    Task(sidebar::SidebarItemTarget),
    WorkContext,
    GoalComposer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsPickerTarget {
    Provider,
    Model,
    SubAgentProvider,
    SubAgentModel,
    ConciergeProvider,
    ConciergeModel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputNoticeKind {
    Warning,
    Success,
}

#[derive(Clone, Debug)]
struct InputNotice {
    text: String,
    kind: InputNoticeKind,
    expires_at_tick: u64,
    dismiss_on_interaction: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingChatActionKind {
    Regenerate,
    Delete,
}

#[derive(Clone, Debug)]
struct PendingChatActionConfirm {
    message_index: usize,
    action: PendingChatActionKind,
}

#[derive(Clone, Debug)]
struct OperatorProfileQuestionVm {
    session_id: String,
    question_id: String,
    field_key: String,
    prompt: String,
    input_kind: String,
    optional: bool,
}

#[derive(Clone, Debug)]
struct OperatorProfileProgressVm {
    answered: u32,
    remaining: u32,
    completion_ratio: f64,
}

#[derive(Clone, Debug, Default)]
struct OperatorProfileOnboardingState {
    visible: bool,
    loading: bool,
    session_id: Option<String>,
    session_kind: Option<String>,
    question: Option<OperatorProfileQuestionVm>,
    progress: Option<OperatorProfileProgressVm>,
    summary_json: Option<String>,
    warning: Option<String>,
}

pub struct TuiModel {
    // State modules
    chat: chat::ChatState,
    input: input::InputState,
    modal: modal::ModalState,
    sidebar: sidebar::SidebarState,
    tasks: task::TaskState,
    config: config::ConfigState,
    approval: approval::ApprovalState,
    anticipatory: AnticipatoryState,
    pub audit: crate::state::audit::AuditState,
    settings: settings::SettingsState,
    pub plugin_settings: settings::PluginSettingsState,
    pub auth: AuthState,
    pub subagents: SubAgentsState,
    pub concierge: ConciergeState,
    pub tier: TierState,

    // UI chrome
    focus: FocusArea,
    theme: ThemeTokens,
    width: u16,
    height: u16,

    // Infrastructure
    daemon_cmd_tx: UnboundedSender<DaemonCommand>,
    daemon_events_rx: Receiver<ClientEvent>,

    // Status
    connected: bool,
    agent_config_loaded: bool,
    status_line: String,
    default_session_id: Option<String>,
    tick_counter: u64,

    // Agent activity state (from daemon events, not local buffers)
    agent_activity: Option<String>,

    // Error state
    last_error: Option<String>,
    error_active: bool,
    error_tick: u64,

    // Pending ChatGPT subscription login flow
    openai_auth_url: Option<String>,
    openai_auth_status_text: Option<String>,
    settings_picker_target: Option<SettingsPickerTarget>,
    last_attention_surface: Option<String>,

    // Vim motion state
    pending_g: bool,

    // Responsive layout override: when Some, overrides breakpoint-based sidebar visibility
    show_sidebar_override: Option<bool>,
    main_pane_view: MainPaneView,
    task_view_scroll: usize,
    task_show_live_todos: bool,
    task_show_timeline: bool,
    task_show_files: bool,

    // Set by /quit command; checked after modal enter to issue quit
    pending_quit: bool,

    // Double-Esc stream stop state
    pending_stop: bool,
    pending_stop_tick: u64,
    input_notice: Option<InputNotice>,
    pending_chat_action_confirm: Option<PendingChatActionConfirm>,
    chat_action_confirm_accept_selected: bool,
    held_key_modifiers: KeyModifiers,

    // Pending file attachments (prepended to next submitted message)
    attachments: Vec<Attachment>,

    // Queue of prompts submitted while streaming (auto-sent after TurnDone)
    queued_prompts: Vec<String>,

    operator_profile: OperatorProfileOnboardingState,

    // Thread ID whose stream was cancelled via double-Esc (ignore further events)
    cancelled_thread_id: Option<String>,

    // Ignore a stale concierge welcome that arrives after the user navigated away.
    ignore_pending_concierge_welcome: bool,

    // Gateway connection statuses received from daemon
    pub gateway_statuses: Vec<chat::GatewayStatusVm>,

    // Recent autonomous actions from heartbeat digests (shown in sidebar)
    pub recent_actions: Vec<RecentActionVm>,

    // Active mouse drag selection in the chat pane
    chat_drag_anchor: Option<Position>,
    chat_drag_current: Option<Position>,
    chat_drag_anchor_point: Option<widgets::chat::SelectionPoint>,
    chat_drag_current_point: Option<widgets::chat::SelectionPoint>,
    chat_selection_snapshot: Option<widgets::chat::CachedSelectionSnapshot>,

    // Active mouse drag selection in the work-context preview pane
    work_context_drag_anchor: Option<Position>,
    work_context_drag_current: Option<Position>,
    work_context_drag_anchor_point: Option<widgets::chat::SelectionPoint>,
    work_context_drag_current_point: Option<widgets::chat::SelectionPoint>,
}

impl TuiModel {
    pub fn new(
        daemon_events_rx: Receiver<ClientEvent>,
        daemon_cmd_tx: UnboundedSender<DaemonCommand>,
    ) -> Self {
        Self {
            chat: chat::ChatState::new(),
            input: input::InputState::new(),
            modal: modal::ModalState::new(),
            sidebar: sidebar::SidebarState::new(),
            tasks: task::TaskState::new(),
            config: config::ConfigState::new(),
            approval: approval::ApprovalState::new(),
            anticipatory: AnticipatoryState::new(),
            audit: crate::state::audit::AuditState::new(),
            settings: settings::SettingsState::new(),
            plugin_settings: settings::PluginSettingsState::new(),
            auth: AuthState::new(),
            subagents: SubAgentsState::new(),
            concierge: ConciergeState::new(),
            tier: TierState::default(),
            focus: FocusArea::Input,
            theme: ThemeTokens::default(),
            width: 120,
            height: 40,
            daemon_cmd_tx,
            daemon_events_rx,
            connected: false,
            agent_config_loaded: false,
            status_line: "Starting...".to_string(),
            default_session_id: None,
            tick_counter: 0,
            agent_activity: None,
            last_error: None,
            error_active: false,
            error_tick: 0,
            openai_auth_url: None,
            openai_auth_status_text: None,
            settings_picker_target: None,
            last_attention_surface: None,
            pending_g: false,
            show_sidebar_override: None,
            main_pane_view: MainPaneView::Conversation,
            task_view_scroll: 0,
            task_show_live_todos: true,
            task_show_timeline: true,
            task_show_files: true,
            pending_quit: false,
            pending_stop: false,
            pending_stop_tick: 0,
            input_notice: None,
            pending_chat_action_confirm: None,
            chat_action_confirm_accept_selected: true,
            held_key_modifiers: KeyModifiers::NONE,
            attachments: Vec::new(),
            queued_prompts: Vec::new(),
            operator_profile: OperatorProfileOnboardingState::default(),
            cancelled_thread_id: None,
            ignore_pending_concierge_welcome: false,
            gateway_statuses: Vec::new(),
            recent_actions: Vec::new(),
            chat_drag_anchor: None,
            chat_drag_current: None,
            chat_drag_anchor_point: None,
            chat_drag_current_point: None,
            chat_selection_snapshot: None,
            work_context_drag_anchor: None,
            work_context_drag_current: None,
            work_context_drag_anchor_point: None,
            work_context_drag_current_point: None,
        }
    }

    fn send_daemon_command(&self, command: DaemonCommand) {
        let _ = self.daemon_cmd_tx.send(command);
    }

    fn show_input_notice(
        &mut self,
        text: impl Into<String>,
        kind: InputNoticeKind,
        duration_ticks: u64,
        dismiss_on_interaction: bool,
    ) {
        self.input_notice = Some(InputNotice {
            text: text.into(),
            kind,
            expires_at_tick: self.tick_counter.saturating_add(duration_ticks),
            dismiss_on_interaction,
        });
    }

    fn clear_dismissable_input_notice(&mut self) {
        if self
            .input_notice
            .as_ref()
            .is_some_and(|notice| notice.dismiss_on_interaction)
        {
            self.input_notice = None;
        }
    }

    fn clear_pending_stop(&mut self) {
        self.pending_stop = false;
        self.clear_dismissable_input_notice();
    }

    fn pending_stop_active(&self) -> bool {
        self.pending_stop && self.tick_counter.saturating_sub(self.pending_stop_tick) < 100
    }

    fn assistant_busy(&self) -> bool {
        self.chat.is_streaming() || self.agent_activity.is_some()
    }

    fn actions_bar_visible(&self) -> bool {
        if self.should_show_local_landing() {
            return false;
        }
        if self.should_show_operator_profile_onboarding() {
            return false;
        }

        self.concierge.loading || !self.chat.active_actions().is_empty()
    }

    fn concierge_banner_visible(&self) -> bool {
        self.actions_bar_visible()
    }

    fn should_show_local_landing(&self) -> bool {
        matches!(self.main_pane_view, MainPaneView::Conversation)
            && self.chat.active_thread().is_none()
            && !self.chat.is_streaming()
            && !self.concierge.loading
            && !self.should_show_operator_profile_onboarding()
            && !self.should_show_provider_onboarding()
    }

    fn should_show_concierge_hero_loading(&self) -> bool {
        self.concierge.loading
            && matches!(self.main_pane_view, MainPaneView::Conversation)
            && self.chat.active_thread().is_none()
            && self.chat.streaming_content().is_empty()
            && !self.should_show_operator_profile_onboarding()
            && !self.concierge.has_active_welcome()
    }

    fn concierge_banner_height(&self) -> u16 {
        if self.should_show_concierge_hero_loading() {
            0
        } else if self.actions_bar_visible() {
            1
        } else {
            0
        }
    }

    fn anticipatory_banner_height(&self) -> u16 {
        if self.anticipatory.has_items() {
            8
        } else {
            0
        }
    }

    fn pane_layout_for_area(&self, area: Rect) -> PaneLayout {
        let input_height = self.input_height().min(area.height.saturating_sub(1));
        let remaining_after_input = area.height.saturating_sub(input_height + 1);
        let anticipatory_height = self
            .anticipatory_banner_height()
            .min(remaining_after_input.saturating_sub(1));
        let remaining_after_anticipatory =
            remaining_after_input.saturating_sub(anticipatory_height);
        let concierge_height = self
            .concierge_banner_height()
            .min(remaining_after_anticipatory.saturating_sub(1));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(anticipatory_height),
                Constraint::Length(concierge_height),
                Constraint::Length(input_height),
                Constraint::Length(1),
            ])
            .split(area);

        let body = chunks[1];
        let chat = if self.sidebar_visible() {
            let sidebar_pct = if self.width >= 120 { 33 } else { 28 };
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(100 - sidebar_pct),
                    Constraint::Percentage(sidebar_pct),
                ])
                .split(body)[0]
        } else {
            body
        };
        let sidebar = if self.sidebar_visible() {
            let sidebar_pct = if self.width >= 120 { 33 } else { 28 };
            Some(
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(100 - sidebar_pct),
                        Constraint::Percentage(sidebar_pct),
                    ])
                    .split(body)[1],
            )
        } else {
            None
        };

        PaneLayout {
            chat,
            sidebar,
            concierge: chunks[3],
            input: chunks[4],
        }
    }

    fn pane_layout(&self) -> PaneLayout {
        self.pane_layout_for_area(Rect::new(0, 0, self.width, self.height))
    }

    fn has_configured_provider(&self) -> bool {
        self.auth.entries.iter().any(|entry| entry.authenticated)
    }

    fn should_show_provider_onboarding(&self) -> bool {
        self.connected
            && self.auth.loaded
            && !self.has_configured_provider()
            && matches!(self.main_pane_view, MainPaneView::Conversation)
            && self.chat.active_thread().is_none()
            && self.chat.streaming_content().is_empty()
            && !self.should_show_operator_profile_onboarding()
    }

    fn should_show_operator_profile_onboarding(&self) -> bool {
        self.operator_profile.visible
            && matches!(self.main_pane_view, MainPaneView::Conversation)
            && self.chat.streaming_content().is_empty()
    }

    fn operator_profile_select_options(field_key: &str) -> Option<&'static [&'static str]> {
        match field_key {
            "notification_preference" => Some(&["minimal", "balanced", "proactive"]),
            _ => None,
        }
    }

    fn current_operator_profile_select_options(&self) -> Option<&'static [&'static str]> {
        self.operator_profile
            .question
            .as_ref()
            .and_then(|question| Self::operator_profile_select_options(&question.field_key))
    }

    fn submit_operator_profile_answer(&mut self) -> bool {
        let Some(question) = self.operator_profile.question.clone() else {
            return false;
        };
        let answer = self.input.buffer().trim();
        if answer.is_empty() && !question.optional {
            self.show_input_notice(
                "Answer required (Ctrl+S to skip, Ctrl+D to defer)",
                InputNoticeKind::Warning,
                80,
                true,
            );
            return true;
        }

        let answer_json = if answer.is_empty() && question.optional {
            "null".to_string()
        } else {
            match question.input_kind.as_str() {
                "bool" => match answer.to_ascii_lowercase().as_str() {
                    "true" | "t" | "yes" | "y" | "1" => "true".to_string(),
                    "false" | "f" | "no" | "n" | "0" => "false".to_string(),
                    _ => {
                        self.show_input_notice(
                            "Use true/false (or yes/no) for this question",
                            InputNoticeKind::Warning,
                            80,
                            true,
                        );
                        return true;
                    }
                },
                "select" => {
                    let normalized = answer.to_ascii_lowercase();
                    if let Some(options) = Self::operator_profile_select_options(&question.field_key) {
                        if !options.iter().any(|option| *option == normalized) {
                            self.show_input_notice(
                                format!(
                                    "Pick one: {}",
                                    options
                                        .iter()
                                        .map(|option| option.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ),
                                InputNoticeKind::Warning,
                                100,
                                true,
                            );
                            return true;
                        }
                    }
                    match serde_json::to_string(&normalized) {
                        Ok(json) => json,
                        Err(_) => return false,
                    }
                }
                _ => match serde_json::to_string(answer) {
                    Ok(json) => json,
                    Err(_) => return false,
                },
            }
        };

        self.operator_profile.loading = true;
        self.operator_profile.question = None;
        self.operator_profile.warning = None;
        self.send_daemon_command(DaemonCommand::SubmitOperatorProfileAnswer {
            session_id: question.session_id,
            question_id: question.question_id,
            answer_json,
        });
        self.input.reduce(input::InputAction::Clear);
        self.status_line = "Submitting operator profile answer…".to_string();
        true
    }

    fn skip_operator_profile_question(&mut self) -> bool {
        let Some(question) = self.operator_profile.question.clone() else {
            return false;
        };
        self.operator_profile.loading = true;
        self.operator_profile.question = None;
        self.operator_profile.warning = None;
        self.send_daemon_command(DaemonCommand::SkipOperatorProfileQuestion {
            session_id: question.session_id,
            question_id: question.question_id,
            reason: Some("tui_skip_shortcut".to_string()),
        });
        self.input.reduce(input::InputAction::Clear);
        self.status_line = "Skipping operator profile question…".to_string();
        true
    }

    fn defer_operator_profile_question(&mut self) -> bool {
        let Some(question) = self.operator_profile.question.clone() else {
            return false;
        };
        let defer_until_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64 + 24 * 60 * 60 * 1_000);
        self.operator_profile.loading = true;
        self.operator_profile.question = None;
        self.operator_profile.warning = None;
        self.send_daemon_command(DaemonCommand::DeferOperatorProfileQuestion {
            session_id: question.session_id,
            question_id: question.question_id,
            defer_until_unix_ms,
        });
        self.input.reduce(input::InputAction::Clear);
        self.status_line = "Deferring operator profile question for 24h…".to_string();
        true
    }

    fn open_settings_tab(&mut self, tab: SettingsTab) {
        if self.modal.top() != Some(modal::ModalKind::Settings) {
            self.modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        }
        self.settings.reduce(SettingsAction::SwitchTab(tab));
        self.send_daemon_command(DaemonCommand::GetProviderAuthStates);
        self.send_daemon_command(DaemonCommand::ListSubAgents);
        self.send_daemon_command(DaemonCommand::GetConciergeConfig);
        if matches!(tab, SettingsTab::Gateway) {
            self.send_daemon_command(DaemonCommand::WhatsAppLinkStatus);
        }
    }

    fn open_provider_setup(&mut self) {
        self.open_settings_tab(SettingsTab::Provider);
        self.status_line = "Configure provider credentials to start chatting".to_string();
    }

    fn set_input_text(&mut self, text: &str) {
        self.input.reduce(input::InputAction::Clear);
        for ch in text.chars() {
            self.input.reduce(input::InputAction::InsertChar(ch));
        }
        self.input.set_mode(input::InputMode::Insert);
    }

    fn close_top_modal(&mut self) {
        if self.modal.top() == Some(modal::ModalKind::WhatsAppLink) {
            self.send_daemon_command(DaemonCommand::WhatsAppLinkStop);
            self.send_daemon_command(DaemonCommand::WhatsAppLinkUnsubscribe);
            self.modal.reset_whatsapp_link();
        }
        self.modal.reduce(modal::ModalAction::Pop);
    }

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
            MainPaneView::WorkContext | MainPaneView::GoalComposer => {
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

    fn sidebar_visible(&self) -> bool {
        if !matches!(
            self.main_pane_view,
            MainPaneView::Conversation | MainPaneView::WorkContext
        ) {
            return false;
        }
        let Some(thread_id) = self.chat.active_thread_id() else {
            return false;
        };
        !self.tasks.todos_for_thread(thread_id).is_empty()
            || self
                .tasks
                .work_context_for_thread(thread_id)
                .is_some_and(|context| !context.entries.is_empty())
    }

    fn current_attention_target(&self) -> (String, Option<String>, Option<String>) {
        if let Some(modal) = self.modal.top() {
            let surface = match modal {
                modal::ModalKind::Settings => {
                    format!(
                        "modal:settings:{}",
                        settings_tab_label(self.settings.active_tab())
                    )
                }
                modal::ModalKind::ApprovalOverlay => "modal:approval".to_string(),
                modal::ModalKind::ChatActionConfirm => "modal:chat_action_confirm".to_string(),
                modal::ModalKind::CommandPalette => "modal:command_palette".to_string(),
                modal::ModalKind::ThreadPicker => "modal:thread_picker".to_string(),
                modal::ModalKind::GoalPicker => "modal:goal_picker".to_string(),
                modal::ModalKind::ProviderPicker => "modal:provider_picker".to_string(),
                modal::ModalKind::ModelPicker => "modal:model_picker".to_string(),
                modal::ModalKind::OpenAIAuth => "modal:openai_auth".to_string(),
                modal::ModalKind::ErrorViewer => "modal:error_viewer".to_string(),
                modal::ModalKind::EffortPicker => "modal:effort_picker".to_string(),
                modal::ModalKind::ToolsPicker => "modal:tools_picker".to_string(),
                modal::ModalKind::ViewPicker => "modal:view_picker".to_string(),
                modal::ModalKind::Help => "modal:help".to_string(),
                modal::ModalKind::WhatsAppLink => "modal:whatsapp_link".to_string(),
            };
            return (
                surface,
                self.chat.active_thread_id().map(str::to_string),
                None,
            );
        }

        match &self.main_pane_view {
            MainPaneView::Conversation => match self.focus {
                FocusArea::Chat => (
                    "conversation:chat".to_string(),
                    self.chat.active_thread_id().map(str::to_string),
                    None,
                ),
                FocusArea::Input => {
                    if self.should_show_provider_onboarding() {
                        ("conversation:onboarding".to_string(), None, None)
                    } else {
                        (
                            "conversation:input".to_string(),
                            self.chat.active_thread_id().map(str::to_string),
                            None,
                        )
                    }
                }
                FocusArea::Sidebar => (
                    format!(
                        "conversation:sidebar:{}",
                        sidebar_tab_label(self.sidebar.active_tab())
                    ),
                    self.chat.active_thread_id().map(str::to_string),
                    None,
                ),
            },
            MainPaneView::Task(target) => (
                "task:detail".to_string(),
                self.target_thread_id(target),
                target_goal_run_id(self, target),
            ),
            MainPaneView::WorkContext => (
                "task:work_context".to_string(),
                self.chat.active_thread_id().map(str::to_string),
                None,
            ),
            MainPaneView::GoalComposer => (
                "task:goal_composer".to_string(),
                self.chat.active_thread_id().map(str::to_string),
                None,
            ),
        }
    }

    fn publish_attention_surface_if_changed(&mut self) {
        if !self.connected {
            return;
        }
        let (surface, thread_id, goal_run_id) = self.current_attention_target();
        let signature = format!(
            "{}|{}|{}",
            surface,
            thread_id.as_deref().unwrap_or_default(),
            goal_run_id.as_deref().unwrap_or_default()
        );
        if self.last_attention_surface.as_deref() == Some(signature.as_str()) {
            return;
        }
        self.last_attention_surface = Some(signature);
        self.send_daemon_command(DaemonCommand::RecordAttention {
            surface,
            thread_id,
            goal_run_id,
        });
    }
}

fn settings_tab_label(tab: SettingsTab) -> &'static str {
    match tab {
        SettingsTab::Provider => "provider",
        SettingsTab::Tools => "tools",
        SettingsTab::WebSearch => "web_search",
        SettingsTab::Chat => "chat",
        SettingsTab::Gateway => "gateway",
        SettingsTab::Auth => "auth",
        SettingsTab::Agent => "agent",
        SettingsTab::SubAgents => "subagents",
        SettingsTab::Concierge => "concierge",
        SettingsTab::Features => "features",
        SettingsTab::Advanced => "advanced",
        SettingsTab::Plugins => "plugins",
    }
}

fn sidebar_tab_label(tab: SidebarTab) -> &'static str {
    match tab {
        SidebarTab::Files => "files",
        SidebarTab::Todos => "todos",
    }
}

fn target_goal_run_id(model: &TuiModel, target: &SidebarItemTarget) -> Option<String> {
    match target {
        SidebarItemTarget::GoalRun { goal_run_id, .. } => Some(goal_run_id.clone()),
        SidebarItemTarget::Task { task_id } => model
            .tasks
            .task_by_id(task_id)
            .and_then(|task| task.goal_run_id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::conversion;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::sync::mpsc;
    use tokio::sync::mpsc::unbounded_channel;

    fn build_model() -> TuiModel {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, _cmd_rx) = unbounded_channel();
        TuiModel::new(daemon_rx, cmd_tx)
    }

    fn unauthenticated_entry() -> ProviderAuthEntry {
        ProviderAuthEntry {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: false,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        }
    }

    fn rendered_chat_area(model: &TuiModel) -> Rect {
        let area = Rect::new(0, 0, model.width, model.height);
        let input_height = model.input_height();
        let anticipatory_height = model.anticipatory_banner_height();
        let concierge_height = model.concierge_banner_height();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(anticipatory_height),
                Constraint::Length(concierge_height),
                Constraint::Length(input_height),
                Constraint::Length(1),
            ])
            .split(area);

        if model.sidebar_visible() {
            let sidebar_pct = if model.width >= 120 { 33 } else { 28 };
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(100 - sidebar_pct),
                    Constraint::Percentage(sidebar_pct),
                ])
                .split(chunks[1])[0]
        } else {
            chunks[1]
        }
    }

    #[test]
    fn provider_onboarding_requires_loaded_auth_state() {
        let mut model = build_model();
        model.connected = true;
        model.auth.entries = vec![unauthenticated_entry()];

        assert!(!model.should_show_provider_onboarding());
    }

    #[test]
    fn provider_onboarding_shows_when_no_provider_is_configured() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        model.auth.entries = vec![unauthenticated_entry()];

        assert!(model.should_show_provider_onboarding());
    }

    #[test]
    fn provider_onboarding_hides_when_provider_is_configured() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        let mut entry = unauthenticated_entry();
        entry.authenticated = true;
        model.auth.entries = vec![entry];

        assert!(!model.should_show_provider_onboarding());
    }

    #[test]
    fn local_landing_shows_only_for_empty_conversation_state() {
        let mut model = build_model();

        assert!(model.should_show_local_landing());

        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        assert!(!model.should_show_local_landing());

        model
            .chat
            .reduce(chat::ChatAction::SelectThread(String::new()));
        model.chat.reduce(chat::ChatAction::Delta {
            thread_id: "stream".to_string(),
            content: "hello".to_string(),
        });
        assert!(!model.should_show_local_landing());

        model.chat.reduce(chat::ChatAction::ResetStreaming);
        model
            .chat
            .reduce(chat::ChatAction::ThreadListReceived(Vec::new()));
        model.connected = true;
        model.auth.loaded = true;
        model.auth.entries = vec![unauthenticated_entry()];
        assert!(!model.should_show_local_landing());
    }

    #[test]
    fn local_landing_yields_to_concierge_loading() {
        let mut model = build_model();
        model.concierge.loading = true;

        assert!(model.should_show_concierge_hero_loading());
        assert!(
            !model.should_show_local_landing(),
            "local landing should not hide concierge loading animation"
        );
    }

    #[test]
    fn local_landing_full_render_does_not_panic_at_width_100() {
        let mut model = build_model();
        model.width = 100;
        model.height = 40;
        model.focus = FocusArea::Input;

        let backend = TestBackend::new(model.width, model.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("local landing render should not panic at width 100");
    }

    #[test]
    fn local_landing_full_render_does_not_panic_at_width_80() {
        let mut model = build_model();
        model.width = 80;
        model.height = 24;
        model.focus = FocusArea::Input;

        let backend = TestBackend::new(model.width, model.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("local landing render should not panic at width 80");
    }

    #[test]
    fn render_uses_frame_area_even_when_model_size_is_stale() {
        let mut model = build_model();
        model.width = 120;
        model.height = 40;
        model.focus = FocusArea::Input;

        let backend = TestBackend::new(100, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("render should honor the live frame size, not stale model dimensions");
    }

    #[test]
    fn concierge_loading_uses_frame_area_even_when_model_size_is_stale() {
        let mut model = build_model();
        model.width = 120;
        model.height = 40;
        model.concierge.loading = true;
        model.focus = FocusArea::Chat;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("concierge loading should render within the live frame size");
    }

    #[test]
    fn render_syncs_model_dimensions_to_live_frame_area() {
        let mut model = build_model();
        model.width = 120;
        model.height = 40;

        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("render should succeed against the live frame area");

        assert_eq!(model.width, 100);
        assert_eq!(model.height, 24);
    }

    #[test]
    fn copy_message_formats_reasoning_and_content_with_separator() {
        let mut model = build_model();
        conversion::reset_last_copied_text();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                reasoning: Some("Private chain".to_string()),
                content: "Public answer".to_string(),
                ..Default::default()
            },
        });

        model.copy_message(0);

        assert_eq!(
            conversion::last_copied_text().as_deref(),
            Some("Reasoning:\nPrivate chain\n\n-------\n\nContent:\nPublic answer")
        );
    }

    #[test]
    fn copy_message_shows_copied_label_until_timeout() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Public answer".to_string(),
                ..Default::default()
            },
        });
        model.chat.select_message(Some(0));

        model.copy_message(0);

        let copied_actions = widgets::chat::message_action_targets(
            &model.chat,
            0,
            model
                .chat
                .active_thread()
                .and_then(|thread| thread.messages.first())
                .expect("message should exist"),
            model.tick_counter,
        );
        assert_eq!(copied_actions[0].0, "[Copied]");

        for _ in 0..100 {
            model.on_tick();
        }

        let reverted_actions = widgets::chat::message_action_targets(
            &model.chat,
            0,
            model
                .chat
                .active_thread()
                .and_then(|thread| thread.messages.first())
                .expect("message should exist"),
            model.tick_counter,
        );
        assert_eq!(reverted_actions[0].0, "[Copy]");
    }

    #[test]
    fn regenerate_message_requires_confirmation_before_sending() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.connected = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "Original prompt".to_string(),
                ..Default::default()
            },
        });
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_regenerate_message(1);

        assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
        assert!(
            cmd_rx.try_recv().is_err(),
            "regenerate should wait for confirmation"
        );

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::ChatActionConfirm,
        );
        assert!(!quit);

        let mut saw_send = false;
        while let Ok(command) = cmd_rx.try_recv() {
            if matches!(command, DaemonCommand::SendMessage { .. }) {
                saw_send = true;
                break;
            }
        }
        assert!(
            saw_send,
            "confirmation should eventually send the regenerated prompt"
        );
    }

    #[test]
    fn delete_message_requires_confirmation_before_removing_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.width = 100;
        model.height = 40;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("m1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_delete_message(0);

        assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(1),
            "message should remain until deletion is confirmed"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "delete should wait for confirmation"
        );

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::ChatActionConfirm,
        );
        assert!(!quit);

        let sent = cmd_rx
            .try_recv()
            .expect("confirmation should send delete command");
        assert!(matches!(sent, DaemonCommand::DeleteMessages { .. }));
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(0),
            "message should be removed after deletion is confirmed"
        );
    }

    #[test]
    fn clicking_cancel_in_chat_action_confirm_does_not_delete_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.width = 100;
        model.height = 40;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("m1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_delete_message(0);
        let (_, overlay_area) = model
            .current_modal_area()
            .expect("chat action confirm modal should be visible");
        let (_, cancel_rect) = render_helpers::chat_action_confirm_button_bounds(overlay_area)
            .expect("confirm modal should expose button bounds");
        let click_col = cancel_rect.x.saturating_add(1);
        let click_row = cancel_rect.y;

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            model.modal.top(),
            None,
            "cancel click should close the modal"
        );
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(1),
            "cancel click must not delete the message"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "cancel click must not send a delete command"
        );
    }

    #[test]
    fn clicking_confirm_in_chat_action_confirm_deletes_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.width = 100;
        model.height = 40;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("m1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_delete_message(0);
        let (_, overlay_area) = model
            .current_modal_area()
            .expect("chat action confirm modal should be visible");
        let (confirm_rect, _) = render_helpers::chat_action_confirm_button_bounds(overlay_area)
            .expect("confirm modal should expose button bounds");
        let click_col = confirm_rect.x.saturating_add(1);
        let click_row = confirm_rect.y;

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });

        let sent = cmd_rx
            .try_recv()
            .expect("confirm click should send a delete command");
        assert!(matches!(sent, DaemonCommand::DeleteMessages { .. }));
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(0),
            "confirm click should delete the message"
        );
    }

    #[test]
    fn resize_clears_drag_snapshots() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "drag me".to_string(),
                ..Default::default()
            },
        });
        model.chat_drag_anchor = Some(Position::new(3, 6));
        model.chat_drag_current = Some(Position::new(8, 9));
        model.chat_drag_anchor_point = Some(widgets::chat::SelectionPoint { row: 1, col: 1 });
        model.chat_drag_current_point = Some(widgets::chat::SelectionPoint { row: 2, col: 4 });
        model.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
            Rect::new(0, 3, 80, 12),
            &model.chat,
            &model.theme,
            model.tick_counter,
        );
        model.work_context_drag_anchor = Some(Position::new(1, 1));
        model.work_context_drag_current = Some(Position::new(2, 2));
        model.work_context_drag_anchor_point =
            Some(widgets::chat::SelectionPoint { row: 0, col: 0 });
        model.work_context_drag_current_point =
            Some(widgets::chat::SelectionPoint { row: 0, col: 1 });

        model.handle_resize(100, 24);

        assert!(model.chat_drag_anchor.is_none());
        assert!(model.chat_drag_current.is_none());
        assert!(model.chat_drag_anchor_point.is_none());
        assert!(model.chat_drag_current_point.is_none());
        assert!(model.chat_selection_snapshot.is_none());
        assert!(model.work_context_drag_anchor.is_none());
        assert!(model.work_context_drag_current.is_none());
        assert!(model.work_context_drag_anchor_point.is_none());
        assert!(model.work_context_drag_current_point.is_none());
    }

    #[test]
    fn cleanup_concierge_on_navigate_hides_local_welcome_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                is_concierge_welcome: true,
                ..Default::default()
            },
        });
        model
            .concierge
            .reduce(crate::state::ConciergeAction::WelcomeReceived {
                content: "Welcome".to_string(),
                actions: vec![crate::state::ConciergeActionVm {
                    label: "Dismiss".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                }],
            });

        model.cleanup_concierge_on_navigate();

        assert!(!model.concierge.welcome_visible);
        assert!(model.ignore_pending_concierge_welcome);
        assert!(
            model.chat.active_actions().is_empty(),
            "dismissed concierge welcome should not leave actionable buttons behind"
        );
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command, got {:?}", other),
        }
    }

    #[test]
    fn submit_prompt_dismisses_concierge_and_avoids_session_binding() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.connected = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                actions: vec![chat::MessageAction {
                    label: "Dismiss".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                }],
                is_concierge_welcome: true,
                ..Default::default()
            },
        });
        model
            .concierge
            .reduce(crate::state::ConciergeAction::WelcomeReceived {
                content: "Welcome".to_string(),
                actions: vec![crate::state::ConciergeActionVm {
                    label: "Dismiss".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                }],
            });
        model.default_session_id = Some("stale-session".to_string());

        model.submit_prompt("hello".to_string());

        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command first, got {:?}", other),
        }
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::SendMessage {
                thread_id,
                content,
                session_id,
            }) => {
                assert_eq!(thread_id.as_deref(), Some("concierge"));
                assert_eq!(content, "hello");
                assert_eq!(session_id, None);
            }
            other => panic!("expected send-message command, got {:?}", other),
        }
        assert!(
            model.chat.active_actions().is_empty(),
            "submitting a prompt should hide concierge welcome actions"
        );
    }

    #[test]
    fn start_goal_run_dismisses_concierge_and_avoids_session_binding() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.connected = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                actions: vec![chat::MessageAction {
                    label: "Goal".to_string(),
                    action_type: "start_goal_run".to_string(),
                    thread_id: None,
                }],
                is_concierge_welcome: true,
                ..Default::default()
            },
        });
        model
            .concierge
            .reduce(crate::state::ConciergeAction::WelcomeReceived {
                content: "Welcome".to_string(),
                actions: vec![crate::state::ConciergeActionVm {
                    label: "Goal".to_string(),
                    action_type: "start_goal_run".to_string(),
                    thread_id: None,
                }],
            });
        model.default_session_id = Some("stale-session".to_string());

        model.start_goal_run_from_prompt("ship it".to_string());

        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command first, got {:?}", other),
        }
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::StartGoalRun {
                goal,
                thread_id,
                session_id,
            }) => {
                assert_eq!(goal, "ship it");
                assert_eq!(thread_id, None);
                assert_eq!(session_id, None);
            }
            other => panic!("expected start-goal-run command, got {:?}", other),
        }
    }

    #[test]
    fn start_new_thread_shows_local_landing_and_does_not_request_concierge() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.concierge.loading = false;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model
            .concierge
            .reduce(crate::state::ConciergeAction::WelcomeReceived {
                content: "Welcome".to_string(),
                actions: vec![crate::state::ConciergeActionVm {
                    label: "Search".to_string(),
                    action_type: "search".to_string(),
                    thread_id: None,
                }],
            });

        model.start_new_thread_view();

        assert!(model.should_show_local_landing());
        assert_eq!(model.chat.active_thread_id(), None);
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command first, got {:?}", other),
        }
        assert!(
            cmd_rx.try_recv().is_err(),
            "unexpected daemon command after /new"
        );
    }

    #[test]
    fn concierge_arrow_keys_navigate_visible_actions() {
        let mut model = build_model();
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                actions: vec![
                    chat::MessageAction {
                        label: "One".to_string(),
                        action_type: "dismiss".to_string(),
                        thread_id: None,
                    },
                    chat::MessageAction {
                        label: "Two".to_string(),
                        action_type: "dismiss".to_string(),
                        thread_id: None,
                    },
                ],
                is_concierge_welcome: true,
                ..Default::default()
            },
        });

        let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);

        assert!(!handled);
        assert_eq!(model.concierge.selected_action, 1);
    }

    #[test]
    fn selected_message_arrow_keys_navigate_inline_actions() {
        let mut model = build_model();
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "first".to_string(),
                ..Default::default()
            },
        });
        model.chat.select_message(Some(0));
        model.chat.select_message_action(0);

        let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);

        assert!(!handled);
        assert_eq!(model.chat.selected_message_action(), 1);
    }

    #[test]
    fn clicking_selected_message_copy_action_copies_that_message() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "first".to_string(),
                ..Default::default()
            },
        });
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "second".to_string(),
                ..Default::default()
            },
        });
        model.chat.select_message(Some(1));

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let copy_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .find_map(|row| {
                (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                    let pos = Position::new(column, row);
                    if widgets::chat::hit_test(
                        chat_area,
                        &model.chat,
                        &model.theme,
                        model.tick_counter,
                        pos,
                    ) == Some(chat::ChatHitTarget::CopyMessage(1))
                    {
                        Some(pos)
                    } else {
                        None
                    }
                })
            })
            .expect("selected message should expose a clickable copy action");

        super::conversion::reset_last_copied_text();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: copy_pos.x,
            row: copy_pos.y,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: copy_pos.x,
            row: copy_pos.y,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some("second")
        );
    }

    #[test]
    fn pressing_enter_executes_selected_inline_message_action() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "second".to_string(),
                ..Default::default()
            },
        });
        model.chat.select_message(Some(0));
        model.chat.select_message_action(0);
        super::conversion::reset_last_copied_text();

        let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        assert!(!handled);
        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some("second")
        );
    }

    #[test]
    fn click_without_drag_uses_press_location_for_message_selection() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "first".to_string(),
                ..Default::default()
            },
        });
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "second".to_string(),
                ..Default::default()
            },
        });

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let message1_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .find_map(|row| {
                (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                    let pos = Position::new(column, row);
                    if widgets::chat::hit_test(
                        chat_area,
                        &model.chat,
                        &model.theme,
                        model.tick_counter,
                        pos,
                    ) == Some(chat::ChatHitTarget::Message(1))
                    {
                        Some(pos)
                    } else {
                        None
                    }
                })
            })
            .expect("second message should expose a clickable body position");
        let message0_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .find_map(|row| {
                (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                    let pos = Position::new(column, row);
                    if widgets::chat::hit_test(
                        chat_area,
                        &model.chat,
                        &model.theme,
                        model.tick_counter,
                        pos,
                    ) == Some(chat::ChatHitTarget::Message(0))
                    {
                        Some(pos)
                    } else {
                        None
                    }
                })
            })
            .expect("first message should expose a clickable body position");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: message1_pos.x,
            row: message1_pos.y,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: message0_pos.x,
            row: message0_pos.y,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(model.chat.selected_message(), Some(1));
    }

    #[test]
    fn concierge_mouse_click_executes_visible_action() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                actions: vec![
                    chat::MessageAction {
                        label: "One".to_string(),
                        action_type: "dismiss".to_string(),
                        thread_id: None,
                    },
                    chat::MessageAction {
                        label: "Two".to_string(),
                        action_type: "start_goal_run".to_string(),
                        thread_id: None,
                    },
                ],
                is_concierge_welcome: true,
                ..Default::default()
            },
        });

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 8,
            row: 35,
            modifiers: KeyModifiers::NONE,
        });

        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command first, got {:?}", other),
        }
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::RequestThread(thread_id)) => {
                assert_eq!(thread_id, "concierge");
            }
            other => panic!("expected thread request command, got {:?}", other),
        }
        assert_eq!(model.focus, FocusArea::Input);
        assert_eq!(model.input.buffer(), "/goal ");
        assert!(model.chat.active_actions().is_empty());
    }

    #[test]
    fn dismissing_concierge_welcome_returns_to_local_landing() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "concierge".to_string(),
            title: "Concierge".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "concierge".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Welcome".to_string(),
                actions: vec![chat::MessageAction {
                    label: "Dismiss".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                }],
                is_concierge_welcome: true,
                ..Default::default()
            },
        });
        model
            .concierge
            .reduce(crate::state::ConciergeAction::WelcomeReceived {
                content: "Welcome".to_string(),
                actions: vec![crate::state::ConciergeActionVm {
                    label: "Dismiss".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                }],
            });

        model.run_concierge_action(crate::state::ConciergeActionVm {
            label: "Dismiss".to_string(),
            action_type: "dismiss".to_string(),
            thread_id: None,
        });

        assert_eq!(model.chat.active_thread_id(), None);
        assert!(model.should_show_local_landing());
        assert_eq!(model.focus, FocusArea::Input);
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => {}
            other => panic!("expected dismiss command, got {:?}", other),
        }
        assert!(cmd_rx.try_recv().is_err(), "unexpected follow-up command");
    }

    #[test]
    fn drag_selection_keeps_original_anchor_point_when_chat_scrolls() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
        let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
            .chain(chat_area.y..preferred_row)
            .find(|row| {
                widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *row),
                )
                .is_some()
            })
            .expect("chat transcript should expose at least one selectable row");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
        let anchor_point = model
            .chat_drag_anchor_point
            .expect("mouse down should capture a document anchor point");

        for _ in 0..4 {
            model.handle_mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 3,
                row: start_row,
                modifiers: KeyModifiers::NONE,
            });
        }

        let current_point = model
            .chat_drag_current_point
            .expect("dragging should keep updating the current document point");
        assert_eq!(
            model.chat_drag_anchor_point,
            Some(anchor_point),
            "autoscroll should not rewrite the original selection anchor"
        );
        assert!(
            current_point.row < anchor_point.row,
            "dragging upward with autoscroll should extend the selection into older transcript rows: anchor={anchor_point:?} current={current_point:?}"
        );
    }

    #[test]
    fn drag_selection_does_not_rebuild_full_transcript_for_every_mouse_event() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "alpha beta gamma".to_string(),
                ..Default::default()
            },
        });

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .find(|candidate| {
                widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *candidate),
                )
                .is_some()
            })
            .expect("chat transcript should expose a selectable row");

        widgets::chat::reset_build_rendered_lines_call_count();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 12,
            row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 12,
            row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            widgets::chat::build_rendered_lines_call_count(),
            1,
            "dragging a static selection should reuse one transcript snapshot"
        );
    }

    #[test]
    fn render_during_active_drag_reuses_cached_snapshot_and_shows_highlight() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
        model.chat.reduce(chat::ChatAction::ScrollChat(8));

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .find(|candidate| {
                widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *candidate),
                )
                .is_some()
            })
            .expect("chat transcript should expose at least one selectable row");

        widgets::chat::reset_build_rendered_lines_call_count();
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 12,
            row,
            modifiers: KeyModifiers::NONE,
        });

        let backend = TestBackend::new(model.width, model.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("model render should succeed");

        assert_eq!(
            widgets::chat::build_rendered_lines_call_count(),
            1,
            "active drag rendering should reuse the cached transcript snapshot"
        );

        let buffer = terminal.backend().buffer();
        let highlighted = (0..model.height)
            .flat_map(|y| (0..model.width).filter_map(move |x| buffer.cell((x, y))))
            .filter(|cell| cell.bg == Color::Indexed(31))
            .count();
        assert!(
            highlighted > 0,
            "active drag should paint a visible selection highlight even while scrolled"
        );
    }

    #[test]
    fn stale_cached_snapshot_is_ignored_after_sidebar_layout_change() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.focus = FocusArea::Chat;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "hello world".to_string(),
                ..Default::default()
            },
        });

        let full_width_area = Rect::new(
            0,
            3,
            model.width,
            model.height.saturating_sub(model.input_height() + 4),
        );
        model.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
            full_width_area,
            &model.chat,
            &model.theme,
            model.tick_counter,
        );
        model.chat_drag_anchor = None;
        model.chat_drag_current = None;
        model.chat_drag_anchor_point = None;
        model.chat_drag_current_point = None;

        model.tasks.reduce(task::TaskAction::WorkContextReceived(
            task::ThreadWorkContext {
                thread_id: "thread-1".to_string(),
                entries: vec![task::WorkContextEntry {
                    path: "/tmp/demo.txt".to_string(),
                    is_text: true,
                    ..Default::default()
                }],
            },
        ));
        model.show_sidebar_override = Some(true);

        widgets::chat::reset_build_rendered_lines_call_count();
        let backend = TestBackend::new(model.width, model.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("render should fall back to fresh layout instead of using stale snapshot");

        assert_eq!(
            widgets::chat::build_rendered_lines_call_count(),
            1,
            "layout changes should ignore stale cached snapshots and rebuild visible chat rows"
        );
    }

    #[test]
    fn mouse_drag_snapshot_uses_rendered_chat_area_without_sidebar() {
        let mut model = build_model();
        model.width = 100;
        model.height = 40;
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model
            .anticipatory
            .reduce(crate::state::AnticipatoryAction::Replace(vec![
                crate::wire::AnticipatoryItem {
                    id: "digest-1".to_string(),
                    ..Default::default()
                },
            ]));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "alpha\nbeta\ngamma\ndelta".to_string(),
                ..Default::default()
            },
        });

        let chat_area = rendered_chat_area(&model);
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: chat_area.x.saturating_add(3),
            row: chat_area
                .y
                .saturating_add(chat_area.height.saturating_sub(2)),
            modifiers: KeyModifiers::NONE,
        });

        let snapshot = model
            .chat_selection_snapshot
            .as_ref()
            .expect("mouse down should create a chat selection snapshot");
        assert!(
            widgets::chat::cached_snapshot_matches_area(snapshot, chat_area),
            "drag snapshots must use the exact rendered chat area"
        );
    }

    #[test]
    fn mouse_drag_snapshot_uses_rendered_chat_area_with_sidebar() {
        let mut model = build_model();
        model.width = 100;
        model.height = 40;
        model.show_sidebar_override = Some(true);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model
            .anticipatory
            .reduce(crate::state::AnticipatoryAction::Replace(vec![
                crate::wire::AnticipatoryItem {
                    id: "digest-1".to_string(),
                    ..Default::default()
                },
            ]));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "alpha\nbeta\ngamma\ndelta".to_string(),
                ..Default::default()
            },
        });

        let chat_area = rendered_chat_area(&model);
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: chat_area.x.saturating_add(3),
            row: chat_area
                .y
                .saturating_add(chat_area.height.saturating_sub(2)),
            modifiers: KeyModifiers::NONE,
        });

        let snapshot = model
            .chat_selection_snapshot
            .as_ref()
            .expect("mouse down should create a chat selection snapshot");
        assert!(
            widgets::chat::cached_snapshot_matches_area(snapshot, chat_area),
            "sidebar drag snapshots must use the exact rendered chat area"
        );
    }

    #[test]
    fn thread_detail_refresh_clears_active_chat_drag_snapshot() {
        let (daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, _cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.show_sidebar_override = Some(true);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "short content".to_string(),
                ..Default::default()
            },
        });

        let chat_area = rendered_chat_area(&model);
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: chat_area.x.saturating_add(3),
            row: chat_area
                .y
                .saturating_add(chat_area.height.saturating_sub(2)),
            modifiers: KeyModifiers::NONE,
        });
        assert!(model.chat_selection_snapshot.is_some());

        daemon_tx
            .send(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
                id: "thread-1".to_string(),
                title: "Thread".to_string(),
                messages: vec![crate::wire::AgentMessage {
                    role: crate::wire::MessageRole::Assistant,
                    content: (1..=120)
                        .map(|idx| format!("line {idx}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    ..Default::default()
                }],
                ..Default::default()
            })))
            .expect("thread detail event should send");
        model.pump_daemon_events();

        assert!(
            model.chat_selection_snapshot.is_none(),
            "thread-detail refresh should invalidate stale drag snapshots"
        );
        assert!(model.chat_drag_anchor.is_none());
        assert!(model.chat_drag_current.is_none());
        assert!(model.chat_drag_anchor_point.is_none());
        assert!(model.chat_drag_current_point.is_none());
    }

    #[test]
    fn drag_selection_copies_expected_text_after_autoscroll() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
        let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
            .chain(chat_area.y..preferred_row)
            .find(|row| {
                widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *row),
                )
                .is_some()
            })
            .expect("chat transcript should expose at least one selectable row");

        super::conversion::reset_last_copied_text();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
        for _ in 0..4 {
            model.handle_mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 3,
                row: start_row,
                modifiers: KeyModifiers::NONE,
            });
        }

        let anchor_point = model
            .chat_drag_anchor_point
            .expect("mouse down should capture a document anchor point");
        let current_point = model
            .chat_drag_current_point
            .expect("autoscroll should extend the current drag point");
        let expected = widgets::chat::selected_text(
            chat_area,
            &model.chat,
            &model.theme,
            model.tick_counter,
            anchor_point,
            current_point,
        )
        .expect("selection should resolve to copied text");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(model.status_line, "Copied selection to clipboard");
    }

    #[test]
    fn work_context_drag_selection_copies_beyond_visible_window() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::WorkContextReceived(
            task::ThreadWorkContext {
                thread_id: "thread-1".to_string(),
                entries: vec![task::WorkContextEntry {
                    path: "/tmp/demo.txt".to_string(),
                    is_text: true,
                    ..Default::default()
                }],
            },
        ));
        model
            .tasks
            .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
                path: "/tmp/demo.txt".to_string(),
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                truncated: false,
                is_text: true,
            }));
        model.tasks.reduce(task::TaskAction::SelectWorkPath {
            thread_id: "thread-1".to_string(),
            path: Some("/tmp/demo.txt".to_string()),
        });
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Chat;

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
        let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
            .chain(chat_area.y..preferred_row)
            .find(|row| {
                widgets::work_context_view::selection_point_from_mouse(
                    chat_area,
                    &model.tasks,
                    model.chat.active_thread_id(),
                    model.sidebar.active_tab(),
                    model.sidebar.selected_item(),
                    &model.theme,
                    model.task_view_scroll,
                    Position::new(3, *row),
                )
                .is_some()
            })
            .expect("work-context preview should expose at least one selectable row");

        super::conversion::reset_last_copied_text();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
        for _ in 0..4 {
            model.handle_mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 3,
                row: start_row,
                modifiers: KeyModifiers::NONE,
            });
        }

        let anchor_point = model
            .work_context_drag_anchor_point
            .expect("mouse down should capture a preview anchor point");
        let current_point = model
            .work_context_drag_current_point
            .expect("scrolling should extend the preview selection");
        let expected = widgets::work_context_view::selected_text(
            chat_area,
            &model.tasks,
            model.chat.active_thread_id(),
            model.sidebar.active_tab(),
            model.sidebar.selected_item(),
            &model.theme,
            model.task_view_scroll,
            anchor_point,
            current_point,
        )
        .expect("selection should resolve to copied preview text");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some(expected.as_str())
        );
    }

    #[test]
    fn esc_closes_work_context_even_from_input_focus() {
        let mut model = build_model();
        model.focus = FocusArea::Input;
        model.main_pane_view = MainPaneView::WorkContext;

        let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

        assert!(!handled);
        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Chat);
    }

    #[test]
    fn reselecting_same_sidebar_file_closes_work_context() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::WorkContextReceived(
            task::ThreadWorkContext {
                thread_id: "thread-1".to_string(),
                entries: vec![task::WorkContextEntry {
                    path: "/tmp/demo.txt".to_string(),
                    is_text: true,
                    ..Default::default()
                }],
            },
        ));
        model.tasks.reduce(task::TaskAction::SelectWorkPath {
            thread_id: "thread-1".to_string(),
            path: Some("/tmp/demo.txt".to_string()),
        });
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Files));
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Sidebar;

        model.handle_sidebar_enter();

        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Sidebar);
    }

    #[test]
    fn reselecting_same_sidebar_todo_closes_work_context() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::ThreadTodosReceived {
            thread_id: "thread-1".to_string(),
            items: vec![task::TodoItem {
                id: "todo-1".to_string(),
                content: "todo".to_string(),
                status: Some(task::TodoStatus::Pending),
                position: 0,
                step_index: None,
                created_at: 0,
                updated_at: 0,
            }],
        });
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Todos));
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Sidebar;

        model.handle_sidebar_enter();

        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Sidebar);
    }

    #[test]
    fn attention_surface_uses_settings_tab_when_modal_open() {
        let mut model = build_model();
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        model
            .settings
            .reduce(SettingsAction::SwitchTab(SettingsTab::SubAgents));

        let (surface, thread_id, goal_run_id) = model.current_attention_target();
        assert_eq!(surface, "modal:settings:subagents");
        assert_eq!(thread_id, None);
        assert_eq!(goal_run_id, None);
    }

    #[test]
    fn attention_surface_uses_sidebar_tab_for_sidebar_focus() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread_1".to_string(),
            title: "Test".to_string(),
        });
        model.tasks.reduce(task::TaskAction::ThreadTodosReceived {
            thread_id: "thread_1".to_string(),
            items: vec![task::TodoItem {
                id: "todo_1".to_string(),
                content: "todo".to_string(),
                status: Some(task::TodoStatus::Pending),
                position: 0,
                step_index: None,
                created_at: 0,
                updated_at: 0,
            }],
        });
        model.focus = FocusArea::Sidebar;
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Todos));

        let (surface, thread_id, goal_run_id) = model.current_attention_target();
        assert_eq!(surface, "conversation:sidebar:todos");
        assert_eq!(thread_id.as_deref(), Some("thread_1"));
        assert_eq!(goal_run_id, None);
    }

    #[test]
    fn operator_profile_onboarding_takes_precedence_over_provider_onboarding() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        model.auth.entries = vec![unauthenticated_entry()];
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(
            model.should_show_operator_profile_onboarding(),
            "operator profile onboarding should be active"
        );
        assert!(
            !model.should_show_provider_onboarding(),
            "provider onboarding should not mask operator profile onboarding"
        );
    }

    #[test]
    fn submit_operator_profile_answer_sends_command_and_clears_input() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });
        model.input.set_text("Milan");

        assert!(model.submit_operator_profile_answer());
        assert_eq!(model.input.buffer(), "");
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when submission starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("submitting answer should emit daemon command");
        match sent {
            DaemonCommand::SubmitOperatorProfileAnswer {
                session_id,
                question_id,
                answer_json,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert_eq!(answer_json, "\"Milan\"");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn submit_operator_profile_answer_allows_empty_input_when_question_is_optional() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "nickname".to_string(),
            field_key: "nickname".to_string(),
            prompt: "Nickname?".to_string(),
            input_kind: "text".to_string(),
            optional: true,
        });

        assert!(model.submit_operator_profile_answer());
        assert!(
            model.operator_profile.loading,
            "optional empty answer should begin submission"
        );
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when submission starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("submitting optional empty answer should emit daemon command");
        match sent {
            DaemonCommand::SubmitOperatorProfileAnswer {
                session_id,
                question_id,
                answer_json,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "nickname");
                assert_eq!(answer_json, "null");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn submit_operator_profile_answer_blocks_empty_input_when_question_is_required() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.submit_operator_profile_answer());
        assert!(
            !model.operator_profile.loading,
            "required empty answer should not start submission"
        );
        assert!(
            model.operator_profile.question.is_some(),
            "question should remain while awaiting required answer"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "required empty answer should not emit daemon command"
        );
    }

    #[test]
    fn skip_operator_profile_question_clears_stale_question_immediately() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.skip_operator_profile_question());
        assert!(model.operator_profile.loading);
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when skip starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("skip should emit daemon command");
        match sent {
            DaemonCommand::SkipOperatorProfileQuestion {
                session_id,
                question_id,
                reason,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert_eq!(reason.as_deref(), Some("tui_skip_shortcut"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn defer_operator_profile_question_clears_stale_question_immediately() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(model.defer_operator_profile_question());
        assert!(model.operator_profile.loading);
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when defer starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("defer should emit daemon command");
        match sent {
            DaemonCommand::DeferOperatorProfileQuestion {
                session_id,
                question_id,
                defer_until_unix_ms,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert!(defer_until_unix_ms.is_some());
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
