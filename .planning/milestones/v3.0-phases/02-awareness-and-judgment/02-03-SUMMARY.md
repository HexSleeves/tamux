---
phase: 02-awareness-and-judgment
plan: 03
subsystem: agent-intelligence
tags: [uncertainty, confidence-scoring, domain-classification, calibration, goal-planning, structural-signals]

# Dependency graph
requires:
  - phase: 02-awareness-and-judgment plan 01
    provides: AwarenessMonitor with per-entity outcome tracking, aggregate_short_term_success_rate
  - phase: 02-awareness-and-judgment plan 02
    provides: EmbodiedMetadata with compute_familiarity, compute_difficulty pure functions
provides:
  - uncertainty/ module with structural confidence scoring (4 signals)
  - DomainClassification enum mapping tools and step kinds to Safety/Reliability/Business/Research
  - DomainThresholds with configurable per-domain blocking behavior
  - CalibrationTracker with conservative cold-start defaults
  - UncertaintyConfig on AgentConfig with serde(default)
  - Goal plan step annotation with [HIGH]/[MEDIUM]/[LOW] labels
  - Plan confidence gate routing LOW plans to operator approval
  - Heartbeat trajectory updates for active goal runs
affects: [phase-03-handoffs, phase-04-operator-controls]

# Tech tracking
tech-stack:
  added: []
  patterns: [structural-signal-based-confidence, domain-specific-escalation, conservative-cold-start-calibration]

key-files:
  created:
    - crates/amux-daemon/src/agent/uncertainty/mod.rs
    - crates/amux-daemon/src/agent/uncertainty/confidence.rs
    - crates/amux-daemon/src/agent/uncertainty/domains.rs
    - crates/amux-daemon/src/agent/uncertainty/calibration.rs
  modified:
    - crates/amux-daemon/src/agent/mod.rs
    - crates/amux-daemon/src/agent/types.rs
    - crates/amux-daemon/src/agent/engine.rs
    - crates/amux-daemon/src/agent/heartbeat_checks.rs
    - crates/amux-daemon/src/agent/goal_llm.rs
    - crates/amux-daemon/src/agent/goal_planner.rs
    - crates/amux-daemon/src/agent/heartbeat.rs

key-decisions:
  - "Confidence from 4 structural signals only (tool_success 0.30, familiarity 0.25, blast_radius 0.25, novelty 0.20) -- no LLM self-assessment"
  - "Labels HIGH/MEDIUM/LOW map from ConfidenceBand: Confident->HIGH, Likely->MEDIUM, Uncertain|Guessing->LOW"
  - "Safety domains block on LOW, Business warns, Research surfaces all -- configurable via DomainThresholds"
  - "CalibrationTracker shifts one step more cautious with <50 observations per band (conservative cold-start)"
  - "GoalRunStepKind mapped to domains via classify_step_kind: Command->Safety, Research/Reason->Research, others->Business"
  - "Plan confidence gate: all HIGH = proceed, any MEDIUM = inform via WorkflowNotice, any LOW = route to AwaitingApproval"

patterns-established:
  - "Structural signal composition: pure scoring functions that combine multiple subsystem outputs into a single assessment"
  - "Domain-based escalation routing: different confidence thresholds per action category"
  - "Conservative cold-start: assume worse until calibration data builds up"

requirements-completed: [UNCR-01, UNCR-04, UNCR-05, UNCR-06, UNCR-07, UNCR-08]

# Metrics
duration: 21min
completed: 2026-03-27
---

# Phase 02 Plan 03: Uncertainty Quantification Summary

**Structural confidence scoring with 4-signal pipeline, domain-specific escalation thresholds, cold-start calibration, and goal planner approval routing**

## Performance

- **Duration:** 21 min
- **Started:** 2026-03-27T09:13:34Z
- **Completed:** 2026-03-27T09:34:34Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Created uncertainty/ module with 4 files providing structural confidence scoring from tool success rate, episodic familiarity, blast radius, and approach novelty
- Domain classification maps tools and step kinds to Safety/Reliability/Business/Research with configurable per-domain blocking thresholds
- CalibrationTracker applies conservative cold-start bias (HIGH->MEDIUM, MEDIUM->LOW) until 50+ observations per band
- Goal plan steps annotated with [HIGH]/[MEDIUM]/[LOW] labels after LLM plan generation, with evidence notes for non-HIGH steps
- Plan confidence gate routes LOW-confidence plans to AwaitingApproval status with ConfidenceWarning events
- Heartbeat emits TrajectoryUpdate events for active goal runs during each cycle

## Task Commits

Each task was committed atomically:

1. **Task 1: Create uncertainty module with confidence scoring, domain classification, calibration, and config** - `54a8cd4` (feat) - TDD with 29 tests
2. **Task 2: Wire uncertainty into goal planning, approval routing, and heartbeat trajectory** - `0ba9078` (feat)

## Files Created/Modified
- `crates/amux-daemon/src/agent/uncertainty/mod.rs` - UncertaintyConfig, PlanConfidenceAction enum, public API
- `crates/amux-daemon/src/agent/uncertainty/confidence.rs` - ConfidenceSignals, ConfidenceAssessment, compute_step_confidence, confidence_label, blast_radius_to_score, approach_novelty_score
- `crates/amux-daemon/src/agent/uncertainty/domains.rs` - DomainClassification enum, classify_domain, classify_step_kind, DomainThresholds, should_block
- `crates/amux-daemon/src/agent/uncertainty/calibration.rs` - CalibrationTracker with conservative cold-start, record_observation, get_calibrated_band, calibration_stats
- `crates/amux-daemon/src/agent/mod.rs` - Added `pub mod uncertainty`
- `crates/amux-daemon/src/agent/types.rs` - Added `uncertainty: UncertaintyConfig` to AgentConfig + Default impl
- `crates/amux-daemon/src/agent/engine.rs` - Added `calibration_tracker: RwLock<CalibrationTracker>` to AgentEngine + init
- `crates/amux-daemon/src/agent/heartbeat_checks.rs` - Added calibration_tracker to test AgentEngine struct
- `crates/amux-daemon/src/agent/goal_llm.rs` - annotate_plan_steps_with_confidence using 4 structural signals
- `crates/amux-daemon/src/agent/goal_planner.rs` - plan_confidence_gate with approval routing (UNCR-08)
- `crates/amux-daemon/src/agent/heartbeat.rs` - Trajectory updates for active goal runs (AWAR-04)

## Decisions Made
- Confidence scoring uses weighted formula: 0.30 tool_success + 0.25 familiarity + 0.25 (1-blast_radius) + 0.20 (1-novelty)
- Labels map from ConfidenceBand enum (Confident->HIGH, Likely->MEDIUM, Uncertain|Guessing->LOW) -- 3-tier user-facing labels over 4-tier internal bands
- GoalRunStepKind mapped to domains via classify_step_kind rather than converting to string and using classify_domain, since step kinds are enums
- blast_radius_score for plan steps derived from domain classification (Safety=0.8, Reliability=0.5, other=0.2) since individual tool names aren't available at plan time
- Annotation happens inside request_goal_plan after LLM plan generation and validation, before return to goal_planner

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Known Stubs
None - all data paths are wired to live subsystem outputs (awareness monitor, episodic store, counter-who).

## Next Phase Readiness
- Phase 02 (Awareness and Judgment) is now complete with all 3 plans executed
- Uncertainty module ready for consumption by Phase 03 (Multi-Agent Handoffs) where confidence scoring can inform specialist selection
- CalibrationTracker ready to receive actual outcome data once goal runs complete and record_observation is called from the reflection phase
- Domain thresholds are operator-configurable via agent config JSON

## Self-Check: PASSED

All created files exist. Both task commits (54a8cd4, 0ba9078) verified in git log.

---
*Phase: 02-awareness-and-judgment*
*Completed: 2026-03-27*
