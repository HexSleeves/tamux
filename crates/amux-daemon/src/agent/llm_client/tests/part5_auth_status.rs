use std::time::Duration;

#[test]
fn pending_login_reuses_flow() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");

    let first = begin_openai_codex_auth_login().expect("first login should succeed");
    let second = begin_openai_codex_auth_login().expect("second login should succeed");

    assert_eq!(first.status.as_deref(), Some("pending"));
    assert_eq!(second.status.as_deref(), Some("pending"));
    assert_eq!(first.auth_url, second.auth_url);
}

#[test]
fn login_timeout_sets_error_state() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");

    let status = mark_openai_codex_auth_timeout_for_tests();

    assert_eq!(status.status.as_deref(), Some("error"));
    assert_eq!(status.available, false);
    assert!(status
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("timed out"));
}

#[test]
fn exchange_failure_sets_error_state() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");

    let status = complete_openai_codex_auth_with_code_for_tests(
        "bad-code",
        &TestExchange {
            result: Err("exchange failed".to_string()),
        },
    );

    assert_eq!(status.status.as_deref(), Some("error"));
    assert_eq!(status.available, false);
    assert_eq!(
        status.error.as_deref(),
        Some("OpenAI authentication failed. Please try signing in again.")
    );
}

#[test]
fn successful_login_persists_auth_and_reports_completed() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");

    let completed = complete_openai_codex_auth_with_code_for_tests(
        "good-code",
        &TestExchange {
            result: Ok(stored_auth_fixture()),
        },
    );

    assert_eq!(completed.status.as_deref(), Some("completed"));
    assert!(completed.available);
    assert!(completed.auth_url.is_none());

    let status = openai_codex_auth_status(false);
    assert!(status.available);
    assert_eq!(status.status.as_deref(), Some("completed"));
    assert_eq!(status.account_id.as_deref(), Some("acct-1"));
}

#[test]
fn status_during_pending_returns_same_auth_url() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");

    let login = begin_openai_codex_auth_login().expect("login should start");
    let status = openai_codex_auth_status(false);

    assert_eq!(status.status.as_deref(), Some("pending"));
    assert_eq!(status.auth_url, login.auth_url);
    assert_eq!(status.available, false);
    assert!(status.account_id.is_none());
    assert!(status.error.is_none());
}

#[test]
fn helper_reports_available_when_only_codex_cli_auth_exists() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    let codex_auth_path = prepare_openai_auth_test(temp_dir.path(), "codex-auth.json");
    write_codex_cli_auth_fixture(&codex_auth_path);

    assert!(read_stored_openai_codex_auth().is_none());
    assert!(has_openai_chatgpt_subscription_auth());
    assert!(read_stored_openai_codex_auth().is_none());
}

#[test]
fn failed_login_does_not_hide_stored_auth_status() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    write_stored_openai_codex_auth(&stored_auth_fixture()).expect("stored auth should persist");
    begin_openai_codex_auth_login().expect("login should start");

    let failed = complete_openai_codex_auth_with_code_for_tests(
        "bad-code",
        &TestExchange {
            result: Err("exchange failed".to_string()),
        },
    );

    assert_eq!(failed.status.as_deref(), Some("error"));
    let status = openai_codex_auth_status(false);
    assert!(status.available);
    assert_eq!(status.status.as_deref(), Some("completed"));
    assert_eq!(status.account_id.as_deref(), Some("acct-1"));
    assert!(provider_auth_state_authenticated());
}

#[test]
fn login_after_error_starts_fresh_flow() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");
    mark_openai_codex_auth_timeout_for_tests();

    let fresh = begin_openai_codex_auth_login().expect("fresh login should start");

    assert_eq!(fresh.status.as_deref(), Some("pending"));
    assert!(fresh.error.is_none());
    assert!(fresh.auth_url.is_some());
}

#[test]
fn browser_callback_timeout_sets_error_state() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");

    let timeout_status = complete_browser_auth_with_timeout_for_tests(
        &TestExchange {
            result: Ok(stored_auth_fixture()),
        },
        Duration::from_millis(10),
    );

    assert_eq!(timeout_status.status.as_deref(), Some("error"));
    assert!(!timeout_status.available);
    assert!(timeout_status
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("timed out"));
}

#[test]
fn stale_flow_completion_returns_current_pending_status() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");

    begin_openai_codex_auth_login().expect("first login should start");
    let stale_flow_id = current_pending_openai_codex_flow_id_for_tests()
        .expect("pending flow id should be present");
    logout_openai_codex_auth().expect("logout should succeed");
    let current = begin_openai_codex_auth_login().expect("second login should start");

    let status = complete_openai_codex_auth_flow_with_result_for_tests(
        &stale_flow_id,
        Ok(stored_auth_fixture()),
    );

    assert_eq!(status.status.as_deref(), Some("pending"));
    assert_eq!(status.auth_url, current.auth_url);
    assert!(!status.available);
}
