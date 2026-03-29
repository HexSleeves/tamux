//! Slack Bot API integration stub.
//!
//! Requires `AMUX_SLACK_TOKEN` environment variable to be set to a valid
//! Slack Bot token (xoxb-...). When the token is present the provider is
//! registered with the gateway; actual Slack API calls require a Slack client
//! crate (e.g. `slack-morphism` or a raw HTTP client) which is not included
//! in this scaffold.
//!
//! Extension points:
//! - Replace `connect()` with Slack WebSocket RTM or Events API setup.
//! - Replace `recv()` with real event deserialization from the Slack stream.
//! - Replace `send()` with `chat.postMessage` API calls.

use amux_protocol::{GatewayProviderBootstrap, GatewaySendRequest};
use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::runtime::{GatewayProvider, GatewayProviderEvent};

/// Slack gateway provider.
pub struct SlackProvider {
    token: String,
    connected: bool,
}

impl SlackProvider {
    /// Create a `SlackProvider` from daemon bootstrap config.
    pub fn from_bootstrap(bootstrap: &GatewayProviderBootstrap) -> Result<Option<Self>> {
        if bootstrap.platform != "slack" || !bootstrap.enabled {
            return Ok(None);
        }
        let credentials: Value =
            serde_json::from_str(&bootstrap.credentials_json).context("parse slack credentials")?;
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

impl GatewayProvider for SlackProvider {
    fn platform(&self) -> &str {
        "slack"
    }

    #[allow(unused)]
    fn connect<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            tracing::info!(
                "Slack provider: would connect using token {}…{}",
                &self.token[..6.min(self.token.len())],
                &self.token[self.token.len().saturating_sub(4)..],
            );
            // TODO: Establish a real Slack RTM/Events API WebSocket here.
            // For now, mark as connected but note the stub status.
            tracing::warn!(
                "Slack API client is not configured — \
                 install a Slack client crate and implement the WebSocket connection"
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
                bail!("Slack provider not connected");
            }
            // TODO: Poll the Slack WebSocket for incoming message events and
            // deserialize them into GatewayMessage structs.
            //
            // Example message mapping:
            //   SlackEvent::Message { channel, user, text, ts }
            //       -> GatewayProviderEvent::Incoming(...)
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
                bail!("Slack provider not connected");
            }
            tracing::info!(
                channel = %request.channel_id,
                text = %request.content,
                "Slack: would send message via chat.postMessage API"
            );
            // TODO: POST to https://slack.com/api/chat.postMessage with:
            //   { "channel": request.channel_id, "text": request.content }
            //   Authorization: Bearer {self.token}
            tracing::warn!("Slack send is a stub — message not actually delivered");
            Ok(Some(format!(
                "slack:{}:{}",
                request.channel_id, request.correlation_id
            )))
        })
    }
}
