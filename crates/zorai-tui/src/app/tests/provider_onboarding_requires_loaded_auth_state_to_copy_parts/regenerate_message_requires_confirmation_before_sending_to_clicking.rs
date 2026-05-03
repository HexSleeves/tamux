    #[test]
    fn regenerate_message_requires_confirmation_before_sending() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.connected = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::User,
                content: "Original prompt".to_string(),
                ..Default::default()
            },
        });
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_regenerate_message(1);

        assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
        assert!(
            cmd_rx.try_recv().is_err(),
            "regenerate should wait for confirmation"
        );

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::ChatActionConfirm,
        );
        assert!(!quit);

        let mut saw_send = false;
        while let Ok(command) = cmd_rx.try_recv() {
            if matches!(command, DaemonCommand::SendMessage { .. }) {
                saw_send = true;
                break;
            }
        }
        assert!(
            saw_send,
            "confirmation should eventually send the regenerated prompt"
        );
    }

    #[test]
    fn pin_action_dispatches_without_confirmation_modal() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("message-1".to_string()),
                role: chat::MessageRole::User,
                content: "Original prompt".to_string(),
                ..Default::default()
            },
        });
        model.chat.select_message(Some(0));
        model.chat.select_message_action(2);

        assert!(model.execute_selected_inline_message_action());
        assert_ne!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::PinThreadMessageForCompaction {
                thread_id,
                message_id,
            }) => {
                assert_eq!(thread_id, "thread-1");
                assert_eq!(message_id, "message-1");
            }
            other => panic!("expected pin command, got {other:?}"),
        }
    }

    #[test]
    fn delete_message_requires_confirmation_before_removing_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.width = 100;
        model.height = 40;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("m1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_delete_message(0);

        assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(1),
            "message should remain until deletion is confirmed"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "delete should wait for confirmation"
        );

        let quit = model.handle_key_modal(
            KeyCode::Enter,
            KeyModifiers::NONE,
            modal::ModalKind::ChatActionConfirm,
        );
        assert!(!quit);

        let sent = cmd_rx
            .try_recv()
            .expect("confirmation should send delete command");
        assert!(matches!(sent, DaemonCommand::DeleteMessages { .. }));
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(0),
            "message should be removed after deletion is confirmed"
        );
    }

    #[test]
    fn clicking_cancel_in_chat_action_confirm_does_not_delete_message() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.width = 100;
        model.height = 40;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                id: Some("m1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Answer".to_string(),
                ..Default::default()
            },
        });

        model.request_delete_message(0);
        let (_, overlay_area) = model
            .current_modal_area()
            .expect("chat action confirm modal should be visible");
        let (_, cancel_rect) = render_helpers::chat_action_confirm_button_bounds(overlay_area)
            .expect("confirm modal should expose button bounds");
        let click_col = cancel_rect.x.saturating_add(1);
        let click_row = cancel_rect.y;

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: click_col,
            row: click_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            model.modal.top(),
            None,
            "cancel click should close the modal"
        );
        assert_eq!(
            model
                .chat
                .active_thread()
                .map(|thread| thread.messages.len()),
            Some(1),
            "cancel click must not delete the message"
        );
        assert!(
            cmd_rx.try_recv().is_err(),
            "cancel click must not send a delete command"
        );
    }
