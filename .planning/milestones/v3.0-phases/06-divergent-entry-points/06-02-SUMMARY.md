---
phase: 06-divergent-entry-points
plan: 02
subsystem: agent
tags: [goal-planner, divergent-sessions, step-routing, llm-prompt, domain-classification]

# Dependency graph
requires:
  - phase: 03-structured-handoffs
    provides: "Divergent subagent infrastructure (start_divergent_session, DivergentSession, Framing)"
  - phase: 06-divergent-entry-points
    plan: 01
    provides: "Tool and IPC entry points for divergent sessions (DIVR-01, DIVR-02)"
provides:
  - "GoalRunStepKind::Divergent enum variant for autonomous goal decomposition"
  - "Goal planner routing of Divergent steps through start_divergent_session"
  - "LLM planning prompt offering 'divergent' as a valid step kind"
  - "Domain classification mapping Divergent to Research"
  - "SQLite round-trip serialization for divergent step kind"
affects: [goal-runner, llm-planning, uncertainty-scoring]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Step-kind routing pattern: if-let chain for Specialist, equality check for Divergent, else for default"
    - "Graceful degradation: divergent session failure falls back to normal task enqueue"

key-files:
  created: []
  modified:
    - crates/amux-daemon/src/agent/types.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/goal_parsing.rs
    - crates/amux-daemon/src/agent/uncertainty/domains.rs
    - crates/amux-daemon/src/history.rs

key-decisions:
  - "Divergent is a unit variant (no payload) -- problem statement comes from step.instructions"
  - "Domain classification maps Divergent to Research (non-blocking on LOW confidence)"
  - "Weight classification uses read_file weight (lightweight, like Reason)"
  - "Placeholder task created with 'divergent' source for goal runner step tracking"

patterns-established:
  - "Step-kind routing chain: Specialist (if-let) -> Divergent (equality) -> default (else)"

requirements-completed: [DIVR-03]

# Metrics
duration: 4min
completed: 2026-03-27
---

# Phase 06 Plan 02: Goal Planner Divergent Step Kind Summary

**GoalRunStepKind::Divergent variant enabling autonomous goal decomposition to spawn divergent framings when a step benefits from multiple perspectives**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-27T17:36:02Z
- **Completed:** 2026-03-27T17:40:53Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added Divergent unit variant to GoalRunStepKind enum with serde "divergent" serialization
- Updated LLM planning prompt to offer "divergent" as a valid step kind with usage guidance
- Routed Divergent steps through start_divergent_session in goal_planner.rs with graceful fallback
- Updated all match arms: weight classification, domain classification, validation message, SQLite serialization

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Divergent variant to GoalRunStepKind and update all match arms** - `d348efb` (feat)
2. **Task 2: Route Divergent steps through start_divergent_session in goal planner** - `0dfe4b9` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/types.rs` - Added Divergent variant to GoalRunStepKind enum
- `crates/amux-daemon/src/agent/goal_llm.rs` - Updated LLM prompt kind list and weight classification match
- `crates/amux-daemon/src/agent/goal_parsing.rs` - Updated validation error message to include "divergent"
- `crates/amux-daemon/src/agent/uncertainty/domains.rs` - Added Divergent -> Research domain classification
- `crates/amux-daemon/src/agent/goal_planner.rs` - Added Divergent step routing through start_divergent_session
- `crates/amux-daemon/src/history.rs` - Added SQLite serialization/deserialization for "divergent" kind

## Decisions Made
- Divergent is a unit variant with no payload (unlike Specialist(String)) -- the problem statement comes from step.instructions, keeping it simple and serde-compatible
- Domain classification maps Divergent to Research (exploratory, non-blocking on LOW confidence) rather than Business
- Weight classification uses "read_file" weight (lightweight like Reason) since divergent spawns analysis, not heavy operations
- Placeholder task with "divergent" source created after session start so goal runner can track step completion

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing Divergent arm in history.rs SQLite serialization**
- **Found during:** Task 1 (cargo check revealed exhaustiveness error)
- **Issue:** history.rs goal_run_step_kind_to_str and parse_goal_run_step_kind did not handle the new Divergent variant
- **Fix:** Added "divergent" serialization in goal_run_step_kind_to_str and parsing in parse_goal_run_step_kind
- **Files modified:** crates/amux-daemon/src/history.rs
- **Verification:** cargo check -p tamux-daemon compiles cleanly
- **Committed in:** d348efb (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Essential for compilation. The plan's interface excerpt did not list history.rs, but the compiler caught the missing match arm. No scope creep.

## Issues Encountered
None - both tasks executed cleanly after the history.rs fix.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three divergent entry points now complete (DIVR-01 tool, DIVR-02 IPC, DIVR-03 goal planner)
- The goal runner can autonomously spawn divergent framings when the LLM outputs a step with kind "divergent"
- All 20 existing divergent tests continue to pass, plus 2 goal_parsing tests

## Self-Check: PASSED

All 6 modified files verified present. Both commit hashes (d348efb, 0dfe4b9) verified in git log.

---
*Phase: 06-divergent-entry-points*
*Completed: 2026-03-27*
