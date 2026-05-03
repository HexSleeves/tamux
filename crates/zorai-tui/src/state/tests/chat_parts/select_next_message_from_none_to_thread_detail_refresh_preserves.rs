fn state_with_messages(count: usize) -> ChatState {
    let mut state = ChatState::new();
    let messages: Vec<AgentMessage> = (0..count)
        .map(|index| AgentMessage {
            role: MessageRole::User,
            content: format!("msg {}", index),
            ..Default::default()
        })
        .collect();
    let thread = AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        messages,
        ..Default::default()
    };
    state.reduce(ChatAction::ThreadListReceived(vec![thread]));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state
}

#[test]
fn select_next_message_from_none() {
    let mut state = state_with_messages(3);
    assert_eq!(state.selected_message(), None);
    state.select_next_message();
    assert_eq!(state.selected_message(), Some(0));
}

#[test]
fn select_next_message_advances() {
    let mut state = state_with_messages(3);
    state.select_next_message();
    state.select_next_message();
    assert_eq!(state.selected_message(), Some(1));
}

#[test]
fn select_next_message_clamps_at_end() {
    let mut state = state_with_messages(2);
    state.select_message(Some(1));
    state.select_next_message();
    assert_eq!(state.selected_message(), Some(1));
}

#[test]
fn select_prev_message_from_none() {
    let mut state = state_with_messages(3);
    state.select_prev_message();
    assert_eq!(state.selected_message(), Some(2));
}

#[test]
fn select_prev_message_decreases() {
    let mut state = state_with_messages(3);
    state.select_message(Some(2));
    state.select_prev_message();
    assert_eq!(state.selected_message(), Some(1));
}

#[test]
fn thread_detail_latest_page_sets_window_metadata() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(thread.total_message_count, 120);
    assert_eq!(thread.loaded_message_start, 70);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.messages.len(), 50);
}

#[test]
fn older_thread_page_prepends_into_loaded_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 70,
        messages: (20..70)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(thread.loaded_message_start, 20);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.messages.len(), 100);
    assert_eq!(
        thread
            .messages
            .first()
            .and_then(|message| message.id.as_deref()),
        Some("msg-20")
    );
    assert_eq!(
        thread
            .messages
            .last()
            .and_then(|message| message.id.as_deref()),
        Some("msg-119")
    );
}

#[test]
fn older_thread_page_preserves_live_context_window_metadata() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        active_context_window_start: Some(105),
        active_context_window_end: Some(120),
        active_context_window_tokens: Some(4_800),
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 70,
        active_context_window_start: Some(20),
        active_context_window_end: Some(70),
        active_context_window_tokens: Some(98_000),
        messages: (20..70)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(thread.loaded_message_start, 20);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.active_context_window_start, Some(105));
    assert_eq!(thread.active_context_window_end, Some(120));
    assert_eq!(thread.active_context_window_tokens, Some(4_800));
}

#[test]
fn stale_pre_compaction_thread_detail_does_not_replace_compacted_context_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 121,
        loaded_message_start: 0,
        loaded_message_end: 121,
        active_context_window_start: Some(0),
        active_context_window_end: Some(121),
        active_context_window_tokens: Some(336_000),
        messages: (0..121)
            .map(|index| AgentMessage {
                id: Some(format!("old-{index}")),
                role: MessageRole::User,
                content: format!("old {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::CompactionApplied {
        thread_id: "t1".into(),
        active_compaction_window_start: 20,
        total_message_count: 122,
    });
    state.reduce(ChatAction::ContextWindowUpdated {
        thread_id: "t1".into(),
        active_context_window_start: 20,
        active_context_window_end: 122,
        active_context_window_tokens: 4_800,
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 121,
        loaded_message_start: 0,
        loaded_message_end: 121,
        active_context_window_start: Some(0),
        active_context_window_end: Some(121),
        active_context_window_tokens: Some(336_000),
        messages: (0..121)
            .map(|index| AgentMessage {
                id: Some(format!("old-{index}")),
                role: MessageRole::User,
                content: format!("old {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(thread.active_compaction_window_start, Some(20));
    assert_eq!(thread.active_context_window_start, Some(20));
    assert_eq!(thread.active_context_window_end, Some(122));
    assert_eq!(thread.active_context_window_tokens, Some(4_800));
}

#[test]
fn older_thread_page_does_not_replace_active_responder_metadata() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Dola".into()),
        profile_provider: Some("chutes".into()),
        profile_model: Some("zaj-org/GLM-5.1-TEE".into()),
        profile_reasoning_effort: Some("xhigh".into()),
        profile_context_window_tokens: Some(128_000),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Svarog".into()),
        profile_provider: Some("openai".into()),
        profile_model: Some("gpt-5.4".into()),
        profile_reasoning_effort: Some("medium".into()),
        profile_context_window_tokens: Some(400_000),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 70,
        messages: (20..70)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(thread.agent_name.as_deref(), Some("Dola"));
    assert_eq!(thread.profile_provider.as_deref(), Some("chutes"));
    assert_eq!(thread.profile_model.as_deref(), Some("zaj-org/GLM-5.1-TEE"));
    assert_eq!(thread.profile_reasoning_effort.as_deref(), Some("xhigh"));
    assert_eq!(thread.profile_context_window_tokens, Some(128_000));
}

#[test]
fn thread_detail_refresh_replaces_window_when_total_visible_messages_shrink() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 5,
        loaded_message_start: 0,
        loaded_message_end: 5,
        messages: (0..5)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("before {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        messages: vec![
            AgentMessage {
                id: Some("msg-compaction".into()),
                role: MessageRole::Assistant,
                content: "Auto compaction applied".into(),
                message_kind: "compaction_artifact".into(),
                compaction_payload: Some("Older context compacted".into()),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-latest".into()),
                role: MessageRole::Assistant,
                content: "Latest visible reply".into(),
                author_agent_id: Some("domowoj".into()),
                author_agent_name: Some("Domowoj".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .unwrap();
    assert_eq!(thread.total_message_count, 2);
    assert_eq!(thread.loaded_message_start, 0);
    assert_eq!(thread.loaded_message_end, 2);
    assert_eq!(thread.messages.len(), 2);
    assert_eq!(
        thread.messages[0].id.as_deref(),
        Some("msg-compaction"),
        "authoritative refresh should discard stale pre-compaction rows"
    );
    assert_eq!(
        thread.messages[1].author_agent_name.as_deref(),
        Some("Domowoj"),
        "participant-authored metadata from the authoritative refresh should survive"
    );
}

#[test]
fn thread_detail_refresh_preserves_optimistic_local_tail_when_smaller_snapshot_matches() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        messages: vec![
            AgentMessage {
                id: Some("msg-0".into()),
                role: MessageRole::Assistant,
                content: "Earlier reply".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-1".into()),
                role: MessageRole::User,
                content: "Previous prompt".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "Please inspect this image\n[image attachment]".into(),
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        messages: vec![
            AgentMessage {
                id: Some("msg-0".into()),
                role: MessageRole::Assistant,
                content: "Earlier reply".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-1".into()),
                role: MessageRole::User,
                content: "Previous prompt".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .unwrap();
    assert_eq!(thread.messages.len(), 3);
    assert_eq!(
        thread
            .messages
            .last()
            .map(|message| message.content.as_str()),
        Some("Please inspect this image\n[image attachment]"),
        "stale detail refresh should not erase the optimistic local tail"
    );
    assert_eq!(thread.total_message_count, 3);
    assert_eq!(thread.loaded_message_end, 3);
}

