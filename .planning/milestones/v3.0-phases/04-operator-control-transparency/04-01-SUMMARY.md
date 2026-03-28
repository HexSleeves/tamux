---
phase: 04-operator-control-transparency
plan: 01
subsystem: agent-engine
tags: [cost-tracking, token-accounting, rate-cards, budget-alerts, goal-runs]

# Dependency graph
requires:
  - phase: 03-multi-agent-orchestration
    provides: "GoalRun struct, AgentEngine, agent_loop, goal_planner infrastructure"
provides:
  - "CostTracker, CostConfig, CostSummary types for per-goal cost accounting"
  - "RateCard system with default pricing for popular models"
  - "Budget alert events when cost exceeds operator threshold"
  - "Cost fields on GoalRun (total_prompt_tokens, total_completion_tokens, estimated_cost_usd)"
affects: [04-02, 04-03, 04-04, frontend-cost-display, tui-cost-display]

# Tech tracking
tech-stack:
  added: []
  patterns: ["per-goal cost tracker lifecycle (create on first accumulate, cleanup on completion/failure)", "single-point cost accumulation to prevent double-counting"]

key-files:
  created:
    - "crates/amux-daemon/src/agent/cost/mod.rs"
    - "crates/amux-daemon/src/agent/cost/rate_cards.rs"
  modified:
    - "crates/amux-daemon/src/agent/types.rs"
    - "crates/amux-daemon/src/agent/engine.rs"
    - "crates/amux-daemon/src/agent/mod.rs"
    - "crates/amux-daemon/src/agent/agent_loop.rs"
    - "crates/amux-daemon/src/agent/goal_planner.rs"
    - "crates/amux-daemon/src/agent/heartbeat_checks.rs"
    - "crates/amux-daemon/src/agent/anticipatory.rs"
    - "crates/amux-daemon/src/agent/liveness/checkpoint.rs"
    - "crates/amux-daemon/src/agent/liveness/state_layers.rs"
    - "crates/amux-daemon/src/agent/task_crud.rs"
    - "crates/amux-daemon/src/history.rs"

key-decisions:
  - "Cost accumulation at exactly two call sites in agent_loop.rs (Done and ToolCalls paths) using a single accumulate_goal_run_cost helper -- prevents double-counting"
  - "Budget alerts are notification-only (no auto-stop) per research locked decision"
  - "Cost tracker cleanup on both goal completion and failure to prevent memory leaks"
  - "All new GoalRun fields use #[serde(default)] for backward compatibility with existing persisted data"

patterns-established:
  - "CostTracker lifecycle: lazily created per goal run, cleaned up on completion/failure"
  - "find_active_goal_run_for_thread: thread-to-running-goal mapping by iterating goal_runs"

requirements-completed: [COST-01, COST-02, COST-03, COST-04]

# Metrics
duration: 17min
completed: 2026-03-27
---

# Phase 04 Plan 01: Per-Goal Cost Accounting Summary

**CostTracker module with provider rate cards, per-goal token accumulation in agent_loop, budget alert events, and cost persistence on GoalRun**

## Performance

- **Duration:** 17 min
- **Started:** 2026-03-27T12:03:10Z
- **Completed:** 2026-03-27T12:20:14Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments
- CostTracker, CostConfig, CostSummary types with full serde support and rate card lookup
- Rate cards for 7 popular models (GPT-4o, GPT-4o-mini, Claude Sonnet/Haiku/Opus, o1-mini) with date-suffix stripping
- Per-goal token + cost accumulation wired into agent_loop at both LLM response paths
- Budget alert event fires once when cumulative cost crosses operator-defined threshold
- Cost data persists on GoalRun completion and failure via existing persist_goal_runs path
- 14 unit tests covering tracker, rate cards, serde roundtrip, and backward compatibility

## Task Commits

Each task was committed atomically:

1. **Task 1: Create cost module with CostTracker, rate cards, and types** - `e0ec115` (feat)
2. **Task 2: Wire cost accumulation into agent_loop and goal completion** - `1592018` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/cost/mod.rs` - CostTracker, CostConfig, CostSummary types and accumulation logic
- `crates/amux-daemon/src/agent/cost/rate_cards.rs` - RateCard struct, default rate cards, lookup function
- `crates/amux-daemon/src/agent/types.rs` - GoalRun cost fields, CostConfig on AgentConfig, BudgetAlert event
- `crates/amux-daemon/src/agent/engine.rs` - cost_trackers field on AgentEngine
- `crates/amux-daemon/src/agent/mod.rs` - pub mod cost declaration
- `crates/amux-daemon/src/agent/agent_loop.rs` - Cost accumulation hooks and helper methods
- `crates/amux-daemon/src/agent/goal_planner.rs` - Cost summary finalization on completion/failure
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Updated test constructor for new field
- `crates/amux-daemon/src/agent/anticipatory.rs` - Updated test GoalRun for new fields
- `crates/amux-daemon/src/agent/liveness/checkpoint.rs` - Updated test GoalRun for new fields
- `crates/amux-daemon/src/agent/liveness/state_layers.rs` - Updated test GoalRun for new fields
- `crates/amux-daemon/src/agent/task_crud.rs` - Updated GoalRun constructor for new fields
- `crates/amux-daemon/src/history.rs` - Updated GoalRun SQL deserializer for new fields

## Decisions Made
- Cost accumulation uses a single `accumulate_goal_run_cost` helper called from both Done and ToolCalls paths -- avoids double-counting while covering all LLM responses
- Budget alerts are notification-only (BudgetAlert event variant) -- no auto-stop behavior
- Cost tracker cleaned up on both goal completion AND failure to prevent memory leaks
- All new GoalRun fields use `#[serde(default)]` for backward compatibility with existing data
- CostSummary also written on goal failure so operators can see how much a failed run cost

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CostSummary serde backward compatibility**
- **Found during:** Task 1 (CostSummary serde test)
- **Issue:** CostSummary fields missing `#[serde(default)]` caused deserialization from empty JSON to fail
- **Fix:** Added `#[serde(default)]` to total_prompt_tokens and total_completion_tokens fields
- **Files modified:** crates/amux-daemon/src/agent/cost/mod.rs
- **Verification:** cost_summary_default_deserialize test passes
- **Committed in:** e0ec115 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Added cost finalization on goal failure**
- **Found during:** Task 2 (goal_planner wiring)
- **Issue:** Plan only specified cost writing on completion; failed goal runs would lose cost data and leak tracker memory
- **Fix:** Added cost summary write and tracker cleanup in fail_goal_run
- **Files modified:** crates/amux-daemon/src/agent/goal_planner.rs
- **Verification:** cargo check passes, fail_goal_run writes cost and removes tracker
- **Committed in:** 1592018 (Task 2 commit)

**3. [Rule 3 - Blocking] Updated GoalRun constructors in history.rs**
- **Found during:** Task 1 (cargo check)
- **Issue:** Two GoalRun struct literals in history.rs (SQL deserialization and test) missing new cost fields
- **Fix:** Added total_prompt_tokens: 0, total_completion_tokens: 0, estimated_cost_usd: None
- **Files modified:** crates/amux-daemon/src/history.rs
- **Verification:** cargo check compiles without errors
- **Committed in:** e0ec115 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 1 missing critical, 1 blocking)
**Impact on plan:** All auto-fixes necessary for correctness and compilation. No scope creep.

## Issues Encountered
- 2 pre-existing test failures in plugin::loader::tests (calendar_manifest and gmail_manifest) unrelated to cost tracking -- these check version strings that changed independently

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Cost tracking infrastructure is complete and ready for UI surfacing
- CostConfig is part of AgentConfig and will be configurable via existing settings path
- BudgetAlert event is ready for frontend/TUI consumption
- Rate cards can be extended by operators via config or future plans

## Self-Check: PASSED
- All 8 key files verified present
- Commit e0ec115 (Task 1) verified in git log
- Commit 1592018 (Task 2) verified in git log
- 14 cost module tests pass
- 1096 daemon tests pass (2 pre-existing failures unrelated to cost tracking)

---
*Phase: 04-operator-control-transparency*
*Completed: 2026-03-27*
