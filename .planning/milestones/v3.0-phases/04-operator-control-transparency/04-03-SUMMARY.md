---
phase: 04-operator-control-transparency
plan: 03
subsystem: agent-engine
tags: [explainability, causal-traces, rejected-alternatives, ipc, goal-planning]

# Dependency graph
requires:
  - phase: 04-01
    provides: "Cost tracking and GoalRun infrastructure"
  - phase: 04-02
    provides: "Autonomy dial and authorship tracking"
  - phase: 01-memory-foundation
    provides: "Episodic memory retrieval, negative knowledge constraints, causal trace infrastructure"
  - phase: 02-situational-awareness
    provides: "Confidence scoring, uncertainty quantification"
provides:
  - "ExplanationResponse and AlternativeConsidered types for structured 'why did you do that?' answers"
  - "handle_explain_action handler with cascade: causal_trace > episodic > negative_knowledge > fallback"
  - "Rejected alternatives captured during goal planning and stored as CausalTrace DecisionOption records"
  - "AgentExplainAction/AgentExplanation IPC variants for client-daemon queries"
  - "list_causal_traces_for_goal_run history method for full causal trace retrieval"
affects: [frontend-explainability-ui, tui-explain-command, cli-explain-subcommand]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cascade explanation resolution: causal_trace > episodic > negative_knowledge > fallback"
    - "Separate explainability module (Phase 4 EXPL) from explanation module (Phase 2 D-03)"

key-files:
  created:
    - crates/amux-daemon/src/agent/explainability.rs
  modified:
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/goal_parsing.rs
    - crates/amux-daemon/src/agent/causal_traces.rs
    - crates/amux-daemon/src/agent/mod.rs
    - crates/amux-daemon/src/history.rs
    - crates/amux-daemon/src/server.rs
    - crates/amux-protocol/src/messages.rs

key-decisions:
  - "Separate explainability.rs module from existing explanation.rs -- different concerns (EXPL vs D-03)"
  - "Cascade resolution order: causal_trace > episodic > negative_knowledge > fallback ensures always-something response"
  - "rejected_alternatives as Vec<String> on GoalPlanResponse -- simple, serde(default), backward compatible"
  - "Rejected alternatives stored as DecisionOption with option_type=plan_alternative in causal traces"
  - "CausalTraceFullRecord struct for complete trace retrieval (distinct from existing CausalTraceRecord)"

patterns-established:
  - "Never-empty explanation pattern: fallback text always provided per research Pitfall 4"
  - "LLM prompt extension pattern: add field to JSON schema + prompt instruction for optional enrichment"

requirements-completed: [EXPL-01, EXPL-02, EXPL-03]

# Metrics
duration: 12min
completed: 2026-03-27
---

# Phase 04 Plan 03: Explainability Mode Summary

**On-demand "why did you do that?" query with causal trace cascade, rejected alternatives capture, and IPC dispatch**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-27T12:43:57Z
- **Completed:** 2026-03-27T12:55:57Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments
- Implemented ExplanationResponse type with full serde round-trip support for structured decision explanations
- Extended goal planning to capture 1-3 rejected alternatives from LLM and store them as CausalTrace records
- Built handle_explain_action with 4-level cascade (causal_trace > episodic > negative_knowledge > fallback) ensuring never-empty responses
- Added AgentExplainAction/AgentExplanation IPC variants and server.rs dispatch handler

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend planning to capture rejected alternatives and build explanation query handler** - `6155103` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/explainability.rs` - New module: ExplanationResponse types, AlternativeConsidered, handle_explain_action cascade handler
- `crates/amux-daemon/src/agent/goal_llm.rs` - Planning prompt updated to request rejected_alternatives from LLM
- `crates/amux-daemon/src/agent/goal_parsing.rs` - GoalPlanResponse extended with rejected_alternatives field, JSON schema updated, 2 new tests
- `crates/amux-daemon/src/agent/causal_traces.rs` - persist_goal_plan_causal_trace now stores rejected alternatives as DecisionOption records
- `crates/amux-daemon/src/agent/mod.rs` - Registered explainability module, updated test struct literal
- `crates/amux-daemon/src/history.rs` - Added list_causal_traces_for_goal_run method and CausalTraceFullRecord struct
- `crates/amux-daemon/src/server.rs` - Added AgentExplainAction dispatch handler
- `crates/amux-protocol/src/messages.rs` - Added AgentExplainAction and AgentExplanation IPC variants

## Decisions Made
- Kept explainability.rs separate from explanation.rs since they serve different purposes (EXPL "why did you do that?" vs D-03 confidence band templates)
- Used cascade resolution (causal_trace > episodic > negative_knowledge > fallback) matching the research recommendation to always return something
- Added rejected_alternatives as Vec<String> with serde(default) for full backward compatibility with existing plan JSON
- Stored rejected alternatives as DecisionOption with option_type="plan_alternative" to integrate with existing causal trace infrastructure
- Created CausalTraceFullRecord as a separate struct from CausalTraceRecord to avoid breaking existing consumers

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing rejected_alternatives in existing test struct literal**
- **Found during:** Task 1 (compilation)
- **Issue:** Adding rejected_alternatives field to GoalPlanResponse broke existing test in mod.rs that used struct literal construction
- **Fix:** Added `rejected_alternatives: Vec::new()` to the test struct literal
- **Files modified:** crates/amux-daemon/src/agent/mod.rs
- **Verification:** cargo check passed
- **Committed in:** 6155103 (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Minimal -- standard field addition requiring existing code update. No scope creep.

## Issues Encountered
None

## Known Stubs
None -- all types are fully implemented, handler cascade is complete, IPC dispatch is wired.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Explainability IPC is ready for frontend/TUI integration
- Clients can send AgentExplainAction and receive AgentExplanation with structured JSON
- Rejected alternatives will populate as goal runs execute with the updated planning prompt

## Self-Check: PASSED

All 8 created/modified files verified present. Commit 6155103 verified in git log.

---
*Phase: 04-operator-control-transparency*
*Completed: 2026-03-27*
