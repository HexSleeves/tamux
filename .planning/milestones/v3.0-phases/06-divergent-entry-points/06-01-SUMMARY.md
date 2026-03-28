---
phase: 06-divergent-entry-points
plan: 01
subsystem: agent
tags: [divergent-sessions, tool-executor, ipc, handoff, parallel-framings]

# Dependency graph
requires:
  - phase: 03-structured-handoffs
    provides: "DivergentSession infrastructure (divergent.rs), start_divergent_session on AgentEngine"
provides:
  - "run_divergent tool callable from agent tool loop"
  - "AgentStartDivergentSession IPC message for external clients"
  - "AgentDivergentSessionStarted IPC response"
affects: [06-divergent-entry-points]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tool definition + dispatch arm + execute function pattern for run_divergent"
    - "IPC ClientMessage/DaemonMessage variant pair with JSON framing passthrough"

key-files:
  created: []
  modified:
    - "crates/amux-daemon/src/agent/tool_executor.rs"
    - "crates/amux-protocol/src/messages.rs"
    - "crates/amux-daemon/src/server.rs"

key-decisions:
  - "Direct Framing type construction in both tool_executor and server.rs since handoff::divergent is pub"
  - "Custom framings require minimum 2 entries (filter applied), matching DivergentSession::new constraint"
  - "goal_run_id passed as None from tool context (no direct access to goal run source)"

patterns-established:
  - "Divergent session entry points follow same tool+IPC dual-path as route_to_specialist+AgentStartGoalRun"

requirements-completed: [DIVR-01, DIVR-02]

# Metrics
duration: 4min
completed: 2026-03-27
---

# Phase 06 Plan 01: Divergent Entry Points Summary

**run_divergent tool and AgentStartDivergentSession IPC wiring two entry points into the existing 739-line DivergentSession infrastructure**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-27T17:36:00Z
- **Completed:** 2026-03-27T17:40:44Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Wired the run_divergent tool so the agent can trigger divergent sessions during its tool loop (DIVR-01)
- Added AgentStartDivergentSession/AgentDivergentSessionStarted IPC variants so external clients (CLI, TUI, Electron) can start divergent sessions (DIVR-02)
- All 20 existing divergent tests continue to pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add run_divergent tool definition and execution handler** - `b3a7ff4` (feat)
2. **Task 2: Add IPC message variants and server handler for divergent sessions** - `7eb4d35` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/tool_executor.rs` - run_divergent tool definition, dispatch arm, execute_run_divergent function
- `crates/amux-protocol/src/messages.rs` - AgentStartDivergentSession ClientMessage + AgentDivergentSessionStarted DaemonMessage
- `crates/amux-daemon/src/server.rs` - IPC handler parsing custom framings and calling start_divergent_session

## Decisions Made
- Used direct `super::handoff::divergent::Framing` / `crate::agent::handoff::divergent::Framing` paths since all intermediate modules are pub -- no helper wrapper needed
- Custom framings filter requires >= 2 entries to match DivergentSession::new's validation constraint
- goal_run_id passed as None from tool executor context since task_id doesn't carry goal run provenance directly

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Both entry points (tool + IPC) now reach start_divergent_session
- Ready for Plan 02 (additional divergent session integration or UI wiring)

## Self-Check: PASSED

All files found. All commits verified.

---
*Phase: 06-divergent-entry-points*
*Completed: 2026-03-27*
