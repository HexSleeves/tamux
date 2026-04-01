pub(super) struct TestExchange {
    pub(super) result: std::result::Result<StoredOpenAICodexAuth, String>,
}

pub(super) const CODEX_CLI_AUTH_FIXTURE_JSON: &str = r#"{"tokens":{"access_token":"header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdC0xIn0sImV4cCI6NDEwMjQ0NDgwMH0.signature","refresh_token":"refresh-token"}}"#;

impl OpenAICodexExchange for TestExchange {
    fn exchange_authorization_code(
        &self,
        _code: &str,
        _verifier: &str,
    ) -> Result<StoredOpenAICodexAuth> {
        match &self.result {
            Ok(auth) => Ok(auth.clone()),
            Err(message) => Err(anyhow::anyhow!(message.clone())),
        }
    }
}

pub(super) fn stored_auth_fixture() -> StoredOpenAICodexAuth {
    StoredOpenAICodexAuth {
        provider: Some("openai-codex".to_string()),
        auth_mode: Some("chatgpt_subscription".to_string()),
        access_token: "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdC0xIn0sImV4cCI6NDEwMjQ0NDgwMH0.signature".to_string(),
        refresh_token: "refresh-token".to_string(),
        account_id: Some("acct-1".to_string()),
        expires_at: Some(4_102_444_800_000),
        source: Some("tamux".to_string()),
        updated_at: Some(4_102_444_800_000),
        created_at: Some(4_102_444_800_000),
    }
}

pub(super) fn write_codex_cli_auth_fixture(path: &std::path::Path) {
    std::fs::write(path, CODEX_CLI_AUTH_FIXTURE_JSON).expect("write codex auth fixture");
}

pub(super) fn set_test_auth_env(root: &std::path::Path, cli_auth_path: &std::path::Path) {
    std::env::set_var("TAMUX_PROVIDER_AUTH_DB_PATH", root.join("provider-auth.db"));
    std::env::set_var("TAMUX_CODEX_CLI_AUTH_PATH", cli_auth_path);
}

pub(super) fn prepare_openai_auth_test(
    root: &std::path::Path,
    cli_auth_name: &str,
) -> std::path::PathBuf {
    let cli_auth_path = root.join(cli_auth_name);
    set_test_auth_env(root, &cli_auth_path);
    clear_openai_codex_auth_test_state();
    cli_auth_path
}
