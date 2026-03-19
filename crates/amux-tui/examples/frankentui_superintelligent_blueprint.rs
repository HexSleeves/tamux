#![allow(dead_code)]

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FxColor {
    Default,
    DimWhite,
    BrightWhite,
    NeonCyan,
    WarmOrange,
    BrightRed,
}

#[derive(Debug, Clone)]
struct Theme {
    bg_main: FxColor,
    fg_text: FxColor,
    fg_active: FxColor,
    accent_primary: FxColor,
    accent_secondary: FxColor,
    accent_warn: FxColor,
}

impl Theme {
    fn retro_hacker() -> Self {
        Self {
            bg_main: FxColor::Default,
            fg_text: FxColor::DimWhite,
            fg_active: FxColor::BrightWhite,
            accent_primary: FxColor::NeonCyan,
            accent_secondary: FxColor::WarmOrange,
            accent_warn: FxColor::BrightRed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Chat,
    Sidebar,
    Input,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SidebarTab {
    Tasks,
    Subagents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranscriptMode {
    Compact,
    Tools,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalKind {
    None,
    CommandPalette,
    ApprovalAlert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
struct HeaderState {
    agent_label: String,
    model_label: String,
    usage_label: String,
}

#[derive(Debug, Clone)]
struct ComposerState {
    input: String,
    multiline: bool,
}

#[derive(Debug, Clone)]
struct MessageVm {
    role: String,
    text: String,
    reasoning: Option<String>,
    reasoning_open: bool,
    tool_name: Option<String>,
    tool_status: Option<String>,
    streaming: bool,
}

#[derive(Debug, Clone)]
struct TaskVm {
    id: String,
    title: String,
    status: TaskStatus,
    depth: usize,
}

#[derive(Debug, Clone)]
struct SubagentVm {
    name: String,
    state: String,
    progress_pct: u8,
}

#[derive(Debug, Clone)]
struct CommandItem {
    command: String,
    description: String,
}

#[derive(Debug, Clone)]
struct ApprovalVm {
    command_text: String,
    risk_level: RiskLevel,
    blast_radius: String,
}

#[derive(Debug, Clone)]
struct ShellState {
    theme: Theme,
    focus: FocusPane,
    sidebar_tab: SidebarTab,
    transcript_mode: TranscriptMode,
    modal: ModalKind,
    header: HeaderState,
    composer: ComposerState,
    messages: Vec<MessageVm>,
    tasks: Vec<TaskVm>,
    subagents: Vec<SubagentVm>,
    command_query: String,
    command_items: Vec<CommandItem>,
    command_selected: usize,
    approval: Option<ApprovalVm>,
    chat_scroll: usize,
    sidebar_scroll: usize,
}

impl ShellState {
    fn new() -> Self {
        Self {
            theme: Theme::retro_hacker(),
            focus: FocusPane::Input,
            sidebar_tab: SidebarTab::Tasks,
            transcript_mode: TranscriptMode::Compact,
            modal: ModalKind::None,
            header: HeaderState {
                agent_label: "[Sisyphus (Ultraworker)]".to_string(),
                model_label: "GLM-4.6 Z.AI Coding Plan".to_string(),
                usage_label: "12.1k tok | $0.93".to_string(),
            },
            composer: ComposerState {
                input: String::new(),
                multiline: false,
            },
            messages: Vec::new(),
            tasks: Vec::new(),
            subagents: Vec::new(),
            command_query: String::new(),
            command_items: default_command_items(),
            command_selected: 0,
            approval: None,
            chat_scroll: 0,
            sidebar_scroll: 0,
        }
    }

    fn render(&self) -> Node {
        let header = render_header(&self.header, &self.theme);
        let left_chat = render_chat_view(self);
        let right_sidebar = render_sidebar(self);
        let main = Node::row(vec![
            left_chat.with_flex(7),
            right_sidebar.with_flex(3),
        ]);
        let footer = render_footer(self);

        let base = Node::column(vec![header, main.with_flex(1), footer]);

        match self.modal {
            ModalKind::CommandPalette => {
                let modal = render_command_modal(self);
                Node::overlay(base, modal)
            }
            ModalKind::ApprovalAlert => {
                let modal = render_approval_alert(self);
                Node::overlay(base, modal)
            }
            ModalKind::None => base,
        }
    }
}

fn render_header(header: &HeaderState, _theme: &Theme) -> Node {
    Node::panel_rounded(
        "header",
        vec![Node::row(vec![
            Node::text(&header.agent_label).with_flex(3),
            Node::text_center(&header.model_label).with_flex(4),
            Node::text_right(&header.usage_label).with_flex(3),
        ])],
    )
}

fn render_footer(state: &ShellState) -> Node {
    let prompt = if state.composer.multiline {
        "> multiline"
    } else {
        "> ask anything"
    };

    Node::panel_rounded(
        "footer",
        vec![
            Node::text(&format!("{} {}", prompt, state.composer.input)),
            Node::text("tab: focus  ctrl+p: commands  ?: help  j/k: nav  ctrl+d/u: page"),
        ],
    )
}

fn render_chat_view(state: &ShellState) -> Node {
    let mut rows = Vec::new();

    for msg in &state.messages {
        rows.push(render_chat_message(msg, state.transcript_mode));
    }

    if rows.is_empty() {
        rows.push(Node::ascii_art(
            "\n  HERMES-AGENT\n  Retro-Hacker Mission Console\n",
        ));
    }

    Node::panel_rounded("chat", rows).with_focus(state.focus == FocusPane::Chat)
}

fn render_chat_message(msg: &MessageVm, mode: TranscriptMode) -> Node {
    let role = Node::badge(&msg.role);

    let body = match mode {
        TranscriptMode::Compact => Node::text(&msg.text),
        TranscriptMode::Tools => {
            if msg.tool_name.is_some() {
                Node::text(&format!(
                    "{} [{}]",
                    msg.tool_name.clone().unwrap_or_else(|| "tool".to_string()),
                    msg.tool_status
                        .clone()
                        .unwrap_or_else(|| "running".to_string())
                ))
            } else {
                Node::text("")
            }
        }
        TranscriptMode::Full => Node::text(&msg.text),
    };

    let mut children = vec![Node::row(vec![role, body.with_flex(1)])];

    if let Some(reasoning) = &msg.reasoning {
        if msg.reasoning_open {
            children.push(Node::details_open("Reasoning", reasoning));
        } else {
            children.push(Node::details_closed("Reasoning (collapsed)"));
        }
    }

    if let Some(tool) = &msg.tool_name {
        let status = msg
            .tool_status
            .clone()
            .unwrap_or_else(|| "running".to_string());
        children.push(Node::status_line(&format!("{} [{}]", tool, status)));
    }

    Node::column(children)
}

fn render_sidebar(state: &ShellState) -> Node {
    let tabs = Node::tabs(vec!["Tasks", "Subagents"], state.sidebar_tab as usize);
    let body = match state.sidebar_tab {
        SidebarTab::Tasks => render_task_tree(&state.tasks),
        SidebarTab::Subagents => render_subagents(&state.subagents),
    };

    Node::panel_rounded("context", vec![tabs, body]).with_focus(state.focus == FocusPane::Sidebar)
}

fn render_task_tree(tasks: &[TaskVm]) -> Node {
    let mut rows = Vec::new();

    for task in tasks {
        let indent = "  ".repeat(task.depth);
        let status = match task.status {
            TaskStatus::Pending => "[ ]",
            TaskStatus::Running => "[~]",
            TaskStatus::Done => "[x]",
            TaskStatus::Failed => "[!]",
        };
        rows.push(Node::text(&format!("{}{} {}", indent, status, task.title)));
    }

    Node::column(rows)
}

fn render_subagents(subagents: &[SubagentVm]) -> Node {
    let rows = subagents
        .iter()
        .map(|a| {
            Node::text(&format!(
                "{}  {}  {}%",
                a.name, a.state, a.progress_pct
            ))
        })
        .collect::<Vec<_>>();

    Node::column(rows)
}

fn render_command_modal(state: &ShellState) -> Node {
    let mut rows = vec![Node::input(&format!("/{}", state.command_query))];

    for (idx, item) in state.command_items.iter().enumerate().take(12) {
        let active = idx == state.command_selected;
        let line = if active {
            format!("> /{}  --  {}", item.command, item.description)
        } else {
            format!("  /{}  --  {}", item.command, item.description)
        };
        rows.push(Node::text(&line));
    }

    Node::modal("command-palette", rows)
}

fn render_approval_alert(state: &ShellState) -> Node {
    let approval = match &state.approval {
        Some(v) => v,
        None => {
            return Node::modal("approval", vec![Node::text("No approval payload")]);
        }
    };

    let risk = match approval.risk_level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    };

    Node::modal(
        "approval",
        vec![
            Node::text(&format!("[{} RISK]", risk)),
            Node::text(&format!("Command: {}", approval.command_text)),
            Node::text(&format!("Blast Radius: {}", approval.blast_radius)),
            Node::text("[Y] Allow once   [A] Allow for session   [N] Reject"),
        ],
    )
}

#[derive(Debug, Clone)]
enum UiAction {
    FocusNext,
    FocusPrev,
    OpenCommandPalette,
    CloseModal,
    SidebarTasks,
    SidebarSubagents,
    ScrollChat(i32),
    ScrollSidebar(i32),
    SubmitInput,
    InsertChar(char),
    ToggleReasoning(usize),
    SelectCommandNext,
    SelectCommandPrev,
    ExecuteCommand,
    ApprovalAllowOnce,
    ApprovalAllowSession,
    ApprovalReject,
}

#[derive(Debug, Clone)]
enum IpcCommand {
    SubmitPrompt { text: String },
    ExecuteSlash { command: String },
    RequestProjectionRefresh,
    ResolveApproval { decision: String },
}

#[derive(Debug, Clone)]
enum DaemonEvent {
    StreamDelta { text: String },
    ToolStatus { name: String, status: String },
    TaskSnapshot { tasks: Vec<TaskVm> },
    SubagentSnapshot { subagents: Vec<SubagentVm> },
    ApprovalRequired { payload: ApprovalVm },
}

trait DaemonIpc {
    fn send(&mut self, cmd: IpcCommand);
    fn poll_events(&mut self) -> VecDeque<DaemonEvent>;
}

fn reduce(state: &mut ShellState, action: UiAction, ipc: &mut dyn DaemonIpc) {
    match action {
        UiAction::FocusNext => {
            state.focus = match state.focus {
                FocusPane::Chat => FocusPane::Sidebar,
                FocusPane::Sidebar => FocusPane::Input,
                FocusPane::Input => FocusPane::Chat,
                FocusPane::Modal => FocusPane::Input,
            };
        }
        UiAction::FocusPrev => {
            state.focus = match state.focus {
                FocusPane::Chat => FocusPane::Input,
                FocusPane::Sidebar => FocusPane::Chat,
                FocusPane::Input => FocusPane::Sidebar,
                FocusPane::Modal => FocusPane::Input,
            };
        }
        UiAction::OpenCommandPalette => {
            state.modal = ModalKind::CommandPalette;
            state.focus = FocusPane::Modal;
        }
        UiAction::CloseModal => {
            state.modal = ModalKind::None;
            state.focus = FocusPane::Input;
        }
        UiAction::SidebarTasks => state.sidebar_tab = SidebarTab::Tasks,
        UiAction::SidebarSubagents => state.sidebar_tab = SidebarTab::Subagents,
        UiAction::ScrollChat(delta) => {
            if delta > 0 {
                state.chat_scroll = state.chat_scroll.saturating_add(delta as usize);
            } else {
                state.chat_scroll = state.chat_scroll.saturating_sub((-delta) as usize);
            }
        }
        UiAction::ScrollSidebar(delta) => {
            if delta > 0 {
                state.sidebar_scroll = state.sidebar_scroll.saturating_add(delta as usize);
            } else {
                state.sidebar_scroll = state.sidebar_scroll.saturating_sub((-delta) as usize);
            }
        }
        UiAction::SubmitInput => {
            let text = state.composer.input.trim().to_string();
            if text.starts_with('/') {
                ipc.send(IpcCommand::ExecuteSlash {
                    command: text.trim_start_matches('/').to_string(),
                });
            } else if !text.is_empty() {
                ipc.send(IpcCommand::SubmitPrompt { text });
            }
            state.composer.input.clear();
        }
        UiAction::InsertChar(c) => state.composer.input.push(c),
        UiAction::ToggleReasoning(_idx) => {
            // In production, toggle message reasoning_open by message index.
        }
        UiAction::SelectCommandNext => {
            let len = state.command_items.len();
            if len > 0 {
                state.command_selected = (state.command_selected + 1).min(len - 1);
            }
        }
        UiAction::SelectCommandPrev => {
            state.command_selected = state.command_selected.saturating_sub(1);
        }
        UiAction::ExecuteCommand => {
            if let Some(cmd) = state.command_items.get(state.command_selected) {
                ipc.send(IpcCommand::ExecuteSlash {
                    command: cmd.command.clone(),
                });
            }
        }
        UiAction::ApprovalAllowOnce => ipc.send(IpcCommand::ResolveApproval {
            decision: "allow_once".to_string(),
        }),
        UiAction::ApprovalAllowSession => ipc.send(IpcCommand::ResolveApproval {
            decision: "allow_session".to_string(),
        }),
        UiAction::ApprovalReject => ipc.send(IpcCommand::ResolveApproval {
            decision: "reject".to_string(),
        }),
    }
}

fn apply_daemon_event(state: &mut ShellState, event: DaemonEvent) {
    match event {
        DaemonEvent::StreamDelta { text } => {
            if let Some(last) = state.messages.last_mut() {
                last.text.push_str(&text);
                last.streaming = true;
            }
        }
        DaemonEvent::ToolStatus { name, status } => {
            state.messages.push(MessageVm {
                role: "TOOL".to_string(),
                text: String::new(),
                reasoning: None,
                reasoning_open: false,
                tool_name: Some(name),
                tool_status: Some(status),
                streaming: false,
            });
        }
        DaemonEvent::TaskSnapshot { tasks } => state.tasks = tasks,
        DaemonEvent::SubagentSnapshot { subagents } => state.subagents = subagents,
        DaemonEvent::ApprovalRequired { payload } => {
            state.approval = Some(payload);
            state.modal = ModalKind::ApprovalAlert;
            state.focus = FocusPane::Modal;
        }
    }
}

#[derive(Debug, Clone)]
enum CtEvent {
    Key { code: KeyCode, ctrl: bool, shift: bool },
    MouseScrollUp,
    MouseScrollDown,
    Resize { width: u16, height: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    Esc,
    Up,
    Down,
    PageUp,
    PageDown,
}

fn map_crossterm_to_action(event: CtEvent, state: &ShellState) -> Option<UiAction> {
    match event {
        CtEvent::Key { code: KeyCode::Tab, .. } => Some(UiAction::FocusNext),
        CtEvent::Key {
            code: KeyCode::BackTab,
            ..
        } => Some(UiAction::FocusPrev),
        CtEvent::Key {
            code: KeyCode::Char('p'),
            ctrl: true,
            ..
        } => Some(UiAction::OpenCommandPalette),
        CtEvent::Key {
            code: KeyCode::Esc, ..
        } => Some(UiAction::CloseModal),
        CtEvent::Key {
            code: KeyCode::Enter,
            shift: false,
            ..
        } => Some(UiAction::SubmitInput),
        CtEvent::Key {
            code: KeyCode::Enter,
            shift: true,
            ..
        } => Some(UiAction::InsertChar('\n')),
        CtEvent::Key {
            code: KeyCode::Char('j'),
            ..
        }
        | CtEvent::Key {
            code: KeyCode::Down,
            ..
        } => {
            if state.focus == FocusPane::Sidebar {
                Some(UiAction::ScrollSidebar(1))
            } else {
                Some(UiAction::ScrollChat(1))
            }
        }
        CtEvent::Key {
            code: KeyCode::Char('k'),
            ..
        }
        | CtEvent::Key {
            code: KeyCode::Up, ..
        } => {
            if state.focus == FocusPane::Sidebar {
                Some(UiAction::ScrollSidebar(-1))
            } else {
                Some(UiAction::ScrollChat(-1))
            }
        }
        CtEvent::Key {
            code: KeyCode::Char(c),
            ctrl: false,
            ..
        } => Some(UiAction::InsertChar(c)),
        CtEvent::MouseScrollUp => {
            if state.focus == FocusPane::Sidebar {
                Some(UiAction::ScrollSidebar(-1))
            } else {
                Some(UiAction::ScrollChat(-1))
            }
        }
        CtEvent::MouseScrollDown => {
            if state.focus == FocusPane::Sidebar {
                Some(UiAction::ScrollSidebar(1))
            } else {
                Some(UiAction::ScrollChat(1))
            }
        }
        CtEvent::Resize { .. } => None,
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    text: Option<String>,
    children: Vec<Node>,
    flex: u16,
    focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Row,
    Column,
    PanelRounded,
    Overlay,
    Modal,
    Text,
    Input,
    Badge,
    Tabs,
    AsciiArt,
    DetailsClosed,
    DetailsOpen,
    StatusLine,
    TextCenter,
    TextRight,
}

impl Node {
    fn row(children: Vec<Node>) -> Self {
        Self::new(NodeKind::Row, None, children)
    }

    fn column(children: Vec<Node>) -> Self {
        Self::new(NodeKind::Column, None, children)
    }

    fn panel_rounded(title: &str, children: Vec<Node>) -> Self {
        Self::new(NodeKind::PanelRounded, Some(title.to_string()), children)
    }

    fn overlay(base: Node, modal: Node) -> Self {
        Self::new(NodeKind::Overlay, None, vec![base, modal])
    }

    fn modal(title: &str, children: Vec<Node>) -> Self {
        Self::new(NodeKind::Modal, Some(title.to_string()), children)
    }

    fn text(text: &str) -> Self {
        Self::new(NodeKind::Text, Some(text.to_string()), vec![])
    }

    fn text_center(text: &str) -> Self {
        Self::new(NodeKind::TextCenter, Some(text.to_string()), vec![])
    }

    fn text_right(text: &str) -> Self {
        Self::new(NodeKind::TextRight, Some(text.to_string()), vec![])
    }

    fn input(value: &str) -> Self {
        Self::new(NodeKind::Input, Some(value.to_string()), vec![])
    }

    fn badge(value: &str) -> Self {
        Self::new(NodeKind::Badge, Some(value.to_string()), vec![])
    }

    fn tabs(labels: Vec<&str>, selected: usize) -> Self {
        let text = labels
            .into_iter()
            .enumerate()
            .map(|(idx, value)| {
                if idx == selected {
                    format!("[{}]", value)
                } else {
                    value.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        Self::new(NodeKind::Tabs, Some(text), vec![])
    }

    fn ascii_art(text: &str) -> Self {
        Self::new(NodeKind::AsciiArt, Some(text.to_string()), vec![])
    }

    fn details_closed(text: &str) -> Self {
        Self::new(NodeKind::DetailsClosed, Some(text.to_string()), vec![])
    }

    fn details_open(title: &str, body: &str) -> Self {
        Self::new(
            NodeKind::DetailsOpen,
            Some(format!("{}\n{}", title, body)),
            vec![],
        )
    }

    fn status_line(text: &str) -> Self {
        Self::new(NodeKind::StatusLine, Some(text.to_string()), vec![])
    }

    fn with_flex(mut self, flex: u16) -> Self {
        self.flex = flex;
        self
    }

    fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn new(kind: NodeKind, text: Option<String>, children: Vec<Node>) -> Self {
        Self {
            kind,
            text,
            children,
            flex: 0,
            focused: false,
        }
    }
}

fn default_command_items() -> Vec<CommandItem> {
    vec![
        CommandItem {
            command: "provider".to_string(),
            description: "Switch LLM backend".to_string(),
        },
        CommandItem {
            command: "model".to_string(),
            description: "Switch active model".to_string(),
        },
        CommandItem {
            command: "tools".to_string(),
            description: "Toggle toolsets".to_string(),
        },
        CommandItem {
            command: "effort".to_string(),
            description: "Set reasoning effort".to_string(),
        },
        CommandItem {
            command: "view chat".to_string(),
            description: "Switch chat view mode".to_string(),
        },
        CommandItem {
            command: "view tasks".to_string(),
            description: "Switch sidebar to task tree".to_string(),
        },
    ]
}

fn main() {
    let _ = ShellState::new().render();
}
