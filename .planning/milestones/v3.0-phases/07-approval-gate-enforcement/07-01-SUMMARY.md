---
phase: 07-approval-gate-enforcement
plan: 01
subsystem: infra
tags: [rust, dispatcher, scheduler, approval-gate, autonomy]

# Dependency graph
requires:
  - phase: 06-divergent-subagents
    provides: goal-run step/task lifecycle and runtime dispatch paths
provides:
  - Runtime fail-closed gate at dispatcher + scheduler boundaries for AwaitingApproval goal runs
  - Supervised-mode step gate that blocks both goal and child task until explicit acknowledgment
  - Explicit `acknowledge`/`ack` goal control transition that clears supervised gate without using `resume`
affects: [goal-run-lifecycle, operator-approval-flow, task-dispatch]

# Tech tracking
tech-stack:
  added: []
  patterns: [fail-closed runtime gating, explicit supervised acknowledgment transitions]

key-files:
  created:
    - .planning/phases/07-approval-gate-enforcement/07-01-SUMMARY.md
  modified:
    - crates/amux-daemon/src/agent/dispatcher.rs
    - crates/amux-daemon/src/agent/task_scheduler.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/task_crud.rs

key-decisions:
  - "Scheduler now fails closed for goal-linked queued tasks when goal metadata is missing."
  - "Supervised step acknowledgment uses a stable autonomy-ack ID propagated to both goal and child task."

patterns-established:
  - "Dispatch boundary checks include both outer-loop filters and inner early returns to prevent bypasses."
  - "Goal control uses explicit action (`acknowledge`/`ack`) to clear supervised gates; `resume` remains pause-only."

requirements-completed: [UNCR-08, AUTO-04]

# Metrics
duration: 6m 32s
completed: 2026-03-27
---

# Phase 07 Plan 01: approval-gate-enforcement Summary

**Unified approval-gate enforcement now fail-closes runtime dispatch and requires explicit supervised acknowledgment before queued goal-step tasks can execute.**

## Performance

- **Duration:** 6m 32s
- **Started:** 2026-03-27T22:13:12Z
- **Completed:** 2026-03-27T22:19:44Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added runtime gate checks so dispatcher no longer advances goal runs in `AwaitingApproval` and scheduler excludes blocked goal-linked queued tasks.
- Added fail-closed scheduler behavior for goal-linked tasks when goal metadata is unavailable, preventing default-allow bypasses.
- Implemented supervised acknowledgment gate semantics where enqueue sets both goal/task to `AwaitingApproval` and explicit `acknowledge`/`ack` unblocks execution.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add fail-closed runtime gate checks to dispatcher and scheduler paths (UNCR-08)** - `8eeb41b` (test), `b2d78ee` (feat)
2. **Task 2: Make supervised acknowledgment an explicit unblock transition across goal + task state (AUTO-04)** - `07a4de1` (test), `6864c87` (feat)

_Note: TDD tasks produced separate RED and GREEN commits._

## Files Created/Modified
- `crates/amux-daemon/src/agent/dispatcher.rs` - Added approval-aware goal-run filtering/early-return and passed goal-run status snapshot into scheduler selection.
- `crates/amux-daemon/src/agent/task_scheduler.rs` - Extended `select_ready_task_indices` to accept goal status context and fail closed for blocked/missing goal metadata; added scheduler tests.
- `crates/amux-daemon/src/agent/goal_planner.rs` - Moved supervised gating into the enqueue transaction and propagated stable `autonomy-ack` IDs to goal + child task.
- `crates/amux-daemon/src/agent/task_crud.rs` - Added explicit `acknowledge`/`ack` control action and preserved `resume` as pause-only unblock.

## Decisions Made
- Used a goal-run status snapshot map (`HashMap<goal_run_id, GoalRunStatus>`) for scheduler gating so selection remains deterministic within a dispatch tick.
- Gate entry for supervised steps now happens in the same state update as step task attachment to eliminate the queued dispatch window.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Prevented planner-status runs from being advanced by dispatcher**
- **Found during:** Task 1
- **Issue:** Runtime dispatcher filter still allowed `GoalRunStatus::Planning`, enabling unnecessary/unsafe advancement attempts during planning.
- **Fix:** Added `GoalRunStatus::Planning` exclusion in `dispatch_goal_runs` and an early return guard in `advance_goal_run`.
- **Files modified:** `crates/amux-daemon/src/agent/dispatcher.rs`
- **Verification:** `cargo test -p tamux-daemon dispatcher:: -- --nocapture`
- **Committed in:** `b2d78ee`

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Deviation tightened fail-closed runtime behavior and did not expand architectural scope.

## Issues Encountered
- The plan’s combined `cargo test` invocation with multiple test-name filters is not accepted by Cargo; tests were executed in equivalent separate commands per module.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Approval-gated runtime paths now consistently deny execution without acknowledgment.
- Goal control and scheduler semantics are aligned for supervised + low-confidence gate handling.

## Known Stubs
- `crates/amux-daemon/src/agent/goal_planner.rs:277` uses “placeholder task” wording for divergent session tracking; this is pre-existing and outside this plan’s enforcement scope.

## Self-Check: PASSED

- FOUND: `.planning/phases/07-approval-gate-enforcement/07-01-SUMMARY.md`
- FOUND commit: `8eeb41b`
- FOUND commit: `b2d78ee`
- FOUND commit: `07a4de1`
- FOUND commit: `6864c87`
