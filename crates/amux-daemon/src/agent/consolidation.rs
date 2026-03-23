//! Idle-time memory consolidation -- trace review, fact aging, heuristic promotion.

use super::*;

/// Default memory fact decay half-life in hours (~69h, per D-04).
const DEFAULT_HALF_LIFE_HOURS: f64 = 69.0;

/// Default idle threshold in milliseconds (5 minutes, per D-01).
#[allow(dead_code)]
const DEFAULT_IDLE_THRESHOLD_MS: u64 = 5 * 60 * 1000;

/// Default consolidation tick budget in seconds (per D-02).
#[allow(dead_code)]
const DEFAULT_BUDGET_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Check all four idle conditions required for consolidation per D-01.
/// Returns true only when ALL conditions are simultaneously met:
/// no active tasks, no active goal runs, no active streams, and operator idle > threshold.
pub(super) fn is_idle_for_consolidation(
    active_task_count: usize,
    running_goal_count: usize,
    active_stream_count: usize,
    last_presence_at: Option<u64>,
    now: u64,
    idle_threshold_ms: u64,
) -> bool {
    if active_task_count > 0 || running_goal_count > 0 || active_stream_count > 0 {
        return false;
    }
    match last_presence_at {
        Some(last) => now.saturating_sub(last) >= idle_threshold_ms,
        None => true,
    }
}

/// Compute current confidence for a memory fact based on exponential decay.
/// Returns a value in 0.0..=1.0. A fact confirmed exactly `half_life_hours` ago
/// will have confidence ~0.5. Active facts (recently confirmed) stay near 1.0.
pub(super) fn compute_decay_confidence(
    last_confirmed_at: u64,
    now: u64,
    half_life_hours: f64,
) -> f64 {
    if last_confirmed_at == 0 || half_life_hours <= 0.0 {
        return 0.0;
    }
    let age_ms = now.saturating_sub(last_confirmed_at) as f64;
    let age_hours = age_ms / 3_600_000.0;
    let lambda = 2.0_f64.ln() / half_life_hours;
    let confidence = (-lambda * age_hours).exp();
    confidence.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Entry point (stub -- filled in Plan 02)
// ---------------------------------------------------------------------------

impl AgentEngine {
    /// Run a consolidation tick if idle conditions are met. Per D-03, this is called
    /// from within run_structured_heartbeat_adaptive() as a sub-phase.
    pub(super) async fn maybe_run_consolidation_if_idle(
        &self,
        _budget: std::time::Duration,
    ) -> Option<ConsolidationResult> {
        let config = self.config.read().await.clone();
        if !config.consolidation.enabled {
            return None;
        }

        // Check idle conditions per D-01
        let active_tasks = self
            .tasks
            .lock()
            .await
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::InProgress))
            .count();
        let running_goals = self
            .goal_runs
            .lock()
            .await
            .iter()
            .filter(|g| matches!(g.status, GoalRunStatus::Running | GoalRunStatus::Planning))
            .count();
        let active_streams = self.stream_cancellations.lock().await.len();
        let last_presence = self.anticipatory.read().await.last_presence_at;
        let now = now_millis();
        let idle_threshold_ms = config.consolidation.idle_threshold_secs * 1000;

        if !is_idle_for_consolidation(
            active_tasks,
            running_goals,
            active_streams,
            last_presence,
            now,
            idle_threshold_ms,
        ) {
            return None;
        }

        tracing::info!("idle conditions met -- starting consolidation tick");
        // Consolidation sub-tasks will be added in Plan 02
        Some(ConsolidationResult::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_idle_for_consolidation tests ──────────────────────────────────

    #[test]
    fn idle_returns_true_when_all_conditions_met() {
        assert!(is_idle_for_consolidation(
            0,
            0,
            0,
            Some(1000),
            1000 + DEFAULT_IDLE_THRESHOLD_MS,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    #[test]
    fn idle_returns_false_with_active_task() {
        assert!(!is_idle_for_consolidation(
            1,
            0,
            0,
            Some(0),
            DEFAULT_IDLE_THRESHOLD_MS + 1,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    #[test]
    fn idle_returns_false_with_active_goal_run() {
        assert!(!is_idle_for_consolidation(
            0,
            1,
            0,
            Some(0),
            DEFAULT_IDLE_THRESHOLD_MS + 1,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    #[test]
    fn idle_returns_false_with_active_stream() {
        assert!(!is_idle_for_consolidation(
            0,
            0,
            1,
            Some(0),
            DEFAULT_IDLE_THRESHOLD_MS + 1,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    #[test]
    fn idle_returns_false_with_recent_presence() {
        // Presence was just 1ms ago — not idle yet
        assert!(!is_idle_for_consolidation(
            0,
            0,
            0,
            Some(10_000),
            10_001,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    #[test]
    fn idle_returns_true_when_no_presence_recorded() {
        // None means no operator has connected — safe to consolidate
        assert!(is_idle_for_consolidation(
            0,
            0,
            0,
            None,
            1000,
            DEFAULT_IDLE_THRESHOLD_MS,
        ));
    }

    // ── compute_decay_confidence tests ───────────────────────────────────

    #[test]
    fn decay_returns_half_at_half_life() {
        let now = 1_000_000_000u64;
        let half_life_ms = (DEFAULT_HALF_LIFE_HOURS * 3_600_000.0) as u64;
        let last_confirmed = now - half_life_ms;
        let confidence = compute_decay_confidence(last_confirmed, now, DEFAULT_HALF_LIFE_HOURS);
        // Should be ~0.5 at exactly one half-life
        assert!(
            (confidence - 0.5).abs() < 0.01,
            "expected ~0.5, got {confidence}"
        );
    }

    #[test]
    fn decay_returns_near_one_for_just_confirmed() {
        let now = 1_000_000_000u64;
        let confidence = compute_decay_confidence(now, now, DEFAULT_HALF_LIFE_HOURS);
        assert!(
            (confidence - 1.0).abs() < 0.001,
            "expected ~1.0, got {confidence}"
        );
    }

    #[test]
    fn decay_returns_zero_for_zero_timestamp() {
        let confidence = compute_decay_confidence(0, 1_000_000, DEFAULT_HALF_LIFE_HOURS);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn decay_returns_zero_for_nonpositive_half_life() {
        let confidence = compute_decay_confidence(500_000, 1_000_000, 0.0);
        assert_eq!(confidence, 0.0);
        let confidence = compute_decay_confidence(500_000, 1_000_000, -5.0);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn decay_clamps_to_valid_range() {
        // Even for edge values, confidence should be in [0.0, 1.0]
        let c1 = compute_decay_confidence(1, 2, DEFAULT_HALF_LIFE_HOURS);
        assert!((0.0..=1.0).contains(&c1));

        let c2 = compute_decay_confidence(1, u64::MAX / 2, DEFAULT_HALF_LIFE_HOURS);
        assert!((0.0..=1.0).contains(&c2));
    }

    #[test]
    fn decay_handles_very_large_age_without_panic() {
        // ~5 billion milliseconds = ~58 days
        let confidence = compute_decay_confidence(0, 5_000_000_000, DEFAULT_HALF_LIFE_HOURS);
        assert_eq!(confidence, 0.0); // last_confirmed_at=0 → always 0.0

        // Large but valid timestamps
        let confidence = compute_decay_confidence(1, 5_000_000_000, DEFAULT_HALF_LIFE_HOURS);
        assert!((0.0..=1.0).contains(&confidence));
        // After many half-lives, confidence should be very close to 0
        assert!(confidence < 0.001);
    }
}
