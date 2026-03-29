//! Gateway message normalization.
//!
//! Platform adapters should normalize inbound provider payloads here and pass
//! the resulting messages upstream. Command interpretation belongs in the
//! daemon, not in `tamux-gateway`.

/// An inbound message received from a chat platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayMessage {
    /// Platform name (for example `slack`, `discord`, `telegram`).
    pub platform: String,
    /// Platform-specific channel identifier.
    pub channel_id: String,
    /// Platform-specific user identifier.
    pub user_id: String,
    /// Normalized message text.
    pub text: String,
    /// Unix timestamp (seconds since epoch).
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawGatewayMessage<'a> {
    pub platform: &'a str,
    pub channel_id: &'a str,
    pub user_id: &'a str,
    pub text: &'a str,
    pub timestamp: u64,
}

/// Normalize a provider payload into a daemon-facing gateway message.
pub fn normalize_message(raw: RawGatewayMessage<'_>) -> Option<GatewayMessage> {
    let platform = raw.platform.trim().to_ascii_lowercase();
    let channel_id = raw.channel_id.trim().to_string();
    let user_id = raw.user_id.trim().to_string();
    let text = raw.text.trim().to_string();

    if platform.is_empty() || channel_id.is_empty() || user_id.is_empty() || text.is_empty() {
        return None;
    }

    Some(GatewayMessage {
        platform,
        channel_id,
        user_id,
        text,
        timestamp: raw.timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_router_normalizes_messages_without_daemon_command_routing() {
        let normalized = normalize_message(RawGatewayMessage {
            platform: "slack",
            channel_id: "C123",
            user_id: "U123",
            text: "  !deploy now  ",
            timestamp: 1_700_000_000,
        })
        .expect("message should normalize");

        assert_eq!(normalized.platform, "slack");
        assert_eq!(normalized.channel_id, "C123");
        assert_eq!(normalized.user_id, "U123");
        assert_eq!(normalized.text, "!deploy now");
        assert_eq!(normalized.timestamp, 1_700_000_000);
    }

    #[test]
    fn gateway_router_rejects_empty_content() {
        assert!(normalize_message(RawGatewayMessage {
            platform: "telegram",
            channel_id: "777",
            user_id: "alice",
            text: "   ",
            timestamp: 1,
        })
        .is_none());
    }
}
