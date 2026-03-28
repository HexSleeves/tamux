---
phase: 03-multi-agent-orchestration
plan: 04
subsystem: agent
tags: [divergent-subagents, multi-agent, parallel-framings, tension-detection, mediation, collaboration]

# Dependency graph
requires:
  - phase: 03-01
    provides: HandoffBroker, SpecialistProfile, ContextBundle, handoff schema, default profiles
provides:
  - DivergentSession with 2-3 framing validation and lifecycle state machine
  - Framing struct for per-perspective system prompt overrides
  - generate_framing_prompts for default analytical/pragmatic lens generation
  - format_tensions for disagreement-to-markdown conversion
  - format_mediator_prompt for tradeoff-acknowledging mediator synthesis
  - AgentEngine integration (start/contribute/complete divergent sessions)
affects: [goal-planner, agent-loop, operator-controls]

# Tech tracking
tech-stack:
  added: []
  patterns: [parallel-framing-pattern, tension-as-output, mediator-prompt-generation]

key-files:
  created:
    - crates/amux-daemon/src/agent/handoff/divergent.rs
  modified:
    - crates/amux-daemon/src/agent/handoff/mod.rs
    - crates/amux-daemon/src/agent/collaboration.rs
    - crates/amux-daemon/src/agent/engine.rs
    - crates/amux-daemon/src/agent/heartbeat_checks.rs

key-decisions:
  - "Divergent sessions use inline now_millis() to avoid cross-module visibility issues with task_prompt::now_millis"
  - "Collaboration types widened from pub(super) to pub(in crate::agent) for handoff submodule access"
  - "Mediator prompt returned as String for caller to decide LLM call vs direct operator presentation"
  - "Virtual parent_task_id created per divergent session to anchor CollaborationSession"

patterns-established:
  - "Parallel framing pattern: spawn 2-3 perspectives, detect disagreements, surface tensions"
  - "Tension-as-output: disagreements are the valuable product, not forced consensus"
  - "Mediator synthesis: tradeoff acknowledgment over winner-picking"

requirements-completed: [DIVR-01, DIVR-02, DIVR-03]

# Metrics
duration: 9min
completed: 2026-03-27
---

# Phase 03 Plan 04: Divergent Subagent Mode Summary

**Divergent parallel framing with 2-3 perspectives, automatic tension detection, and mediator prompt generation that surfaces tradeoffs without forcing consensus**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-27T10:34:41Z
- **Completed:** 2026-03-27T10:44:38Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- DivergentSession with validated 2-3 framing constraint and Spawning->Running->Mediating->Complete state machine
- Default framing generation (analytical-lens + pragmatic-lens) with problem-specific system prompts
- Tension formatting that maps disagreements to per-framing markdown with evidence sections
- Mediator prompt that explicitly instructs "do NOT force consensus" and "acknowledge tradeoffs"
- Full AgentEngine integration: start_divergent_session creates CollaborationSession + enqueues per-framing tasks
- 20 unit tests covering all pure functions and state machine transitions

## Task Commits

Each task was committed atomically:

1. **Task 1: DivergentSession types, framing generation, tension formatting, mediator prompts** - `3a5e9ac` (feat)
2. **Task 2: Wire divergent mode into AgentEngine via CollaborationSession** - `ec2e168` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/handoff/divergent.rs` - DivergentSession, Framing, DivergentStatus, format_tensions, format_mediator_prompt, AgentEngine impl block
- `crates/amux-daemon/src/agent/handoff/mod.rs` - Added `pub mod divergent;` declaration
- `crates/amux-daemon/src/agent/collaboration.rs` - Widened visibility from pub(super) to pub(in crate::agent) for types and detect_disagreements
- `crates/amux-daemon/src/agent/engine.rs` - Added divergent_sessions field to AgentEngine
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Added divergent_sessions to test constructor

## Decisions Made
- Used inline `now_millis()` in divergent.rs instead of importing from task_prompt to avoid pub(super) visibility barrier across nested modules
- Widened collaboration.rs struct visibility from `pub(super)` to `pub(in crate::agent)` -- necessary for handoff submodule to use CollaborationSession, Disagreement, Contribution types
- Mediator prompt is returned as a String rather than triggering an LLM call -- the caller (goal runner or agent loop) decides whether to invoke the LLM or surface tensions directly to operator
- Created virtual parent_task_id for each divergent session to anchor the CollaborationSession in the existing collaboration HashMap

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Inline now_millis() to avoid module visibility barrier**
- **Found during:** Task 1
- **Issue:** task_prompt::now_millis is pub(super) and inaccessible from agent::handoff::divergent (grandchild module)
- **Fix:** Added a local now_millis() function in divergent.rs (identical 4-line implementation)
- **Files modified:** crates/amux-daemon/src/agent/handoff/divergent.rs
- **Verification:** cargo check clean, all tests pass
- **Committed in:** 3a5e9ac (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed lifetime error in format_tensions**
- **Found during:** Task 1
- **Issue:** format! macro created a temporary String returned by reference, causing E0515
- **Fix:** Simplified position-to-framing mapping to use index-based lookup avoiding temporary references
- **Files modified:** crates/amux-daemon/src/agent/handoff/divergent.rs
- **Verification:** cargo check clean, all tests pass
- **Committed in:** 3a5e9ac (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed blocking issues above.

## Known Stubs
None -- all functions are fully implemented with real logic.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Divergent mode infrastructure is complete and ready for goal planner / agent loop integration
- Mediator prompt can be fed to any LLM for synthesis or presented directly to operator
- Future work: tool integration to trigger divergent sessions from chat/goal runs, and mediator LLM call orchestration

## Self-Check: PASSED

All 6 files verified present. Both task commits (3a5e9ac, ec2e168) verified in git log.

---
*Phase: 03-multi-agent-orchestration*
*Completed: 2026-03-27*
