---
phase: 05-integration-wiring
plan: 02
subsystem: agent
tags: [handoff, escalation, episodic, specialist, goal-planner, session-lifecycle]

# Dependency graph
requires:
  - phase: 03-specialist-handoffs
    provides: "HandoffBroker, escalation triggers, acceptance validation, specialist profiles"
  - phase: 01-episodic-memory
    provides: "Episodic store with record_session_end_episode"
  - phase: 05-integration-wiring plan 01
    provides: "Initial wiring of embodied metadata, calibration, and IPC routing"
provides:
  - "Production callers for evaluate_escalation_triggers in goal step failure path"
  - "Production callers for validate_specialist_output in goal step completion path"
  - "Production caller for record_session_end_episode in KillSession IPC handler"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Escalation trigger evaluation pattern: read broker profiles, match role, check escalation_chain"
    - "Specialist output validation pattern: check task.source == handoff, validate on completion"
    - "Non-blocking episode recording in IPC handlers: fire-and-forget with warn on failure"

key-files:
  created: []
  modified:
    - "crates/amux-daemon/src/agent/goal_planner.rs"
    - "crates/amux-daemon/src/server.rs"

key-decisions:
  - "SessionId is Uuid not u32 -- used Display formatting for tracing and episode recording"

patterns-established:
  - "Escalation triggers evaluated between snapshot load and replan logic in handle_goal_run_step_failure"
  - "Specialist output validation runs between provenance recording and auto-checkpoint in handle_goal_run_step_completion"
  - "Session-end episodes recorded before SessionKilled response in KillSession handler"

requirements-completed: [HAND-04, HAND-05, EPIS-08]

# Metrics
duration: 5min
completed: 2026-03-27
---

# Phase 05 Plan 02: Escalation/Validation/Session-Episode Wiring Summary

**Production callers wired for escalation triggers (HAND-04), specialist output validation (HAND-05), and session-end episode recording (EPIS-08) closing remaining handoff and episodic integration gaps**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-27T17:17:43Z
- **Completed:** 2026-03-27T17:22:46Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- evaluate_escalation_triggers called during specialist step failure handling with consecutive_failures, elapsed_secs, and confidence_band from profile escalation chains
- validate_specialist_output called on handoff-sourced task completion for specialist steps with structural acceptance checks
- record_session_end_episode called from KillSession IPC handler with session summary and entity tags

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire escalation triggers and specialist output validation into goal step lifecycle** - `f7a51ef` (feat)
2. **Task 2: Wire session-end episode recording into KillSession lifecycle** - `8362003` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/goal_planner.rs` - Added escalation trigger evaluation in handle_goal_run_step_failure and specialist output validation in handle_goal_run_step_completion
- `crates/amux-daemon/src/server.rs` - Added session-end episode recording in KillSession handler

## Decisions Made
- SessionId is Uuid (not u32 as plan interface section suggested) -- used `id.to_string()` and `%` Display formatting for tracing compatibility

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed SessionId type mismatch in tracing macro**
- **Found during:** Task 2 (KillSession wiring)
- **Issue:** Plan assumed SessionId is u32 but it is actually `Uuid`. Using `session_id = id` in tracing::warn! failed because Uuid does not implement tracing::Value
- **Fix:** Converted id to string first (`let session_id_str = id.to_string()`) and used `%session_id_str` Display formatting in tracing macro
- **Files modified:** crates/amux-daemon/src/server.rs
- **Verification:** cargo check -p tamux-daemon succeeds
- **Committed in:** 8362003 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Minor type correction needed for Uuid tracing compatibility. No scope creep.

## Issues Encountered
- Package name is `tamux-daemon` not `amux-daemon` (plan verification commands used wrong name) -- corrected during execution

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 05 integration wiring plans complete
- All orphaned functions now have production callers: escalation triggers, specialist output validation, session-end episodes
- Phase 05 closes the final integration gaps for the v3.0 intelligence layer

## Self-Check: PASSED

- FOUND: crates/amux-daemon/src/agent/goal_planner.rs
- FOUND: crates/amux-daemon/src/server.rs
- FOUND: .planning/phases/05-integration-wiring/05-02-SUMMARY.md
- FOUND: f7a51ef (Task 1 commit)
- FOUND: 8362003 (Task 2 commit)

---
*Phase: 05-integration-wiring*
*Completed: 2026-03-27*
