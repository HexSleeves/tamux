---
phase: 03-multi-agent-orchestration
plan: 03
subsystem: agent
tags: [handoff, broker, route-to-specialist, goal-planner, orchestration, multi-agent]

requires:
  - phase: 03-multi-agent-orchestration
    plan: 01
    provides: "HandoffBroker types, SpecialistProfile, match_specialist(), handoff_log schema"
  - phase: 03-multi-agent-orchestration
    plan: 02
    provides: "ContextBundle assembly, escalation evaluation, acceptance validation, WORM audit trail"
  - phase: 01-memory-foundation
    provides: "Episodic memory retrieval, negative knowledge constraints"
provides:
  - "route_handoff() full orchestration: match specialist -> assemble bundle -> audit -> enqueue task"
  - "assemble_context_bundle() pulling episodic refs and negative constraints from Phase 1"
  - "validate_specialist_output() with structural checks and WORM audit"
  - "route_to_specialist tool registered in tool dispatch and tool definitions"
  - "GoalRunStepKind::Specialist(String) variant for planned specialist routing"
  - "Goal planner Specialist step dispatch through handoff broker with fallback"
affects: [03-04 divergent-subagents]

tech-stack:
  added: []
  patterns: ["Specialist routing via handoff broker from both tool calls and goal planner", "GoalRunStepKind::Specialist(role) for goal-originated specialist handoffs"]

key-files:
  created:
    - crates/amux-daemon/src/agent/handoff/broker.rs
  modified:
    - crates/amux-daemon/src/agent/handoff/mod.rs
    - crates/amux-daemon/src/agent/tool_executor.rs
    - crates/amux-daemon/src/agent/types.rs
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/goal_parsing.rs
    - crates/amux-daemon/src/agent/system_prompt.rs
    - crates/amux-daemon/src/agent/uncertainty/domains.rs
    - crates/amux-daemon/src/history.rs

key-decisions:
  - "GoalRunStepKind::Specialist(String) requires removing Copy derive -- all step.kind call sites updated to .clone()"
  - "classify_step_kind changed to take &GoalRunStepKind (reference) since Copy removed"
  - "Specialist steps serialized as 'specialist:ROLE' in SQLite for backward-compatible round-tripping"
  - "Goal planner falls back to normal enqueue if specialist handoff fails (graceful degradation)"
  - "Generalist profile (last in list) used as fallback when no specialist matches capability tags"
  - "route_to_specialist tool returns structured JSON response on success, plain error string on failure"

patterns-established:
  - "Broker orchestration pattern: match -> bundle -> audit -> enqueue -> validate"
  - "Specialist routing from two entry points: tool call (mid-task) and goal planner (planned steps)"
  - "Graceful degradation: specialist handoff failure falls back to normal task enqueue"

requirements-completed: [HAND-05, HAND-09]

duration: 12min
completed: 2026-03-27
---

# Phase 03 Plan 03: Broker Orchestration and Agent Pipeline Wiring Summary

**HandoffBroker route_handoff() orchestrating full match-bundle-audit-enqueue flow, route_to_specialist tool for mid-task handoffs, and GoalRunStepKind::Specialist for planned specialist routing**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-27T10:50:24Z
- **Completed:** 2026-03-27T11:02:29Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Implemented broker.rs with three core orchestration methods: route_handoff(), assemble_context_bundle(), validate_specialist_output()
- route_handoff integrates all Plan 01/02 subsystems: match_specialist, ContextBundle assembly with episodic refs and negative constraints, WORM audit recording, and task enqueue
- Context bundles respect 2000-token ceiling and 3-hop depth limit
- route_to_specialist tool registered in tool dispatch (tool_executor.rs) and tool definitions for LLM access
- GoalRunStepKind::Specialist(String) variant enables goal planner to specify specialist routing on plan steps
- Goal planner dispatches Specialist steps through handoff broker with graceful fallback to normal enqueue
- System prompt updated with specialist routing guidance

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement broker orchestration** - `46279bb` (feat)
2. **Task 2: Wire route_to_specialist tool and GoalRunStepKind::Specialist** - `4a2ad61` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/handoff/broker.rs` - route_handoff(), assemble_context_bundle(), validate_specialist_output() orchestration
- `crates/amux-daemon/src/agent/handoff/mod.rs` - Added `pub mod broker;` declaration
- `crates/amux-daemon/src/agent/tool_executor.rs` - route_to_specialist tool definition and dispatch, execute_route_to_specialist handler
- `crates/amux-daemon/src/agent/types.rs` - GoalRunStepKind::Specialist(String) variant, removed Copy derive
- `crates/amux-daemon/src/agent/goal_llm.rs` - Updated classify_step_kind call to pass reference
- `crates/amux-daemon/src/agent/goal_planner.rs` - Specialist step dispatch through handoff broker with fallback
- `crates/amux-daemon/src/agent/goal_parsing.rs` - Updated step.kind usages to .clone(), validation message includes specialist
- `crates/amux-daemon/src/agent/system_prompt.rs` - Specialist routing guidance in subagent supervision section
- `crates/amux-daemon/src/agent/uncertainty/domains.rs` - classify_step_kind takes &GoalRunStepKind, handles Specialist variant
- `crates/amux-daemon/src/history.rs` - goal_run_step_kind_to_str returns String (specialist:ROLE format), takes reference

## Decisions Made
- **GoalRunStepKind Copy removal**: Adding Specialist(String) requires removing Copy derive. All implicit copy sites updated to explicit .clone(). This is a minor API change but necessary for the string payload.
- **classify_step_kind signature change**: Changed from taking by value to taking by reference since Copy was removed. Specialist maps to Business domain classification.
- **SQLite serialization format**: Specialist steps stored as "specialist:ROLE" string, parsed with strip_prefix on load. Backward-compatible: unknown kinds still fall through to Research.
- **Graceful degradation in goal planner**: If specialist handoff fails (e.g., depth limit reached), the step falls back to normal task enqueue with a warning log.
- **Tool response format**: route_to_specialist returns pretty-printed JSON with task_id, specialist_name, profile_id, handoff_log_id, and context_bundle_tokens on success.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed Copy derive from GoalRunStepKind**
- **Found during:** Task 2 (adding Specialist(String) variant)
- **Issue:** GoalRunStepKind had Copy derive, but Specialist(String) contains a heap-allocated String which cannot be Copy
- **Fix:** Removed Copy from derive, updated all call sites using step.kind to use .clone(), changed classify_step_kind to take &GoalRunStepKind
- **Files modified:** types.rs, goal_parsing.rs, goal_planner.rs, goal_llm.rs, uncertainty/domains.rs, history.rs
- **Verification:** cargo check clean, all 1073 tests pass (2 pre-existing plugin failures)
- **Committed in:** 4a2ad61 (Task 2 commit)

**2. [Rule 3 - Blocking] Updated history.rs step kind serialization**
- **Found during:** Task 2 (GoalRunStepKind change)
- **Issue:** goal_run_step_kind_to_str took GoalRunStepKind by value (required Copy) and returned &'static str (cannot include dynamic role string)
- **Fix:** Changed to take &GoalRunStepKind and return String, added specialist:ROLE format. Updated parse_goal_run_step_kind to handle specialist: prefix.
- **Files modified:** crates/amux-daemon/src/history.rs
- **Verification:** cargo check clean, all tests pass
- **Committed in:** 4a2ad61 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes necessary to support String payload in enum variant. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Full handoff pipeline operational: agent calls route_to_specialist -> broker matches specialist -> assembles bundle -> records WORM audit -> enqueues task -> validates output
- Goal planner supports Specialist step kind for planned specialist routing
- Ready for Plan 04 (divergent subagents) which builds parallel interpretation mode on top of this handoff infrastructure
- All handoff subsystems (types, profiles, matching, context bundles, escalation, acceptance, audit, broker) are complete and wired

---
*Phase: 03-multi-agent-orchestration*
*Completed: 2026-03-27*

## Self-Check: PASSED

- broker.rs: FOUND (395 lines, exceeds 100 minimum)
- 03-03-SUMMARY.md: FOUND
- Commit 46279bb (Task 1): FOUND
- Commit 4a2ad61 (Task 2): FOUND
- cargo check: clean (103 pre-existing warnings)
- cargo test handoff: 75 passed, 0 failed
- cargo test all: 1073 passed, 2 failed (pre-existing plugin::loader failures)
