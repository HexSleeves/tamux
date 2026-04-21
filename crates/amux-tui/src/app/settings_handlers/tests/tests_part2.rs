#[test]
fn operator_model_inspect_field_requests_operator_model_snapshot() {
    let (mut model, mut daemon_rx) = make_model();
    focus_settings_field(&mut model, SettingsTab::Chat, "operator_model_inspect");
    assert_eq!(model.settings.current_field_name(), "operator_model_inspect");

    model.activate_settings_field();

    assert!(matches!(
        daemon_rx.try_recv().expect("expected inspect command"),
        DaemonCommand::GetOperatorModel
    ));
    assert!(daemon_rx.try_recv().is_err());
}

#[test]
fn operator_model_reset_field_requests_model_reset() {
    let (mut model, mut daemon_rx) = make_model();
    focus_settings_field(&mut model, SettingsTab::Chat, "operator_model_reset");
    assert_eq!(model.settings.current_field_name(), "operator_model_reset");

    model.activate_settings_field();

    assert!(matches!(
        daemon_rx.try_recv().expect("expected reset command"),
        DaemonCommand::ResetOperatorModel
    ));
    assert!(daemon_rx.try_recv().is_err());
}
