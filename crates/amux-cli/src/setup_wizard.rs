//! First-run setup wizard for tamux.
//!
//! Detects fresh installations (no config.json or no provider set) and guides
//! the user through LLM provider configuration. Per D-16, this covers daemon
//! auto-start + setup wizard only; capability tiers and concierge onboarding
//! are Phase 10.

use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Result of a completed setup wizard run.
#[derive(Debug, Clone)]
pub struct SetupResult {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub api_transport: String,
    pub preferred_client: String,
    pub data_dir: PathBuf,
    pub capability_tier: Option<String>,
}

struct ProviderInfo {
    id: &'static str,
    name: &'static str,
    default_model: &'static str,
    default_base_url: &'static str,
    api_transport: &'static str,
}

pub(crate) const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        id: "anthropic",
        name: "Anthropic (Claude)",
        default_model: "claude-sonnet-4-20250514",
        default_base_url: "https://api.anthropic.com",
        api_transport: "anthropic",
    },
    ProviderInfo {
        id: "openai",
        name: "OpenAI (GPT)",
        default_model: "gpt-4o",
        default_base_url: "https://api.openai.com/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "openrouter",
        name: "OpenRouter",
        default_model: "anthropic/claude-sonnet-4",
        default_base_url: "https://openrouter.ai/api/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "groq",
        name: "Groq",
        default_model: "llama-3.3-70b-versatile",
        default_base_url: "https://api.groq.com/openai/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "together",
        name: "Together AI",
        default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        default_base_url: "https://api.together.xyz/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "fireworks",
        name: "Fireworks AI",
        default_model: "accounts/fireworks/models/llama-v3p3-70b-instruct",
        default_base_url: "https://api.fireworks.ai/inference/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "deepseek",
        name: "DeepSeek",
        default_model: "deepseek-chat",
        default_base_url: "https://api.deepseek.com/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "mistral",
        name: "Mistral AI",
        default_model: "mistral-large-latest",
        default_base_url: "https://api.mistral.ai/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "featherless",
        name: "Featherless AI",
        default_model: "Qwen/Qwen2.5-72B-Instruct",
        default_base_url: "https://api.featherless.ai/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "ollama",
        name: "Ollama (Local)",
        default_model: "llama3.3",
        default_base_url: "http://localhost:11434/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "lmstudio",
        name: "LM Studio (Local)",
        default_model: "loaded-model",
        default_base_url: "http://localhost:1234/v1",
        api_transport: "openai",
    },
    ProviderInfo {
        id: "custom",
        name: "Custom OpenAI-compatible",
        default_model: "",
        default_base_url: "",
        api_transport: "openai",
    },
];

/// Returns true if a provider is local (no API key required).
fn is_local_provider(id: &str) -> bool {
    matches!(id, "ollama" | "lmstudio")
}

/// Check whether the first-run setup wizard should run, using the default config
/// path at `~/.tamux/agent/config.json`.
pub fn needs_setup() -> bool {
    let config_path = amux_protocol::amux_data_dir()
        .join("agent")
        .join("config.json");
    needs_setup_at(&config_path)
}

/// Check whether setup is needed for a specific config path.
/// Returns true if the config file doesn't exist, can't be read, or has no
/// provider configured.
pub(crate) fn needs_setup_at(config_path: &Path) -> bool {
    let data = match std::fs::read_to_string(config_path) {
        Ok(d) => d,
        Err(_) => return true,
    };

    let value: serde_json::Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return true,
    };

    match value.get("provider").and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => false,
        _ => true,
    }
}

/// Read a line from stdin with a prompt.
fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .context("Failed to read input")?;
    Ok(line.trim().to_string())
}

/// Read an API key with masked input using crossterm raw mode.
fn read_api_key(provider_name: &str) -> Result<String> {
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    use crossterm::terminal;

    print!("Enter API key for {}: ", provider_name);
    io::stdout().flush()?;

    terminal::enable_raw_mode().context("Failed to enable raw mode for masked input")?;

    let mut key = String::new();
    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Enter => {
                    terminal::disable_raw_mode()?;
                    println!();
                    break;
                }
                KeyCode::Backspace => {
                    if key.pop().is_some() {
                        print!("\x08 \x08");
                        io::stdout().flush()?;
                    }
                }
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    terminal::disable_raw_mode()?;
                    anyhow::bail!("Setup cancelled by user");
                }
                KeyCode::Char(c) => {
                    key.push(c);
                    print!("*");
                    io::stdout().flush()?;
                }
                _ => {}
            }
        }
    }

    Ok(key)
}

/// Run the setup wizard, collecting provider configuration from the user.
pub async fn run_setup_wizard() -> Result<SetupResult> {
    // Step 1: Welcome banner
    println!();
    println!("+-----------------------------------------+");
    println!("|   tamux -- The Agent That Lives          |");
    println!("|   First-time setup                       |");
    println!("+-----------------------------------------+");
    println!();

    // Step 1: Provider selection
    println!("Select your LLM provider:\n");
    for (i, p) in PROVIDERS.iter().enumerate() {
        println!("  {:>2}. {}", i + 1, p.name);
    }
    println!();

    let provider_idx = loop {
        let input = prompt(&format!("Select provider [1-{}]: ", PROVIDERS.len()))?;
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= PROVIDERS.len() => break n - 1,
            _ => println!("Invalid selection. Enter a number between 1 and {}.", PROVIDERS.len()),
        }
    };

    let provider = &PROVIDERS[provider_idx];
    let mut model = provider.default_model.to_string();
    let mut base_url = provider.default_base_url.to_string();
    let mut api_transport = provider.api_transport.to_string();

    // For custom provider, prompt for base URL and transport
    if provider.id == "custom" {
        base_url = prompt("Enter base URL (e.g., http://localhost:8080/v1): ")?;
        if base_url.is_empty() {
            anyhow::bail!("Base URL is required for custom provider");
        }
        model = prompt("Enter model name: ")?;
        let transport_input = prompt("API transport [1] OpenAI  [2] Anthropic (default: 1): ")?;
        api_transport = match transport_input.as_str() {
            "2" => "anthropic".to_string(),
            _ => "openai".to_string(),
        };
    }

    // Confirm or override the default model
    if provider.id != "custom" && !model.is_empty() {
        let model_input = prompt(&format!("Model [{}]: ", model))?;
        if !model_input.is_empty() {
            model = model_input;
        }
    }

    // Step 2: API key
    let api_key = if is_local_provider(provider.id) {
        println!("Local provider detected -- no API key needed.");
        String::new()
    } else {
        read_api_key(provider.name)?
    };

    // Step 3: Default client
    println!();
    let client_input = prompt("Default client [1] TUI  [2] Electron Desktop (default: 1): ")?;
    let preferred_client = match client_input.as_str() {
        "2" => "electron".to_string(),
        _ => "tui".to_string(),
    };

    // Step 4: Experience level (capability tier self-assessment)
    println!();
    println!("How familiar are you with AI agents?\n");
    println!("  1. Just getting started");
    println!("  2. I've used chatbots");
    println!("  3. I run automations");
    println!("  4. I build agent systems");
    println!();

    let capability_tier = loop {
        let input = prompt("Select experience level [1-4] (default: 1): ")?;
        if input.is_empty() {
            break Some("newcomer".to_string());
        }
        match input.as_str() {
            "1" => break Some("newcomer".to_string()),
            "2" => break Some("familiar".to_string()),
            "3" => break Some("power_user".to_string()),
            "4" => break Some("expert".to_string()),
            _ => println!("Invalid selection. Enter a number between 1 and 4."),
        }
    };

    // Step 5: Data directory (renumbered from Step 4)
    let default_data_dir = amux_protocol::amux_data_dir();
    let data_dir_input = prompt(&format!(
        "Data directory [{}]: ",
        default_data_dir.display()
    ))?;
    let data_dir = if data_dir_input.is_empty() {
        default_data_dir
    } else {
        PathBuf::from(&data_dir_input)
    };

    // Step 6: Connectivity test
    println!();
    println!("Testing connection to {}...", provider.name);
    let test_result = test_connectivity(&base_url, &model, &api_key, &api_transport).await;
    match test_result {
        ConnectivityResult::Success => {
            println!("Connection successful!");
        }
        ConnectivityResult::AuthError => {
            println!(
                "Connection reached provider but API key was rejected. \
                 You can fix this later in ~/.tamux/agent/config.json"
            );
        }
        ConnectivityResult::ConnectionError(e) => {
            println!(
                "Could not reach provider: {}. \
                 You can fix this later in ~/.tamux/agent/config.json",
                e
            );
        }
    }

    // Write config atomically (temp file + rename) per Pitfall 6
    write_config_atomic(&data_dir, &SetupResult {
        provider: provider.id.to_string(),
        model: model.clone(),
        api_key: api_key.clone(),
        base_url: base_url.clone(),
        api_transport: api_transport.clone(),
        preferred_client: preferred_client.clone(),
        data_dir: data_dir.clone(),
        capability_tier: capability_tier.clone(),
    })
    .context("Failed to write configuration")?;

    let config_path = data_dir.join("agent").join("config.json");
    println!("Configuration saved to {}", config_path.display());

    Ok(SetupResult {
        provider: provider.id.to_string(),
        model,
        api_key,
        base_url,
        api_transport,
        preferred_client,
        data_dir,
        capability_tier,
    })
}

enum ConnectivityResult {
    Success,
    AuthError,
    ConnectionError(String),
}

/// Test connectivity to the selected provider endpoint.
async fn test_connectivity(
    base_url: &str,
    model: &str,
    api_key: &str,
    api_transport: &str,
) -> ConnectivityResult {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => return ConnectivityResult::ConnectionError(e.to_string()),
    };

    let (url, body, request) = if api_transport == "anthropic" {
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "hi"}]
        });
        let req = client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body);
        (url, body, req)
    } else {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "hi"}]
        });
        let req = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&body);
        (url, body, req)
    };

    tracing::debug!(url = %url, body = %body, "testing provider connectivity");

    match request.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if status == 401 || status == 403 {
                ConnectivityResult::AuthError
            } else {
                // Any response (including 4xx like 400/422) means the endpoint is reachable
                ConnectivityResult::Success
            }
        }
        Err(e) => ConnectivityResult::ConnectionError(e.to_string()),
    }
}

/// Write agent config atomically: write to temp file, then rename.
fn write_config_atomic(data_dir: &Path, result: &SetupResult) -> Result<()> {
    let agent_dir = data_dir.join("agent");
    std::fs::create_dir_all(&agent_dir)
        .context("Failed to create agent config directory")?;

    let mut config = serde_json::json!({
        "provider": result.provider,
        "model": result.model,
        "api_key": result.api_key,
        "base_url": result.base_url,
        "api_transport": result.api_transport,
        "preferred_client": result.preferred_client,
    });

    // Include capability tier self-assessment if set.
    if let Some(ref tier) = result.capability_tier {
        config["tier"] = serde_json::json!({
            "enabled": true,
            "user_self_assessment": tier,
        });
    }

    let config_str = serde_json::to_string_pretty(&config)
        .context("Failed to serialize config")?;

    let config_path = agent_dir.join("config.json");
    let tmp_path = agent_dir.join("config.json.tmp");

    std::fs::write(&tmp_path, &config_str)
        .context("Failed to write temporary config file")?;

    std::fs::rename(&tmp_path, &config_path)
        .context("Failed to atomically rename config file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_needs_setup_returns_true_when_no_config() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("agent").join("config.json");
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_needs_setup_returns_false_when_config_exists_with_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agent");
        fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        fs::write(
            &config_path,
            r#"{"provider": "anthropic", "model": "claude-sonnet-4-20250514"}"#,
        )
        .unwrap();
        assert!(!needs_setup_at(&config_path));
    }

    #[test]
    fn test_needs_setup_returns_true_when_provider_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agent");
        fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        fs::write(&config_path, r#"{"provider": "", "model": ""}"#).unwrap();
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_needs_setup_returns_true_when_invalid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agent");
        fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        fs::write(&config_path, "not valid json").unwrap();
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_needs_setup_returns_true_when_no_provider_field() {
        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agent");
        fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        fs::write(&config_path, r#"{"model": "gpt-4o"}"#).unwrap();
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_providers_list_has_minimum_count() {
        assert!(
            PROVIDERS.len() >= 10,
            "Expected at least 10 providers, got {}",
            PROVIDERS.len()
        );
    }

    #[test]
    fn test_providers_have_required_entries() {
        let required = [
            "anthropic",
            "openai",
            "openrouter",
            "groq",
            "together",
            "fireworks",
            "deepseek",
            "mistral",
            "featherless",
            "ollama",
            "lmstudio",
            "custom",
        ];
        for req in &required {
            assert!(
                PROVIDERS.iter().any(|p| p.id == *req),
                "Missing provider: {}",
                req
            );
        }
    }

    #[test]
    fn test_is_local_provider() {
        assert!(is_local_provider("ollama"));
        assert!(is_local_provider("lmstudio"));
        assert!(!is_local_provider("anthropic"));
        assert!(!is_local_provider("openai"));
    }

    #[test]
    fn test_write_config_atomic() {
        let tmp = tempfile::tempdir().unwrap();
        let result = SetupResult {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: "sk-test-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            api_transport: "anthropic".to_string(),
            preferred_client: "tui".to_string(),
            data_dir: tmp.path().to_path_buf(),
            capability_tier: Some("familiar".to_string()),
        };

        write_config_atomic(tmp.path(), &result).unwrap();

        let config_path = tmp.path().join("agent").join("config.json");
        assert!(config_path.exists());

        // Verify temp file was cleaned up (renamed away)
        let tmp_path = tmp.path().join("agent").join("config.json.tmp");
        assert!(!tmp_path.exists());

        // Verify config content
        let content: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(content["provider"], "anthropic");
        assert_eq!(content["model"], "claude-sonnet-4-20250514");
        assert_eq!(content["api_key"], "sk-test-key");
        assert_eq!(content["api_transport"], "anthropic");
        assert_eq!(content["preferred_client"], "tui");
    }
}
