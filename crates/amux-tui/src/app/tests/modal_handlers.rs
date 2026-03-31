use super::*;
use tokio::sync::mpsc::unbounded_channel;

fn make_model() -> (
    TuiModel,
    tokio::sync::mpsc::UnboundedReceiver<DaemonCommand>,
) {
    let (_event_tx, event_rx) = std::sync::mpsc::channel();
    let (daemon_tx, daemon_rx) = unbounded_channel();
    (TuiModel::new(event_rx, daemon_tx), daemon_rx)
}

#[test]
fn whatsapp_modal_esc_sends_stop_and_closes() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));
    assert_eq!(model.modal.top(), Some(modal::ModalKind::WhatsAppLink));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected stop command"),
        DaemonCommand::WhatsAppLinkStop
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
}

#[test]
fn whatsapp_modal_esc_keeps_connected_session_running() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .set_whatsapp_link_connected(Some("+48663977535".to_string()));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
    assert!(daemon_rx.try_recv().is_err());
}

#[test]
fn whatsapp_modal_cancel_sends_stop_and_closes() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));

    let quit = model.handle_key_modal(
        KeyCode::Char('c'),
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected stop command"),
        DaemonCommand::WhatsAppLinkStop
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
}

#[test]
fn stacked_modal_pop_only_cleans_whatsapp_when_top() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::WhatsAppLink));
    assert!(daemon_rx.try_recv().is_err());

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected stop command"),
        DaemonCommand::WhatsAppLinkStop
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
}

#[test]
fn stacked_modal_pop_preserves_connected_whatsapp_session() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .set_whatsapp_link_connected(Some("+48663977535".to_string()));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::WhatsAppLink));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::CommandPalette,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::WhatsAppLink));
    assert!(daemon_rx.try_recv().is_err());

    let quit = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::WhatsAppLink,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert!(matches!(
        daemon_rx.try_recv().expect("expected unsubscribe command"),
        DaemonCommand::WhatsAppLinkUnsubscribe
    ));
    assert!(daemon_rx.try_recv().is_err());
}

#[test]
fn selecting_custom_provider_does_not_chain_into_model_picker() {
    let (mut model, _daemon_rx) = make_model();
    let custom_index = providers::PROVIDERS
        .iter()
        .position(|provider| provider.id == "custom")
        .expect("custom provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model
        .modal
        .set_picker_item_count(providers::PROVIDERS.len());
    if custom_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(custom_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, "custom");
    assert_ne!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
}

#[test]
fn selecting_custom_provider_focuses_model_field_for_inline_entry() {
    let (mut model, _daemon_rx) = make_model();
    let custom_index = providers::PROVIDERS
        .iter()
        .position(|provider| provider.id == "custom")
        .expect("custom provider to exist");

    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::Provider));
    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model
        .modal
        .set_picker_item_count(providers::PROVIDERS.len());
    if custom_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(custom_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, "custom");
    assert_eq!(model.settings.current_field_name(), "model");
    assert_eq!(model.settings.field_cursor(), 3);
}

#[test]
fn thread_picker_right_arrow_switches_to_rarog_tab() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));

    let quit = model.handle_key_modal(
        KeyCode::Right,
        KeyModifiers::NONE,
        modal::ModalKind::ThreadPicker,
    );

    assert!(!quit);
    assert_eq!(
        model.modal.thread_picker_tab(),
        modal::ThreadPickerTab::Rarog
    );
}

#[test]
fn thread_picker_enter_selects_filtered_rarog_thread() {
    let (mut model, mut daemon_rx) = make_model();
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadListReceived(vec![
        chat::AgentThread {
            id: "regular-thread".into(),
            title: "Regular work".into(),
            ..Default::default()
        },
        chat::AgentThread {
            id: "heartbeat-1".into(),
            title: "HEARTBEAT SYNTHESIS".into(),
            ..Default::default()
        },
    ]));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));
    model
        .modal
        .set_thread_picker_tab(modal::ThreadPickerTab::Rarog);
    model.sync_thread_picker_item_count();
    model.modal.reduce(modal::ModalAction::Navigate(1));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ThreadPicker,
    );

    assert!(!quit);
    assert_eq!(model.chat.active_thread_id(), Some("heartbeat-1"));
    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread(thread_id)) => assert_eq!(thread_id, "heartbeat-1"),
        other => panic!("expected thread request, got {:?}", other),
    }
}

#[test]
fn thread_picker_mouse_click_switches_to_rarog_tab() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));
    let (_, overlay_area) = model
        .current_modal_area()
        .expect("thread picker modal should be visible");

    let rarog_pos = (overlay_area.y..overlay_area.y.saturating_add(overlay_area.height))
        .find_map(|row| {
            (overlay_area.x..overlay_area.x.saturating_add(overlay_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::thread_picker::hit_test(overlay_area, &model.chat, &model.modal, pos)
                    == Some(widgets::thread_picker::ThreadPickerHitTarget::Tab(
                        modal::ThreadPickerTab::Rarog,
                    ))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("thread picker should expose a clickable Rarog tab");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rarog_pos.x,
        row: rarog_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        model.modal.thread_picker_tab(),
        modal::ThreadPickerTab::Rarog
    );
}

#[test]
fn subagent_inline_edit_does_not_sync_main_config() {
    let (mut model, mut daemon_rx) = make_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.agent_config_raw = Some(serde_json::json!({}));

    let mut editor = crate::state::subagents::SubAgentEditorState::new(
        None,
        1,
        "openai".to_string(),
        "gpt-5.4".to_string(),
    );
    editor.name = "Draft".to_string();
    model.subagents.editor = Some(editor);
    model.settings.start_editing("subagent_name", "Draft");
    model.settings.reduce(SettingsAction::InsertChar('X'));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );

    assert!(!quit);
    assert_eq!(
        model
            .subagents
            .editor
            .as_ref()
            .map(|editor| editor.name.as_str()),
        Some("DraftX")
    );
    assert!(
        daemon_rx.try_recv().is_err(),
        "sub-agent field edits should stay local until Save"
    );
}
