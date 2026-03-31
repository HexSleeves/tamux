    #[test]
    fn drag_selection_copies_expected_text_after_autoscroll() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
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
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
        let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
            .chain(chat_area.y..preferred_row)
            .find(|row| {
                widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *row),
                )
                .is_some()
            })
            .expect("chat transcript should expose at least one selectable row");

        super::conversion::reset_last_copied_text();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
        for _ in 0..4 {
            model.handle_mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 3,
                row: start_row,
                modifiers: KeyModifiers::NONE,
            });
        }

        let anchor_point = model
            .chat_drag_anchor_point
            .expect("mouse down should capture a document anchor point");
        let current_point = model
            .chat_drag_current_point
            .expect("autoscroll should extend the current drag point");
        let expected = widgets::chat::selected_text(
            chat_area,
            &model.chat,
            &model.theme,
            model.tick_counter,
            anchor_point,
            current_point,
        )
        .expect("selection should resolve to copied text");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(model.status_line, "Copied selection to clipboard");
    }

    #[test]
    fn work_context_drag_selection_copies_beyond_visible_window() {
        let mut model = build_model();
        model.show_sidebar_override = Some(false);
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::WorkContextReceived(
            task::ThreadWorkContext {
                thread_id: "thread-1".to_string(),
                entries: vec![task::WorkContextEntry {
                    path: "/tmp/demo.txt".to_string(),
                    is_text: true,
                    ..Default::default()
                }],
            },
        ));
        model
            .tasks
            .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
                path: "/tmp/demo.txt".to_string(),
                content: (1..=80)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                truncated: false,
                is_text: true,
            }));
        model.tasks.reduce(task::TaskAction::SelectWorkPath {
            thread_id: "thread-1".to_string(),
            path: Some("/tmp/demo.txt".to_string()),
        });
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Chat;

        let input_start_row = model.height.saturating_sub(model.input_height() + 1);
        let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
        let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
        let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
            .chain(chat_area.y..preferred_row)
            .find(|row| {
                widgets::work_context_view::selection_point_from_mouse(
                    chat_area,
                    &model.tasks,
                    model.chat.active_thread_id(),
                    model.sidebar.active_tab(),
                    model.sidebar.selected_item(),
                    &model.theme,
                    model.task_view_scroll,
                    Position::new(3, *row),
                )
                .is_some()
            })
            .expect("work-context preview should expose at least one selectable row");

        super::conversion::reset_last_copied_text();

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
        for _ in 0..4 {
            model.handle_mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 3,
                row: start_row,
                modifiers: KeyModifiers::NONE,
            });
        }

        let anchor_point = model
            .work_context_drag_anchor_point
            .expect("mouse down should capture a preview anchor point");
        let current_point = model
            .work_context_drag_current_point
            .expect("scrolling should extend the preview selection");
        let expected = widgets::work_context_view::selected_text(
            chat_area,
            &model.tasks,
            model.chat.active_thread_id(),
            model.sidebar.active_tab(),
            model.sidebar.selected_item(),
            &model.theme,
            model.task_view_scroll,
            anchor_point,
            current_point,
        )
        .expect("selection should resolve to copied preview text");

        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });

        assert_eq!(
            super::conversion::last_copied_text().as_deref(),
            Some(expected.as_str())
        );
    }

    #[test]
    fn esc_closes_work_context_even_from_input_focus() {
        let mut model = build_model();
        model.focus = FocusArea::Input;
        model.main_pane_view = MainPaneView::WorkContext;

        let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

        assert!(!handled);
        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Chat);
    }

    #[test]
    fn reselecting_same_sidebar_file_closes_work_context() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::WorkContextReceived(
            task::ThreadWorkContext {
                thread_id: "thread-1".to_string(),
                entries: vec![task::WorkContextEntry {
                    path: "/tmp/demo.txt".to_string(),
                    is_text: true,
                    ..Default::default()
                }],
            },
        ));
        model.tasks.reduce(task::TaskAction::SelectWorkPath {
            thread_id: "thread-1".to_string(),
            path: Some("/tmp/demo.txt".to_string()),
        });
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Files));
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Sidebar;

        model.handle_sidebar_enter();

        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Sidebar);
    }

    #[test]
    fn reselecting_same_sidebar_todo_closes_work_context() {
        let mut model = build_model();
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-1".to_string(),
            title: "Thread".to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
        model.tasks.reduce(task::TaskAction::ThreadTodosReceived {
            thread_id: "thread-1".to_string(),
            items: vec![task::TodoItem {
                id: "todo-1".to_string(),
                content: "todo".to_string(),
                status: Some(task::TodoStatus::Pending),
                position: 0,
                step_index: None,
                created_at: 0,
                updated_at: 0,
            }],
        });
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Todos));
        model.main_pane_view = MainPaneView::WorkContext;
        model.focus = FocusArea::Sidebar;

        model.handle_sidebar_enter();

        assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
        assert_eq!(model.focus, FocusArea::Sidebar);
    }

    #[test]
    fn attention_surface_uses_settings_tab_when_modal_open() {
        let mut model = build_model();
        model
            .modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        model
            .settings
            .reduce(SettingsAction::SwitchTab(SettingsTab::SubAgents));

        let (surface, thread_id, goal_run_id) = model.current_attention_target();
        assert_eq!(surface, "modal:settings:subagents");
        assert_eq!(thread_id, None);
        assert_eq!(goal_run_id, None);
    }

    #[test]
    fn attention_surface_uses_sidebar_tab_for_sidebar_focus() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread_1".to_string(),
            title: "Test".to_string(),
        });
        model.tasks.reduce(task::TaskAction::ThreadTodosReceived {
            thread_id: "thread_1".to_string(),
            items: vec![task::TodoItem {
                id: "todo_1".to_string(),
                content: "todo".to_string(),
                status: Some(task::TodoStatus::Pending),
                position: 0,
                step_index: None,
                created_at: 0,
                updated_at: 0,
            }],
        });
        model.focus = FocusArea::Sidebar;
        model
            .sidebar
            .reduce(SidebarAction::SwitchTab(SidebarTab::Todos));

        let (surface, thread_id, goal_run_id) = model.current_attention_target();
        assert_eq!(surface, "conversation:sidebar:todos");
        assert_eq!(thread_id.as_deref(), Some("thread_1"));
        assert_eq!(goal_run_id, None);
    }

    #[test]
    fn operator_profile_onboarding_takes_precedence_over_provider_onboarding() {
        let mut model = build_model();
        model.connected = true;
        model.auth.loaded = true;
        model.auth.entries = vec![unauthenticated_entry()];
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });

        assert!(
            model.should_show_operator_profile_onboarding(),
            "operator profile onboarding should be active"
        );
        assert!(
            !model.should_show_provider_onboarding(),
            "provider onboarding should not mask operator profile onboarding"
        );
    }

    #[test]
    fn submit_operator_profile_answer_sends_command_and_clears_input() {
        let (_daemon_tx, daemon_rx) = mpsc::channel();
        let (cmd_tx, mut cmd_rx) = unbounded_channel();
        let mut model = TuiModel::new(daemon_rx, cmd_tx);
        model.operator_profile.visible = true;
        model.operator_profile.question = Some(OperatorProfileQuestionVm {
            session_id: "sess-1".to_string(),
            question_id: "name".to_string(),
            field_key: "name".to_string(),
            prompt: "What should I call you?".to_string(),
            input_kind: "text".to_string(),
            optional: false,
        });
        model.input.set_text("Milan");

        assert!(model.submit_operator_profile_answer());
        assert_eq!(model.input.buffer(), "");
        assert!(
            model.operator_profile.question.is_none(),
            "question should clear when submission starts"
        );

        let sent = cmd_rx
            .try_recv()
            .expect("submitting answer should emit daemon command");
        match sent {
            DaemonCommand::SubmitOperatorProfileAnswer {
                session_id,
                question_id,
                answer_json,
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(question_id, "name");
                assert_eq!(answer_json, "\"Milan\"");
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

