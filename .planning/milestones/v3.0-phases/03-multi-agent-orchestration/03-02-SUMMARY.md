---
phase: 03-multi-agent-orchestration
plan: 02
subsystem: agent
tags: [handoff, context-bundle, escalation, acceptance, audit, worm, multi-agent]

requires:
  - phase: 03-multi-agent-orchestration
    plan: 01
    provides: "HandoffBroker types (ContextBundle, AcceptanceCriteria, escalation types), handoff_log schema"
  - phase: 01-memory-foundation
    provides: "Episodic memory WORM pattern, EpisodeRef struct"
  - phase: 02-awareness-and-judgment
    provides: "ConfidenceBand enum, confidence scoring"
provides:
  - "ContextBundle assembly with chars/4 token estimation and 2000-token ceiling enforcement"
  - "Depth limit enforcement at 3 hops (HAND-08)"
  - "Escalation trigger evaluation for ConfidenceBelow, ToolFails, TimeExceeds"
  - "confidence_band_order mapping with internal + user-facing aliases"
  - "AcceptanceCriteria structural validation (non_empty, min_length, contains)"
  - "ValidationResult with all-failures collection and needs_llm_validation flag"
  - "WORM handoff audit trail via append_telemetry('handoff', ...)"
  - "SQLite handoff_log detail recording and outcome updates"
affects: [03-03 broker-routing, 03-04 divergent-subagents]

tech-stack:
  added: []
  patterns: ["Token ceiling enforcement via progressive field summarization", "Structural check DSL (non_empty, min_length:N, contains:TEXT)", "WORM audit via history.append_telemetry with kind=handoff"]

key-files:
  created:
    - crates/amux-daemon/src/agent/handoff/context_bundle.rs
    - crates/amux-daemon/src/agent/handoff/escalation.rs
    - crates/amux-daemon/src/agent/handoff/acceptance.rs
    - crates/amux-daemon/src/agent/handoff/audit.rs
  modified:
    - crates/amux-daemon/src/agent/handoff/mod.rs
    - crates/amux-daemon/src/history.rs

key-decisions:
  - "Token estimation uses chars/4 (consistent with APPROX_CHARS_PER_TOKEN elsewhere)"
  - "Ceiling enforcement order: summarize parent_context first, then trim partial_outputs oldest-first, then truncate negative_constraints"
  - "confidence_band_order maps both internal (guessing/uncertain/likely/confident) and user-facing (low/medium/high) names"
  - "Made history.append_telemetry pub(crate) for cross-module WORM access"
  - "Structural checks collected all failures (not short-circuit) for comprehensive validation feedback"
  - "needs_llm_validation only set when structural checks pass (avoids wasted LLM calls on obviously bad output)"

patterns-established:
  - "Context bundle token ceiling: progressive summarization then field trimming"
  - "Acceptance criteria DSL: simple string-based structural checks parseable without regex"
  - "WORM audit for handoffs: kind='handoff' with structured JSON payload including handoff_log_id cross-reference"

requirements-completed: [HAND-02, HAND-03, HAND-04, HAND-05, HAND-06, HAND-08]

duration: 10min
completed: 2026-03-27
---

# Phase 03 Plan 02: Handoff Context Bundle and Audit Summary

**Context bundle assembly with 2000-token ceiling enforcement, escalation trigger evaluation for 3 trigger types, structural acceptance validation, and WORM handoff audit trail**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-27T10:35:29Z
- **Completed:** 2026-03-27T10:46:11Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Implemented ContextBundle with token estimation (chars/4), ceiling enforcement via progressive summarization, and depth limit at 3 hops
- Implemented escalation trigger evaluation supporting ConfidenceBelow, ToolFails, TimeExceeds with dual-name confidence mapping
- Implemented AcceptanceCriteria structural validation with 3 check types (non_empty, min_length:N, contains:TEXT) plus default factories
- Implemented WORM handoff audit trail via format_handoff_audit_payload and AgentEngine methods for SQLite + WORM recording
- 43 unit tests across 4 new files (13 context_bundle + 14 escalation + 13 acceptance + 3 audit)

## Task Commits

Each task was committed atomically:

1. **Task 1: Context bundle assembly, token ceiling, and escalation trigger evaluation**
   - `5cea73c` (test) - RED: failing tests for context bundle and escalation
   - `eea4e48` (feat) - GREEN: implementation with 27 tests passing
2. **Task 2: Acceptance criteria validation and WORM handoff audit trail**
   - `9cbf33f` (test) - RED: failing tests for acceptance and audit
   - `0b5a659` (feat) - GREEN: implementation with 16 tests passing

## Files Created/Modified
- `crates/amux-daemon/src/agent/handoff/context_bundle.rs` - ContextBundle impl: new, estimate_tokens, recompute, enforce_token_ceiling, depth limit
- `crates/amux-daemon/src/agent/handoff/escalation.rs` - confidence_band_order mapping, evaluate_escalation_triggers for 3 trigger types
- `crates/amux-daemon/src/agent/handoff/acceptance.rs` - AcceptanceCriteria.validate_structural with non_empty/min_length/contains checks, default factories
- `crates/amux-daemon/src/agent/handoff/audit.rs` - format_handoff_audit_payload, AgentEngine WORM + SQLite handoff logging
- `crates/amux-daemon/src/agent/handoff/mod.rs` - Added module declarations and ValidationResult re-export
- `crates/amux-daemon/src/history.rs` - Made append_telemetry pub(crate) for cross-module WORM access

## Decisions Made
- Token estimation uses chars/4 (consistent with APPROX_CHARS_PER_TOKEN used elsewhere in codebase)
- Ceiling enforcement follows priority order: summarize parent_context_summary with progressively smaller limits, then trim partial_outputs oldest-first, then truncate negative_constraints
- confidence_band_order handles both internal names (guessing/uncertain/likely/confident) and user-facing aliases (low/medium/high) with case-insensitive matching
- Made history.append_telemetry pub(crate) rather than adding a new wrapper method -- minimal change to enable WORM access from agent module
- Acceptance structural checks collect all failures (not short-circuit) so validation provides comprehensive feedback
- needs_llm_validation flag only set when all structural checks pass to avoid wasting LLM calls on structurally invalid output

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Made append_telemetry pub(crate)**
- **Found during:** Task 2 (WORM audit trail)
- **Issue:** history.rs append_telemetry was private, preventing AgentEngine handoff audit from calling it
- **Fix:** Changed visibility from private to pub(crate) -- minimal access widening
- **Files modified:** crates/amux-daemon/src/history.rs
- **Verification:** cargo check clean, all tests pass
- **Committed in:** 0b5a659 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Essential for WORM audit trail access. No scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Context bundle assembly ready for broker routing (Plan 03) to use when dispatching handoffs
- Escalation triggers ready for broker to evaluate during handoff monitoring
- Acceptance validation ready for broker to validate specialist output before accepting
- WORM audit trail ready for broker to record every handoff event
- All 4 files provide the intelligence layer that makes handoffs reliable rather than mechanical

---
*Phase: 03-multi-agent-orchestration*
*Completed: 2026-03-27*
