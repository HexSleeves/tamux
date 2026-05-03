include!("message_parts/message.rs");
#[path = "message_markdown_table.rs"]
mod markdown_table;

include!("message_parts/format_weles_review_badge_to_render_markdown.rs");
include!("message_parts/render_markdown_segment_to_format_tool_status.rs");
include!("message_parts/wrap_text_to_split_text_by_width.rs");
#[cfg(test)]
#[path = "tests/message.rs"]
mod tests;
