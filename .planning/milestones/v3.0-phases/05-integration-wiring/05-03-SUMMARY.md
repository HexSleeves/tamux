---
phase: 05-integration-wiring
plan: 03
subsystem: api
tags: [uncertainty, embodied, handoff, validation, sqlite]
requires:
  - phase: 05-01
    provides: calibration wiring and embodied-weight confidence path
  - phase: 05-02
    provides: specialist handoff validation entry points and escalation plumbing
provides:
  - temperature is computed from real thread user-message pacing and fed into active confidence scoring
  - specialist handoff tasks persist to_task_id linkage and resolve real handoff_log_id at completion
  - specialist validation now fail-closes and routes to failure/replan handling before acceptance mutation
affects: [phase-05-verification, uncertainty, handoff-broker]
tech-stack:
  added: []
  patterns:
    - "Pre-acceptance gate for specialist completion: validate first, mutate goal-run state second"
    - "SQLite linkage helpers for handoff_log id<->to_task_id resolution"
key-files:
  created: []
  modified:
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/handoff/audit.rs
    - crates/amux-daemon/src/agent/handoff/broker.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
key-decisions:
  - "Temperature signal uses last-5-minute user message count plus average gap from last up-to-5 user messages."
  - "Specialist completion is fail-closed: missing linkage, validation failure, or validation error routes through handle_goal_run_step_failure."
patterns-established:
  - "Route_handoff binds persisted handoff_log.to_task_id immediately after task enqueue."
  - "Goal planner resolves handoff_log_id by task_id instead of synthetic fallback IDs."
requirements-completed: [UNCR-07, EPIS-08, HAND-04, HAND-05, COST-03, EMBD-01, EMBD-02, EMBD-03, EMBD-04]
duration: 12m
completed: 2026-03-27
---

# Phase 05 Plan 03: Integration Wiring Gap-Closure Summary

**Confidence scoring now consumes real operator pacing temperature, and specialist handoff completion is enforced through real log linkage with fail-closed validation gating.**

## Performance

- **Duration:** 12m
- **Started:** 2026-03-27T22:58:00Z
- **Completed:** 2026-03-27T23:09:44Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Wired `compute_temperature(recent_message_count, avg_gap_secs)` into `annotate_plan_steps_with_confidence` and fed it into `blast_radius_score` before confidence computation.
- Passed thread context from `request_goal_plan` using `goal_run.thread_id.as_deref()` and derived temperature inputs from real `MessageRole::User` timestamps.
- Added handoff persistence/query helpers to bind `to_task_id` and resolve `handoff_log_id` by task.
- Updated specialist completion flow to validate before acceptance mutation and fail closed to `handle_goal_run_step_failure` on linkage or validation failure.

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire EMBD-02 temperature into active confidence scoring path** - `bdaec2a` (feat)
2. **Task 2: Enforce HAND-05 validation gate with real handoff_log linkage** - `cecfe7d` (fix)

## Files Created/Modified
- `crates/amux-daemon/src/agent/goal_llm.rs` - adds thread-aware operator pacing extraction and active temperature scoring blend.
- `crates/amux-daemon/src/agent/handoff/audit.rs` - adds `bind_handoff_task_id` and `resolve_handoff_log_id_by_task_id` SQLite helpers.
- `crates/amux-daemon/src/agent/handoff/broker.rs` - persists `to_task_id` linkage after specialist task enqueue.
- `crates/amux-daemon/src/agent/goal_planner.rs` - enforces pre-acceptance specialist validation with fail-closed replan/failure routing.

## Decisions Made
- Compute one per-plan temperature signal from live thread history (last 5 minutes + recent gap cadence) and blend it into blast-radius score with clamping.
- Execute specialist output validation as a hard gate before setting step completion state to prevent bypass on validation/linkage failures.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Corrected package/target test invocation during verification**
- **Found during:** Task 1 verification
- **Issue:** Plan command used `cargo test -p tamux-daemon --lib ...`, but package has no lib target.
- **Fix:** Ran equivalent package-scoped tests without `--lib`: `cargo test -p tamux-daemon embodied::dimensions -- --nocapture` and continued with `cargo check`.
- **Files modified:** None (verification command adjustment only)
- **Verification:** Command completed successfully.
- **Committed in:** N/A (no source changes)

**2. [Rule 3 - Blocking] Resolved cross-module visibility for handoff lookup helper**
- **Found during:** Task 2 implementation
- **Issue:** `resolve_handoff_log_id_by_task_id` was not visible from `goal_planner.rs`, causing compile failure.
- **Fix:** Promoted helper visibility from `pub(super)` to `pub(crate)` for both new handoff linkage helpers.
- **Files modified:** `crates/amux-daemon/src/agent/handoff/audit.rs`
- **Verification:** `cargo check -p tamux-daemon` passed.
- **Committed in:** `cecfe7d`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required to complete and verify planned functionality; no scope creep.

## Issues Encountered
- Repository had extensive pre-existing unrelated modifications; task commits were staged strictly by explicit file path to preserve atomicity.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 05 verification gaps EMBD-02 and HAND-05 are closed in production code paths.
- Re-verification can now assert no synthetic `hlid_` fallback and active temperature scoring behavior.

## Self-Check: PASSED
- Found summary file: `.planning/phases/05-integration-wiring/05-03-SUMMARY.md`
- Found task commit: `bdaec2a`
- Found task commit: `cecfe7d`
