---
phase: 04-operator-control-transparency
plan: 02
subsystem: agent-engine
tags: [autonomy-dial, authorship, event-filtering, supervised-mode, goal-runs]

# Dependency graph
requires:
  - phase: 04-01
    provides: "GoalRun cost fields, AgentEvent BudgetAlert, cost tracker lifecycle"
provides:
  - "AutonomyLevel enum (Autonomous/Aware/Supervised) with event filtering"
  - "AuthorshipTag enum (Operator/Agent/Joint) with classification logic"
  - "Per-goal autonomy dial wired through IPC to event emission"
  - "Supervised mode acknowledgment gate at step boundaries"
  - "Authorship metadata set on goal completion"
affects: [04-03, 04-04, frontend-goal-display, tui-goal-display]

# Tech tracking
tech-stack:
  added: []
  patterns: ["autonomy event filtering gate in emit_goal_run_update", "GoalRunStatus-to-event_kind mapping for autonomy filtering", "supervised acknowledgment gate reusing AwaitingApproval status"]

key-files:
  created:
    - "crates/amux-daemon/src/agent/autonomy.rs"
    - "crates/amux-daemon/src/agent/authorship.rs"
  modified:
    - "crates/amux-daemon/src/agent/types.rs"
    - "crates/amux-daemon/src/agent/mod.rs"
    - "crates/amux-daemon/src/agent/work_context.rs"
    - "crates/amux-daemon/src/agent/goal_planner.rs"
    - "crates/amux-daemon/src/agent/task_crud.rs"
    - "crates/amux-protocol/src/messages.rs"
    - "crates/amux-daemon/src/server.rs"
    - "crates/amux-cli/src/client.rs"
    - "crates/amux-mcp/src/main.rs"
    - "crates/amux-tui/src/client.rs"
    - "crates/amux-daemon/src/history.rs"
    - "crates/amux-daemon/src/agent/anticipatory.rs"
    - "crates/amux-daemon/src/agent/heartbeat_checks.rs"
    - "crates/amux-daemon/src/agent/liveness/state_layers.rs"
    - "crates/amux-daemon/src/agent/liveness/checkpoint.rs"

key-decisions:
  - "Default autonomy level is Aware (matches current behavior, no behavioral change for existing users)"
  - "Event filtering gate in emit_goal_run_update maps GoalRunStatus to event_kind string for should_emit_event check"
  - "Supervised mode reuses existing AwaitingApproval GoalRunStatus with autonomy_acknowledgment event phase"
  - "Authorship defaults to Joint for all completed goal runs (operator provides goal text, agent executes)"
  - "autonomy_level is a string on the wire (protocol crate) and parsed to enum in daemon — keeps protocol dependency-free"

patterns-established:
  - "Autonomy event filtering: check should_emit_event before sending GoalRunUpdate events"
  - "GoalRunStatus-to-event_kind mapping: completed/failed/planning/step_started/step_detail"
  - "Supervised acknowledgment gate: set AwaitingApproval after enqueue, operator resumes via existing approval flow"

requirements-completed: [AUTO-01, AUTO-02, AUTO-03, AUTO-04, AUTH-01, AUTH-02]

# Metrics
duration: 14min
completed: 2026-03-27
---

# Phase 04 Plan 02: Per-Goal Autonomy Dial and Shared Authorship Summary

**AutonomyLevel enum with event filtering gate, supervised-mode acknowledgment at step boundaries, and AuthorshipTag on goal completion**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-27T12:25:46Z
- **Completed:** 2026-03-27T12:40:24Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments
- AutonomyLevel (Autonomous/Aware/Supervised) with should_emit_event filtering and requires_acknowledgment gate
- AuthorshipTag (Operator/Agent/Joint) with classify_authorship logic based on participation signals
- Full IPC wiring: autonomy_level field flows from protocol message through server to GoalRun creation
- Event filtering gate in emit_goal_run_update suppresses intermediate events in Autonomous mode
- Supervised mode pauses goal runs at step boundaries via existing AwaitingApproval mechanism
- Authorship tag set on goal completion as metadata (not inline commentary)
- 21 unit tests covering all autonomy and authorship behavior
- Backward compatible: all new fields use serde(default) for existing persisted data

## Task Commits

Each task was committed atomically:

1. **Task 1: Create autonomy and authorship modules with types and logic** - `d5b235c` (feat)
2. **Task 2: Wire autonomy into IPC, event emission, and goal completion with authorship** - `64ba112` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/autonomy.rs` - AutonomyLevel enum, should_emit_event, requires_acknowledgment, from_str_or_default, 15 unit tests
- `crates/amux-daemon/src/agent/authorship.rs` - AuthorshipTag enum, classify_authorship, 6 unit tests
- `crates/amux-daemon/src/agent/types.rs` - GoalRun extended with autonomy_level and authorship_tag fields
- `crates/amux-daemon/src/agent/mod.rs` - Module declarations for autonomy and authorship
- `crates/amux-daemon/src/agent/work_context.rs` - Event filtering gate + GoalRunStatus-to-event_kind mapper
- `crates/amux-daemon/src/agent/goal_planner.rs` - Supervised acknowledgment gate + authorship tag on completion
- `crates/amux-daemon/src/agent/task_crud.rs` - start_goal_run accepts and parses autonomy_level parameter
- `crates/amux-protocol/src/messages.rs` - autonomy_level field on AgentStartGoalRun
- `crates/amux-daemon/src/server.rs` - Pass-through of autonomy_level to start_goal_run
- `crates/amux-cli/src/client.rs` - AgentBridgeCommand updated for autonomy_level
- `crates/amux-mcp/src/main.rs` - MCP tool_start_goal_run parses autonomy_level from args
- `crates/amux-tui/src/client.rs` - TUI client updated for new protocol field
- `crates/amux-daemon/src/history.rs` - GoalRun SQL deserializer updated for new fields
- `crates/amux-daemon/src/agent/anticipatory.rs` - Test GoalRun updated for new fields
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Test GoalRun updated for new fields
- `crates/amux-daemon/src/agent/liveness/state_layers.rs` - Test GoalRun updated for new fields
- `crates/amux-daemon/src/agent/liveness/checkpoint.rs` - Test GoalRun updated for new fields

## Decisions Made
- Default autonomy level is Aware per locked decision -- no behavioral change for existing users
- Event filtering maps GoalRunStatus to event_kind strings (completed/failed/planning/step_started/step_detail)
- Supervised mode reuses existing AwaitingApproval status with "autonomy_acknowledgment" event phase to distinguish from policy approvals
- All completed goal runs get Joint authorship (operator provides goal text + agent executes plan)
- autonomy_level is a string on the protocol wire to keep the protocol crate dependency-free

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated GoalRun constructors across all test files**
- **Found during:** Task 1 (cargo check)
- **Issue:** 7 GoalRun struct literals across test and production files missing new autonomy_level and authorship_tag fields
- **Fix:** Added `autonomy_level: Default::default(), authorship_tag: None` to all struct literals
- **Files modified:** history.rs, anticipatory.rs, heartbeat_checks.rs, liveness/state_layers.rs, liveness/checkpoint.rs, mod.rs
- **Verification:** cargo check compiles without errors
- **Committed in:** d5b235c (Task 1 commit)

**2. [Rule 3 - Blocking] Updated CLI bridge, MCP, and TUI for new protocol field**
- **Found during:** Task 2 (cargo check on all crates)
- **Issue:** AgentStartGoalRun struct literal in CLI, MCP, and TUI crates missing new autonomy_level field
- **Fix:** Added autonomy_level field to all three call sites (None for TUI/CLI default, parsed from args for MCP)
- **Files modified:** amux-cli/src/client.rs, amux-mcp/src/main.rs, amux-tui/src/client.rs
- **Verification:** cargo check on all affected crates compiles
- **Committed in:** 64ba112 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary for compilation across all crates. No scope creep.

## Issues Encountered
- 2 pre-existing test failures in plugin::loader::tests (calendar_manifest and gmail_manifest) unrelated to autonomy/authorship -- same as documented in 04-01 SUMMARY
- Pre-existing compilation errors in tamux-cli for OperatorProfile protocol variants (from parallel task) -- unrelated to this plan

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Autonomy dial is fully wired end-to-end (IPC -> GoalRun -> event filtering -> acknowledgment gates)
- Authorship tag is set on goal completion and persisted with GoalRun
- Ready for frontend/TUI to surface autonomy level selector when starting goal runs
- Ready for frontend/TUI to display authorship tag on completed goal runs
- Supervised mode uses existing approval flow -- no new UI needed for acknowledgment

## Self-Check: PASSED
- All 2 created files verified present
- All 7 key modified files verified present
- Commit d5b235c (Task 1) verified in git log
- Commit 64ba112 (Task 2) verified in git log
- 21 autonomy + authorship tests pass
- 1117 daemon tests pass (2 pre-existing failures unrelated to autonomy/authorship)

---
*Phase: 04-operator-control-transparency*
*Completed: 2026-03-27*
