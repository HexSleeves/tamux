//! First-run setup wizard for tamux.
//!
//! Connects to the daemon via IPC socket and configures the agent through
//! protocol messages. All config writes go through daemon IPC -- config.json
//! is never written or referenced as a daemon config source.
//!
//! Navigation uses crossterm arrow-key selection (not number input).
//! Provider list is queried from the daemon at runtime (no hardcoded list).

use amux_protocol::{AmuxCodec, ClientMessage, DaemonMessage};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{self, Stylize};
use crossterm::terminal;
use futures::{SinkExt, StreamExt};
use std::io::{self, Write};
use std::path::Path;
use tokio_util::codec::Framed;

// ---------------------------------------------------------------------------
// Local mirror of ProviderAuthState (daemon-side struct)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
struct ProviderAuthState {
    provider_id: String,
    provider_name: String,
    #[allow(dead_code)]
    authenticated: bool,
    auth_source: String,
    model: String,
    base_url: String,
}

// ---------------------------------------------------------------------------
// IPC connection helpers (private to wizard)
// ---------------------------------------------------------------------------

#[cfg(unix)]
async fn wizard_connect(
) -> Result<Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = std::path::PathBuf::from(runtime_dir).join("tamux-daemon.sock");
    let stream = tokio::net::UnixStream::connect(&path)
        .await
        .with_context(|| format!("cannot connect to daemon at {}", path.display()))?;
    Ok(Framed::new(stream, AmuxCodec))
}

#[cfg(windows)]
async fn wizard_connect(
) -> Result<Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>> {
    let addr = amux_protocol::default_tcp_addr();
    let stream = tokio::net::TcpStream::connect(&addr)
        .await
        .with_context(|| format!("cannot connect to daemon on {addr}"))?;
    Ok(Framed::new(stream, AmuxCodec))
}

async fn wizard_send(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    msg: ClientMessage,
) -> Result<()> {
    framed.send(msg).await.map_err(Into::into)
}

async fn wizard_recv(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
) -> Result<DaemonMessage> {
    framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("daemon closed connection"))?
        .map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Ensure daemon is running (Step 0 per D-02)
// ---------------------------------------------------------------------------

async fn ensure_daemon_running() -> Result<()> {
    // Try to connect first
    if wizard_connect().await.is_ok() {
        return Ok(());
    }

    // Daemon not reachable -- try to start it
    println!("Starting daemon...");
    let mut cmd = std::process::Command::new("tamux-daemon");
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Err(e) = cmd.spawn() {
        anyhow::bail!(
            "Could not start daemon: {e}\nPlease start it manually with: tamux-daemon"
        );
    }

    // Poll for daemon socket up to 5 seconds
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if wizard_connect().await.is_ok() {
            println!("Daemon started.");
            return Ok(());
        }
    }

    anyhow::bail!(
        "Daemon did not become reachable within 5 seconds.\n\
         Please start it manually with: tamux-daemon"
    )
}

// ---------------------------------------------------------------------------
// Crossterm arrow-key select list (per D-04)
// ---------------------------------------------------------------------------

/// Interactive select list with arrow-key navigation.
/// Returns `Some(index)` on Enter, `None` on Esc (only if `allow_esc` is true).
fn select_list(title: &str, items: &[(&str, &str)], allow_esc: bool) -> Result<Option<usize>> {
    use crossterm::cursor;

    let mut selected: usize = 0;

    terminal::enable_raw_mode().context("Failed to enable raw mode")?;

    // Helper to clean up raw mode on any exit path
    let result = (|| -> Result<Option<usize>> {
        loop {
            // Clear and render
            // Move cursor to start of list area and clear
            print!("\r");
            // Print title
            print!(
                "{}{}",
                style::SetForegroundColor(style::Color::White),
                style::SetAttribute(style::Attribute::Bold)
            );
            print!("{title}");
            println!(
                "{}{}",
                style::SetAttribute(style::Attribute::Reset),
                style::SetForegroundColor(style::Color::Reset)
            );
            println!();

            for (i, (label, desc)) in items.iter().enumerate() {
                if i == selected {
                    print!(
                        "  {} {} ",
                        style::SetForegroundColor(style::Color::Green),
                        ">".bold()
                    );
                    print!("{}", label.bold());
                    if !desc.is_empty() {
                        print!(" {}", style::SetForegroundColor(style::Color::DarkGrey));
                        print!("({desc})");
                    }
                    println!(
                        "{}",
                        style::SetForegroundColor(style::Color::Reset)
                    );
                } else {
                    print!("    ");
                    print!(
                        "{}{}",
                        style::SetForegroundColor(style::Color::Grey),
                        label
                    );
                    if !desc.is_empty() {
                        print!(" ({desc})");
                    }
                    println!("{}", style::SetForegroundColor(style::Color::Reset));
                }
            }

            io::stdout().flush()?;

            // Read key
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Up => {
                        if selected == 0 {
                            selected = items.len().saturating_sub(1);
                        } else {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        selected += 1;
                        if selected >= items.len() {
                            selected = 0;
                        }
                    }
                    KeyCode::Enter => {
                        return Ok(Some(selected));
                    }
                    KeyCode::Esc if allow_esc => {
                        return Ok(None);
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        anyhow::bail!("Setup cancelled by user");
                    }
                    _ => {}
                }
            }

            // Move cursor up to redraw (title + blank line + items)
            let lines_to_clear = items.len() + 2;
            print!(
                "{}",
                cursor::MoveUp(lines_to_clear as u16)
            );
            // Clear from cursor to end of screen
            print!("{}", terminal::Clear(terminal::ClearType::FromCursorDown));
        }
    })();

    terminal::disable_raw_mode().context("Failed to disable raw mode")?;

    result
}

// ---------------------------------------------------------------------------
// Crossterm text input (for API key etc.)
// ---------------------------------------------------------------------------

/// Interactive text input with optional masking.
/// Returns `Some(text)` on Enter, `None` on Esc.
fn text_input(prompt_text: &str, default: &str, masked: bool) -> Result<Option<String>> {
    if !default.is_empty() {
        print!("{prompt_text} [{default}]: ");
    } else {
        print!("{prompt_text}: ");
    }
    io::stdout().flush()?;

    terminal::enable_raw_mode().context("Failed to enable raw mode for input")?;

    let result = (|| -> Result<Option<String>> {
        let mut input = String::new();
        loop {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => {
                        println!();
                        let value = if input.is_empty() && !default.is_empty() {
                            default.to_string()
                        } else {
                            input
                        };
                        return Ok(Some(value));
                    }
                    KeyCode::Esc => {
                        println!();
                        return Ok(None);
                    }
                    KeyCode::Backspace => {
                        if input.pop().is_some() {
                            print!("\x08 \x08");
                            io::stdout().flush()?;
                        }
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        anyhow::bail!("Setup cancelled by user");
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        if masked {
                            print!("*");
                        } else {
                            print!("{c}");
                        }
                        io::stdout().flush()?;
                    }
                    _ => {}
                }
            }
        }
    })();

    terminal::disable_raw_mode().context("Failed to disable raw mode")?;

    result
}

// ---------------------------------------------------------------------------
// Helper: is local provider (no API key required)
// ---------------------------------------------------------------------------

/// Returns true if a provider is local (no API key required).
fn is_local_provider(id: &str) -> bool {
    matches!(id, "ollama" | "lmstudio")
}

// ---------------------------------------------------------------------------
// Setup detection
// ---------------------------------------------------------------------------

/// Check whether setup is needed by querying daemon config via IPC.
/// Returns `true` if daemon is unreachable or has no provider set.
pub async fn needs_setup_via_ipc() -> bool {
    let mut framed = match wizard_connect().await {
        Ok(f) => f,
        Err(_) => return true, // Can't reach daemon = needs setup
    };
    if wizard_send(&mut framed, ClientMessage::AgentGetConfig)
        .await
        .is_err()
    {
        return true;
    }
    match wizard_recv(&mut framed).await {
        Ok(DaemonMessage::AgentConfigResponse { config_json }) => {
            let value: serde_json::Value = match serde_json::from_str(&config_json) {
                Ok(v) => v,
                Err(_) => return true,
            };
            match value.get("provider").and_then(|v| v.as_str()) {
                Some(s) if !s.is_empty() => false,
                _ => true,
            }
        }
        _ => true,
    }
}

/// Legacy check: inspect config.json on disk.
/// Used only as fallback when daemon is not running yet (first-ever run).
pub fn needs_setup_legacy() -> bool {
    let config_path = amux_protocol::amux_data_dir()
        .join("agent")
        .join("config.json");
    needs_setup_at(&config_path)
}

/// Check whether setup is needed for a specific config path (legacy file check).
fn needs_setup_at(config_path: &Path) -> bool {
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

// ---------------------------------------------------------------------------
// Main wizard entry point
// ---------------------------------------------------------------------------

/// Run the setup wizard. Connects to the daemon via IPC, queries provider list,
/// and configures the agent through IPC messages. Never writes config.json.
pub async fn run_setup_wizard() -> Result<()> {
    // Step 0: Ensure daemon is running
    ensure_daemon_running().await?;

    // Open a long-lived IPC connection for the wizard
    let mut framed = wizard_connect()
        .await
        .context("Failed to connect to daemon for setup")?;

    // Step 1: Welcome banner
    println!();
    println!(
        "{}",
        "tamux -- The Agent That Lives".bold()
    );
    println!("First-time setup");
    println!();

    // Step 2: Tier self-assessment (per D-06)
    let tier_items: Vec<(&str, &str)> = vec![
        ("Just getting started", "newcomer"),
        ("I've used chatbots and assistants", "familiar"),
        ("I run automations and scripting", "power_user"),
        ("I build agent systems", "expert"),
    ];

    let tier_idx = select_list(
        "How familiar are you with AI agents?",
        &tier_items,
        false,
    )?
    .expect("tier selection is required");

    let tier_string = tier_items[tier_idx].1.to_string();

    // Send tier override via IPC
    wizard_send(
        &mut framed,
        ClientMessage::AgentSetTierOverride {
            tier: Some(tier_string.clone()),
        },
    )
    .await
    .context("Failed to set tier override")?;

    // AgentSetTierOverride is fire-and-forget (no response expected per server.rs)
    // Brief pause to let daemon process it
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    println!();

    // Step 3: Provider selection (per D-07)
    // Query daemon for provider list
    wizard_send(&mut framed, ClientMessage::AgentGetProviderAuthStates)
        .await
        .context("Failed to query provider list")?;

    let providers: Vec<ProviderAuthState> = match wizard_recv(&mut framed).await? {
        DaemonMessage::AgentProviderAuthStates { states_json } => {
            serde_json::from_str(&states_json)
                .context("Failed to parse provider auth states from daemon")?
        }
        other => {
            anyhow::bail!("Unexpected daemon response: {other:?}");
        }
    };

    if providers.is_empty() {
        anyhow::bail!("Daemon returned empty provider list. Is the daemon running correctly?");
    }

    let provider_items: Vec<(&str, &str)> = providers
        .iter()
        .map(|p| (p.provider_name.as_str(), p.provider_id.as_str()))
        .collect();

    let provider_idx = select_list("Select your LLM provider:", &provider_items, false)?
        .expect("provider selection is required");

    let selected_provider = &providers[provider_idx];
    let provider_id = selected_provider.provider_id.clone();
    let provider_name = selected_provider.provider_name.clone();
    let base_url = selected_provider.base_url.clone();
    let default_model = selected_provider.model.clone();
    let auth_source = selected_provider.auth_source.clone();

    println!();

    // Step 4: API key (per D-05)
    if is_local_provider(&provider_id) {
        println!("Local provider -- no API key needed.");
    } else {
        let api_key = text_input(
            &format!("Enter API key for {provider_name}"),
            "",
            true,
        )?
        .unwrap_or_default();

        if api_key.is_empty() {
            println!("No API key entered. You can set it later with `tamux setup`.");
        } else {
            // Send via IPC: AgentLoginProvider
            wizard_send(
                &mut framed,
                ClientMessage::AgentLoginProvider {
                    provider_id: provider_id.clone(),
                    api_key: api_key.clone(),
                    base_url: String::new(),
                },
            )
            .await
            .context("Failed to send API key to daemon")?;

            // Await response (AgentProviderAuthStates) to confirm
            match wizard_recv(&mut framed).await? {
                DaemonMessage::AgentProviderAuthStates { .. } => {
                    println!("API key saved.");
                }
                DaemonMessage::AgentError { message } => {
                    println!("Warning: daemon error setting API key: {message}");
                }
                _ => {
                    // Accept any response -- key was sent
                }
            }

            println!();

            // Step 6: Connectivity test (required per D-08)
            println!("Testing connection to {provider_name}...");
            wizard_send(
                &mut framed,
                ClientMessage::AgentValidateProvider {
                    provider_id: provider_id.clone(),
                    base_url: base_url.clone(),
                    api_key,
                    auth_source: auth_source.clone(),
                },
            )
            .await
            .context("Failed to send connectivity test")?;

            match wizard_recv(&mut framed).await? {
                DaemonMessage::AgentProviderValidation {
                    valid, error, ..
                } => {
                    if valid {
                        println!(
                            "{}",
                            "Connection successful!".with(style::Color::Green)
                        );
                    } else if let Some(ref err) = error {
                        if err.contains("401") || err.contains("403") {
                            println!(
                                "API key was rejected. You can update it later with `tamux setup`."
                            );
                        } else {
                            println!(
                                "Could not reach provider: {err}. You can fix this later with `tamux setup`."
                            );
                        }
                    } else {
                        println!(
                            "Validation returned invalid with no error detail. You can retry with `tamux setup`."
                        );
                    }
                }
                DaemonMessage::AgentError { message } => {
                    println!(
                        "Could not validate provider: {message}. You can fix this later with `tamux setup`."
                    );
                }
                _ => {
                    println!("Unexpected response during connectivity test.");
                }
            }
        }
    }

    println!();

    // Step 5: Set as active provider
    wizard_send(
        &mut framed,
        ClientMessage::AgentSetConfigItem {
            key_path: "provider".to_string(),
            value_json: format!("\"{}\"", provider_id),
        },
    )
    .await
    .context("Failed to set active provider")?;

    // Set default model
    if !default_model.is_empty() {
        wizard_send(
            &mut framed,
            ClientMessage::AgentSetConfigItem {
                key_path: "model".to_string(),
                value_json: format!("\"{}\"", default_model),
            },
        )
        .await
        .context("Failed to set default model")?;
    }

    // Brief pause for daemon to process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Step 7: Completion
    println!(
        "{}",
        format!("Setup complete! Provider: {provider_name}").bold()
    );
    println!("Run 'tamux' to start using tamux.");

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_list_wraps_index() {
        // Test the wrapping logic used in select_list.
        // We can't run the full interactive function in tests, but we can verify
        // the index arithmetic.
        let len = 4usize;

        // Wrap down from 0 -> last
        let mut idx = 0usize;
        if idx == 0 {
            idx = len.saturating_sub(1);
        } else {
            idx -= 1;
        }
        assert_eq!(idx, 3);

        // Wrap up from last -> 0
        idx = 3;
        idx += 1;
        if idx >= len {
            idx = 0;
        }
        assert_eq!(idx, 0);

        // Normal move down
        idx = 1;
        idx += 1;
        if idx >= len {
            idx = 0;
        }
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_needs_setup_legacy_returns_true_when_no_config() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("agent").join("config.json");
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_needs_setup_returns_false_when_config_has_provider() {
        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agent");
        std::fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        std::fs::write(
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
        std::fs::create_dir_all(&agent_dir).unwrap();
        let config_path = agent_dir.join("config.json");
        std::fs::write(&config_path, r#"{"provider": "", "model": ""}"#).unwrap();
        assert!(needs_setup_at(&config_path));
    }

    #[test]
    fn test_is_local_provider() {
        assert!(is_local_provider("ollama"));
        assert!(is_local_provider("lmstudio"));
        assert!(!is_local_provider("anthropic"));
        assert!(!is_local_provider("openai"));
    }
}
