//! Concierge agent — proactive welcome greetings and lightweight ops assistant.

use super::types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Well-known thread ID for the concierge.
pub const CONCIERGE_THREAD_ID: &str = "concierge";

pub struct ConciergeEngine {
    config: Arc<RwLock<AgentConfig>>,
    event_tx: broadcast::Sender<AgentEvent>,
    http_client: reqwest::Client,
    pending_welcome_ids: RwLock<Vec<String>>,
}

impl ConciergeEngine {
    pub fn new(
        config: Arc<RwLock<AgentConfig>>,
        event_tx: broadcast::Sender<AgentEvent>,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            config,
            event_tx,
            http_client,
            pending_welcome_ids: RwLock::new(Vec::new()),
        }
    }

    /// Initialize the concierge — ensure the pinned thread exists.
    pub async fn initialize(
        &self,
        threads: &RwLock<std::collections::HashMap<String, AgentThread>>,
    ) {
        let mut threads_guard = threads.write().await;
        if !threads_guard.contains_key(CONCIERGE_THREAD_ID) {
            let now = super::now_millis();
            let thread = AgentThread {
                id: CONCIERGE_THREAD_ID.to_string(),
                title: "Concierge".to_string(),
                created_at: now,
                updated_at: now,
                messages: Vec::new(),
                pinned: true,
                upstream_thread_id: None,
                upstream_transport: None,
                upstream_provider: None,
                upstream_model: None,
                upstream_assistant_id: None,
                total_input_tokens: 0,
                total_output_tokens: 0,
            };
            threads_guard.insert(CONCIERGE_THREAD_ID.to_string(), thread);
            tracing::info!("concierge: created pinned thread");
        }
    }

    /// Called when a client subscribes to agent events.
    /// Prunes stale welcome messages, gathers context, and emits a new welcome.
    pub async fn on_client_connected(
        &self,
        threads: &RwLock<std::collections::HashMap<String, AgentThread>>,
    ) {
        let config = self.config.read().await;
        if !config.concierge.enabled {
            return;
        }

        // Prune any stale welcome messages from a previous session.
        self.prune_welcome_messages(threads).await;

        let detail_level = config.concierge.detail_level;
        drop(config);

        // Gather context and build welcome.
        let (content, actions) = self.gather_and_compose(threads, detail_level).await;

        if content.is_empty() {
            return;
        }

        // Add welcome message to the concierge thread.
        let msg_id = format!("welcome_{}", super::now_millis());
        {
            let mut threads_guard = threads.write().await;
            if let Some(thread) = threads_guard.get_mut(CONCIERGE_THREAD_ID) {
                thread.messages.push(AgentMessage {
                    role: MessageRole::Assistant,
                    content: content.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                    tool_name: None,
                    tool_arguments: None,
                    tool_status: None,
                    input_tokens: 0,
                    output_tokens: 0,
                    provider: Some("concierge".into()),
                    model: None,
                    api_transport: None,
                    response_id: None,
                    reasoning: None,
                    timestamp: super::now_millis(),
                });
                thread.updated_at = super::now_millis();
            }
        }

        // Track for later pruning.
        self.pending_welcome_ids.write().await.push(msg_id);

        // Emit the welcome event.
        let actions_json = serde_json::to_string(&actions).unwrap_or_else(|_| "[]".into());
        let _ = self.event_tx.send(AgentEvent::ConciergeWelcome {
            thread_id: CONCIERGE_THREAD_ID.to_string(),
            content,
            detail_level: format!("{:?}", detail_level).to_lowercase(),
            actions_json,
        });
    }

    /// Prune pending welcome messages from the concierge thread.
    pub async fn prune_welcome_messages(
        &self,
        threads: &RwLock<std::collections::HashMap<String, AgentThread>>,
    ) {
        let ids = {
            let mut guard = self.pending_welcome_ids.write().await;
            std::mem::take(&mut *guard)
        };
        if ids.is_empty() {
            return;
        }
        // Remove from in-memory thread (remove last N assistant messages matching welcome pattern).
        let mut threads_guard = threads.write().await;
        if let Some(thread) = threads_guard.get_mut(CONCIERGE_THREAD_ID) {
            let count = ids.len();
            // Remove the last `count` assistant messages (the welcome batch).
            let mut removed = 0;
            thread.messages.retain(|msg| {
                if removed < count
                    && msg.role == MessageRole::Assistant
                    && msg.provider.as_deref() == Some("concierge")
                {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Gather context based on detail level and compose the welcome message.
    async fn gather_and_compose(
        &self,
        threads: &RwLock<std::collections::HashMap<String, AgentThread>>,
        detail_level: ConciergeDetailLevel,
    ) -> (String, Vec<ConciergeAction>) {
        let threads_guard = threads.read().await;

        // Find most recent non-concierge thread.
        let mut recent_threads: Vec<&AgentThread> = threads_guard
            .values()
            .filter(|t| t.id != CONCIERGE_THREAD_ID && !t.messages.is_empty())
            .collect();
        recent_threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let last_thread = recent_threads.first();
        let mut actions = Vec::new();

        // Build content based on detail level.
        let content = match detail_level {
            ConciergeDetailLevel::Minimal => {
                if let Some(thread) = last_thread {
                    let date = format_timestamp(thread.updated_at);
                    actions.push(ConciergeAction {
                        label: format!("Continue: {}", truncate_str(&thread.title, 40)),
                        action_type: ConciergeActionType::ContinueSession,
                        thread_id: Some(thread.id.clone()),
                    });
                    actions.push(ConciergeAction {
                        label: "Start new session".into(),
                        action_type: ConciergeActionType::StartNew,
                        thread_id: None,
                    });
                    actions.push(ConciergeAction {
                        label: "Search history".into(),
                        action_type: ConciergeActionType::Search,
                        thread_id: None,
                    });
                    format!(
                        "Welcome back! Last session: **{}** ({}). {} messages.",
                        thread.title,
                        date,
                        thread.messages.len()
                    )
                } else {
                    actions.push(ConciergeAction {
                        label: "Start new session".into(),
                        action_type: ConciergeActionType::StartNew,
                        thread_id: None,
                    });
                    "Welcome to tamux! Ready to start your first session.".into()
                }
            }
            _ => {
                // For ContextSummary, ProactiveTriage, DailyBriefing — build a richer prompt.
                // For now, use the same template as Minimal but with more context.
                // LLM-based summarization will be added when the concierge LLM call path is wired.
                let mut parts = Vec::new();

                if let Some(thread) = last_thread {
                    let date = format_timestamp(thread.updated_at);
                    let msg_count = thread.messages.len();
                    let last_msgs: Vec<String> = thread
                        .messages
                        .iter()
                        .rev()
                        .take(3)
                        .map(|m| {
                            let role = match m.role {
                                MessageRole::User => "You",
                                MessageRole::Assistant => "Agent",
                                _ => "System",
                            };
                            let snippet: String = m.content.chars().take(80).collect();
                            format!(
                                "  {} — {}: {}",
                                format_timestamp(m.timestamp),
                                role,
                                snippet
                            )
                        })
                        .collect();
                    let last_msgs_reversed: Vec<&String> = last_msgs.iter().rev().collect();

                    parts.push(format!(
                        "**Last session:** {} ({}, {} messages)",
                        thread.title, date, msg_count
                    ));
                    if !last_msgs_reversed.is_empty() {
                        parts.push("Recent activity:".into());
                        for msg in last_msgs_reversed {
                            parts.push(msg.clone());
                        }
                    }

                    actions.push(ConciergeAction {
                        label: format!("Continue: {}", truncate_str(&thread.title, 40)),
                        action_type: ConciergeActionType::ContinueSession,
                        thread_id: Some(thread.id.clone()),
                    });
                }

                // Show additional recent sessions.
                if recent_threads.len() > 1 {
                    let others: Vec<String> = recent_threads
                        [1..std::cmp::min(4, recent_threads.len())]
                        .iter()
                        .map(|t| format!("  - {} ({})", t.title, format_timestamp(t.updated_at)))
                        .collect();
                    if !others.is_empty() {
                        parts.push("**Other recent sessions:**".into());
                        for other in others {
                            parts.push(other);
                        }
                    }
                }

                actions.push(ConciergeAction {
                    label: "Start new session".into(),
                    action_type: ConciergeActionType::StartNew,
                    thread_id: None,
                });
                actions.push(ConciergeAction {
                    label: "Search history".into(),
                    action_type: ConciergeActionType::Search,
                    thread_id: None,
                });
                actions.push(ConciergeAction {
                    label: "Dismiss".into(),
                    action_type: ConciergeActionType::Dismiss,
                    thread_id: None,
                });

                if parts.is_empty() {
                    "Welcome to tamux! Ready to start your first session.".into()
                } else {
                    parts.join("\n")
                }
            }
        };

        (content, actions)
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}\u{2026}")
    }
}

fn format_timestamp(ts: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let dt = UNIX_EPOCH + Duration::from_millis(ts);
    // Simple relative time formatting.
    let now = std::time::SystemTime::now();
    let elapsed = now.duration_since(dt).unwrap_or_default();
    let secs = elapsed.as_secs();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}
