use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::hash::{Hash, Hasher};

use crate::app::RecentActionVm;
use crate::state::chat::GatewayStatusVm;
use crate::state::chat::{ChatState, MessageRole};
use crate::state::sidebar::{SidebarState, SidebarTab};
use crate::state::task::TaskState;
use crate::state::tier::TierState;
use crate::theme::ThemeTokens;

