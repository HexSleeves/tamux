#![allow(dead_code)]

/// TUI-side concierge state.
pub struct ConciergeState {
    pub enabled: bool,
    pub detail_level: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub welcome_content: Option<String>,
    pub welcome_actions: Vec<ConciergeActionVm>,
    pub welcome_visible: bool,
}

#[derive(Debug, Clone)]
pub struct ConciergeActionVm {
    pub label: String,
    pub action_type: String,
    pub thread_id: Option<String>,
}

impl ConciergeState {
    pub fn new() -> Self {
        Self {
            enabled: true,
            detail_level: "proactive_triage".into(),
            provider: None,
            model: None,
            welcome_content: None,
            welcome_actions: Vec::new(),
            welcome_visible: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConciergeAction {
    WelcomeReceived {
        content: String,
        actions: Vec<ConciergeActionVm>,
    },
    WelcomeDismissed,
}

impl ConciergeState {
    pub fn reduce(&mut self, action: ConciergeAction) {
        match action {
            ConciergeAction::WelcomeReceived { content, actions } => {
                self.welcome_content = Some(content);
                self.welcome_actions = actions;
                self.welcome_visible = true;
            }
            ConciergeAction::WelcomeDismissed => {
                self.welcome_content = None;
                self.welcome_actions.clear();
                self.welcome_visible = false;
            }
        }
    }
}
