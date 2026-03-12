//! Message router: translates incoming chat platform messages into daemon
//! managed-command requests.
//!
//! The routing logic recognises two command prefixes:
//!
//! - `!<command>`    — bang prefix (e.g. `!ls -la`)
//! - `/run <command>` — explicit run prefix (e.g. `/run cargo build`)
//!
//! Messages that do not match either prefix are ignored (return `None`).

use amux_protocol::{ManagedCommandRequest, ManagedCommandSource, SecurityLevel};

// ---------------------------------------------------------------------------
// Gateway message / response types
// ---------------------------------------------------------------------------

/// An inbound message received from a chat platform.
#[derive(Debug, Clone)]
pub struct GatewayMessage {
    /// Platform name (e.g. "slack", "telegram", "discord").
    pub platform: String,
    /// Platform-specific channel identifier.
    pub channel_id: String,
    /// Platform-specific user identifier.
    pub user_id: String,
    /// Raw message text.
    pub text: String,
    /// Unix timestamp (seconds since epoch).
    pub timestamp: u64,
}

/// A response to be delivered back to a chat platform channel.
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    /// Text body of the response.
    pub text: String,
    /// Target channel identifier.
    pub channel_id: String,
}

// ---------------------------------------------------------------------------
// Routing logic
// ---------------------------------------------------------------------------

/// Attempt to extract a daemon command from an incoming chat message.
///
/// Returns `Some(ManagedCommandRequest)` when the message matches a recognised
/// command prefix, or `None` if it should be ignored.
pub fn route_to_command(msg: &GatewayMessage) -> Option<ManagedCommandRequest> {
    let text = msg.text.trim();

    // Match `!<command>` or `/run <command>`.
    let command = if let Some(cmd) = text.strip_prefix('!') {
        cmd.trim().to_string()
    } else if let Some(cmd) = text.strip_prefix("/run ") {
        cmd.trim().to_string()
    } else {
        return None;
    };

    if command.is_empty() {
        return None;
    }

    Some(ManagedCommandRequest {
        command,
        rationale: format!(
            "Gateway command from {} user {} in #{}",
            msg.platform, msg.user_id, msg.channel_id,
        ),
        allow_network: true,
        sandbox_enabled: true,
        security_level: SecurityLevel::Moderate,
        cwd: None,
        language_hint: None,
        source: ManagedCommandSource::Gateway,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(text: &str) -> GatewayMessage {
        GatewayMessage {
            platform: "test".into(),
            channel_id: "ch-1".into(),
            user_id: "u-1".into(),
            text: text.into(),
            timestamp: 0,
        }
    }

    #[test]
    fn bang_prefix_routes() {
        let req = route_to_command(&make_msg("!ls -la")).unwrap();
        assert_eq!(req.command, "ls -la");
    }

    #[test]
    fn run_prefix_routes() {
        let req = route_to_command(&make_msg("/run cargo build")).unwrap();
        assert_eq!(req.command, "cargo build");
    }

    #[test]
    fn plain_text_ignored() {
        assert!(route_to_command(&make_msg("hello world")).is_none());
    }

    #[test]
    fn empty_bang_ignored() {
        assert!(route_to_command(&make_msg("!")).is_none());
    }

    #[test]
    fn whitespace_trimmed() {
        let req = route_to_command(&make_msg("  !echo hi  ")).unwrap();
        assert_eq!(req.command, "echo hi");
    }
}
