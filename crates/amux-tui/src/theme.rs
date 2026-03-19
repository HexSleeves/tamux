#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Success,
    Warning,
    Danger,
    Mission,
}

impl Tone {
    pub fn short(self) -> &'static str {
        match self {
            Self::Neutral => "n",
            Self::Success => "ok",
            Self::Warning => "warn",
            Self::Danger => "err",
            Self::Mission => "mission",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeTokens {
    pub name: String,
    pub border: Tone,
    pub accent: Tone,
    pub focus: Tone,
    pub warning: Tone,
    pub danger: Tone,
}

impl ThemeTokens {
    pub fn mission_control_default() -> Self {
        Self {
            name: "mission-control-rich".to_string(),
            border: Tone::Neutral,
            accent: Tone::Mission,
            focus: Tone::Success,
            warning: Tone::Warning,
            danger: Tone::Danger,
        }
    }
}
