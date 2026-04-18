use crate::app::tests::test_harness::make_test_model;
use crate::state::settings::{SettingsAction, SettingsTab};
use crate::client::DaemonCommand;
use serde_json::{json, Value};

fn focus_audio_field(model: &mut crate::app::TuiModel, field_idx: usize) {
    model.settings.reduce(SettingsAction::SwitchTab(SettingsTab::Features));
    model.settings.reduce(SettingsAction::NavigateField(field_idx as i32));
}

#[test]
fn audio_stt_enabled_toggle_sends_expected_daemon_command() {
    let (mut model, rx) = make_test_model();
    let mut rx = rx;
    
    // Set up config with audio settings
    model.config.agent_config_raw = Some(json!({
        "extra": {
            "audio_stt_enabled": false
        }
    }));
    
    focus_audio_field(&mut model, 16);
    model.activate_settings_field();
    
    // Should send SetConfigItem command
    let cmd: Option<DaemonCommand> = rx.try_recv().ok();
    match cmd {
        Some(DaemonCommand::SetConfigItem { key_path, value_json }) => {
            assert_eq!(key_path, "/extra/audio_stt_enabled");
            assert_eq!(value_json, "true");
        }
        _ => panic!("Expected SetConfigItem command for audio_stt_enabled"),
    }
    
    // Config should be updated locally
    let enabled = model.config.agent_config_raw
        .as_ref()
        .and_then(|r: &Value| r.get("extra"))
        .and_then(|e: &Value| e.get("audio_stt_enabled"))
        .and_then(|v: &Value| v.as_bool())
        .unwrap_or(false);
    assert!(enabled, "audio_stt_enabled should be toggled to true");
}

#[test]
fn audio_tts_enabled_toggle_sends_expected_daemon_command() {
    let (mut model, rx) = make_test_model();
    let mut rx = rx;
    
    model.config.agent_config_raw = Some(json!({
        "extra": {
            "audio_tts_enabled": false
        }
    }));
    
    focus_audio_field(&mut model, 19);
    model.activate_settings_field();
    
    let cmd: Option<DaemonCommand> = rx.try_recv().ok();
    match cmd {
        Some(DaemonCommand::SetConfigItem { key_path, value_json }) => {
            assert_eq!(key_path, "/extra/audio_tts_enabled");
            assert_eq!(value_json, "true");
        }
        _ => panic!("Expected SetConfigItem command for audio_tts_enabled"),
    }
}

#[test]
fn audio_stt_provider_edit_starts_with_current_value() {
    let (mut model, _rx) = make_test_model();
    
    model.config.agent_config_raw = Some(json!({
        "extra": {
            "audio_stt_provider": "anthropic"
        }
    }));
    
    focus_audio_field(&mut model, 17);
    model.activate_settings_field();
    
    assert!(model.settings.is_editing());
    assert_eq!(model.settings.editing_field(), Some("feat_audio_stt_provider"));
    assert_eq!(model.settings.edit_buffer(), "anthropic");
}

#[test]
fn audio_tts_voice_edit_sends_expected_daemon_command_on_confirm() {
    let (mut model, rx) = make_test_model();
    let mut rx = rx;
    
    model.config.agent_config_raw = Some(json!({
        "extra": {
            "audio_tts_voice": "alloy"
        }
    }));
    
    focus_audio_field(&mut model, 22);
    model.activate_settings_field();
    
    // Edit the value
    model.settings.reduce(SettingsAction::Backspace);
    model.settings.reduce(SettingsAction::Backspace);
    model.settings.reduce(SettingsAction::Backspace);
    model.settings.reduce(SettingsAction::Backspace);
    model.settings.reduce(SettingsAction::Backspace);
    model.settings.reduce(SettingsAction::InsertChar('s'));
    model.settings.reduce(SettingsAction::InsertChar('h'));
    model.settings.reduce(SettingsAction::InsertChar('i'));
    model.settings.reduce(SettingsAction::InsertChar('m'));
    model.settings.reduce(SettingsAction::InsertChar('m'));
    model.settings.reduce(SettingsAction::InsertChar('e'));
    model.settings.reduce(SettingsAction::InsertChar('r'));
    
    model.settings.reduce(SettingsAction::ConfirmEdit);
    
    let cmd: Option<DaemonCommand> = rx.try_recv().ok();
    match cmd {
        Some(DaemonCommand::SetConfigItem { key_path, value_json }) => {
            assert_eq!(key_path, "/extra/audio_tts_voice");
            assert!(value_json.contains("shimmer"));
        }
        _ => panic!("Expected SetConfigItem command for audio_tts_voice"),
    }
    
    let voice = model.config.agent_config_raw
        .as_ref()
        .and_then(|r: &Value| r.get("extra"))
        .and_then(|e: &Value| e.get("audio_tts_voice"))
        .and_then(|v: &Value| v.as_str())
        .unwrap_or("");
    assert!(voice.contains("shimmer"));
}