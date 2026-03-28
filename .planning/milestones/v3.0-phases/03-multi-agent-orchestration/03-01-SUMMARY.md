---
phase: 03-multi-agent-orchestration
plan: 01
subsystem: agent
tags: [handoff, specialist-profiles, capability-matching, sqlite, multi-agent]

requires:
  - phase: 01-memory-foundation
    provides: "Episodic memory schema pattern (execute_batch), EpisodicStore on AgentEngine"
  - phase: 02-awareness-and-judgment
    provides: "CalibrationTracker on AgentEngine, awareness module pattern"
provides:
  - "HandoffBroker struct with 5 default specialist profiles"
  - "CapabilityTag-based match_specialist() scoring algorithm"
  - "13 handoff types: Proficiency, CapabilityTag, SpecialistProfile, escalation types, ContextBundle, AcceptanceCriteria, HandoffResult, HandoffBroker"
  - "SQLite specialist_profiles and handoff_log tables"
  - "handoff_broker: RwLock<HandoffBroker> field on AgentEngine"
affects: [03-02 context-bundling, 03-03 escalation-chains, 03-04 divergent-subagents]

tech-stack:
  added: []
  patterns: ["Capability-weighted specialist matching with proficiency scoring", "Handoff audit schema with indexed log tables"]

key-files:
  created:
    - crates/amux-daemon/src/agent/handoff/mod.rs
    - crates/amux-daemon/src/agent/handoff/profiles.rs
    - crates/amux-daemon/src/agent/handoff/schema.rs
  modified:
    - crates/amux-daemon/src/agent/mod.rs
    - crates/amux-daemon/src/agent/engine.rs
    - crates/amux-daemon/src/agent/heartbeat_checks.rs
    - crates/amux-daemon/src/history.rs

key-decisions:
  - "Proficiency weights: Expert=1.0, Advanced=0.75, Competent=0.5, Familiar=0.25"
  - "Match threshold default 0.3 -- allows generalist to serve as fallback"
  - "10% tie-break rule: when top-2 candidates are within 10% score, prefer higher max single-tag proficiency"
  - "Schema uses execute_batch pattern (consistent with episodic schema), not rusqlite_migration"

patterns-established:
  - "Handoff module structure: mod.rs (types + broker), profiles.rs (defaults + matching), schema.rs (SQLite)"
  - "Specialist profile matching via weighted capability tags with configurable threshold"

requirements-completed: [HAND-01, HAND-07, HAND-09]

duration: 6min
completed: 2026-03-27
---

# Phase 03 Plan 01: Handoff Module Foundation Summary

**HandoffBroker with 5 specialist profiles, capability-weighted matching algorithm, and SQLite persistence schema wired into AgentEngine**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-27T10:25:06Z
- **Completed:** 2026-03-27T10:31:06Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Defined all 13 handoff types (Proficiency, CapabilityTag, SpecialistProfile, escalation types, ContextBundle, EpisodeRef, PartialOutput, AcceptanceCriteria, HandoffResult, HandoffBroker)
- Implemented 5 default specialist profiles with capability tags and proficiency levels (researcher, backend-developer, frontend-developer, reviewer, generalist)
- Implemented match_specialist() scoring algorithm with 10% tie-break rule and configurable threshold
- Created SQLite schema with specialist_profiles and handoff_log tables plus 4 indexes
- Wired HandoffBroker as RwLock field on AgentEngine, initialized at startup
- 12 unit tests covering all matching scenarios

## Task Commits

Each task was committed atomically:

1. **Task 1: Create handoff module types, specialist profiles, and capability matching** - `a892dec` (feat)
2. **Task 2: Create handoff SQLite schema and wire HandoffBroker into AgentEngine** - `23f0c7b` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/handoff/mod.rs` - 13 handoff types, HandoffBroker struct with Default impl
- `crates/amux-daemon/src/agent/handoff/profiles.rs` - 5 default specialist profiles, match_specialist() algorithm, 12 unit tests
- `crates/amux-daemon/src/agent/handoff/schema.rs` - SQLite schema for specialist_profiles and handoff_log tables
- `crates/amux-daemon/src/agent/mod.rs` - Added `pub mod handoff;` declaration
- `crates/amux-daemon/src/agent/engine.rs` - Added handoff_broker field to AgentEngine struct and new_with_storage()
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Added handoff_broker field to test constructor
- `crates/amux-daemon/src/history.rs` - Added init_handoff_schema call in init_schema startup path

## Decisions Made
- Proficiency weights (Expert=1.0, Advanced=0.75, Competent=0.5, Familiar=0.25) provide clear differentiation for scoring
- Match threshold default 0.3 ensures generalist can serve as last-resort fallback
- When top-2 candidates score within 10%, prefer the one with higher max single-tag proficiency (rewards deep expertise over broad mediocrity)
- Used execute_batch for schema init (consistent with episodic schema pattern from Phase 1)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- HandoffBroker and all types ready for Plan 02 (context bundling) to implement ContextBundle methods
- Escalation types ready for Plan 03 (escalation chains) to implement trigger evaluation
- SpecialistProfile matching ready for broker dispatch logic
- handoff_log table ready for WORM audit trail writes

---
*Phase: 03-multi-agent-orchestration*
*Completed: 2026-03-27*
