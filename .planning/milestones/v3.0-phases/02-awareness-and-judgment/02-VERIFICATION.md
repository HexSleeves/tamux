---
phase: 02-awareness-and-judgment
verified: 2026-03-27T09:46:12Z
status: passed
score: 20/20 must-haves verified
---

# Phase 02: Awareness and Judgment Verification Report

**Phase Goal:** The agent senses when it is stuck, tracks the texture of its own activity, and expresses honest confidence grounded in structural evidence
**Verified:** 2026-03-27T09:46:12Z
**Status:** PASSED
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Every tool call in agent_loop records an outcome to the awareness monitor | VERIFIED | `agent_loop.rs` L1148: `self.record_awareness_outcome(...)` called after counter-who update |
| 2 | When 3+ consecutive same tool+args patterns fire with no progress, awareness detects diminishing returns | VERIFIED | `awareness/mod.rs` L111: `consecutive_same_pattern < 3` threshold check, `short_term_success_rate < 0.3` dual condition, 32 unit tests pass |
| 3 | Counter-who is consulted before any mode shift fires (false positive guard) | VERIFIED | `awareness/mod.rs` L194-200: `detect_repeated_approaches` called before `evaluate_mode_shift`; mode_shift.rs requires both signals |
| 4 | Trajectory (converging/diverging/stalled) computed from progress vs failure ratio in sliding window | VERIFIED | `awareness/trajectory.rs` L33-40: `compute_trajectory` returns -1.0..1.0; L43-57: `compute_trajectory_state` classifies direction with 0.1 stalled zone |
| 5 | Three-tier sliding windows track short-term (5 actions), medium-term (30 min), long-term (session) | VERIFIED | `awareness/tracker.rs` L12: `SHORT_TERM_COUNT = 5`, L15: `MEDIUM_TERM_SECS = 30 * 60`, L96-134: `recompute_rates` computes all three |
| 6 | TrajectoryUpdate, ModeShift, and ConfidenceWarning AgentEvent variants exist and can be broadcast | VERIFIED | `types.rs` L2310: TrajectoryUpdate, L2318: ModeShift, L2325: ConfidenceWarning; all used in engine/goal_planner/heartbeat |
| 7 | Difficulty scalar (0.0-1.0) computed from error rate and retry count | VERIFIED | `embodied/dimensions.rs` L12-15: `compute_difficulty` with 0.6/0.4 weighting |
| 8 | Familiarity scalar (0.0-1.0) from episodic FTS5 hit count | VERIFIED | `embodied/dimensions.rs` L21-24: `compute_familiarity` with 0-5 linear scaling |
| 9 | Trajectory scalar (-1.0 to 1.0) from progress/failure ratio | VERIFIED | `embodied/dimensions.rs` L30-38: `compute_trajectory_score` maps ratio to -1..1 |
| 10 | Temperature scalar (0.0-1.0) from operator message frequency | VERIFIED | `embodied/dimensions.rs` L47-59: `compute_temperature` with freq+pacing dual signal |
| 11 | Weight scalar (0.0-1.0) classifying actions by impact | VERIFIED | `embodied/dimensions.rs` L67-80: `compute_weight` with static tool name match table |
| 12 | EmbodiedMetadata struct aggregates all 5 dimensions | VERIFIED | `embodied/mod.rs` L20-32: struct with all 5 fields; L61-75: `compute_embodied_metadata` calls all dimension functions |
| 13 | Each goal plan step gets a confidence label [HIGH], [MEDIUM], or [LOW] prepended | VERIFIED | `goal_llm.rs` L173: `step.title = format!("[{}] {}", assessment.label, step.title)` |
| 14 | Confidence derives from 4 structural signals, not LLM self-assessment | VERIFIED | `uncertainty/confidence.rs` L83-86: weighted formula 0.30 tool_success + 0.25 familiarity + 0.25 (1-blast) + 0.20 (1-novelty) |
| 15 | LOW-confidence actions in Safety/Reliability domains block and require operator approval | VERIFIED | `uncertainty/domains.rs` L116-127: `should_block` returns true for Safety with LOW; goal_planner.rs L154: ConfidenceWarning with blocked=true |
| 16 | LOW-confidence actions in Research/Business domains surface without blocking | VERIFIED | `uncertainty/domains.rs` tests confirm Research never blocks; Business threshold set to Guessing (nothing below) |
| 17 | Operator can configure per-domain confidence thresholds via UncertaintyConfig | VERIFIED | `types.rs` L1251: `pub uncertainty: UncertaintyConfig` on AgentConfig with serde(default); `uncertainty/mod.rs` L29-38: DomainThresholds configurable |
| 18 | CalibrationTracker records predicted band vs actual outcome with conservative cold-start | VERIFIED | `uncertainty/calibration.rs` L75-88: `get_calibrated_band` shifts one step more cautious when < threshold observations |
| 19 | All HIGH proceed / any MEDIUM inform / any LOW require approval (UNCR-08) | VERIFIED | `goal_planner.rs` L131-178: `plan_confidence_gate` routes LOW to AwaitingApproval, MEDIUM to WorkflowNotice, HIGH to Proceed |
| 20 | Trajectory visible in heartbeat check results during active goal runs (AWAR-04) | VERIFIED | `heartbeat.rs` L310-314: TrajectoryUpdate event emitted for each running goal run |

**Score:** 20/20 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/amux-daemon/src/agent/awareness/mod.rs` | AwarenessMonitor struct, re-exports | VERIFIED | 366 lines, AwarenessMonitor with per-entity tracking, AgentEngine integration methods, 14 unit tests |
| `crates/amux-daemon/src/agent/awareness/tracker.rs` | Per-entity OutcomeWindow with 3-tier sliding windows | VERIFIED | 243 lines, OutcomeWindow struct with push/recompute, 8 unit tests |
| `crates/amux-daemon/src/agent/awareness/mode_shift.rs` | Strategy rotation and operator notification logic | VERIFIED | 120 lines, ModeShiftDecision with dual-signal guard, 5 unit tests |
| `crates/amux-daemon/src/agent/awareness/trajectory.rs` | TrajectoryState computation | VERIFIED | 124 lines, compute_trajectory + compute_trajectory_state, 7 unit tests |
| `crates/amux-daemon/src/agent/embodied/mod.rs` | EmbodiedMetadata struct, compute_embodied_metadata | VERIFIED | 126 lines, EmbodiedMetadata + EmbodiedSignals structs, aggregator function, 2 unit tests |
| `crates/amux-daemon/src/agent/embodied/dimensions.rs` | Pure computation functions for all 5 scalar dimensions | VERIFIED | 201 lines, 5 pure functions, 15 unit tests |
| `crates/amux-daemon/src/agent/uncertainty/mod.rs` | UncertaintyConfig with per-domain thresholds | VERIFIED | 70 lines, UncertaintyConfig + PlanConfidenceAction enum, 1 unit test |
| `crates/amux-daemon/src/agent/uncertainty/confidence.rs` | compute_step_confidence from structural signals | VERIFIED | 229 lines, ConfidenceSignals/Assessment, weighted formula, 12 unit tests |
| `crates/amux-daemon/src/agent/uncertainty/domains.rs` | DomainClassification enum and tool-to-domain mapping | VERIFIED | 198 lines, 4-domain enum, classify_domain, classify_step_kind, DomainThresholds with should_block, 8 unit tests |
| `crates/amux-daemon/src/agent/uncertainty/calibration.rs` | CalibrationTracker with conservative cold-start | VERIFIED | 191 lines, observation recording, calibrated band with conservative shift, stats, 5 unit tests |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `agent_loop.rs` | `awareness/tracker.rs` | `record_awareness_outcome` called after counter-who update | WIRED | L1148: called with entity_id, tool_name, args_hash, success, is_progress |
| `awareness/mode_shift.rs` | `episodic/counter_who.rs` | `detect_repeated_approaches` check before mode shift | WIRED | L196: `detect_repeated_approaches(&store.counter_who.tried_approaches, 3)` |
| `engine.rs` | `awareness/mod.rs` | `RwLock<AwarenessMonitor>` field on AgentEngine | WIRED | L134: field declaration, L236: initialization in new_with_storage |
| `goal_llm.rs` | `uncertainty/confidence.rs` | `annotate_plan_steps_with_confidence` calls `compute_step_confidence` | WIRED | L95: called after plan generation, L168: compute_step_confidence invoked per step |
| `goal_planner.rs` | `uncertainty/mod.rs` | `plan_confidence_gate` checks min band and routes | WIRED | L103: called after plan update, L154: emits ConfidenceWarning, L104: routes to AwaitingApproval |
| `uncertainty/confidence.rs` | `embodied/dimensions.rs` | episodic_familiarity from compute_familiarity | WIRED | goal_llm.rs L130: `compute_familiarity(episodes.len())` feeds into signals.episodic_familiarity |
| `uncertainty/confidence.rs` | `awareness/tracker.rs` | tool_success_rate from aggregate_short_term_success_rate | WIRED | goal_llm.rs L120-122: reads from awareness monitor |
| `heartbeat.rs` | `awareness/trajectory.rs` | trajectory summary in heartbeat results | WIRED | L310: `get_awareness_trajectory(gr_id)`, L311: emits TrajectoryUpdate event |
| `engine.rs` | `uncertainty/calibration.rs` | CalibrationTracker field on AgentEngine | WIRED | L136: field declaration, L237: initialization |
| `types.rs` | `uncertainty/mod.rs` | UncertaintyConfig on AgentConfig | WIRED | L1251: `pub uncertainty: UncertaintyConfig`, L1815: Default impl |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `awareness/mod.rs` | AwarenessMonitor.windows | `record_outcome` from agent_loop tool calls | Yes -- every tool call in hot loop writes real outcomes | FLOWING |
| `awareness/trajectory.rs` | TrajectoryState | `OutcomeWindow.total_progress_count/total_failure_count` | Yes -- computed from real recorded outcomes | FLOWING |
| `embodied/dimensions.rs` | 5 scalar dimensions | Pure functions from input signals | Yes -- pure computations on real inputs | FLOWING |
| `uncertainty/confidence.rs` | ConfidenceAssessment | 4 structural signals from awareness+episodic+counter_who | Yes -- signals gathered from live subsystems in goal_llm.rs L118-159 | FLOWING |
| `goal_llm.rs` (annotation) | step.title with label | compute_step_confidence output | Yes -- labels prepended to real plan steps in live goal planning | FLOWING |
| `goal_planner.rs` (gate) | PlanConfidenceAction | step title prefix parsing | Yes -- reads from annotated steps and routes to real approval flow | FLOWING |
| `heartbeat.rs` (trajectory) | TrajectoryUpdate event | get_awareness_trajectory from live monitor | Yes -- emitted for running goal runs during heartbeat cycle | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Awareness module tests pass | `cargo test -p tamux-daemon awareness` | 32 passed, 0 failed | PASS |
| Embodied module tests pass | `cargo test -p tamux-daemon embodied` | 19 passed, 0 failed | PASS |
| Uncertainty module tests pass | `cargo test -p tamux-daemon uncertainty` | 29 passed, 0 failed | PASS |
| Full daemon compiles | `cargo check -p tamux-daemon` | Finished dev (warnings only, pre-existing) | PASS |
| Full daemon test suite | `cargo test -p tamux-daemon --bin tamux-daemon` | 998 passed, 2 failed (pre-existing plugin::loader failures, unrelated) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| AWAR-01 | 02-01 | Empirical failure tracking across all agent activity | SATISFIED | `agent_loop.rs` L1148 records every tool call outcome to awareness monitor |
| AWAR-02 | 02-01 | Automatic mode shift when diminishing returns detected | SATISFIED | `awareness/mod.rs` L183-212: dual-signal mode shift fires ModeShift event |
| AWAR-03 | 02-01 | Counter-who consulted before mode shifts fire | SATISFIED | `awareness/mod.rs` L194-200: detect_repeated_approaches called before evaluate_mode_shift |
| AWAR-04 | 02-01 (used in 02-03) | Trajectory tracking surfaced to operator | SATISFIED | `heartbeat.rs` L310-314: TrajectoryUpdate events for running goal runs |
| AWAR-05 | 02-01 | Sliding window analysis (5 actions / 30 min / session) | SATISFIED | `tracker.rs` L12-15: SHORT_TERM_COUNT=5, MEDIUM_TERM_SECS=1800; all three rates computed |
| EMBD-01 | 02-02 | Scalar dimensions: difficulty, familiarity, trajectory | SATISFIED | `dimensions.rs` L12-38: compute_difficulty, compute_familiarity, compute_trajectory_score |
| EMBD-02 | 02-02 | Temperature dimension from operator urgency | SATISFIED | `dimensions.rs` L47-59: compute_temperature from message frequency and pacing |
| EMBD-03 | 02-02 | Weight dimension distinguishing light from heavy actions | SATISFIED | `dimensions.rs` L67-80: compute_weight with static tool name classification |
| EMBD-04 | 02-02 | Embodied metadata feeds into uncertainty scoring | SATISFIED | `goal_llm.rs` L130: compute_familiarity feeds into confidence signals; EmbodiedMetadata struct available |
| UNCR-01 | 02-03 | Planning confidence: each goal plan step rated HIGH/MEDIUM/LOW | SATISFIED | `goal_llm.rs` L173: `[HIGH/MEDIUM/LOW]` prepended to step titles |
| UNCR-04 | 02-03 | Domain-specific escalation: Safety blocks, Research surfaces | SATISFIED | `domains.rs` L116-127: should_block with domain-specific thresholds |
| UNCR-05 | 02-03 | Operator preferences: configurable thresholds per domain | SATISFIED | `types.rs` L1251: UncertaintyConfig on AgentConfig; DomainThresholds is serde-configurable |
| UNCR-06 | 02-03 | Confidence from structural signals, not LLM alone | SATISFIED | `confidence.rs` L83-86: 4 structural signals only (tool_success, familiarity, blast_radius, novelty) |
| UNCR-07 | 02-03 | Calibration feedback loop: operator corrections adjust model | SATISFIED | `calibration.rs` L34-110: CalibrationTracker with record_observation, conservative cold-start at <50 obs |
| UNCR-08 | 02-03 | All HIGH proceed / MEDIUM inform / LOW require approval | SATISFIED | `goal_planner.rs` L131-178: plan_confidence_gate with three-tier routing |

**Note:** UNCR-02 (tool-call confidence) and UNCR-03 (output confidence) were moved to Phase 4 during plan verification per CONTEXT.md decision (confidence in non-goal contexts deferred to v3.1). Confirmed in REQUIREMENTS.md: both mapped to Phase 4 with Pending status.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | - | - | - | No anti-patterns found in any phase 02 files |

Zero TODO/FIXME/PLACEHOLDER/stub patterns found across all 10 created files.

### Human Verification Required

### 1. Confidence Labels in Goal Plan Output

**Test:** Start a goal run with uncertainty enabled, observe the plan steps in GUI/TUI
**Expected:** Each step should show `[HIGH] Step title`, `[MEDIUM] Step title`, or `[LOW] Step title` in the plan display
**Why human:** Requires running daemon with a real LLM to generate plans and observing the output rendering

### 2. LOW-Confidence Approval Routing

**Test:** Trigger a goal run where a step gets LOW confidence (e.g., a novel, destructive command with low tool success rate)
**Expected:** Plan should enter AwaitingApproval status and ConfidenceWarning event should surface in UI
**Why human:** Requires specific conditions that are hard to trigger programmatically without manipulating awareness state

### 3. Heartbeat Trajectory Display

**Test:** Start a long-running goal run and observe heartbeat output in TUI
**Expected:** TrajectoryUpdate events should show convergence/divergence/stalled status for the running goal
**Why human:** Requires running daemon with active goal runs and observing heartbeat cycle output

### Gaps Summary

No gaps found. All 20 must-have truths verified, all 10 artifacts exist, are substantive, and are wired. All 10 key links verified as connected. Data flows from live subsystems through the entire pipeline. All 15 Phase 2 requirements are satisfied. All 80 unit tests pass. Compilation succeeds. No anti-patterns detected.

The 2 pre-existing test failures in `plugin::loader::tests` are unrelated to Phase 2 work and do not represent regressions.

---

_Verified: 2026-03-27T09:46:12Z_
_Verifier: Claude (gsd-verifier)_
