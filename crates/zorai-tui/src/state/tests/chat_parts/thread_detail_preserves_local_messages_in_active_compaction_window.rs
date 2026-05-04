#[test]
fn thread_detail_preserves_local_messages_in_active_compaction_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Thread".into(),
    });
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "rule based".into(),
            message_kind: "compaction_artifact".into(),
            compaction_payload: Some("Older context compacted for continuity".into()),
            timestamp: 10,
            ..Default::default()
        },
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Local assistant follow-up".into(),
            timestamp: 20,
            ..Default::default()
        },
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "Newest local user prompt".into(),
            timestamp: 30,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "rule based".into(),
            message_kind: "compaction_artifact".into(),
            compaction_payload: Some("Older context compacted for continuity".into()),
            timestamp: 10,
            ..Default::default()
        }],
        ..Default::default()
    }));

    let thread = state.active_thread().expect("thread should exist");
    assert!(
        thread
            .messages
            .iter()
            .any(|message| message.content == "Local assistant follow-up"),
        "detail merge should preserve local assistant messages after the latest compaction artifact"
    );
    assert!(
        thread
            .messages
            .iter()
            .any(|message| message.content == "Newest local user prompt"),
        "detail merge should preserve local user messages after the latest compaction artifact"
    );
}

#[test]
fn append_message_replaces_previous_concierge_welcome() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "concierge".into(),
        title: "Concierge".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome 1".into(),
            is_concierge_welcome: true,
            ..Default::default()
        },
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome 2".into(),
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    let thread = state
        .active_thread()
        .expect("concierge thread should exist");
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].content, "Welcome 2");
    assert!(thread.messages[0].is_concierge_welcome);
}

#[test]
fn append_message_does_not_duplicate_persisted_optimistic_user_tail() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        total_message_count: 1,
        loaded_message_start: 0,
        loaded_message_end: 1,
        messages: vec![AgentMessage {
            id: Some("persisted-user".into()),
            role: MessageRole::User,
            content: "same prompt".into(),
            timestamp: 100,
            ..Default::default()
        }],
        ..Default::default()
    }));

    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "same prompt".into(),
            timestamp: 101,
            ..Default::default()
        },
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.messages.len(),
        1,
        "optimistic user echo should collapse into the persisted message"
    );
    assert_eq!(thread.total_message_count, 1);
    assert_eq!(thread.messages[0].id.as_deref(), Some("persisted-user"));
}

#[test]
fn append_message_does_not_duplicate_adjacent_optimistic_user_tail() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Thread".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "same prompt".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "same prompt".into(),
            timestamp: 101,
            ..Default::default()
        },
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.messages.len(),
        1,
        "double-submit optimistic user echo should collapse at the tail"
    );
    assert_eq!(thread.total_message_count, 1);
}

#[test]
fn append_message_does_not_duplicate_persisted_normal_echo_after_optimistic_tail() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Thread".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "same prompt".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            id: Some("persisted-user".into()),
            role: MessageRole::User,
            content: "same prompt".into(),
            message_kind: "normal".into(),
            timestamp: 101,
            ..Default::default()
        },
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.messages.len(),
        1,
        "persisted normal echo should replace the optimistic user tail"
    );
    assert_eq!(thread.total_message_count, 1);
    assert_eq!(thread.messages[0].id.as_deref(), Some("persisted-user"));
    assert_eq!(thread.messages[0].message_kind, "normal");
}

#[test]
fn disjoint_latest_page_reanchors_optimistic_prompt_from_unloaded_thread() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        ..Default::default()
    }]));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "new prompt".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("old {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        total_message_count: 121,
        loaded_message_start: 71,
        loaded_message_end: 121,
        messages: (71..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("old {index}"),
                ..Default::default()
            })
            .chain(std::iter::once(AgentMessage {
                id: Some("persisted-user".into()),
                role: MessageRole::User,
                content: "new prompt".into(),
                message_kind: "normal".into(),
                timestamp: 101,
                ..Default::default()
            }))
            .collect(),
        ..Default::default()
    }));

    let thread = state
        .active_thread()
        .expect("thread should remain selected");
    let prompt_count = thread
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::User && message.content == "new prompt")
        .count();
    assert_eq!(
        prompt_count, 1,
        "persisted prompt should replace the optimistic prompt instead of duplicating it"
    );
    assert_eq!(thread.loaded_message_start, 70);
    assert_eq!(thread.loaded_message_end, 121);
    assert_eq!(thread.total_message_count, 121);
    assert_eq!(
        thread
            .messages
            .last()
            .and_then(|message| message.id.as_deref()),
        Some("persisted-user")
    );
}

#[test]
fn disjoint_latest_page_drops_optimistic_prompt_when_persisted_echo_already_loaded() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        ..Default::default()
    }]));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "new prompt".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        total_message_count: 121,
        loaded_message_start: 71,
        loaded_message_end: 121,
        messages: (71..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("old {index}"),
                ..Default::default()
            })
            .chain(std::iter::once(AgentMessage {
                id: Some("persisted-user".into()),
                role: MessageRole::User,
                content: "new prompt".into(),
                message_kind: "normal".into(),
                timestamp: 101,
                ..Default::default()
            }))
            .collect(),
        ..Default::default()
    }));

    let thread = state
        .active_thread()
        .expect("thread should remain selected");
    let prompt_count = thread
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::User && message.content == "new prompt")
        .count();
    assert_eq!(
        prompt_count, 1,
        "first loaded detail containing the persisted prompt should collapse the optimistic prompt"
    );
    assert_eq!(thread.loaded_message_start, 71);
    assert_eq!(thread.loaded_message_end, 121);
    assert_eq!(thread.total_message_count, 121);
    assert_eq!(
        thread
            .messages
            .last()
            .and_then(|message| message.id.as_deref()),
        Some("persisted-user")
    );
}

#[test]
fn shifted_latest_page_collapses_adjacent_optimistic_prompt_echo() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("old {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "new prompt".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        total_message_count: 122,
        loaded_message_start: 121,
        loaded_message_end: 122,
        messages: vec![AgentMessage {
            id: Some("persisted-user".into()),
            role: MessageRole::User,
            content: "new prompt".into(),
            message_kind: "normal".into(),
            timestamp: 101,
            ..Default::default()
        }],
        ..Default::default()
    }));

    let thread = state
        .active_thread()
        .expect("thread should remain selected");
    let prompt_count = thread
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::User && message.content == "new prompt")
        .count();
    assert_eq!(
        prompt_count, 1,
        "shifted latest-page refresh should collapse adjacent optimistic and persisted prompt echoes"
    );
    assert_eq!(
        thread
            .messages
            .last()
            .and_then(|message| message.id.as_deref()),
        Some("persisted-user")
    );
}

#[test]
fn dismiss_concierge_welcome_removes_only_welcome_messages() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "concierge".into(),
        title: "Concierge".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome".into(),
            is_concierge_welcome: true,
            ..Default::default()
        },
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Follow-up".into(),
            ..Default::default()
        },
    });

    state.reduce(ChatAction::DismissConciergeWelcome);

    let thread = state
        .active_thread()
        .expect("concierge thread should exist");
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].content, "Follow-up");
    assert!(!thread.messages[0].is_concierge_welcome);
}

