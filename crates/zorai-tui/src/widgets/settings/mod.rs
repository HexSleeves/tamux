use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use zorai_protocol::has_whatsapp_allowed_contacts;

use crate::providers;
use crate::state::concierge::ConciergeState;
use crate::state::config::ConfigState;
use crate::state::modal::{ModalState, WhatsAppLinkPhase};
use crate::state::settings::{PluginListItem, PluginSettingsState, SettingsState, SettingsTab};
use crate::state::subagents::SubAgentsState;
use crate::theme::ThemeTokens;
use crate::widgets::message::wrap_text;

include!("render_edit_buffer_with_cursor_to_editing_cursor_hit_test_to_content.rs");
include!("advanced_single_line_edit_layout_to_subagent_row_action_offsets.rs");
include!("render_provider_tab_to_render_tools_tab.rs");
include!("render_websearch_tab.rs");
include!("render_chat_tab_to_render_honcho_editor_actions.rs");
include!("wrap_textarea_visual_line_to_render_wrapped_textarea_buffer_to_render.rs");
include!("render_concierge_tab_to_render_feature_toggle_line.rs");
include!("render_features_tab.rs");
include!("render_advanced_value_to_render_advanced_tab.rs");
include!("render_gateway_text_field.rs");
include!("render_auth_tab_to_render_agent_tab.rs");
include!("render_plugins_tab_to_connector_readiness_style.rs");
include!("render_about_tab.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::config::ConfigState;
    use crate::state::modal::ModalState;
    use crate::state::settings::SettingsState;

    include!("tests/settings_handles_empty_state_to_auth_tab_shows_chatgpt_logout.rs");
}
