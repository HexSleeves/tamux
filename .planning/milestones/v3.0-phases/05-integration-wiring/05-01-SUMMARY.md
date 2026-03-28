---
phase: 05-integration-wiring
plan: 01
subsystem: agent-runtime
tags: [calibration, confidence, embodied-dimensions, ipc, event-forwarding, uncertainty]

# Dependency graph
requires:
  - phase: 02-awareness-embodied-uncertainty
    provides: "CalibrationTracker, embodied dimensions, confidence pipeline, awareness monitor"
  - phase: 04-operator-control-transparency
    provides: "BudgetAlert, cost tracking, autonomy events, confidence warnings"
provides:
  - "Production callers for CalibrationTracker (record_observation on goal completion/failure)"
  - "Calibrated confidence bands via get_calibrated_band before label assignment"
  - "Embodied difficulty and weight signals in confidence scoring pipeline"
  - "IPC forwarding for 6 new AgentEvent variants to connected clients"
affects: [05-integration-wiring, tui, frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Calibration feedback loop: predict -> execute -> record -> adjust"
    - "Weight-blended blast radius: 0.6 domain + 0.4 embodied weight"
    - "Thread-scoped vs broadcast event routing in IPC"

key-files:
  created: []
  modified:
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/server.rs

key-decisions:
  - "Calibration observation extracted from step title prefix ([HIGH]/[MEDIUM]/[LOW]) since that is the predicted label"
  - "Weight-blended blast_radius_score (0.6 domain + 0.4 embodied weight) instead of replacing domain score entirely"
  - "compute_temperature not wired in planning phase -- it is a runtime operator urgency signal"
  - "Thread-scoped routing for ModeShift/ConfidenceWarning/CounterWhoAlert; broadcast for BudgetAlert/TrajectoryUpdate/EpisodeRecorded"

patterns-established:
  - "Calibration feedback pattern: extract predicted band from step title prefix, record against actual outcome"
  - "Embodied dimension blending: structural dimensions influence blast_radius_score via weighted combination"

requirements-completed: [UNCR-07, COST-03, EMBD-01, EMBD-02, EMBD-03, EMBD-04]

# Metrics
duration: 4min
completed: 2026-03-27
---

# Phase 05 Plan 01: Integration Wiring Summary

**Calibration feedback loop wired on goal completion/failure, embodied difficulty+weight in confidence pipeline, all 6 new AgentEvent variants forwarded over IPC**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-27T17:09:41Z
- **Completed:** 2026-03-27T17:14:38Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- CalibrationTracker.record_observation() called on every goal completion (success=true) and failure (success=false) with predicted confidence band extracted from step title prefix
- get_calibrated_band() adjusts confidence labels before plan step annotation, closing the calibration feedback loop (UNCR-07)
- compute_difficulty and compute_weight wired into confidence pipeline alongside existing compute_familiarity, with weight blending into blast_radius_score
- All 6 new AgentEvent variants (BudgetAlert, TrajectoryUpdate, ModeShift, ConfidenceWarning, EpisodeRecorded, CounterWhoAlert) forwarded to connected IPC clients with correct routing (thread-scoped vs broadcast)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire calibration feedback loop and embodied dimensions into confidence pipeline** - `e7340e5` (feat)
2. **Task 2: Forward 6 new AgentEvent variants over IPC to connected clients** - `79031b1` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/goal_planner.rs` - Added calibration recording on goal completion and failure
- `crates/amux-daemon/src/agent/goal_llm.rs` - Added calibrated band adjustment, embodied difficulty/weight computation, weight-blended blast radius
- `crates/amux-daemon/src/server.rs` - Added thread_id extraction for 3 thread-scoped events, broadcast routing for 3 non-thread-scoped events

## Decisions Made
- Calibration observation extracted from step title prefix ([HIGH]/[MEDIUM]/[LOW]) since that is the predicted label at planning time
- Weight-blended blast_radius_score (0.6 domain + 0.4 embodied weight) preserves domain classification primacy while incorporating embodied signal
- compute_temperature intentionally not wired in planning phase -- it is an operator urgency signal computed at runtime for action scoring
- Thread-scoped routing for ModeShift/ConfidenceWarning/CounterWhoAlert (have thread_id); broadcast for BudgetAlert/TrajectoryUpdate/EpisodeRecorded (have goal_run_id or episode_id, not thread_id)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Package name is `tamux-daemon` not `amux-daemon` (plan used crate directory name) -- trivially resolved by using correct package name for cargo commands.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Calibration feedback loop is live -- confidence labels will self-correct as goal runs accumulate
- All new events reach IPC clients -- frontend and TUI can render them when ready
- Ready for 05-02 plan execution (remaining integration wiring)

## Self-Check: PASSED

- All 3 modified files exist on disk
- Both task commits (e7340e5, 79031b1) found in git history
- cargo check passes, all existing tests pass (5 calibration, 17 embodied dimensions, 6 server)

---
*Phase: 05-integration-wiring*
*Completed: 2026-03-27*
