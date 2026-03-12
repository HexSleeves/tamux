//! Ephemeral per-command network isolation.
//! When sandbox is disabled, this provides standalone network toggling
//! via `unshare -n` on Linux.

pub struct NetworkWrappedCommand {
    pub program: String,
    pub args: Vec<String>,
}

/// Wrap a command with network isolation if `allow_network` is false.
/// On Linux, uses `unshare -n`. On other platforms, no-op.
pub fn wrap_network(command: &str, allow_network: bool) -> NetworkWrappedCommand {
    if allow_network {
        return NetworkWrappedCommand {
            program: "sh".to_string(),
            args: vec!["-c".to_string(), command.to_string()],
        };
    }

    #[cfg(target_os = "linux")]
    {
        if which_exists("unshare") {
            return NetworkWrappedCommand {
                program: "unshare".to_string(),
                args: vec![
                    "-n".to_string(),
                    "sh".to_string(),
                    "-c".to_string(),
                    command.to_string(),
                ],
            };
        }
    }

    tracing::warn!(
        "network isolation requested but no mechanism available; running without restriction"
    );
    NetworkWrappedCommand {
        program: "sh".to_string(),
        args: vec!["-c".to_string(), command.to_string()],
    }
}

#[cfg(target_os = "linux")]
fn which_exists(binary: &str) -> bool {
    std::process::Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
