//! Discord Bot integration stub.
//!
//! Requires `AMUX_DISCORD_TOKEN` environment variable to be set to a valid
//! Discord Bot token. When the token is present the provider is registered
//! with the gateway; actual Discord API calls require a Discord client crate
//! (e.g. `serenity` or `twilight`) which is not included in this scaffold.
//!
//! Extension points:
//! - Replace `connect()` with Discord Gateway WebSocket handshake + IDENTIFY.
//! - Replace `recv()` with MESSAGE_CREATE event deserialization.
//! - Replace `send()` with `POST /channels/{id}/messages` REST call.

use amux_protocol::{GatewayProviderBootstrap, GatewaySendRequest};
use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::runtime::{GatewayProvider, GatewayProviderEvent};

/// Discord gateway provider.
pub struct DiscordProvider {
    token: String,
    connected: bool,
}

impl DiscordProvider {
    /// Create a `DiscordProvider` from daemon bootstrap config.
    pub fn from_bootstrap(bootstrap: &GatewayProviderBootstrap) -> Result<Option<Self>> {
        if bootstrap.platform != "discord" || !bootstrap.enabled {
            return Ok(None);
        }
        let credentials: Value = serde_json::from_str(&bootstrap.credentials_json)
            .context("parse discord credentials")?;
        let token = credentials
            .get("token")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if token.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            token: token.to_string(),
            connected: false,
        }))
    }
}

impl GatewayProvider for DiscordProvider {
    fn platform(&self) -> &str {
        "discord"
    }

    fn connect<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            tracing::info!(
                "Discord provider: would connect to Gateway WebSocket (token: {}…)",
                &self.token[..8.min(self.token.len())],
            );
            // TODO: Implement Discord Gateway connection:
            // 1. GET /gateway/bot to obtain WebSocket URL.
            // 2. Connect via WebSocket, receive Hello (opcode 10).
            // 3. Send Identify (opcode 2) with token and intents.
            // 4. Begin heartbeat loop.
            tracing::warn!(
                "Discord API client is not configured — \
                 install a Discord client crate and implement the Gateway connection"
            );
            self.connected = true;
            Ok(())
        })
    }

    fn recv<'a>(
        &'a mut self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<GatewayProviderEvent>>> + Send + 'a>,
    > {
        Box::pin(async move {
            if !self.connected {
                bail!("Discord provider not connected");
            }
            // TODO: Read from the Discord Gateway WebSocket and match on
            // MESSAGE_CREATE dispatch events:
            //
            //   GatewayProviderEvent::Incoming(...)
            //
            // Filter out messages from bots (message.author.bot == true).
            Ok(None)
        })
    }

    fn send<'a>(
        &'a mut self,
        request: GatewaySendRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send + 'a>>
    {
        Box::pin(async move {
            if !self.connected {
                bail!("Discord provider not connected");
            }
            tracing::info!(
                channel = %request.channel_id,
                text = %request.content,
                "Discord: would send message via Create Message API"
            );
            // TODO: POST https://discord.com/api/v10/channels/{channel_id}/messages
            //   { "content": request.content }
            //   Authorization: Bot {self.token}
            tracing::warn!("Discord send is a stub — message not actually delivered");
            Ok(Some(format!(
                "discord:{}:{}",
                request.channel_id, request.correlation_id
            )))
        })
    }
}
