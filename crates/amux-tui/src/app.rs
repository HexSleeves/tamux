use std::{collections::VecDeque, sync::mpsc::Receiver};
use std::fs;

use ftui_core::event::{Event, KeyCode, MouseButton, MouseEventKind};
use ftui_runtime::{
    program::Cmd,
    string_model::StringModel,
};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use web_time::Duration;

use crate::actions::{AppAction, FocusTarget, ModalKind};
use crate::client::ClientEvent;
use crate::layout::{mission_layout, Rect};
use crate::modal::ModalStack;
use crate::panes::{PaneId, PaneRegistry};
use crate::state::{
    AgentConfigSnapshot, AgentMessage, AgentTask, AgentThread, FetchedModel, GoalRun,
    GoalRunStatus, GoalRunStepKind, GoalRunStepStatus, HeartbeatItem, MessageRole, OutputKind,
    OutputLine, TaskStatus,
};
use crate::theme::ThemeTokens;

#[derive(Debug, Clone)]
pub enum Msg {
    Event(Event),
}

impl From<Event> for Msg {
    fn from(value: Event) -> Self {
        Self::Event(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Threads,
    Chat,
    Mission,
    Composer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoStatus {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranscriptMode {
    Compact,
    Tools,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissionMode {
    Summary,
    Timeline,
    Approvals,
    Events,
    Todo,
}

#[derive(Debug, Clone)]
struct UiTodo {
    title: String,
    status: TodoStatus,
}

#[derive(Debug, Clone)]
pub enum DaemonCommand {
    Refresh,
    RefreshServices,
    RequestThread(String),
    RequestGoalRunDetail(String),
    SendMessage {
        thread_id: Option<String>,
        content: String,
        session_id: Option<String>,
    },
    FetchModels {
        provider_id: String,
        base_url: String,
        api_key: String,
    },
    SetConfigJson(String),
    ControlGoalRun {
        goal_run_id: String,
        action: String,
    },
    ResolveTaskApproval {
        approval_id: String,
        decision: String,
    },
    SpawnSession {
        shell: Option<String>,
        cwd: Option<String>,
        cols: u16,
        rows: u16,
    },
}

pub struct TuiModel {
    daemon_events_rx: Receiver<ClientEvent>,
    daemon_cmd_tx: UnboundedSender<DaemonCommand>,
    action_queue: VecDeque<AppAction>,
    pane_registry: PaneRegistry,
    modal_stack: ModalStack,
    theme: ThemeTokens,

    threads: Vec<AgentThread>,
    selected_thread: usize,
    tasks: Vec<AgentTask>,
    goal_runs: Vec<GoalRun>,
    selected_goal_run: usize,
    mission_scroll: usize,
    mission_mode: MissionMode,
    heartbeat_items: Vec<HeartbeatItem>,
    agent_config: Option<AgentConfigSnapshot>,
    agent_config_raw: Option<Value>,
    fetched_models: Vec<FetchedModel>,
    selected_model_index: usize,
    plugin_items: Vec<String>,
    plugin_cursor: usize,
    mcp_items: Vec<String>,
    mcp_cursor: usize,
    skills_items: Vec<String>,
    skills_cursor: usize,
    pending_approval_ids: Vec<String>,
    todo_items: Vec<UiTodo>,
    todo_cursor: usize,
    transcript_mode: TranscriptMode,

    input: String,
    input_mode: InputMode,
    focus: FocusArea,

    connected: bool,
    status: String,

    default_session_id: Option<String>,

    force_new_chat: bool,
    pending_local_thread_id: Option<String>,
    pending_new_thread_prompt: Option<String>,

    chat_scroll: usize,

    output: Vec<OutputLine>,
    booted_at: u64,
    last_live_refresh_at: u64,
    last_service_refresh_at: u64,

    width: u16,
    height: u16,
}

impl TuiModel {
    pub fn new(
        daemon_events_rx: Receiver<ClientEvent>,
        daemon_cmd_tx: UnboundedSender<DaemonCommand>,
    ) -> Self {
        Self {
            daemon_events_rx,
            daemon_cmd_tx,
            action_queue: VecDeque::new(),
            pane_registry: PaneRegistry::bootstrap(),
            modal_stack: ModalStack::default(),
            theme: ThemeTokens::mission_control_default(),
            threads: Vec::new(),
            selected_thread: 0,
            tasks: Vec::new(),
            goal_runs: Vec::new(),
            selected_goal_run: 0,
            mission_scroll: 0,
            mission_mode: MissionMode::Summary,
            heartbeat_items: Vec::new(),
            agent_config: None,
            agent_config_raw: None,
            fetched_models: Vec::new(),
            selected_model_index: 0,
            plugin_items: discover_plugin_items(),
            plugin_cursor: 0,
            mcp_items: discover_mcp_items(),
            mcp_cursor: 0,
            skills_items: discover_skills_items(),
            skills_cursor: 0,
            pending_approval_ids: Vec::new(),
            todo_items: default_todos(),
            todo_cursor: 0,
            transcript_mode: TranscriptMode::Compact,
            input: String::new(),
            input_mode: InputMode::Insert,
            focus: FocusArea::Composer,
            connected: false,
            status: "Starting Tamux TUI…".to_string(),
            default_session_id: None,
            force_new_chat: false,
            pending_local_thread_id: None,
            pending_new_thread_prompt: None,
            chat_scroll: 0,
            output: Vec::new(),
            booted_at: now_millis(),
            last_live_refresh_at: now_millis(),
            last_service_refresh_at: now_millis(),
            width: 120,
            height: 40,
        }
    }

    fn send_daemon_command(&self, command: DaemonCommand) {
        let _ = self.daemon_cmd_tx.send(command);
    }

    fn enqueue_action(&mut self, action: AppAction) {
        self.action_queue.push_back(action);
    }

    fn drain_action_queue(&mut self) {
        while let Some(action) = self.action_queue.pop_front() {
            self.apply_action(action);
        }
    }

    fn apply_action(&mut self, action: AppAction) {
        match action {
            AppAction::Tick => self.pump_daemon_events(),
            AppAction::Resize { width, height } => {
                self.width = width;
                self.height = height;
            }
            AppAction::Focus(target) => {
                self.focus = match target {
                    FocusTarget::Threads => FocusArea::Threads,
                    FocusTarget::Chat => FocusArea::Chat,
                    FocusTarget::Mission => FocusArea::Mission,
                    FocusTarget::Composer => FocusArea::Composer,
                };

                if self.focus == FocusArea::Composer {
                    self.input_mode = InputMode::Insert;
                }
            }
            AppAction::OpenModal(kind) => {
                self.modal_stack.push(kind);
                self.status = format!("Opened {:?}", kind);
            }
            AppAction::CloseTopModal => {
                let _ = self.modal_stack.pop();
                self.status = "Closed overlay".to_string();
            }
        }
    }

    fn pump_daemon_events(&mut self) {
        while let Ok(event) = self.daemon_events_rx.try_recv() {
            self.handle_client_event(event);
        }

        self.refresh_live_thread_if_needed();
    }

    fn handle_client_event(&mut self, event: ClientEvent) {
        match event {
            ClientEvent::Connected => {
                self.connected = true;
                self.status = "Connected. Type prompt and Enter.".to_string();
                self.add_output("daemon connected", OutputKind::Success);
                self.send_daemon_command(DaemonCommand::Refresh);
                self.send_daemon_command(DaemonCommand::RefreshServices);
                self.bind_launch_terminal_session();
            }
            ClientEvent::Disconnected => {
                self.connected = false;
                self.default_session_id = None;
                self.pending_local_thread_id = None;
                self.status = "Disconnected from daemon".to_string();
                self.add_output("daemon disconnected", OutputKind::Warning);
            }
            ClientEvent::SessionSpawned { session_id } => {
                self.default_session_id = Some(session_id.clone());
                self.status = format!("Bound to terminal session {}", session_id);
                self.add_output(
                    &format!("agent tools bound to session {}", session_id),
                    OutputKind::Info,
                );
            }
            ClientEvent::ThreadList(threads) => self.replace_threads(threads),
            ClientEvent::ThreadDetail(Some(thread)) => self.upsert_thread(thread),
            ClientEvent::ThreadDetail(None) => {}
            ClientEvent::TaskList(tasks) => {
                self.tasks = tasks;
                self.refresh_pending_approvals();
                self.status = format!(
                    "Service sync: {} tasks · {} approvals",
                    self.tasks.len(),
                    self.pending_approval_ids.len()
                );
            }
            ClientEvent::TaskUpdate(task) => {
                self.upsert_task(task);
                self.refresh_pending_approvals();
            }
            ClientEvent::GoalRunList(goal_runs) => {
                self.goal_runs = goal_runs;
                if self.goal_runs.is_empty() {
                    self.selected_goal_run = 0;
                } else {
                    self.selected_goal_run = self
                        .selected_goal_run
                        .min(self.goal_runs.len().saturating_sub(1));
                    if let Some(selected) = self.goal_runs.get(self.selected_goal_run) {
                        self.send_daemon_command(DaemonCommand::RequestGoalRunDetail(
                            selected.id.clone(),
                        ));
                    }
                }
            }
            ClientEvent::GoalRunDetail(Some(goal_run)) => {
                self.upsert_goal_run(goal_run);
            }
            ClientEvent::GoalRunDetail(None) => {}
            ClientEvent::GoalRunUpdate(goal_run) => {
                let goal_run_id = goal_run.id.clone();
                self.upsert_goal_run(goal_run);
                if !goal_run_id.is_empty() {
                    self.send_daemon_command(DaemonCommand::RequestGoalRunDetail(goal_run_id));
                }
            }
            ClientEvent::AgentConfig(config) => {
                self.agent_config = Some(config);
            }
            ClientEvent::AgentConfigRaw(raw) => {
                self.agent_config_raw = Some(raw);
            }
            ClientEvent::ModelsFetched(models) => {
                self.fetched_models = models;
                if self.fetched_models.is_empty() {
                    self.selected_model_index = 0;
                    self.status = "No models returned by provider".to_string();
                } else {
                    self.selected_model_index = self
                        .selected_model_index
                        .min(self.fetched_models.len().saturating_sub(1));
                    self.status = format!("Fetched {} models", self.fetched_models.len());
                }
            }
            ClientEvent::HeartbeatItems(items) => {
                self.heartbeat_items = items;
            }
            ClientEvent::ThreadCreated { thread_id, title } => {
                self.promote_pending_to_real(&thread_id, Some(&title));
                {
                    let thread = self.ensure_thread_mut(&thread_id, &title);
                    thread.updated_at = now_millis();
                }
                self.select_thread_by_id(&thread_id);
                self.status = format!("New thread ready: {}", title);
                self.send_daemon_command(DaemonCommand::RequestThread(thread_id));
            }
            ClientEvent::Delta { thread_id, content } => {
                self.promote_pending_to_real(&thread_id, None);
                self.push_delta(&thread_id, &content);
                self.select_thread_by_id(&thread_id);
                self.chat_scroll = 0;
            }
            ClientEvent::Reasoning { thread_id, content } => {
                self.promote_pending_to_real(&thread_id, None);
                self.push_reasoning(&thread_id, &content);
                self.select_thread_by_id(&thread_id);
                self.chat_scroll = 0;
            }
            ClientEvent::ToolCall {
                thread_id,
                call_id,
                name,
                arguments,
            } => {
                self.promote_pending_to_real(&thread_id, None);
                self.push_tool_call(&thread_id, &call_id, &name, &arguments);
                self.select_thread_by_id(&thread_id);
                self.add_output(
                    &format!("tool {} {}", name, compact_excerpt(&arguments, 160)),
                    OutputKind::Tool,
                );
            }
            ClientEvent::ToolResult {
                thread_id,
                call_id,
                name,
                content,
                is_error,
            } => {
                self.promote_pending_to_real(&thread_id, None);
                self.push_tool_result(&thread_id, &call_id, &name, &content, is_error);
                self.select_thread_by_id(&thread_id);
                self.add_output(
                    &format!("{} -> {}", name, compact_excerpt(&content, 160)),
                    if is_error {
                        OutputKind::Error
                    } else {
                        OutputKind::Success
                    },
                );
            }
            ClientEvent::Done {
                thread_id,
                input_tokens,
                output_tokens,
                cost,
                provider,
                model,
                tps,
                generation_ms,
            } => {
                self.promote_pending_to_real(&thread_id, None);
                self.finalize_turn(
                    &thread_id,
                    input_tokens,
                    output_tokens,
                    cost,
                    tps,
                    generation_ms,
                );

                let provider = provider.unwrap_or_else(|| "provider".to_string());
                let model = model.unwrap_or_else(|| "model".to_string());
                self.add_output(
                    &format!(
                        "turn complete {} / {} · {} in / {} out",
                        provider, model, input_tokens, output_tokens
                    ),
                    OutputKind::Success,
                );

                self.chat_scroll = 0;
            }
            ClientEvent::Error(message) => {
                self.status = message.clone();
                self.add_output(&message, OutputKind::Error);
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Cmd<Msg> {
        match event {
            Event::Key(key) => {
                if key.kind != ftui_core::event::KeyEventKind::Press {
                    return Cmd::none();
                }
                self.handle_key(key.code)
            }
            Event::Mouse(mouse) => {
                self.handle_mouse(mouse);
                self.drain_action_queue();
                Cmd::none()
            }
            Event::Resize { width, height } => {
                self.enqueue_action(AppAction::Resize { width, height });
                self.drain_action_queue();
                Cmd::none()
            }
            Event::Tick => {
                self.enqueue_action(AppAction::Tick);
                self.drain_action_queue();
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> Cmd<Msg> {
        match self.input_mode {
            InputMode::Normal => self.handle_key_normal(code),
            InputMode::Insert => self.handle_key_insert(code),
        }
    }

    fn handle_key_normal(&mut self, code: KeyCode) -> Cmd<Msg> {
        if self.modal_stack.top() == Some(ModalKind::ModelPicker) {
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.select_model_delta(-1);
                    return Cmd::none();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.select_model_delta(1);
                    return Cmd::none();
                }
                KeyCode::Enter => {
                    self.apply_selected_model();
                    return Cmd::none();
                }
                _ => {}
            }
        }

        match code {
            KeyCode::Char('q') => return Cmd::quit(),
            KeyCode::Escape => {
                if self.modal_stack.is_empty() {
                    return Cmd::quit();
                }
                self.enqueue_action(AppAction::CloseTopModal);
            }
            KeyCode::Char('/') => {
                self.input.clear();
                self.input.push('/');
                self.input_mode = InputMode::Insert;
                self.focus = FocusArea::Composer;
                self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
                self.status = "Slash command mode".to_string();
            }
            KeyCode::Char('i') | KeyCode::Enter => {
                self.input_mode = InputMode::Insert;
                self.focus = FocusArea::Composer;
                self.status = "Insert mode".to_string();
            }
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_prev(),
            KeyCode::Up => match self.focus {
                FocusArea::Threads => self.navigate_threads(-1),
                FocusArea::Chat => self.scroll_chat(3),
                FocusArea::Mission => self.scroll_mission(3),
                FocusArea::Composer => {}
            },
            KeyCode::Down => match self.focus {
                FocusArea::Threads => self.navigate_threads(1),
                FocusArea::Chat => self.scroll_chat(-3),
                FocusArea::Mission => self.scroll_mission(-3),
                FocusArea::Composer => {}
            },
            KeyCode::PageUp => match self.focus {
                FocusArea::Mission => self.scroll_mission(12),
                _ => self.scroll_chat(12),
            },
            KeyCode::PageDown => match self.focus {
                FocusArea::Mission => self.scroll_mission(-12),
                _ => self.scroll_chat(-12),
            },
            KeyCode::Home => {
                if self.focus == FocusArea::Mission {
                    self.mission_scroll = usize::MAX / 4;
                    self.status = "Viewing older mission lines".to_string();
                } else {
                    self.chat_scroll = usize::MAX / 4;
                    self.status = "Viewing older transcript lines".to_string();
                }
            }
            KeyCode::End => {
                if self.focus == FocusArea::Mission {
                    self.mission_scroll = 0;
                    self.status = "Following mission tail".to_string();
                } else {
                    self.chat_scroll = 0;
                    self.status = "Following live transcript tail".to_string();
                }
            }
            _ => {}
        }

        self.drain_action_queue();

        Cmd::none()
    }

    fn handle_key_insert(&mut self, code: KeyCode) -> Cmd<Msg> {
        match code {
            KeyCode::Escape => {
                self.input_mode = InputMode::Normal;
                self.status = "Normal mode".to_string();
            }
            KeyCode::Enter => {
                if self.input.trim().is_empty() {
                    self.status = "Empty prompt".to_string();
                } else {
                    self.submit_prompt();
                }
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Tab => {
                self.input_mode = InputMode::Normal;
                self.focus_next();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                if self.input.starts_with('/') && self.modal_stack.top() != Some(ModalKind::CommandPalette) {
                    self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
                    self.drain_action_queue();
                }
            }
            _ => {}
        }

        Cmd::none()
    }

    fn handle_mouse(&mut self, mouse: ftui_core::event::MouseEvent) {
        let layout = mission_layout(self.width, self.height);
        let point = (mouse.x, mouse.y);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if layout.hints.contains(point.0, point.1) {
                    self.input.clear();
                    self.input.push('/');
                    self.enqueue_action(AppAction::Focus(FocusTarget::Composer));
                    self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
                } else if layout.threads.contains(point.0, point.1) {
                    self.enqueue_action(AppAction::Focus(FocusTarget::Threads));
                    if let Some(index) = click_list_index(
                        rect_to_area(layout.threads),
                        point.1,
                        self.threads.len(),
                        self.selected_thread,
                        1,
                    ) {
                        self.selected_thread = index;
                        self.chat_scroll = 0;
                        if let Some(thread_id) = self.selected_thread().map(|thread| thread.id.clone()) {
                            self.send_daemon_command(DaemonCommand::RequestThread(thread_id));
                        }
                    }
                } else if layout.chat.contains(point.0, point.1) {
                    self.enqueue_action(AppAction::Focus(FocusTarget::Chat));
                } else if layout.mission.contains(point.0, point.1) {
                    self.enqueue_action(AppAction::Focus(FocusTarget::Mission));
                } else if layout.composer_input.contains(point.0, point.1)
                    || layout.composer_meta.contains(point.0, point.1)
                {
                    self.enqueue_action(AppAction::Focus(FocusTarget::Composer));
                }
            }
            MouseEventKind::ScrollUp => {
                if layout.threads.contains(point.0, point.1) {
                    self.navigate_threads(-1);
                } else if layout.chat.contains(point.0, point.1) {
                    self.scroll_chat(3);
                } else if layout.mission.contains(point.0, point.1) {
                    self.scroll_mission(3);
                }
            }
            MouseEventKind::ScrollDown => {
                if layout.threads.contains(point.0, point.1) {
                    self.navigate_threads(1);
                } else if layout.chat.contains(point.0, point.1) {
                    self.scroll_chat(-3);
                } else if layout.mission.contains(point.0, point.1) {
                    self.scroll_mission(-3);
                }
            }
            _ => {}
        }
    }

    fn submit_prompt(&mut self) {
        let prompt = self.input.trim().to_string();
        if self.execute_slash_command(&prompt) {
            self.input.clear();
            self.input_mode = InputMode::Normal;
            self.focus = FocusArea::Chat;
            self.chat_scroll = 0;
            return;
        }

        if !self.connected {
            self.status = "Daemon not connected".to_string();
            return;
        }

        if self.default_session_id.is_none() {
            self.bind_launch_terminal_session();
        }

        self.input.clear();
        self.input_mode = InputMode::Normal;
        self.focus = FocusArea::Chat;

        let session_id = self.default_session_id.clone();
        if self.force_new_chat || self.selected_thread().is_none() {
            let local_thread_id = format!("local-thread-{}", now_millis());
            self.create_local_pending_thread(&local_thread_id, &prompt);
            self.pending_local_thread_id = Some(local_thread_id.clone());
            self.pending_new_thread_prompt = Some(prompt.clone());
            self.select_thread_by_id(&local_thread_id);
            self.send_daemon_command(DaemonCommand::SendMessage {
                thread_id: None,
                content: prompt.clone(),
                session_id,
            });
        } else if let Some(thread_id) = self.selected_thread().map(|thread| thread.id.clone()) {
            self.append_user_turn(&thread_id, &prompt);
            self.ensure_tail_assistant(&thread_id);
            self.send_daemon_command(DaemonCommand::SendMessage {
                thread_id: Some(thread_id.clone()),
                content: prompt.clone(),
                session_id,
            });
            self.select_thread_by_id(&thread_id);
        }

        self.force_new_chat = false;
        self.chat_scroll = 0;
        self.status = "Prompt sent".to_string();
        self.add_output(&format!("chat ▶ {}", prompt), OutputKind::Info);
    }

    fn bind_launch_terminal_session(&mut self) {
        if self.default_session_id.is_some() {
            return;
        }

        let cwd = std::env::current_dir()
            .ok()
            .map(|value| value.to_string_lossy().to_string());
        let shell = std::env::var("SHELL").ok().filter(|value| !value.trim().is_empty());

        self.send_daemon_command(DaemonCommand::SpawnSession {
            shell,
            cwd,
            cols: self.width.max(80),
            rows: self.height.max(24),
        });
        self.status = "Binding launch shell session…".to_string();
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            FocusArea::Threads => FocusArea::Chat,
            FocusArea::Chat => FocusArea::Mission,
            FocusArea::Mission => FocusArea::Composer,
            FocusArea::Composer => FocusArea::Threads,
        };

        if self.focus == FocusArea::Composer {
            self.input_mode = InputMode::Insert;
        }
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            FocusArea::Threads => FocusArea::Composer,
            FocusArea::Chat => FocusArea::Threads,
            FocusArea::Mission => FocusArea::Chat,
            FocusArea::Composer => FocusArea::Mission,
        };

        if self.focus != FocusArea::Composer {
            self.input_mode = InputMode::Normal;
        }
    }

    fn navigate_threads(&mut self, delta: isize) {
        if self.threads.is_empty() {
            return;
        }

        self.selected_thread = offset_index(self.selected_thread, self.threads.len(), delta);
        self.chat_scroll = 0;
        if let Some(thread_id) = self.selected_thread().map(|thread| thread.id.clone()) {
            self.send_daemon_command(DaemonCommand::RequestThread(thread_id));
        }
    }

    fn scroll_chat(&mut self, delta: isize) {
        if delta >= 0 {
            self.chat_scroll = self.chat_scroll.saturating_add(delta as usize);
            self.status = "Transcript scroll locked above live tail".to_string();
        } else {
            self.chat_scroll = self.chat_scroll.saturating_sub((-delta) as usize);
            if self.chat_scroll == 0 {
                self.status = "Following live transcript tail".to_string();
            }
        }
    }

    fn scroll_mission(&mut self, delta: isize) {
        if delta >= 0 {
            self.mission_scroll = self.mission_scroll.saturating_add(delta as usize);
            self.status = "Mission scroll locked above live tail".to_string();
        } else {
            self.mission_scroll = self.mission_scroll.saturating_sub((-delta) as usize);
            if self.mission_scroll == 0 {
                self.status = "Following mission tail".to_string();
            }
        }
    }

    fn execute_slash_command(&mut self, prompt: &str) -> bool {
        let trimmed = prompt.trim();
        if !trimmed.starts_with('/') {
            return false;
        }

        let body = trimmed.trim_start_matches('/').trim();
        if body.is_empty() {
            self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
            self.drain_action_queue();
            self.status = "Slash command browser".to_string();
            return true;
        }

        let parts = body.split_whitespace().collect::<Vec<_>>();
        let cmd = parts.first().copied().unwrap_or_default().to_ascii_lowercase();
        let arg1 = parts.get(1).copied().unwrap_or_default().to_ascii_lowercase();
        let arg2 = parts.get(2).copied().unwrap_or_default().to_ascii_lowercase();

        match cmd.as_str() {
            "help" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
                self.status = "Opened slash command browser".to_string();
            }
            "new" => {
                self.force_new_chat = true;
                self.pending_local_thread_id = None;
                self.pending_new_thread_prompt = None;
                self.status = "New conversation armed".to_string();
            }
            "refresh" => match arg1.as_str() {
                "threads" | "chat" => {
                    self.send_daemon_command(DaemonCommand::Refresh);
                    self.status = "Refreshed threads".to_string();
                }
                "services" | "svc" => {
                    self.send_daemon_command(DaemonCommand::RefreshServices);
                    self.status = "Refreshed services".to_string();
                }
                "all" => {
                    self.send_daemon_command(DaemonCommand::Refresh);
                    self.send_daemon_command(DaemonCommand::RefreshServices);
                    self.status = "Refreshed threads + services".to_string();
                }
                _ => {
                    self.send_daemon_command(DaemonCommand::Refresh);
                    self.send_daemon_command(DaemonCommand::RefreshServices);
                    self.status = "Refreshed threads + services".to_string();
                }
            },
            "view" => self.execute_view_command(&arg1, &arg2),
            "goal" => self.execute_goal_command(&arg1),
            "approval" | "approve" => self.execute_approval_command(&arg1),
            "model" => self.execute_model_command(&arg1),
            "plugin" | "plugins" => self.execute_plugin_command(&arg1),
            "mcp" => self.execute_mcp_command(&arg1),
            "skill" | "skills" => self.execute_skills_command(&arg1),
            "todo" => self.execute_todo_command(&arg1),
            _ => {
                self.status = format!("Unknown slash command: /{}", cmd);
                self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
            }
        }

        self.add_output(&format!("slash ▶ /{}", body), OutputKind::Info);
        self.drain_action_queue();
        true
    }

    fn execute_view_command(&mut self, view: &str, sub: &str) {
        match view {
            "threads" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Threads));
                self.enqueue_action(AppAction::OpenModal(ModalKind::ThreadPicker));
                self.status = "View: threads".to_string();
            }
            "chat" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Chat));
                let _ = self.modal_stack.pop();
                self.transcript_mode = match sub {
                    "tools" => TranscriptMode::Tools,
                    "full" => TranscriptMode::Full,
                    _ => TranscriptMode::Compact,
                };
                self.status = format!(
                    "View: conversation ({})",
                    format_transcript_mode(self.transcript_mode)
                );
            }
            "mission" | "goal" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Mission));
                self.mission_mode = match sub {
                    "timeline" => MissionMode::Timeline,
                    "approvals" => MissionMode::Approvals,
                    "events" => MissionMode::Events,
                    "todo" => MissionMode::Todo,
                    _ => MissionMode::Summary,
                };
                self.status = format!(
                    "View: mission ({})",
                    format_mission_mode(self.mission_mode)
                );
            }
            "provider" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::ProviderPicker));
                self.status = "View: provider profile".to_string();
            }
            "model" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::ModelPicker));
                self.fetch_models_for_active_provider();
                self.status = "View: model picker".to_string();
            }
            "plugin" | "plugins" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::PluginPicker));
                self.status = "View: plugin picker".to_string();
            }
            "mcp" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::McpPicker));
                self.status = "View: MCP picker".to_string();
            }
            "skill" | "skills" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::SkillsPicker));
                self.status = "View: skills picker".to_string();
            }
            "session" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::SessionPicker));
                self.status = "View: session picker".to_string();
            }
            "approval" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::ApprovalOverlay));
                self.status = "View: approval overlay".to_string();
            }
            "commands" | "palette" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::CommandPalette));
                self.status = "View: command browser".to_string();
            }
            "todo" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Mission));
                self.mission_mode = MissionMode::Todo;
                self.status = "View: TODO tracker".to_string();
            }
            _ => {
                self.status = "Usage: /view threads | /view chat [compact|tools|full] | /view mission [summary|timeline|approvals|events|todo] | provider|model|plugin|mcp|skills|session|approval|todo|commands".to_string();
            }
        }
    }

    fn execute_goal_command(&mut self, action: &str) {
        match action {
            "next" => self.select_goal_run_delta(1),
            "prev" => self.select_goal_run_delta(-1),
            "pause" => self.control_selected_goal_run("pause"),
            "resume" => self.control_selected_goal_run("resume"),
            "cancel" => self.control_selected_goal_run("cancel"),
            "rerun" => self.control_selected_goal_run("rerun"),
            "retry" => self.control_selected_goal_run("retry"),
            "view" | "open" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Mission));
                self.status = "Goal detail opened in mission pane".to_string();
            }
            _ => {
                self.status = "Usage: /goal next|prev|pause|resume|cancel|retry|rerun|view".to_string();
            }
        }
    }

    fn execute_approval_command(&mut self, action: &str) {
        match action {
            "approve" | "approve_once" => self.resolve_first_pending_approval("approve_once"),
            "deny" => self.resolve_first_pending_approval("deny"),
            "view" | "open" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::ApprovalOverlay));
                self.status = "Approval queue opened".to_string();
            }
            _ => {
                self.status = "Usage: /approval approve|deny|view".to_string();
            }
        }
    }

    fn execute_model_command(&mut self, action: &str) {
        match action {
            "fetch" => self.fetch_models_for_active_provider(),
            "next" => self.select_model_delta(1),
            "prev" => self.select_model_delta(-1),
            "apply" => self.apply_selected_model(),
            "view" | "open" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::ModelPicker));
                self.fetch_models_for_active_provider();
                self.status = "Model picker opened".to_string();
            }
            _ => {
                self.status = "Usage: /model fetch|next|prev|apply|view".to_string();
            }
        }
    }

    fn execute_todo_command(&mut self, action: &str) {
        match action {
            "next" => {
                self.todo_cursor = offset_index(self.todo_cursor, self.todo_items.len(), 1);
                self.status = format!("TODO selected: {}", self.todo_items[self.todo_cursor].title);
            }
            "prev" => {
                self.todo_cursor = offset_index(self.todo_cursor, self.todo_items.len(), -1);
                self.status = format!("TODO selected: {}", self.todo_items[self.todo_cursor].title);
            }
            "doing" => {
                if let Some(item) = self.todo_items.get_mut(self.todo_cursor) {
                    item.status = TodoStatus::InProgress;
                    self.status = format!("TODO in progress: {}", item.title);
                }
            }
            "done" => {
                if let Some(item) = self.todo_items.get_mut(self.todo_cursor) {
                    item.status = TodoStatus::Completed;
                    self.status = format!("TODO completed: {}", item.title);
                }
            }
            "open" | "view" | "list" | "" => {
                self.enqueue_action(AppAction::Focus(FocusTarget::Mission));
                self.status = "TODO tracker in mission pane".to_string();
            }
            _ => {
                self.status = "Usage: /todo list|next|prev|doing|done|view".to_string();
            }
        }
    }

    fn execute_plugin_command(&mut self, action: &str) {
        match action {
            "next" => {
                self.plugin_cursor = offset_index(self.plugin_cursor, self.plugin_items.len(), 1);
                self.status = format!("Plugin selected: {}", selected_item(&self.plugin_items, self.plugin_cursor));
            }
            "prev" => {
                self.plugin_cursor = offset_index(self.plugin_cursor, self.plugin_items.len(), -1);
                self.status = format!("Plugin selected: {}", selected_item(&self.plugin_items, self.plugin_cursor));
            }
            "refresh" => {
                self.plugin_items = discover_plugin_items();
                self.plugin_cursor = self.plugin_cursor.min(self.plugin_items.len().saturating_sub(1));
                self.status = format!("Plugins refreshed ({})", self.plugin_items.len());
            }
            "inspect" => {
                self.status = format!("Plugin inspect: {}", selected_item(&self.plugin_items, self.plugin_cursor));
            }
            "view" | "open" | "list" | "" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::PluginPicker));
                self.status = "Plugin picker opened".to_string();
            }
            _ => self.status = "Usage: /plugin list|next|prev|inspect|refresh|view".to_string(),
        }
    }

    fn execute_mcp_command(&mut self, action: &str) {
        match action {
            "next" => {
                self.mcp_cursor = offset_index(self.mcp_cursor, self.mcp_items.len(), 1);
                self.status = format!("MCP selected: {}", selected_item(&self.mcp_items, self.mcp_cursor));
            }
            "prev" => {
                self.mcp_cursor = offset_index(self.mcp_cursor, self.mcp_items.len(), -1);
                self.status = format!("MCP selected: {}", selected_item(&self.mcp_items, self.mcp_cursor));
            }
            "refresh" => {
                self.mcp_items = discover_mcp_items();
                self.mcp_cursor = self.mcp_cursor.min(self.mcp_items.len().saturating_sub(1));
                self.status = format!("MCP inventory refreshed ({})", self.mcp_items.len());
            }
            "inspect" => {
                self.status = format!("MCP inspect: {}", selected_item(&self.mcp_items, self.mcp_cursor));
            }
            "view" | "open" | "list" | "" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::McpPicker));
                self.status = "MCP picker opened".to_string();
            }
            _ => self.status = "Usage: /mcp list|next|prev|inspect|refresh|view".to_string(),
        }
    }

    fn execute_skills_command(&mut self, action: &str) {
        match action {
            "next" => {
                self.skills_cursor = offset_index(self.skills_cursor, self.skills_items.len(), 1);
                self.status = format!("Skill selected: {}", selected_item(&self.skills_items, self.skills_cursor));
            }
            "prev" => {
                self.skills_cursor = offset_index(self.skills_cursor, self.skills_items.len(), -1);
                self.status = format!("Skill selected: {}", selected_item(&self.skills_items, self.skills_cursor));
            }
            "refresh" => {
                self.skills_items = discover_skills_items();
                self.skills_cursor = self.skills_cursor.min(self.skills_items.len().saturating_sub(1));
                self.status = format!("Skills refreshed ({})", self.skills_items.len());
            }
            "inspect" => {
                self.status = format!("Skill inspect: {}", selected_item(&self.skills_items, self.skills_cursor));
            }
            "view" | "open" | "list" | "" => {
                self.enqueue_action(AppAction::OpenModal(ModalKind::SkillsPicker));
                self.status = "Skills picker opened".to_string();
            }
            _ => self.status = "Usage: /skills list|next|prev|inspect|refresh|view".to_string(),
        }
    }

    fn refresh_live_thread_if_needed(&mut self) {
        if !self.connected {
            return;
        }

        let now = now_millis();
        if now.saturating_sub(self.last_service_refresh_at) >= 2200 {
            self.last_service_refresh_at = now;
            self.send_daemon_command(DaemonCommand::RefreshServices);
        }

        if now.saturating_sub(self.last_live_refresh_at) < 1200 {
            return;
        }

        let Some(thread_id) = self
            .selected_thread()
            .filter(|thread| {
                thread.messages.iter().rev().take(10).any(|message| {
                    (message.role == MessageRole::Assistant && message.is_streaming)
                        || message.tool_status.as_deref() == Some("executing")
                })
            })
            .map(|thread| thread.id.clone())
        else {
            return;
        };

        self.last_live_refresh_at = now;
        self.send_daemon_command(DaemonCommand::RequestThread(thread_id));
    }

    fn create_local_pending_thread(&mut self, thread_id: &str, prompt: &str) {
        let title = prompt
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ");
        let title = if title.is_empty() {
            "New Conversation".to_string()
        } else {
            title
        };

        let thread = self.ensure_thread_mut(thread_id, &title);
        if thread.messages.is_empty() {
            thread
                .messages
                .push(Self::make_message(MessageRole::User, prompt.to_string()));
            let mut assistant = Self::make_message(MessageRole::Assistant, String::new());
            assistant.is_streaming = true;
            thread.messages.push(assistant);
        }
        thread.updated_at = now_millis();
    }

    fn append_user_turn(&mut self, thread_id: &str, content: &str) {
        let thread = self.ensure_thread_mut(thread_id, "Conversation");
        thread
            .messages
            .push(Self::make_message(MessageRole::User, content.to_string()));
        thread.updated_at = now_millis();
    }

    fn ensure_tail_assistant(&mut self, thread_id: &str) {
        let thread = self.ensure_thread_mut(thread_id, "Conversation");
        let needs = !matches!(
            thread.messages.last(),
            Some(message) if message.role == MessageRole::Assistant && message.is_streaming
        );

        if needs {
            let mut assistant = Self::make_message(MessageRole::Assistant, String::new());
            assistant.is_streaming = true;
            thread.messages.push(assistant);
        }
    }

    fn push_delta(&mut self, thread_id: &str, content: &str) {
        self.ensure_tail_assistant(thread_id);
        let thread = self.ensure_thread_mut(thread_id, "Conversation");
        if let Some(last) = thread.messages.last_mut() {
            last.content.push_str(content);
            last.is_streaming = true;
            last.timestamp = now_millis();
        }
        thread.updated_at = now_millis();
    }

    fn push_reasoning(&mut self, thread_id: &str, content: &str) {
        self.ensure_tail_assistant(thread_id);
        let thread = self.ensure_thread_mut(thread_id, "Conversation");
        if let Some(last) = thread.messages.last_mut() {
            let reasoning = last.reasoning.get_or_insert_with(String::new);
            reasoning.push_str(content);
            last.is_streaming = true;
            last.timestamp = now_millis();
        }
        thread.updated_at = now_millis();
    }

    fn push_tool_call(&mut self, thread_id: &str, call_id: &str, name: &str, arguments: &str) {
        let thread = self.ensure_thread_mut(thread_id, "Conversation");

        let found = if !call_id.is_empty() {
            thread
                .messages
                .iter_mut()
                .rev()
                .find(|message| message.tool_call_id.as_deref() == Some(call_id))
        } else {
            thread
                .messages
                .iter_mut()
                .rev()
                .find(|message| {
                    message.role == MessageRole::Tool
                        && message.tool_name.as_deref() == Some(name)
                        && message.tool_status.as_deref() == Some("executing")
                })
        };

        if let Some(message) = found {
            message.tool_name = Some(name.to_string());
            message.tool_arguments = Some(arguments.to_string());
            if !call_id.is_empty() {
                message.tool_call_id = Some(call_id.to_string());
            }
            message.tool_status = Some("executing".to_string());
            message.timestamp = now_millis();
        } else {
            let mut tool = Self::make_message(MessageRole::Tool, String::new());
            tool.tool_name = Some(name.to_string());
            tool.tool_arguments = Some(arguments.to_string());
            tool.tool_call_id = if call_id.is_empty() {
                None
            } else {
                Some(call_id.to_string())
            };
            tool.tool_status = Some("executing".to_string());
            thread.messages.push(tool);
        }

        thread.updated_at = now_millis();
    }

    fn push_tool_result(&mut self, thread_id: &str, call_id: &str, name: &str, content: &str, is_error: bool) {
        let thread = self.ensure_thread_mut(thread_id, "Conversation");

        let found = if !call_id.is_empty() {
            thread
                .messages
                .iter_mut()
                .rev()
                .find(|message| message.tool_call_id.as_deref() == Some(call_id))
        } else {
            thread
                .messages
                .iter_mut()
                .rev()
                .find(|message| {
                    message.role == MessageRole::Tool
                        && message.tool_name.as_deref() == Some(name)
                        && message.tool_status.as_deref() == Some("executing")
                })
        };

        if let Some(message) = found {
            message.tool_name = Some(name.to_string());
            message.content = content.to_string();
            message.tool_status = Some(if is_error { "error" } else { "done" }.to_string());
            if !call_id.is_empty() {
                message.tool_call_id = Some(call_id.to_string());
            }
            message.timestamp = now_millis();
        } else {
            let mut tool = Self::make_message(MessageRole::Tool, content.to_string());
            tool.tool_name = Some(name.to_string());
            tool.tool_call_id = if call_id.is_empty() {
                None
            } else {
                Some(call_id.to_string())
            };
            tool.tool_status = Some(if is_error { "error" } else { "done" }.to_string());
            thread.messages.push(tool);
        }

        thread.updated_at = now_millis();
    }

    fn finalize_turn(
        &mut self,
        thread_id: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost: Option<f64>,
        tps: Option<f64>,
        generation_ms: Option<u64>,
    ) {
        let thread = self.ensure_thread_mut(thread_id, "Conversation");

        let target_index = thread
            .messages
            .iter()
            .rposition(|message| message.role == MessageRole::Assistant && message.is_streaming)
            .or_else(|| {
                thread
                    .messages
                    .iter()
                    .rposition(|message| message.role == MessageRole::Assistant)
            });

        if let Some(index) = target_index {
            let (existing_input, existing_output) = {
                let message = &thread.messages[index];
                (message.input_tokens, message.output_tokens)
            };

            let input_delta = input_tokens.saturating_sub(existing_input);
            let output_delta = output_tokens.saturating_sub(existing_output);
            thread.total_input_tokens = thread.total_input_tokens.saturating_add(input_delta);
            thread.total_output_tokens = thread.total_output_tokens.saturating_add(output_delta);

            let message = &mut thread.messages[index];

            message.input_tokens = message.input_tokens.max(input_tokens);
            message.output_tokens = message.output_tokens.max(output_tokens);
            message.cost = choose_max_f64(message.cost, cost);
            message.tps = choose_max_f64(message.tps, tps);
            message.generation_ms = match (message.generation_ms, generation_ms) {
                (Some(left), Some(right)) => Some(left.max(right)),
                (None, right) => right,
                (left, None) => left,
            };
            message.is_streaming = false;
            message.timestamp = now_millis();
        }

        for message in thread
            .messages
            .iter_mut()
            .filter(|message| message.role == MessageRole::Tool && message.tool_status.as_deref() == Some("executing"))
        {
            message.tool_status = Some("done".to_string());
            if message.content.trim().is_empty() {
                message.content = "Tool finished".to_string();
            }
            message.timestamp = now_millis();
        }

        thread.updated_at = now_millis();
    }

    fn promote_pending_to_real(&mut self, thread_id: &str, title: Option<&str>) {
        if self.pending_new_thread_prompt.is_none() {
            return;
        }

        let Some(local_id) = self.pending_local_thread_id.clone() else {
            return;
        };
        if local_id == thread_id {
            return;
        }

        let pending_is_selected = self
            .selected_thread()
            .map(|thread| thread.id == local_id)
            .unwrap_or(false);
        let target_has_messages = self
            .threads
            .iter()
            .find(|thread| thread.id == thread_id)
            .map(|thread| !thread.messages.is_empty())
            .unwrap_or(false);

        if !pending_is_selected && target_has_messages {
            return;
        }

        let local_messages = self.take_thread(&local_id).map(|thread| thread.messages);
        self.pending_local_thread_id = None;

        let pending_prompt = self.pending_new_thread_prompt.clone();
        {
            let thread = self.ensure_thread_mut(thread_id, title.unwrap_or("Conversation"));
            if thread.messages.is_empty() {
                if let Some(messages) = local_messages {
                    if !messages.is_empty() {
                        thread.messages = messages;
                    }
                }

                if thread.messages.is_empty() {
                    if let Some(prompt) = pending_prompt {
                        thread.messages.push(Self::make_message(MessageRole::User, prompt));
                        let mut assistant = Self::make_message(MessageRole::Assistant, String::new());
                        assistant.is_streaming = true;
                        thread.messages.push(assistant);
                    }
                }
            }
            thread.updated_at = now_millis();
        }

        self.pending_new_thread_prompt = None;
    }

    fn replace_threads(&mut self, incoming_threads: Vec<AgentThread>) {
        let selected_id = self.selected_thread().map(|thread| thread.id.clone());
        let pending_id = self.pending_local_thread_id.clone();

        let mut merged = Vec::with_capacity(incoming_threads.len() + 1);
        for thread in incoming_threads {
            let merged_thread = if let Some(existing) = self.threads.iter().find(|candidate| candidate.id == thread.id) {
                merge_thread(existing, thread)
            } else {
                thread
            };
            merged.push(merged_thread);
        }

        if let Some(pending_id) = pending_id {
            if !merged.iter().any(|thread| thread.id == pending_id) {
                if let Some(local_pending) = self.threads.iter().find(|thread| thread.id == pending_id) {
                    merged.push(local_pending.clone());
                }
            }
        }

        merged.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        self.threads = merged;

        if let Some(selected_id) = selected_id {
            self.select_thread_by_id(&selected_id);
        } else {
            self.selected_thread = self.selected_thread.min(self.threads.len().saturating_sub(1));
        }
    }

    fn upsert_thread(&mut self, thread: AgentThread) {
        if let Some(existing) = self.threads.iter_mut().find(|candidate| candidate.id == thread.id) {
            *existing = merge_thread(existing, thread);
        } else {
            self.threads.push(thread);
        }

        self.threads
            .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    }

    fn upsert_task(&mut self, task: AgentTask) {
        if task.id.is_empty() {
            return;
        }

        if let Some(existing) = self.tasks.iter_mut().find(|candidate| candidate.id == task.id) {
            *existing = task;
        } else {
            self.tasks.push(task);
        }
    }

    fn upsert_goal_run(&mut self, goal_run: GoalRun) {
        if goal_run.id.is_empty() {
            return;
        }

        if let Some(existing) = self
            .goal_runs
            .iter_mut()
            .find(|candidate| candidate.id == goal_run.id)
        {
            *existing = goal_run;
        } else {
            self.goal_runs.push(goal_run);
        }
    }

    fn refresh_pending_approvals(&mut self) {
        self.pending_approval_ids = self
            .tasks
            .iter()
            .filter_map(|task| task.awaiting_approval_id.clone())
            .collect();
    }

    fn select_goal_run_delta(&mut self, delta: isize) {
        if self.goal_runs.is_empty() {
            self.status = "No goal runs available".to_string();
            return;
        }

        self.selected_goal_run = offset_index(self.selected_goal_run, self.goal_runs.len(), delta);
        if let Some(goal_run) = self.goal_runs.get(self.selected_goal_run) {
            let status = goal_run
                .status
                .map(format_goal_status)
                .unwrap_or("unknown");
            self.send_daemon_command(DaemonCommand::RequestGoalRunDetail(goal_run.id.clone()));
            self.status = format!(
                "Selected goal {} ({})",
                truncate_inline(&goal_run.title, 48),
                status
            );
        }
    }

    fn fetch_models_for_active_provider(&mut self) {
        let Some((provider_id, base_url, api_key)) = self.active_provider_fetch_params() else {
            self.status = "Cannot fetch models: provider config unavailable".to_string();
            return;
        };

        self.send_daemon_command(DaemonCommand::FetchModels {
            provider_id: provider_id.clone(),
            base_url,
            api_key,
        });
        self.status = format!("Fetching models for {}", provider_id);
    }

    fn select_model_delta(&mut self, delta: isize) {
        if self.fetched_models.is_empty() {
            self.status = "No fetched models to select".to_string();
            return;
        }

        self.selected_model_index = offset_index(self.selected_model_index, self.fetched_models.len(), delta);
        if let Some(model) = self.fetched_models.get(self.selected_model_index) {
            self.status = format!("Model selected: {}", model.id);
        }
    }

    fn apply_selected_model(&mut self) {
        let Some(selected) = self.fetched_models.get(self.selected_model_index).cloned() else {
            self.status = "No model selected".to_string();
            return;
        };

        let Some(mut raw_config) = self.agent_config_raw.clone() else {
            self.status = "Cannot apply model: raw config unavailable".to_string();
            return;
        };

        let provider_id = self
            .agent_config
            .as_ref()
            .map(|cfg| cfg.provider.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| get_json_string(&raw_config, "provider"))
            .unwrap_or_else(|| "openai".to_string());

        let fetch_params = self.active_provider_fetch_params();

        let Value::Object(root) = &mut raw_config else {
            self.status = "Cannot apply model: invalid config payload".to_string();
            return;
        };

        root.insert("provider".to_string(), Value::String(provider_id.clone()));
        root.insert("model".to_string(), Value::String(selected.id.clone()));

        let providers_value = root
            .entry("providers".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(providers) = providers_value {
            let provider_value = providers
                .entry(provider_id.clone())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            if let Value::Object(provider_cfg) = provider_value {
                provider_cfg.insert("model".to_string(), Value::String(selected.id.clone()));

                if let Some((_, base_url, api_key)) = fetch_params {
                    if !base_url.is_empty() {
                        provider_cfg
                            .entry("base_url".to_string())
                            .or_insert_with(|| Value::String(base_url));
                    }
                    if !api_key.is_empty() {
                        provider_cfg
                            .entry("api_key".to_string())
                            .or_insert_with(|| Value::String(api_key));
                    }
                }

                if let Some(reasoning) = self
                    .agent_config
                    .as_ref()
                    .map(|cfg| cfg.reasoning_effort.clone())
                    .filter(|value| !value.trim().is_empty())
                {
                    provider_cfg
                        .entry("reasoning_effort".to_string())
                        .or_insert_with(|| Value::String(reasoning));
                }
            }
        }

        let Ok(config_json) = serde_json::to_string(&raw_config) else {
            self.status = "Cannot apply model: failed to serialize config".to_string();
            return;
        };

        self.send_daemon_command(DaemonCommand::SetConfigJson(config_json));
        self.send_daemon_command(DaemonCommand::RefreshServices);
        self.agent_config_raw = Some(raw_config);
        if let Some(config) = &mut self.agent_config {
            config.provider = provider_id;
            config.model = selected.id.clone();
        }

        self.status = format!("Applied model {}", selected.id);
    }

    fn active_provider_fetch_params(&self) -> Option<(String, String, String)> {
        let provider_id = self
            .agent_config
            .as_ref()
            .map(|cfg| cfg.provider.clone())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                self.agent_config_raw
                    .as_ref()
                    .and_then(|raw| get_json_string(raw, "provider"))
            })?;

        let base_url = self
            .agent_config_raw
            .as_ref()
            .and_then(|raw| get_provider_field(raw, &provider_id, "base_url"))
            .or_else(|| {
                self.agent_config_raw
                    .as_ref()
                    .and_then(|raw| get_json_string(raw, "base_url"))
            })
            .unwrap_or_else(|| {
                self.agent_config
                    .as_ref()
                    .map(|cfg| cfg.base_url.clone())
                    .unwrap_or_default()
            });

        let api_key = self
            .agent_config_raw
            .as_ref()
            .and_then(|raw| get_provider_field(raw, &provider_id, "api_key"))
            .or_else(|| {
                self.agent_config_raw
                    .as_ref()
                    .and_then(|raw| get_json_string(raw, "api_key"))
            })
            .unwrap_or_else(|| {
                self.agent_config
                    .as_ref()
                    .map(|cfg| cfg.api_key.clone())
                    .unwrap_or_default()
            });

        Some((provider_id, base_url, api_key))
    }

    fn control_selected_goal_run(&mut self, action: &str) {
        let Some(goal_run) = self.goal_runs.get(self.selected_goal_run) else {
            self.status = "No goal run selected".to_string();
            return;
        };

        self.send_daemon_command(DaemonCommand::ControlGoalRun {
            goal_run_id: goal_run.id.clone(),
            action: action.to_string(),
        });
        self.status = format!(
            "Goal action {} sent for {}",
            action,
            truncate_inline(&goal_run.title, 48)
        );
    }

    fn resolve_first_pending_approval(&mut self, decision: &str) {
        let Some(approval_id) = self.pending_approval_ids.first().cloned() else {
            self.status = "No pending approvals".to_string();
            return;
        };

        self.send_daemon_command(DaemonCommand::ResolveTaskApproval {
            approval_id: approval_id.clone(),
            decision: decision.to_string(),
        });

        self.status = format!("Approval {} -> {}", approval_id, decision);
    }

    fn ensure_thread_mut(&mut self, thread_id: &str, title: &str) -> &mut AgentThread {
        if let Some(index) = self.threads.iter().position(|thread| thread.id == thread_id) {
            return &mut self.threads[index];
        }

        self.threads.push(AgentThread {
            id: thread_id.to_string(),
            title: if title.trim().is_empty() {
                "Conversation".to_string()
            } else {
                title.to_string()
            },
            created_at: now_millis(),
            updated_at: now_millis(),
            messages: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
        });

        let index = self.threads.len().saturating_sub(1);
        &mut self.threads[index]
    }

    fn take_thread(&mut self, thread_id: &str) -> Option<AgentThread> {
        self.threads
            .iter()
            .position(|thread| thread.id == thread_id)
            .map(|index| self.threads.remove(index))
    }

    fn select_thread_by_id(&mut self, thread_id: &str) {
        if let Some(index) = self.threads.iter().position(|thread| thread.id == thread_id) {
            self.selected_thread = index;
        }
    }

    fn selected_thread(&self) -> Option<&AgentThread> {
        self.threads.get(self.selected_thread)
    }

    fn make_message(role: MessageRole, content: String) -> AgentMessage {
        AgentMessage {
            role,
            content,
            reasoning: None,
            tool_name: None,
            tool_arguments: None,
            tool_call_id: None,
            tool_status: None,
            input_tokens: 0,
            output_tokens: 0,
            tps: None,
            generation_ms: None,
            cost: None,
            is_streaming: false,
            timestamp: now_millis(),
        }
    }

    fn add_output(&mut self, content: &str, kind: OutputKind) {
        self.output.push(OutputLine {
            timestamp: now_millis(),
            kind,
            content: content.to_string(),
        });
        if self.output.len() > 500 {
            self.output.remove(0);
        }
    }

    fn view_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let layout = mission_layout(self.width, self.height);
        let width = layout.header.width as usize;
        let height = layout.status.y.saturating_add(1) as usize;

        let threads_w = layout.threads.width as usize;
        let chat_w = layout.chat.width as usize;
        let mission_w = layout.mission.width as usize;
        let body_h = layout.chat.height as usize;

        let threads_label = self
            .pane_registry
            .get(PaneId::Threads)
            .map(|pane| pane.title)
            .unwrap_or("Threads");
        let chat_label = self
            .pane_registry
            .get(PaneId::Chat)
            .map(|pane| pane.title)
            .unwrap_or("Conversation");
        let active_overlay = self
            .modal_stack
            .top()
            .map(|overlay| format!("{:?}", overlay))
            .unwrap_or_else(|| "none".to_string());
        let running_tasks = self
            .tasks
            .iter()
            .filter(|task| {
                matches!(
                    task.status,
                    Some(TaskStatus::InProgress | TaskStatus::Queued | TaskStatus::AwaitingApproval)
                )
            })
            .count();
        let config_label = self
            .agent_config
            .as_ref()
            .map(|cfg| {
                format!(
                    "{}/{}:{}",
                    cfg.provider,
                    cfg.model,
                    if cfg.reasoning_effort.is_empty() {
                        "-"
                    } else {
                        cfg.reasoning_effort.as_str()
                    }
                )
            })
            .unwrap_or_else(|| "unconfigured".to_string());
        let selected_goal_label = self
            .goal_runs
            .get(self.selected_goal_run)
            .map(|goal| {
                format!(
                    "{}:{}",
                    truncate_inline(&goal.title, 20),
                    goal.status.map(format_goal_status).unwrap_or("unknown")
                )
            })
            .unwrap_or_else(|| "none".to_string());

        let splash = now_millis().saturating_sub(self.booted_at) < 1600
            && self.threads.is_empty()
            && self.goal_runs.is_empty();
        if splash {
            return render_startup_splash(width, height, self.connected);
        }

        lines.push(pad(
            &format!(
                "TAMUX [{}]  sess={}  mode={:?}  focus={:?}  ovl={}({})  theme={}",
                if self.connected { "connected" } else { "disconnected" },
                self.default_session_id
                    .clone()
                    .unwrap_or_else(|| "binding...".to_string()),
                self.input_mode,
                self.focus,
                self.modal_stack.len(),
                active_overlay,
                self.theme.name,
            ),
            width,
        ));

        lines.push(pad(
            &format!(
                "slash /help /view /goal /approval /model /todo  |  run={}/{} goals={} appr={} hb={} models={} cfg={} goal={} palette={}/{}/{}/{}/{}",
                running_tasks,
                self.tasks.len(),
                self.goal_runs.len(),
                self.pending_approval_ids.len(),
                self.heartbeat_items.len(),
                self.fetched_models.len(),
                config_label,
                selected_goal_label,
                self.theme.border.short(),
                self.theme.accent.short(),
                self.theme.focus.short(),
                self.theme.warning.short(),
                self.theme.danger.short(),
            ),
            width,
        ));

        let body_content_h = body_h.saturating_sub(2);
        let show_floating_palette = self.modal_stack.top() == Some(ModalKind::CommandPalette);

        let thread_rows = self.render_thread_rows(body_content_h);
        let chat_rows = match self.modal_stack.top() {
            Some(modal) if !show_floating_palette => {
                self.render_overlay_rows(modal, body_content_h, chat_w.saturating_sub(2))
            }
            _ => self.render_chat_rows(body_content_h, chat_w.saturating_sub(2)),
        };
        let mission_rows = self.render_goal_panel_rows(body_content_h, mission_w.saturating_sub(2));

        let threads_panel = render_panel_box(
            &format!(" {} ({}) ", threads_label, self.threads.len()),
            &thread_rows,
            threads_w,
            body_h,
            self.focus == FocusArea::Threads,
        );
        let chat_panel = render_panel_box(
            &match self.modal_stack.top() {
                Some(modal) => format!(" Overlay {:?} ", modal),
                None => format!(
                    " {} [{}] ",
                    chat_label,
                    format_transcript_mode(self.transcript_mode)
                ),
            },
            &chat_rows,
            chat_w,
            body_h,
            self.focus == FocusArea::Chat,
        );
        let mission_panel = render_panel_box(
            &format!(" Mission [{}] ", format_mission_mode(self.mission_mode)),
            &mission_rows,
            mission_w,
            body_h,
            self.focus == FocusArea::Mission,
        );

        for row in 0..body_h {
            let left = threads_panel.get(row).cloned().unwrap_or_default();
            let right = chat_panel.get(row).cloned().unwrap_or_default();
            let mission = mission_panel.get(row).cloned().unwrap_or_default();
            lines.push(format!("{} | {} | {}", left, right, mission));
        }

        let compose_armed = if self.force_new_chat {
            "fresh-thread"
        } else {
            "selected-thread"
        };
        let session_label = self.default_session_id.as_deref().unwrap_or("none");

        lines.push(pad(
            &format!(
                "composer [{}] target={} session={}",
                match self.input_mode {
                    InputMode::Insert => "insert",
                    InputMode::Normal => "normal",
                },
                compose_armed,
                session_label,
            ),
            width,
        ));

        let mut input_line = self.input.clone();
        if self.input_mode == InputMode::Insert {
            input_line.push('|');
        }
        lines.push(pad(&format!("> {}", truncate_inline(&input_line, width.saturating_sub(2))), width));

        let latest_output = self
            .output
            .last()
            .map(|line| {
                let label = match line.kind {
                    OutputKind::Info => "info",
                    OutputKind::Success => "ok",
                    OutputKind::Warning => "warn",
                    OutputKind::Error => "error",
                    OutputKind::Tool => "tool",
                };
                format!(
                    "{} {}: {}",
                    format_time(line.timestamp),
                    label,
                    compact_excerpt(&line.content, 100)
                )
            })
            .unwrap_or_else(|| "ready".to_string());

        lines.push(pad(
            &format!("{} | {}", self.status, latest_output),
            width,
        ));

        if show_floating_palette {
            lines = overlay_centered_palette(lines, width, height, self.input.as_str());
        }

        while lines.len() < height {
            lines.push(String::new());
        }

        lines
    }

    fn render_thread_rows(&self, body_h: usize) -> Vec<String> {
        if self.threads.is_empty() {
            return vec!["(no threads yet)".to_string()];
        }

        let mut rows = vec![format!("showing {} threads", self.threads.len())];
        let start = list_start(self.threads.len(), self.selected_thread, body_h.saturating_sub(1).max(1));
        for (index, thread) in self
            .threads
            .iter()
            .enumerate()
            .skip(start)
            .take(body_h.saturating_sub(1))
        {
            let marker = if index == self.selected_thread { ">" } else { " " };
            let activity = thread_activity_glyph(thread);
            let pending = if self.pending_local_thread_id.as_deref() == Some(thread.id.as_str()) {
                "*"
            } else {
                ""
            };
            let excerpt = last_thread_excerpt(thread);
            rows.push(format!(
                "{}{}{} {} | {}",
                marker,
                activity,
                pending,
                truncate_inline(&thread.title, 18),
                truncate_inline(&excerpt, 20)
            ));
        }

        rows
    }

    fn render_chat_rows(&self, body_h: usize, chat_w: usize) -> Vec<String> {
        let Some(thread) = self.selected_thread() else {
            let mut home = vec![
                "TAMUX OPERATIONS CONSOLE".to_string(),
                "".to_string(),
                "Open slash commands: /help".to_string(),
                "Create chat: /new then type prompt".to_string(),
                "Switch views: /view threads|chat|mission|todo".to_string(),
                "Goal controls: /goal next|prev|pause|resume|cancel".to_string(),
                "Approvals: /approval view|approve|deny".to_string(),
                "Models: /model fetch|next|prev|apply|view".to_string(),
                "".to_string(),
                "Mouse: click pane to focus, wheel to scroll.".to_string(),
            ];

            let mut rows = home
                .drain(..)
                .flat_map(|line| wrap_line(&line, chat_w.max(8)))
                .collect::<Vec<_>>();
            rows.truncate(body_h.max(1));
            return rows;
        };

        let mut rows = vec![format!(
            "thread={} msgs={} tok={} scroll={} mode={}",
            truncate_inline(&thread.title, 24),
            thread.messages.len(),
            thread.total_input_tokens + thread.total_output_tokens,
            self.chat_scroll,
            format_transcript_mode(self.transcript_mode),
        )];

        let mut transcript = build_transcript_rows(thread, self.transcript_mode, chat_w);
        transcript = collapse_repeated_lines(transcript);

        if transcript.is_empty() {
            transcript.push("(empty transcript)".to_string());
        }

        let visible = body_h.saturating_sub(1).max(1);
        let max_scroll = transcript.len().saturating_sub(visible);
        let start = if self.chat_scroll == 0 {
            max_scroll
        } else {
            max_scroll.saturating_sub(self.chat_scroll.min(max_scroll))
        };

        let visible_rows = transcript
            .into_iter()
            .skip(start)
            .take(visible)
            .map(|line| truncate_inline(&line, chat_w.max(8)))
            .collect::<Vec<_>>();

        rows.extend(visible_rows);
        rows
    }

    fn render_goal_panel_rows(&self, body_h: usize, mission_w: usize) -> Vec<String> {
        let mut rows = Vec::new();

        if self.mission_mode == MissionMode::Todo {
            rows.push("todo tracker".to_string());
            rows.extend(self.render_todo_rows());
            return mission_rows_window(rows, body_h, mission_w, self.mission_scroll);
        }

        let done_count = self
            .todo_items
            .iter()
            .filter(|item| item.status == TodoStatus::Completed)
            .count();
        rows.push(format!(
            "todo {}/{} (cursor {})",
            done_count,
            self.todo_items.len(),
            self.todo_cursor + 1
        ));
        for (idx, item) in self.todo_items.iter().enumerate() {
            let marker = if idx == self.todo_cursor { ">" } else { " " };
            rows.push(format!(
                "{} [{}] {}",
                marker,
                format_todo_status(item.status),
                truncate_inline(&item.title, 34)
            ));
        }
        rows.push("".to_string());
        rows.push("goal detail".to_string());

        let Some(goal) = self.goal_runs.get(self.selected_goal_run) else {
            rows.push("No goal run selected".to_string());
            rows.push("Use /goal next or /goal prev".to_string());
            return mission_rows_window(rows, body_h, mission_w, self.mission_scroll);
        };

        let status = goal.status.map(format_goal_status).unwrap_or("unknown");
        let completed_steps = goal
            .steps
            .iter()
            .filter(|step| step.status == Some(GoalRunStepStatus::Completed))
            .count();
        let total_steps = goal.steps.len();
        let step_progress = if total_steps == 0 {
            "-".to_string()
        } else {
            format!("{}/{}", completed_steps, total_steps)
        };

        rows.push(format!("{} [{}]", truncate_inline(&goal.title, 28), status));
        rows.push(format!("goal: {}", truncate_inline(&goal.goal, 46)));
        rows.push(format!(
            "step {} · active {} · approvals {}",
            step_progress,
            goal.child_task_count,
            goal.approval_count
        ));

        if let Some(current_step) = &goal.current_step_title {
            rows.push(format!("now: {}", truncate_inline(current_step, 46)));
        }

        if let Some(error) = &goal.last_error {
            rows.push(format!("error: {}", truncate_inline(error, 46)));
        }

        if let Some(reflection) = &goal.reflection_summary {
            rows.push(format!(
                "reflection: {}",
                truncate_inline(reflection, 46)
            ));
        }

        if !goal.memory_updates.is_empty() {
            rows.push(format!("memory updates: {}", goal.memory_updates.len()));
        }

        if let Some(skill) = &goal.generated_skill_path {
            rows.push(format!("skill: {}", truncate_inline(skill, 46)));
        }

        match self.mission_mode {
            MissionMode::Summary => {
                rows.push("summary view".to_string());
                rows.extend(render_goal_timeline_rows(goal, mission_w, 6));
                rows.push("events:".to_string());
                rows.extend(render_goal_event_rows(goal, 4));
            }
            MissionMode::Timeline => {
                rows.push("timeline view".to_string());
                rows.extend(render_goal_timeline_rows(goal, mission_w, usize::MAX));
            }
            MissionMode::Approvals => {
                rows.push("approvals view".to_string());
                rows.extend(self.render_goal_approval_rows(goal, mission_w));
            }
            MissionMode::Events => {
                rows.push("events view".to_string());
                rows.extend(render_goal_event_rows(goal, usize::MAX));
            }
            MissionMode::Todo => {}
        }

        mission_rows_window(rows, body_h, mission_w, self.mission_scroll)
    }

    fn render_todo_rows(&self) -> Vec<String> {
        let done_count = self
            .todo_items
            .iter()
            .filter(|item| item.status == TodoStatus::Completed)
            .count();
        let mut rows = vec![format!(
            "todo {}/{} (cursor {})",
            done_count,
            self.todo_items.len(),
            self.todo_cursor + 1
        )];
        for (idx, item) in self.todo_items.iter().enumerate() {
            let marker = if idx == self.todo_cursor { ">" } else { " " };
            rows.push(format!(
                "{} [{}] {}",
                marker,
                format_todo_status(item.status),
                truncate_inline(&item.title, 34)
            ));
        }
        rows
    }

    fn render_goal_approval_rows(&self, goal: &GoalRun, width: usize) -> Vec<String> {
        let mut rows = Vec::new();

        let mut approval_tasks = self
            .tasks
            .iter()
            .filter(|task| {
                task.awaiting_approval_id.is_some()
                    && (task.goal_run_id.as_deref() == Some(goal.id.as_str())
                        || task.goal_run_id.is_none())
            })
            .collect::<Vec<_>>();
        approval_tasks.sort_by(|left, right| left.title.cmp(&right.title));

        rows.push(format!(
            "pending approvals: {}",
            approval_tasks.len()
        ));
        if approval_tasks.is_empty() {
            rows.push("(none)".to_string());
            rows.push("Use /approval view for global queue.".to_string());
            return rows;
        }

        for task in approval_tasks {
            let approval_id = task
                .awaiting_approval_id
                .as_deref()
                .unwrap_or("unknown");
            rows.push(format!(
                "- {}",
                truncate_inline(&format!("{} [{}]", task.title, approval_id), width.saturating_sub(2).max(8))
            ));
            if let Some(reason) = &task.blocked_reason {
                rows.push(format!(
                    "  reason: {}",
                    truncate_inline(reason, width.saturating_sub(10).max(8))
                ));
            }
            if let Some(step) = &task.goal_step_title {
                rows.push(format!(
                    "  step: {}",
                    truncate_inline(step, width.saturating_sub(8).max(8))
                ));
            }
        }
        rows.push("/approval approve  /approval deny".to_string());
        rows
    }

    fn render_overlay_rows(&self, modal: ModalKind, body_h: usize, chat_w: usize) -> Vec<String> {
        let mut rows = Vec::new();

        match modal {
            ModalKind::CommandPalette => {
                rows.push("Slash Command Browser".to_string());
                rows.push("/help".to_string());
                rows.push("/new".to_string());
                rows.push("/refresh threads|services|all".to_string());
                rows.push("/view chat compact|tools|full".to_string());
                rows.push("/view mission summary|timeline|approvals|events|todo".to_string());
                rows.push("/view threads|provider|model|plugin|mcp|skills|session|approval|commands".to_string());
                rows.push("/goal next|prev|pause|resume|cancel|retry|rerun|view".to_string());
                rows.push("/approval approve|deny|view".to_string());
                rows.push("/model fetch|next|prev|apply|view".to_string());
                rows.push("/plugin list|next|prev|inspect|refresh|view".to_string());
                rows.push("/mcp list|next|prev|inspect|refresh|view".to_string());
                rows.push("/skills list|next|prev|inspect|refresh|view".to_string());
                rows.push("/todo list|next|prev|doing|done|view".to_string());
                rows.push("Tip: click this hint bar to auto-start '/' input.".to_string());
            }
            ModalKind::ProviderPicker => {
                rows.push("Provider Profile".to_string());
                if let Some(config) = &self.agent_config {
                    rows.push(format!("provider: {}", config.provider));
                    rows.push(format!("model: {}", config.model));
                    rows.push(format!("reasoning: {}", config.reasoning_effort));
                } else {
                    rows.push("No config snapshot yet. Press g to sync services.".to_string());
                }
            }
            ModalKind::ThreadPicker => {
                rows.push(format!("Thread Picker ({})", self.threads.len()));
                for (idx, thread) in self.threads.iter().take(body_h.saturating_sub(2)).enumerate() {
                    let marker = if idx == self.selected_thread { ">" } else { " " };
                    rows.push(format!(
                        "{} {}",
                        marker,
                        truncate_inline(&thread.title, chat_w.saturating_sub(4).max(8))
                    ));
                }
            }
            ModalKind::SessionPicker => {
                rows.push("Session Picker".to_string());
                rows.push(format!(
                    "default session: {}",
                    self.default_session_id
                        .clone()
                        .unwrap_or_else(|| "not bound".to_string())
                ));
                rows.push(format!("heartbeat checks: {}", self.heartbeat_items.len()));
                for item in self.heartbeat_items.iter().take(body_h.saturating_sub(4)) {
                    let state = item
                        .last_result
                        .map(|result| format!("{:?}", result).to_lowercase())
                        .unwrap_or_else(|| "never".to_string());
                    rows.push(format!("- {} [{}]", item.label, state));
                }
            }
            ModalKind::ModelPicker => {
                rows.push("Model Picker".to_string());
                if let Some(config) = &self.agent_config {
                    rows.push(format!("active: {}/{}", config.provider, config.model));
                }
                rows.push("Use /model next|prev|apply (or arrows + enter).".to_string());
                if self.fetched_models.is_empty() {
                    rows.push("No fetched models yet. Use /model fetch.".to_string());
                } else {
                    for (idx, model) in self
                        .fetched_models
                        .iter()
                        .take(body_h.saturating_sub(4))
                        .enumerate()
                    {
                        let marker = if idx == self.selected_model_index { ">" } else { " " };
                        let label = model.name.as_deref().unwrap_or(model.id.as_str());
                        let context = model
                            .context_window
                            .map(|value| format!("{}", value))
                            .unwrap_or_else(|| "?".to_string());
                        rows.push(format!(
                            "{} {} ({})",
                            marker,
                            truncate_inline(label, chat_w.saturating_sub(14).max(10)),
                            context
                        ));
                    }
                }
            }
            ModalKind::PluginPicker => {
                rows.push(format!("Plugin Picker ({})", self.plugin_items.len()));
                rows.push("/plugin next|prev|inspect|refresh".to_string());
                rows.extend(render_picker_rows(
                    &self.plugin_items,
                    self.plugin_cursor,
                    body_h.saturating_sub(2),
                    chat_w,
                ));
            }
            ModalKind::McpPicker => {
                rows.push(format!("MCP Picker ({})", self.mcp_items.len()));
                rows.push("/mcp next|prev|inspect|refresh".to_string());
                rows.extend(render_picker_rows(
                    &self.mcp_items,
                    self.mcp_cursor,
                    body_h.saturating_sub(2),
                    chat_w,
                ));
            }
            ModalKind::SkillsPicker => {
                rows.push(format!("Skills Picker ({})", self.skills_items.len()));
                rows.push("/skills next|prev|inspect|refresh".to_string());
                rows.extend(render_picker_rows(
                    &self.skills_items,
                    self.skills_cursor,
                    body_h.saturating_sub(2),
                    chat_w,
                ));
            }
            ModalKind::ApprovalOverlay => {
                rows.push(format!(
                    "Approval Queue ({})",
                    self.pending_approval_ids.len()
                ));
                if self.pending_approval_ids.is_empty() {
                    rows.push("No pending approvals".to_string());
                } else {
                    rows.push("Use /approval approve or /approval deny".to_string());
                    for approval_id in self.pending_approval_ids.iter().take(body_h.saturating_sub(4)) {
                        rows.push(format!("- {}", approval_id));
                    }
                }
            }
        }

        let mut visible_rows = rows
            .into_iter()
            .flat_map(|line| wrap_line(&line, chat_w.max(10)))
            .collect::<Vec<_>>();
        visible_rows.truncate(body_h.max(1));
        visible_rows
    }
}

impl StringModel for TuiModel {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        self.send_daemon_command(DaemonCommand::Refresh);
        self.send_daemon_command(DaemonCommand::RefreshServices);
        Cmd::tick(Duration::from_millis(80))
    }

    fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message> {
        match msg {
            Msg::Event(event) => self.handle_event(event),
        }
    }

    fn view_string(&self) -> String {
        self.view_lines().join("\n")
    }
}

fn rect_to_area(rect: Rect) -> (u16, u16, u16, u16) {
    (rect.x, rect.y, rect.width, rect.height)
}

fn click_list_index(
    area: (u16, u16, u16, u16),
    y: u16,
    total: usize,
    selection: usize,
    item_height: usize,
) -> Option<usize> {
    if total == 0 {
        return None;
    }

    let (ax, ay, aw, ah) = area;
    let _ = ax;
    let _ = aw;

    if y < ay || y >= ay.saturating_add(ah) {
        return None;
    }

    let capacity = ((ah as usize) / item_height.max(1)).max(1);
    let start = list_start(total, selection, capacity);
    let row = (y.saturating_sub(ay) as usize) / item_height.max(1);
    Some((start + row).min(total.saturating_sub(1)))
}

fn list_start(total: usize, selection: usize, capacity: usize) -> usize {
    if total == 0 || capacity == 0 {
        return 0;
    }

    if selection < capacity {
        0
    } else {
        selection
            .saturating_sub(capacity.saturating_sub(1))
            .min(total.saturating_sub(capacity))
    }
}

fn offset_index(current: usize, len: usize, delta: isize) -> usize {
    if len == 0 {
        return 0;
    }

    let len = len as isize;
    let current = current as isize;
    let next = (current + delta).clamp(0, len - 1);
    next as usize
}

fn merge_thread(existing: &AgentThread, mut incoming: AgentThread) -> AgentThread {
    if incoming.messages.is_empty() || incoming.messages.len() < existing.messages.len() {
        incoming.messages = existing.messages.clone();
    }

    incoming.total_input_tokens = incoming.total_input_tokens.max(existing.total_input_tokens);
    incoming.total_output_tokens = incoming.total_output_tokens.max(existing.total_output_tokens);
    incoming.updated_at = incoming.updated_at.max(existing.updated_at);

    if incoming.title.trim().is_empty() {
        incoming.title = existing.title.clone();
    }

    incoming
}

fn choose_max_f64(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (None, value) => value,
        (value, None) => value,
    }
}

#[allow(dead_code)]
fn render_message_lines(message: &AgentMessage) -> Vec<String> {
    let mut lines = Vec::new();
    let role_label = match message.role {
        MessageRole::System => "SYSTEM",
        MessageRole::User => "USER",
        MessageRole::Assistant => "ASSISTANT",
        MessageRole::Tool => "TOOL",
        MessageRole::Unknown => "MESSAGE",
    };

    let mut metrics = vec![format!("{} in / {} out", message.input_tokens, message.output_tokens)];
    if let Some(tps) = message.tps {
        metrics.push(format!("{:.1} tok/s", tps));
    }
    if let Some(ms) = message.generation_ms {
        metrics.push(format!("{} ms", ms));
    }
    if let Some(cost) = message.cost {
        metrics.push(format!("${:.4}", cost));
    }

    lines.push(format!(
        "[{}] {} · {}",
        role_label,
        format_time(message.timestamp),
        metrics.join(" · ")
    ));

    if message.role == MessageRole::Tool {
        let name = message.tool_name.as_deref().unwrap_or("tool");
        let status = message.tool_status.as_deref().unwrap_or("requested");
        let elapsed = if status == "executing" {
            format!(" · {}s", tool_elapsed_seconds(message.timestamp))
        } else {
            String::new()
        };
        lines.push(format!("{} [{}]{}", name, status, elapsed));

        if let Some(arguments) = &message.tool_arguments {
            let args = single_line(sanitize_for_display(arguments));
            if !args.trim().is_empty() {
                lines.push(format!("args {}", truncate_inline(&args, 140)));
            }
        }

        let result = single_line(sanitize_for_display(&message.content));
        if !result.trim().is_empty() && status != "executing" {
            lines.push(format!("result {}", truncate_inline(&result, 160)));
        }

        lines.push("details collapsed by default · open Activity for full traces".to_string());
        return lines;
    }

    if !message.content.trim().is_empty() {
        let mut content = sanitize_for_display(&message.content);
        if message.is_streaming {
            content.push('▌');
        }
        lines.push(content);
    } else if message.is_streaming {
        lines.push("waiting for response…".to_string());
    }

    if let Some(reasoning) = &message.reasoning {
        let reasoning = sanitize_for_display(reasoning);
        if !reasoning.trim().is_empty() {
            lines.push("reasoning".to_string());
            lines.push(reasoning);
        }
    }

    lines
}

fn last_thread_excerpt(thread: &AgentThread) -> String {
    if let Some(message) = thread
        .messages
        .iter()
        .rev()
        .find(|message| !message.content.trim().is_empty())
    {
        let summary = compact_excerpt(&sanitize_for_display(&message.content), 56);
        if is_boilerplate_prompt(&summary) {
            return "queued task".to_string();
        }
        return summary;
    }

    if let Some(tool) = thread
        .messages
        .iter()
        .rev()
        .find(|message| message.role == MessageRole::Tool)
    {
        let name = tool.tool_name.as_deref().unwrap_or("tool");
        let status = tool.tool_status.as_deref().unwrap_or("requested");
        return format!("{} [{}]", name, status);
    }

    "No content yet".to_string()
}

fn thread_activity_glyph(thread: &AgentThread) -> &'static str {
    if let Some(last) = thread.messages.last() {
        if last.is_streaming {
            return "~";
        }
        return match last.role {
            MessageRole::User => "U",
            MessageRole::Assistant => "A",
            MessageRole::Tool => "T",
            MessageRole::System => "S",
            MessageRole::Unknown => "?",
        };
    }
    "-"
}

fn render_message_compact_line(message: &AgentMessage, width: usize) -> String {
    let role = match message.role {
        MessageRole::System => "S",
        MessageRole::User => "U",
        MessageRole::Assistant => "A",
        MessageRole::Tool => "T",
        MessageRole::Unknown => "?",
    };

    let mut body = if message.role == MessageRole::Tool {
        let name = message.tool_name.as_deref().unwrap_or("tool");
        let status = message.tool_status.as_deref().unwrap_or("requested");
        let result = compact_excerpt(&sanitize_for_display(&message.content), 120);
        if result.is_empty() {
            format!("{} [{}]", name, status)
        } else {
            format!("{} [{}] {}", name, status, result)
        }
    } else {
        let mut content = compact_excerpt(&sanitize_for_display(&message.content), 120);
        if content.is_empty() && message.is_streaming {
            content = "waiting for response".to_string();
        }
        if content.is_empty() {
            content = "(empty)".to_string();
        }
        if is_boilerplate_prompt(&content) {
            content = "queued task".to_string();
        }
        content
    };

    if message.is_streaming {
        body.push_str(" [stream]");
    }

    truncate_inline(
        &format!("[{}] {}", role, body),
        width.max(8),
    )
}

fn build_transcript_rows(thread: &AgentThread, mode: TranscriptMode, width: usize) -> Vec<String> {
    let mut rows: Vec<(String, String)> = Vec::new();

    for message in &thread.messages {
        match message.role {
            MessageRole::Tool => {
                if matches!(mode, TranscriptMode::Compact) {
                    let key = format!(
                        "tool:{}:{}",
                        message.tool_call_id.as_deref().unwrap_or("-"),
                        message.tool_name.as_deref().unwrap_or("tool")
                    );
                    let line = render_tool_grouped_line(message, width);
                    if let Some((last_key, last_line)) = rows.last_mut() {
                        if *last_key == key {
                            *last_line = line;
                            continue;
                        }
                    }
                    rows.push((key, line));
                } else {
                    rows.push((
                        format!("raw:{}", rows.len()),
                        render_message_compact_line(message, width),
                    ));
                }
            }
            MessageRole::Assistant => {
                if mode != TranscriptMode::Full {
                    let cleaned = sanitize_for_display(&message.content);
                    if cleaned.trim().is_empty() && !message.is_streaming {
                        continue;
                    }
                }
                rows.push((
                    format!("raw:{}", rows.len()),
                    render_message_compact_line(message, width),
                ));
            }
            _ => {
                rows.push((
                    format!("raw:{}", rows.len()),
                    render_message_compact_line(message, width),
                ));
            }
        }
    }

    if mode == TranscriptMode::Tools {
        rows.retain(|(_, line)| line.starts_with("[T]"));
    }

    rows.into_iter().map(|(_, line)| line).collect()
}

fn render_tool_grouped_line(message: &AgentMessage, width: usize) -> String {
    let name = message.tool_name.as_deref().unwrap_or("tool");
    let status = message.tool_status.as_deref().unwrap_or("requested");
    let args = message
        .tool_arguments
        .as_deref()
        .map(sanitize_for_display)
        .unwrap_or_default();
    let result = sanitize_for_display(&message.content);

    let mut payload = String::new();
    if !args.trim().is_empty() {
        payload.push_str(&format!(" args:{}", compact_excerpt(&args, 28)));
    }
    if !result.trim().is_empty() {
        payload.push_str(&format!(" => {}", compact_excerpt(&result, 48)));
    }

    truncate_inline(
        &format!(
            "[T] {} [{}]{}",
            name,
            status,
            if payload.is_empty() { "" } else { payload.as_str() }
        ),
        width.max(8),
    )
}

fn is_boilerplate_prompt(content: &str) -> bool {
    let normalized = content.to_ascii_lowercase();
    normalized.contains("execute the following queued task")
}

fn collapse_repeated_lines(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() {
        return lines;
    }

    let mut out = Vec::new();
    let mut current = lines[0].clone();
    let mut count = 1usize;

    for line in lines.into_iter().skip(1) {
        if line == current {
            count += 1;
        } else {
            if count > 1 {
                out.push(format!("{} x{}", current, count));
            } else {
                out.push(current);
            }
            current = line;
            count = 1;
        }
    }

    if count > 1 {
        out.push(format!("{} x{}", current, count));
    } else {
        out.push(current);
    }

    out
}

fn render_panel_box(
    title: &str,
    rows: &[String],
    width: usize,
    height: usize,
    focused: bool,
) -> Vec<String> {
    let width = width.max(8);
    let height = height.max(3);
    let fill = if focused { '=' } else { '-' };
    let corner = if focused { '#' } else { '+' };
    let mut lines = Vec::with_capacity(height);

    let mut top = truncate_inline(&format!(" {} ", title.trim()), width.saturating_sub(2));
    let top_len = top.chars().count();
    if top_len < width.saturating_sub(2) {
        top.push_str(&fill.to_string().repeat(width.saturating_sub(2) - top_len));
    }
    lines.push(format!("{}{}{}", corner, top, corner));

    let content_h = height.saturating_sub(2);
    for idx in 0..content_h {
        let content = rows.get(idx).map(String::as_str).unwrap_or("");
        lines.push(format!("|{}|", pad(content, width.saturating_sub(2))));
    }

    lines.push(format!(
        "{}{}{}",
        corner,
        fill.to_string().repeat(width.saturating_sub(2)),
        corner
    ));
    lines
}

fn render_startup_splash(width: usize, height: usize, connected: bool) -> Vec<String> {
    let mut lines = vec![" ".repeat(width); height.max(1)];
    let logo = [
        "  _______   ___   __  __  _   _  __  __ ",
        " |__   __| / _ \\ |  \\/  || | | ||  \\/  |",
        "    | |   | | | || \\  / || | | || \\  / |",
        "    | |   | |_| || |\\/| || |_| || |\\/| |",
        "    |_|    \\___/ |_|  |_| \\___/ |_|  |_|",
    ];
    let subtitle = if connected {
        "Mission control connected. Booting operator surfaces..."
    } else {
        "Waiting for daemon handshake..."
    };
    let hint = "Tip: use wide terminal (>= 120 cols) for full mission layout.";

    let total = logo.len() + 2;
    let start = height.saturating_sub(total) / 2;

    for (idx, row) in logo.iter().enumerate() {
        let line_idx = start + idx;
        if line_idx < lines.len() {
            lines[line_idx] = centered_line(width, row);
        }
    }

    if start + logo.len() < lines.len() {
        lines[start + logo.len()] = centered_line(width, subtitle);
    }
    if start + logo.len() + 1 < lines.len() {
        lines[start + logo.len() + 1] = centered_line(width, hint);
    }

    lines
}

fn centered_line(width: usize, text: &str) -> String {
    let clipped = truncate_inline(text, width);
    let clipped_len = clipped.chars().count();
    if clipped_len >= width {
        return clipped;
    }

    let left = (width - clipped_len) / 2;
    format!("{}{}", " ".repeat(left), clipped)
}

fn overlay_centered_palette(
    mut base_lines: Vec<String>,
    screen_w: usize,
    screen_h: usize,
    input: &str,
) -> Vec<String> {
    if base_lines.is_empty() || screen_w < 40 || screen_h < 10 {
        return base_lines;
    }

    let query = input
        .trim()
        .strip_prefix('/')
        .unwrap_or(input.trim())
        .to_ascii_lowercase();

    let commands = slash_command_catalog()
        .into_iter()
        .filter(|(name, _)| query.is_empty() || name.contains(&query))
        .collect::<Vec<_>>();

    let card_w = (screen_w.saturating_sub(18)).clamp(48, 96);
    let card_h = (screen_h.saturating_sub(8)).clamp(10, 18);

    let mut rows = vec![
        format!("filter: /{}", query),
        "".to_string(),
        format!("{:26} {}", "command", "description"),
    ];

    for (name, desc) in commands.iter().take(card_h.saturating_sub(6)) {
        rows.push(format!("/{:25} {}", truncate_inline(name, 25), desc));
    }

    if commands.is_empty() {
        rows.push("(no commands match filter)".to_string());
    }

    rows.push("".to_string());
    rows.push("enter executes slash command in composer".to_string());

    let panel = render_panel_box(" Slash Commands ", &rows, card_w, card_h, true);
    let top = (screen_h.saturating_sub(card_h)) / 2;
    let left = (screen_w.saturating_sub(card_w)) / 2;

    let max_rows = panel.len().min(screen_h.saturating_sub(top));
    for row in 0..max_rows {
        if let Some(target) = base_lines.get_mut(top + row) {
            *target = overlay_at(target, panel[row].as_str(), left);
        }
    }

    base_lines
}

fn overlay_at(target: &str, overlay: &str, x: usize) -> String {
    let mut chars = target.chars().collect::<Vec<_>>();
    for (idx, ch) in overlay.chars().enumerate() {
        let pos = x + idx;
        if pos >= chars.len() {
            break;
        }
        chars[pos] = ch;
    }
    chars.into_iter().collect()
}

fn slash_command_catalog() -> Vec<(&'static str, &'static str)> {
    vec![
        ("help", "open slash command browser"),
        ("new", "arm a new conversation"),
        ("refresh", "refresh threads + services"),
        ("refresh threads", "refresh thread list"),
        ("refresh services", "refresh goals/tasks/config"),
        ("view threads", "open thread picker view"),
        ("view chat compact", "focus conversation compact mode"),
        ("view chat tools", "focus conversation grouped-tool mode"),
        ("view chat full", "focus conversation verbose mode"),
        ("view mission summary", "focus mission summary"),
        ("view mission timeline", "focus mission step timeline"),
        ("view mission approvals", "focus mission approvals"),
        ("view mission events", "focus mission event feed"),
        ("view mission todo", "focus mission todo tracker"),
        ("view provider", "open provider profile"),
        ("view model", "open model picker"),
        ("view plugin", "open plugin picker"),
        ("view mcp", "open MCP picker"),
        ("view skills", "open skills picker"),
        ("view session", "open session picker"),
        ("view approval", "open approval overlay"),
        ("view todo", "focus todo tracker in mission pane"),
        ("goal next", "select next goal run"),
        ("goal prev", "select previous goal run"),
        ("goal pause", "pause selected goal run"),
        ("goal resume", "resume selected goal run"),
        ("goal cancel", "cancel selected goal run"),
        ("goal retry", "retry selected goal run"),
        ("goal rerun", "rerun selected goal run"),
        ("approval view", "show approval queue"),
        ("approval approve", "approve first pending approval"),
        ("approval deny", "deny first pending approval"),
        ("model fetch", "fetch models for active provider"),
        ("model next", "select next fetched model"),
        ("model prev", "select previous fetched model"),
        ("model apply", "apply selected model"),
        ("plugin list", "open plugin inventory"),
        ("plugin next", "select next plugin"),
        ("plugin prev", "select previous plugin"),
        ("plugin inspect", "inspect selected plugin"),
        ("plugin refresh", "refresh plugin inventory"),
        ("mcp list", "open MCP inventory"),
        ("mcp next", "select next MCP server"),
        ("mcp prev", "select previous MCP server"),
        ("mcp inspect", "inspect selected MCP server"),
        ("mcp refresh", "refresh MCP inventory"),
        ("skills list", "open skills inventory"),
        ("skills next", "select next skill"),
        ("skills prev", "select previous skill"),
        ("skills inspect", "inspect selected skill"),
        ("skills refresh", "refresh skills inventory"),
        ("todo view", "open todo tracker"),
        ("todo next", "select next todo item"),
        ("todo prev", "select previous todo item"),
        ("todo doing", "mark current todo in progress"),
        ("todo done", "mark current todo completed"),
    ]
}

fn wrap_line(line: &str, width: usize) -> Vec<String> {
    if width <= 1 {
        return vec![line.to_string()];
    }

    let clean = sanitize_for_display(line);
    if clean.chars().count() <= width {
        return vec![clean];
    }

    let mut rows = Vec::new();
    let mut current = String::new();
    for ch in clean.chars() {
        current.push(ch);
        if current.chars().count() >= width {
            rows.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }

    rows
}

fn pad(content: &str, width: usize) -> String {
    let mut line = truncate_inline(content, width);
    let current = line.chars().count();
    if current < width {
        line.push_str(&" ".repeat(width - current));
    }
    line
}

fn truncate_inline(content: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let mut chars = content.chars();
    let clipped: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        if max_chars > 3 {
            let mut short: String = clipped.chars().take(max_chars - 3).collect();
            short.push_str("...");
            short
        } else {
            clipped
        }
    } else {
        clipped
    }
}

fn compact_excerpt(content: &str, max_chars: usize) -> String {
    let single = content
        .replace('\n', " ")
        .replace('\r', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    truncate_inline(&single, max_chars)
}

#[allow(dead_code)]
fn single_line(content: String) -> String {
    content
        .lines()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_for_display(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' && chars.peek().copied() == Some('\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        if ch.is_control() && ch != '\n' && ch != '\t' {
            continue;
        }

        output.push(ch);
    }

    output
}

fn format_time(ms: u64) -> String {
    if ms == 0 {
        return "--:--:--".to_string();
    }
    let secs = ms / 1000;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn format_goal_status(status: GoalRunStatus) -> &'static str {
    match status {
        GoalRunStatus::Queued => "queued",
        GoalRunStatus::Planning => "planning",
        GoalRunStatus::Running => "running",
        GoalRunStatus::AwaitingApproval => "awaiting_approval",
        GoalRunStatus::Paused => "paused",
        GoalRunStatus::Completed => "completed",
        GoalRunStatus::Failed => "failed",
        GoalRunStatus::Cancelled => "cancelled",
    }
}

fn format_transcript_mode(mode: TranscriptMode) -> &'static str {
    match mode {
        TranscriptMode::Compact => "compact",
        TranscriptMode::Tools => "tools",
        TranscriptMode::Full => "full",
    }
}

fn format_mission_mode(mode: MissionMode) -> &'static str {
    match mode {
        MissionMode::Summary => "summary",
        MissionMode::Timeline => "timeline",
        MissionMode::Approvals => "approvals",
        MissionMode::Events => "events",
        MissionMode::Todo => "todo",
    }
}

fn render_goal_timeline_rows(goal: &GoalRun, width: usize, limit: usize) -> Vec<String> {
    let mut rows = vec!["steps:".to_string()];
    for step in goal.steps.iter().take(limit) {
        let marker = if step.position == goal.current_step_index {
            ">"
        } else {
            " "
        };
        let status = step
            .status
            .map(format_goal_step_status)
            .unwrap_or("pending");
        rows.push(format!(
            "{} {}. {} [{} {}]",
            marker,
            step.position + 1,
            truncate_inline(&step.title, 28),
            format_goal_step_kind(step.kind),
            status
        ));
        if let Some(summary) = &step.summary {
            rows.push(format!(
                "  {}",
                truncate_inline(summary, width.saturating_sub(4).max(8))
            ));
        }
        if let Some(error) = &step.error {
            rows.push(format!(
                "  err: {}",
                truncate_inline(error, width.saturating_sub(8).max(8))
            ));
        }
    }
    rows
}

fn render_goal_event_rows(goal: &GoalRun, limit: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for event in goal.events.iter().rev().take(limit) {
        rows.push(format!(
            "- {} {}",
            truncate_inline(&event.phase, 10),
            truncate_inline(&event.message, 32)
        ));
    }
    if rows.is_empty() {
        rows.push("(no events)".to_string());
    }
    rows
}

fn mission_rows_window(rows: Vec<String>, body_h: usize, mission_w: usize, mission_scroll: usize) -> Vec<String> {
    let visible_rows = rows
        .into_iter()
        .flat_map(|line| wrap_line(&line, mission_w.max(10)))
        .collect::<Vec<_>>();

    let visible = body_h.max(1);
    let max_scroll = visible_rows.len().saturating_sub(visible);
    let start = if mission_scroll == 0 {
        max_scroll
    } else {
        max_scroll.saturating_sub(mission_scroll.min(max_scroll))
    };
    visible_rows.into_iter().skip(start).take(visible).collect()
}

fn render_picker_rows(items: &[String], cursor: usize, limit: usize, width: usize) -> Vec<String> {
    if items.is_empty() {
        return vec!["(none discovered)".to_string()];
    }

    let mut rows = Vec::new();
    let visible = limit.max(1);
    let start = list_start(items.len(), cursor, visible);

    for (idx, item) in items.iter().enumerate().skip(start).take(visible) {
        let marker = if idx == cursor { ">" } else { " " };
        rows.push(format!(
            "{} {}",
            marker,
            truncate_inline(item, width.saturating_sub(4).max(8))
        ));
    }

    rows
}

fn selected_item(items: &[String], cursor: usize) -> String {
    items
        .get(cursor)
        .cloned()
        .or_else(|| items.first().cloned())
        .unwrap_or_else(|| "none".to_string())
}

fn discover_plugin_items() -> Vec<String> {
    let mut out = Vec::new();

    out.extend(list_files_without_ext("frontend/src/plugins", &["ts", "tsx"]));
    out.extend(list_files_without_ext("src/plugins", &["rs", "toml", "yaml", "yml"]));

    if out.is_empty() {
        out.extend([
            "code-review".to_string(),
            "component-refactoring".to_string(),
            "connection".to_string(),
            "workspace-tools".to_string(),
        ]);
    }

    normalize_unique_sorted(out)
}

fn discover_mcp_items() -> Vec<String> {
    let mut out = Vec::new();

    if let Ok(content) = fs::read_to_string("frontend/config/mcporter.json") {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            if let Some(obj) = json.as_object() {
                out.extend(obj.keys().cloned());
            } else if let Some(array) = json.as_array() {
                for item in array {
                    if let Some(name) = item.get("name").and_then(Value::as_str) {
                        out.push(name.to_string());
                    }
                }
            }
        }
    }

    out.extend(list_dirs("docs/skills/connection"));

    if out.is_empty() {
        out.extend([
            "native-mcp".to_string(),
            "mcporter".to_string(),
            "filesystem".to_string(),
            "github".to_string(),
        ]);
    }

    normalize_unique_sorted(out)
}

fn discover_skills_items() -> Vec<String> {
    let mut out = Vec::new();

    out.extend(list_dirs("docs/skills"));
    out.extend(list_files_without_ext("docs/skills", &["md"]));

    if out.is_empty() {
        out.extend([
            "building".to_string(),
            "connection".to_string(),
            "operating".to_string(),
        ]);
    }

    normalize_unique_sorted(out)
}

fn list_dirs(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

fn list_files_without_ext(path: &str, allowed_exts: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_type = match entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }

            let p = entry.path();
            let ext = p.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if !allowed_exts.iter().any(|candidate| candidate.eq_ignore_ascii_case(ext)) {
                continue;
            }

            if let Some(stem) = p.file_stem().and_then(|v| v.to_str()) {
                out.push(stem.to_string());
            }
        }
    }
    out
}

fn normalize_unique_sorted(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort_by(|left, right| left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()));
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
}

fn default_todos() -> Vec<UiTodo> {
    vec![
        UiTodo {
            title: "Architecture foundation modules".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Daemon transport interactivity".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Intelligent slash router + command views".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Mission-control shell + boxed layout".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Transcript and tool-row parity".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Goal timeline + approval UX depth".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Pickers and ecosystem manager surfaces".to_string(),
            status: TodoStatus::Completed,
        },
        UiTodo {
            title: "Frontend parity closure".to_string(),
            status: TodoStatus::NotStarted,
        },
        UiTodo {
            title: "Deterministic render + reducer tests".to_string(),
            status: TodoStatus::Completed,
        },
    ]
}

fn format_todo_status(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::NotStarted => "todo",
        TodoStatus::InProgress => "doing",
        TodoStatus::Completed => "done",
    }
}

fn format_goal_step_kind(kind: GoalRunStepKind) -> &'static str {
    match kind {
        GoalRunStepKind::Reason => "reason",
        GoalRunStepKind::Command => "command",
        GoalRunStepKind::Research => "research",
        GoalRunStepKind::Memory => "memory",
        GoalRunStepKind::Skill => "skill",
        GoalRunStepKind::Unknown => "unknown",
    }
}

fn format_goal_step_status(status: GoalRunStepStatus) -> &'static str {
    match status {
        GoalRunStepStatus::Pending => "pending",
        GoalRunStepStatus::InProgress => "running",
        GoalRunStepStatus::Completed => "done",
        GoalRunStepStatus::Failed => "failed",
        GoalRunStepStatus::Skipped => "skipped",
    }
}

fn get_json_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn get_provider_field(value: &Value, provider_id: &str, key: &str) -> Option<String> {
    value
        .get("providers")
        .and_then(Value::as_object)
        .and_then(|providers| providers.get(provider_id))
        .and_then(Value::as_object)
        .and_then(|provider| provider.get(key))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

#[allow(dead_code)]
fn tool_elapsed_seconds(timestamp_ms: u64) -> u64 {
    now_millis().saturating_sub(timestamp_ms) / 1000
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AgentMessage, AgentThread, MessageRole};

    #[test]
    fn collapse_repeated_lines_compacts_runs() {
        let input = vec![
            "alpha".to_string(),
            "alpha".to_string(),
            "beta".to_string(),
            "beta".to_string(),
            "beta".to_string(),
            "gamma".to_string(),
        ];

        let output = collapse_repeated_lines(input);
        assert_eq!(
            output,
            vec![
                "alpha x2".to_string(),
                "beta x3".to_string(),
                "gamma".to_string(),
            ]
        );
    }

    #[test]
    fn transcript_compact_groups_tool_updates_by_call() {
        let thread = AgentThread {
            id: "t1".to_string(),
            title: "demo".to_string(),
            messages: vec![
                AgentMessage {
                    role: MessageRole::User,
                    content: "run check".to_string(),
                    ..AgentMessage::default()
                },
                AgentMessage {
                    role: MessageRole::Tool,
                    tool_name: Some("bash_command".to_string()),
                    tool_call_id: Some("call-1".to_string()),
                    tool_status: Some("running".to_string()),
                    content: "".to_string(),
                    ..AgentMessage::default()
                },
                AgentMessage {
                    role: MessageRole::Tool,
                    tool_name: Some("bash_command".to_string()),
                    tool_call_id: Some("call-1".to_string()),
                    tool_status: Some("done".to_string()),
                    content: "ok".to_string(),
                    ..AgentMessage::default()
                },
            ],
            ..AgentThread::default()
        };

        let rows = build_transcript_rows(&thread, TranscriptMode::Compact, 80);
        let tool_rows = rows
            .iter()
            .filter(|row| row.starts_with("[T] bash_command"))
            .collect::<Vec<_>>();

        assert_eq!(tool_rows.len(), 1);
        assert!(tool_rows[0].contains("done"));
    }

    #[test]
    fn panel_box_snapshot_is_stable() {
        let rows = vec!["row one".to_string(), "row two".to_string()];
        let panel = render_panel_box("Snapshot", &rows, 18, 5, true);
        let rendered = panel.join("\n");

        let expected = "# Snapshot ======#\n|row one         |\n|row two         |\n|                |\n#================#";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn centered_palette_overlay_contains_title() {
        let base = vec![" ".repeat(80); 22];
        let with_overlay = overlay_centered_palette(base, 80, 22, "/view");
        let joined = with_overlay.join("\n");
        assert!(joined.contains("Slash Commands"));
        assert!(joined.contains("filter: /view"));
    }

    #[test]
    fn ecosystem_discovery_has_fallback_inventory() {
        assert!(!discover_plugin_items().is_empty());
        assert!(!discover_mcp_items().is_empty());
        assert!(!discover_skills_items().is_empty());
    }
}
