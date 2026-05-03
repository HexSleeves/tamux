#[test]
fn drag_selection_does_not_rebuild_full_transcript_for_every_mouse_event() {
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
            content: "alpha beta gamma".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|candidate| {
            *candidate > chat_area.y.saturating_add(1)
                && *candidate
                    < chat_area
                        .y
                        .saturating_add(chat_area.height)
                        .saturating_sub(2)
                &&
            widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(3, *candidate),
            )
            .is_some()
        })
        .expect("chat transcript should expose a selectable row");

    let (anchor_col, drag_col) = (chat_area.x..chat_area.x.saturating_add(chat_area.width))
        .find_map(|start_col| {
            let start_point = widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(start_col, row),
            )?;
            ((start_col + 1)..chat_area.x.saturating_add(chat_area.width)).find_map(|end_col| {
                let end_point = widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(end_col, row),
                )?;
                (end_point != start_point).then_some((start_col, end_col))
            })
        })
        .expect("chat transcript should expose two distinct selectable columns");

    widgets::chat::reset_build_rendered_lines_call_count();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: anchor_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        2,
        "dragging a static selection should not rebuild beyond the initial hit-test pass and one cached transcript snapshot"
    );
}

#[test]
fn render_during_active_drag_reuses_cached_snapshot_and_shows_highlight() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
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
            role: chat::MessageRole::Assistant,
            content: (1..=80)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::ScrollChat(8));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|candidate| {
            *candidate > chat_area.y.saturating_add(1)
                && *candidate
                    < chat_area
                        .y
                        .saturating_add(chat_area.height)
                        .saturating_sub(2)
                &&
            widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(3, *candidate),
            )
            .is_some()
        })
        .expect("chat transcript should expose at least one selectable row");

    let (anchor_col, drag_col) = (chat_area.x..chat_area.x.saturating_add(chat_area.width))
        .find_map(|start_col| {
            let start_point = widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(start_col, row),
            )?;
            ((start_col + 1)..chat_area.x.saturating_add(chat_area.width)).find_map(|end_col| {
                let end_point = widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(end_col, row),
                )?;
                (end_point != start_point).then_some((start_col, end_col))
            })
        })
        .expect("chat transcript should expose two distinct selectable columns");

    widgets::chat::reset_build_rendered_lines_call_count();
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: anchor_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("model render should succeed");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        2,
        "active drag rendering should not rebuild beyond the initial hit-test pass and one cached transcript snapshot"
    );

    let buffer = terminal.backend().buffer();
    let highlighted = (0..model.height)
        .flat_map(|y| (0..model.width).filter_map(move |x| buffer.cell((x, y))))
        .filter(|cell| cell.bg == Color::Indexed(31))
        .count();
    assert!(
        highlighted > 0,
        "active drag should paint a visible selection highlight even while scrolled"
    );
}

#[test]
fn repeated_chat_renders_reuse_cached_snapshot_when_transcript_is_unchanged() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
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
            content: (1..=200)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    widgets::chat::reset_build_rendered_lines_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    terminal
        .draw(|frame| model.render(frame))
        .expect("second render should succeed");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        1,
        "unchanged chat renders should reuse the cached transcript snapshot"
    );
}

#[test]
fn scrolling_reuses_cached_snapshot_and_updates_visible_window() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
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
            content: (1..=200)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    widgets::chat::reset_build_rendered_lines_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    let before = terminal.backend().buffer().clone();

    model.chat.reduce(chat::ChatAction::ScrollChat(8));

    terminal
        .draw(|frame| model.render(frame))
        .expect("second render after scroll should succeed");
    let after = terminal.backend().buffer().clone();

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        1,
        "scrolling should reuse the cached transcript snapshot instead of rebuilding all lines"
    );
    assert_ne!(
        before, after,
        "scrolling should still change the visible transcript window"
    );
}

#[test]
fn stale_cached_snapshot_is_ignored_after_sidebar_layout_change() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
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
            role: chat::MessageRole::Assistant,
            content: "hello world".to_string(),
            ..Default::default()
        },
    });

    let full_width_area = Rect::new(
        0,
        3,
        model.width,
        model.height.saturating_sub(model.input_height() + 4),
    );
    model.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
        full_width_area,
        &model.chat,
        &model.theme,
        model.tick_counter,
        model.retry_wait_start_selected,
    );
    model.chat_drag_anchor = None;
    model.chat_drag_current = None;
    model.chat_drag_anchor_point = None;
    model.chat_drag_current_point = None;

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
    model.show_sidebar_override = Some(true);

    widgets::chat::reset_build_rendered_lines_call_count();
    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("render should fall back to fresh layout instead of using stale snapshot");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        1,
        "layout changes should ignore stale cached snapshots and rebuild visible chat rows"
    );
}

#[test]
fn mouse_drag_snapshot_uses_rendered_chat_area_without_sidebar() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .anticipatory
        .reduce(crate::state::AnticipatoryAction::Replace(vec![
            crate::wire::AnticipatoryItem {
                id: "digest-1".to_string(),
                ..Default::default()
            },
        ]));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "alpha\nbeta\ngamma\ndelta".to_string(),
            ..Default::default()
        },
    });

    let chat_area = rendered_chat_area(&model);
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x.saturating_add(3),
        row: chat_area
            .y
            .saturating_add(chat_area.height.saturating_sub(2)),
        modifiers: KeyModifiers::NONE,
    });

    let snapshot = model
        .chat_selection_snapshot
        .as_ref()
        .expect("mouse down should create a chat selection snapshot");
    assert!(
        widgets::chat::cached_snapshot_matches_area(snapshot, chat_area),
        "drag snapshots must use the exact rendered chat area"
    );
}

#[test]
fn repeated_sidebar_renders_reuse_cached_snapshot_when_history_is_unchanged() {
    let mut model = build_model();
    model.show_sidebar_override = Some(true);
    model.focus = FocusArea::Sidebar;
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
            entries: (0..200)
                .map(|idx| task::WorkContextEntry {
                    path: format!("/tmp/file-{idx:03}.rs"),
                    change_kind: Some("modified".to_string()),
                    is_text: true,
                    ..Default::default()
                })
                .collect(),
        },
    ));
    model.activate_sidebar_tab(SidebarTab::Files);

    widgets::sidebar::reset_build_cached_snapshot_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    terminal
        .draw(|frame| model.render(frame))
        .expect("second render should succeed");

    assert_eq!(
        widgets::sidebar::build_cached_snapshot_call_count(),
        1,
        "unchanged sidebar renders should reuse the cached sidebar snapshot"
    );
}

