use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::message::wrap_text;
use crate::state::chat::{
    AgentMessage, ChatHitTarget, ChatState, MessageRole, RetryPhase, TranscriptMode,
};
use crate::theme::ThemeTokens;

const MESSAGE_PADDING_X: usize = 2;
const MESSAGE_PADDING_Y: usize = 1;
const TOGGLE_BUTTON_HIT_WIDTH: usize = 2;
const SCROLLBAR_WIDTH: u16 = 1;

#[cfg(test)]
thread_local! {
    static BUILD_RENDERED_LINES_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

include!("render_streaming_markdown_to_message_block_style_to_message_action.rs");
include!("build_rendered_lines_to_build_visible_window_from_snapshot_to_apply.rs");
include!("resolved_scroll_to_highlight_line_range_to_selected_text_to_selection.rs");
include!("selection_point_from_snapshot_to_render.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::chat::{AgentThread, ChatAction, MessageRole};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn chat_with_messages(messages: Vec<AgentMessage>) -> ChatState {
        let mut chat = ChatState::new();
        chat.reduce(ChatAction::ThreadCreated {
            thread_id: "t1".into(),
            title: "Test".into(),
        });
        chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
            id: "t1".into(),
            title: "Test".into(),
            messages,
            ..Default::default()
        }));
        chat
    }

    include!("tests/chat_handles_empty_state_to_all_file_mutation_tool_rows_use_filename.rs");
    include!("tests/compaction_artifact_lines_use_standard_message_left_padding_to_concierge.rs");
}
