#[test]
fn reasoning_renders_before_multiline_content() {
    let msg = AgentMessage {
        role: MessageRole::Assistant,
        content: "First line that wraps a bit for the test".into(),
        reasoning: Some("Let me think...".into()),
        ..Default::default()
    };
    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        20,
        &empty_expanded(),
        &empty_tools(),
    );
    let first_text: String = lines[0]
        .spans
        .iter()
        .map(|span| span.content.to_string())
        .collect();
    let second_text: String = lines[1]
        .spans
        .iter()
        .map(|span| span.content.to_string())
        .collect();
    assert!(
        first_text.contains("Reasoning"),
        "First line should be reasoning, got: {}",
        first_text
    );
    assert!(
        !second_text.contains("Reasoning"),
        "Content should start after reasoning, got: {}",
        second_text
    );
}

#[test]
fn reasoning_expandable() {
    let msg = AgentMessage {
        role: MessageRole::Assistant,
        content: "Answer".into(),
        reasoning: Some("Thinking step by step".into()),
        ..Default::default()
    };
    let collapsed = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let mut exp = empty_expanded();
    exp.insert(0);
    let expanded = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &exp,
        &empty_tools(),
    );
    assert!(
        expanded.len() > collapsed.len(),
        "Expanded should have more lines"
    );
}

#[test]
fn meta_cognition_message_collapses_by_default() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Meta-cognitive intervention: warning before tool execution.\nPlanned tool: read_file\nDetected risks:\n- overconfidence".into(),
        ..Default::default()
    };

    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let plain = plain_lines(&lines).join("\n");

    assert!(
        plain.contains("Meta-cognition"),
        "collapsed meta-cognition should show a disclosure header: {plain}"
    );
    assert!(
        !plain.contains("Planned tool: read_file") && !plain.contains("overconfidence"),
        "collapsed meta-cognition should hide details: {plain}"
    );
}

#[test]
fn meta_cognition_message_expands_with_reasoning_state() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Meta-cognitive intervention: warning before tool execution.\nPlanned tool: read_file\nDetected risks:\n- overconfidence".into(),
        ..Default::default()
    };
    let mut expanded = empty_expanded();
    expanded.insert(0);

    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &expanded,
        &empty_tools(),
    );
    let plain = plain_lines(&lines).join("\n");

    assert!(
        plain.contains("Planned tool: read_file") && plain.contains("overconfidence"),
        "expanded meta-cognition should show details: {plain}"
    );
}

#[test]
fn tools_mode_skips_non_tool_messages() {
    let msg = AgentMessage {
        role: MessageRole::User,
        content: "Hello".into(),
        ..Default::default()
    };
    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Tools,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    assert!(lines.is_empty());
}

#[test]
fn wrap_text_empty_string() {
    let lines = wrap_text("", 80);
    assert_eq!(lines, vec![""]);
}

#[test]
fn wrap_text_zero_width() {
    let lines = wrap_text("hello", 0);
    assert_eq!(lines, vec!["hello"]);
}
