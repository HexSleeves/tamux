//! Heartbeat system — periodic health checks and notifications.

use super::*;
use chrono::Timelike;

/// Pure function: check if a given hour falls within a quiet window.
///
/// Returns `true` when heartbeat execution should be suppressed.
/// Handles midnight-wrap ranges (e.g., start=22, end=6 means 22:00-05:59).
/// When `dnd` is `true`, always returns `true` (manual do-not-disturb).
pub(super) fn check_quiet_window(
    hour: u32,
    start: Option<u32>,
    end: Option<u32>,
    dnd: bool,
) -> bool {
    if dnd {
        return true;
    }
    let (s, e) = match (start, end) {
        (Some(s), Some(e)) => (s, e),
        _ => return false,
    };
    if s <= e {
        // Same-day range: e.g., 9..17 means 9:00-16:59
        hour >= s && hour < e
    } else {
        // Midnight wrap: e.g., 22..6 means 22:00-05:59
        hour >= s || hour < e
    }
}

/// Resolve the effective cron expression from config fields.
///
/// Prefers `heartbeat_cron` if set, otherwise converts `heartbeat_interval_mins`
/// via the legacy helper. Per D-08.
pub(super) fn resolve_cron_from_config(config: &AgentConfig) -> String {
    config
        .heartbeat_cron
        .clone()
        .unwrap_or_else(|| interval_mins_to_cron(config.heartbeat_interval_mins))
}

impl AgentEngine {
    /// Check if current time falls within quiet hours or DND is active. Per D-07.
    pub(super) async fn is_quiet_hours(&self) -> bool {
        let config = self.config.read().await;
        let hour = chrono::Local::now().hour();
        check_quiet_window(
            hour,
            config.quiet_hours_start,
            config.quiet_hours_end,
            config.dnd_enabled,
        )
    }

    /// Resolve the effective cron expression from config. Per D-08.
    /// Prefers heartbeat_cron if set, otherwise converts heartbeat_interval_mins.
    pub(super) async fn resolve_heartbeat_cron(&self) -> String {
        let config = self.config.read().await;
        resolve_cron_from_config(&config)
    }

    pub async fn get_heartbeat_items(&self) -> Vec<HeartbeatItem> {
        self.heartbeat_items.read().await.clone()
    }

    pub async fn set_heartbeat_items(&self, items: Vec<HeartbeatItem>) {
        *self.heartbeat_items.write().await = items;
        self.persist_heartbeat().await;
    }

    pub(super) async fn run_heartbeat(&self) -> Result<()> {
        let items = self.heartbeat_items.read().await.clone();
        let now = now_millis();

        for item in &items {
            if !item.enabled {
                continue;
            }

            let interval_ms = if item.interval_minutes > 0 {
                item.interval_minutes * 60 * 1000
            } else {
                self.config.read().await.heartbeat_interval_mins * 60 * 1000
            };

            let due = match item.last_run_at {
                Some(last) => now - last >= interval_ms,
                None => true,
            };

            if !due {
                continue;
            }

            let prompt = format!(
                "Heartbeat check: {}\n\n\
                 Respond with HEARTBEAT_OK if everything is normal, \
                 or HEARTBEAT_ALERT: <explanation> if something needs attention.",
                item.prompt
            );

            let result = match self.send_message(None, &prompt).await {
                Ok(thread_id) => {
                    // Check the last assistant message for OK/ALERT
                    let threads = self.threads.read().await;
                    let response = threads
                        .get(&thread_id)
                        .and_then(|t| {
                            t.messages
                                .iter()
                                .rev()
                                .find(|m| m.role == MessageRole::Assistant)
                                .map(|m| m.content.clone())
                        })
                        .unwrap_or_default();

                    if response.contains("HEARTBEAT_OK") {
                        (HeartbeatOutcome::Ok, "OK".into())
                    } else if response.contains("HEARTBEAT_ALERT") {
                        (HeartbeatOutcome::Alert, response)
                    } else {
                        (HeartbeatOutcome::Ok, response)
                    }
                }
                Err(e) => (HeartbeatOutcome::Error, format!("Error: {e}")),
            };

            let _ = self.event_tx.send(AgentEvent::HeartbeatResult {
                item_id: item.id.clone(),
                result: result.0,
                message: result.1.clone(),
            });

            // Update item state
            {
                let mut items = self.heartbeat_items.write().await;
                if let Some(i) = items.iter_mut().find(|i| i.id == item.id) {
                    i.last_run_at = Some(now);
                    i.last_result = Some(result.0);
                    i.last_message = Some(result.1);
                }
            }

            // If alert and notify enabled, send notification
            if result.0 == HeartbeatOutcome::Alert && item.notify_on_alert {
                let _ = self.event_tx.send(AgentEvent::Notification {
                    title: format!("Heartbeat Alert: {}", item.label),
                    body: item.last_message.clone().unwrap_or_default(),
                    severity: NotificationSeverity::Alert,
                    channels: item.notify_channels.clone(),
                });
            }
        }

        self.persist_heartbeat().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── check_quiet_window pure function tests ─────────────────────────

    #[test]
    fn quiet_hours_within_midnight_wrap_window() {
        // start=22, end=6, hour=23 → quiet
        assert!(check_quiet_window(23, Some(22), Some(6), false));
    }

    #[test]
    fn quiet_hours_outside_midnight_wrap_window() {
        // start=22, end=6, hour=12 → not quiet
        assert!(!check_quiet_window(12, Some(22), Some(6), false));
    }

    #[test]
    fn quiet_hours_midnight_wrap_early_morning() {
        // start=22, end=6, hour=3 → quiet (early morning within wrap)
        assert!(check_quiet_window(3, Some(22), Some(6), false));
    }

    #[test]
    fn quiet_hours_midnight_wrap_boundary_end() {
        // start=22, end=6, hour=6 → NOT quiet (end hour is exclusive)
        assert!(!check_quiet_window(6, Some(22), Some(6), false));
    }

    #[test]
    fn quiet_hours_midnight_wrap_boundary_start() {
        // start=22, end=6, hour=22 → quiet (start hour is inclusive)
        assert!(check_quiet_window(22, Some(22), Some(6), false));
    }

    #[test]
    fn dnd_enabled_overrides_everything() {
        // dnd=true → always quiet regardless of hour or window config
        assert!(check_quiet_window(12, None, None, true));
        assert!(check_quiet_window(12, Some(22), Some(6), true));
        assert!(check_quiet_window(0, Some(9), Some(17), true));
    }

    #[test]
    fn no_quiet_hours_configured_and_no_dnd() {
        // No quiet hours, no DND → never quiet
        assert!(!check_quiet_window(12, None, None, false));
        assert!(!check_quiet_window(0, None, None, false));
        assert!(!check_quiet_window(23, None, None, false));
    }

    #[test]
    fn same_day_range_inside() {
        // start=9, end=17, hour=12 → quiet
        assert!(check_quiet_window(12, Some(9), Some(17), false));
    }

    #[test]
    fn same_day_range_outside() {
        // start=9, end=17, hour=20 → not quiet
        assert!(!check_quiet_window(20, Some(9), Some(17), false));
    }

    #[test]
    fn partial_config_only_start_set() {
        // Only start set (no end) → not quiet
        assert!(!check_quiet_window(23, Some(22), None, false));
    }

    #[test]
    fn partial_config_only_end_set() {
        // Only end set (no start) → not quiet
        assert!(!check_quiet_window(3, None, Some(6), false));
    }

    // ── resolve_cron_from_config tests ─────────────────────────────────

    #[test]
    fn resolve_cron_prefers_explicit_cron() {
        let config = AgentConfig {
            heartbeat_cron: Some("0 * * * *".to_string()),
            heartbeat_interval_mins: 15,
            ..AgentConfig::default()
        };
        assert_eq!(resolve_cron_from_config(&config), "0 * * * *");
    }

    #[test]
    fn resolve_cron_falls_back_to_interval_mins() {
        let config = AgentConfig {
            heartbeat_cron: None,
            heartbeat_interval_mins: 15,
            ..AgentConfig::default()
        };
        assert_eq!(resolve_cron_from_config(&config), "*/15 * * * *");
    }

    #[test]
    fn resolve_cron_with_hourly_interval() {
        let config = AgentConfig {
            heartbeat_cron: None,
            heartbeat_interval_mins: 60,
            ..AgentConfig::default()
        };
        assert_eq!(resolve_cron_from_config(&config), "0 * * * *");
    }

    #[test]
    fn resolve_cron_explicit_overrides_interval() {
        let config = AgentConfig {
            heartbeat_cron: Some("30 2 * * *".to_string()),
            heartbeat_interval_mins: 60,
            ..AgentConfig::default()
        };
        assert_eq!(resolve_cron_from_config(&config), "30 2 * * *");
    }
}
