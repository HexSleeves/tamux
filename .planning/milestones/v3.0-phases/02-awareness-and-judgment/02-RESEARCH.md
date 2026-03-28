# Phase 2: Awareness and Judgment - Research

**Researched:** 2026-03-27
**Domain:** Situational awareness, embodied metadata, uncertainty quantification for Rust daemon agent runtime
**Confidence:** HIGH

## Summary

Phase 2 adds three interconnected intelligence subsystems to the tamux daemon: situational awareness (empirical failure tracking across all agent activity, mode shifts when stuck, sliding window analysis), embodied metadata (5 scalar dimensions per action: difficulty, familiarity, trajectory, temperature, weight), and uncertainty quantification (structural confidence labels on goal plan steps, domain-specific escalation, calibration feedback). These subsystems build directly on Phase 1's episodic memory, counter-who, and negative knowledge infrastructure.

The critical architectural insight is that this phase creates NO new SQLite tables for awareness (it operates on in-memory sliding windows), adds one optional table for embodied metadata persistence, and extends the existing uncertainty pipeline (explanation.rs ConfidenceBand, escalation.rs EscalationCriteria, causal_traces.rs blast_radius_advisory) rather than building from scratch. The existing stuck_detection.rs, health_monitor.rs, and self_assessment.rs provide proven patterns that awareness generalizes from goal-runner scope to all agent activity. The uncertainty engine is primarily a computation layer that combines structural signals (tool success rates, episodic familiarity, blast radius) into ConfidenceBand labels -- it extends, not replaces, the existing explanation.rs confidence pipeline.

**Primary recommendation:** Build awareness first (it produces signals for confidence), then embodied metadata (it computes the 5 scalar dimensions that feed into confidence scoring), then uncertainty quantification (it consumes both awareness trajectory and embodied familiarity to produce honest confidence labels). All three integrate as extension modules within AgentEngine following the established RwLock/Mutex-guarded field + impl AgentEngine pattern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Stuck detection triggers after 3+ same tool+args pattern with no new information gained (SHA-256 approach hashing from counter-who)
- When stuck is detected: auto-shift strategy (try different approach) + notify operator with "diminishing returns detected"
- Sliding window sizes: short-term 5 actions, medium-term 30 minutes, long-term full session
- Counter-who is consulted before ALL mode shifts fire (prevents false positives from productive repetition)
- Confidence labels displayed inline with plan steps: `[HIGH] Step 1: ...` -- compact, scannable
- Structural signals feeding confidence: tool success rate + episodic familiarity + blast radius + approach novelty
- Default domain escalation: Safety domains block on LOW, Business domains warn on LOW, Research domains surface all levels
- Confidence visible in goal planning only for v3.0 -- expand to chat/tasks in v3.1
- Provider-agnostic: confidence derives from structural signals, not LLM logits or model-specific features
- 5 core scalar dimensions: difficulty (retries, error rate), familiarity (episodic match), trajectory (converging/diverging), temperature (operator urgency), weight (conceptual mass)
- Trajectory calculated as ratio of "progress events" vs "retry/failure events" in sliding window
- Trajectory displayed in goal run status updates to operator + available in heartbeat
- Heartbeat uses trajectory and difficulty to adjust check frequency (stuck + hard = more frequent checks)

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions and are at Claude's discretion.

### Deferred Ideas (OUT OF SCOPE)
- Confidence in non-goal contexts (chat, tasks) -- v3.1
- Momentum and coherence dimensions -- v3.1 if needed
- Dashboard widget for trajectory visualization -- future UI work
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AWAR-01 | Empirical failure tracking across all agent activity (tool calls, sessions, browsing, goal runs) | Generalize existing stuck_detection.rs DetectionSnapshot from goal-runner scope to per-entity sliding windows in new awareness/ module |
| AWAR-02 | Automatic mode shift when diminishing returns detected | counter-who approach hashing (threshold 3) + new strategy rotation logic in mode_shift submodule |
| AWAR-03 | Counter-who consulted before mode shifts fire | Locked decision: counter_who.detect_repeated_approaches checked first; only fire mode shift when counter-who confirms |
| AWAR-04 | Trajectory tracking: converging vs diverging from goal | TrajectoryState struct with progress/retry ratio in sliding window; surfaced via AgentEvent::TrajectoryUpdate |
| AWAR-05 | Sliding window analysis (short-term: 5 actions, medium-term: 30 min, long-term: session) | Three-tier OutcomeWindow with VecDeque ring buffers keyed by entity_id |
| EMBD-01 | Scalar dimensions: difficulty, familiarity, trajectory | Pure computation functions: difficulty from error_rate + retry_count, familiarity from episodic match count, trajectory from awareness sliding window |
| EMBD-02 | Temperature dimension: urgency signals from operator messages | Parse operator message recency + frequency; fast follow-ups = high temperature |
| EMBD-03 | Weight dimension: conceptual mass | Classify actions by impact: config changes/deployments = heavy, reads/queries = light |
| EMBD-04 | Embodied metadata feeds into uncertainty scoring | EmbodiedMetadata struct passed to confidence computation; unfamiliar + difficult = lower confidence band |
| UNCR-01 | Planning confidence: each goal plan step rated HIGH/MEDIUM/LOW | Post-plan annotation in goal_llm.rs: compute_step_confidence() per step, append `[HIGH/MEDIUM/LOW]` to step title |
| UNCR-02 | Tool-call confidence: pre-execution warnings with blast-radius uncertainty | Pre-tool check in agent_loop.rs: combine blast_radius_advisory + structural signals; emit ConfidenceWarning if LOW |
| UNCR-03 | Output confidence: research results by source authority and freshness | Label source in tool results: official docs > community > unknown; tag with last-verified date |
| UNCR-04 | Domain-specific escalation: Safety/Reliability block on LOW | DomainClassification enum (Safety, Reliability, Research, Business); match tool/action to domain; block/warn per threshold |
| UNCR-05 | Operator preferences: configurable thresholds per domain | UncertaintyConfig on AgentConfig with per-domain ConfidenceBand thresholds, deserialized from config.json |
| UNCR-06 | Confidence derives from hybrid signals, not LLM alone | Four structural signals computed independently: tool_success_rate, episodic_familiarity, blast_radius_score, approach_novelty; combined into ConfidenceBand |
| UNCR-07 | Calibration feedback loop: operator corrections adjust confidence | CalibrationTracker records predicted_band vs actual_outcome; applies conservative bias when historical calibration is poor |
| UNCR-08 | All HIGH -> proceed; any MEDIUM -> inform; any LOW -> require approval | Decision logic in goal_planner.rs after plan annotation: inspect all step confidence bands, route to appropriate approval flow |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| No new crates | - | All Phase 2 work uses existing dependencies | Phase 2 is computation + in-memory state, not new I/O |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `sha2` | 0.10 (existing) | Approach hashing for stuck detection | Already used by counter_who.rs |
| `serde`/`serde_json` | 1.x (existing) | Serialization for config, events, persistence | All new types |
| `tokio` | 1.x (existing) | Async runtime, RwLock, broadcast | All async integration |
| `tracing` | 0.1 (existing) | Structured logging for awareness events | All new modules |
| `uuid` | 1.x (existing) | Event and audit IDs | New AgentEvent variants |

**No new crate dependencies required for Phase 2.** All subsystems are built on existing infrastructure.

## Architecture Patterns

### Recommended Project Structure
```
crates/amux-daemon/src/agent/
  awareness/
    mod.rs          # AwarenessMonitor struct, re-exports
    tracker.rs      # Per-entity OutcomeWindow with 3-tier sliding windows
    mode_shift.rs   # Strategy rotation + operator notification on diminishing returns
    trajectory.rs   # TrajectoryState: converging/diverging/stalled assessment
  embodied/
    mod.rs          # EmbodiedMetadata struct, compute functions
    dimensions.rs   # difficulty(), familiarity(), trajectory(), temperature(), weight()
  uncertainty/
    mod.rs          # UncertaintyConfig, public API
    confidence.rs   # compute_step_confidence(), compute_tool_confidence()
    domains.rs      # DomainClassification enum, domain matching, per-domain thresholds
    calibration.rs  # CalibrationTracker: predicted vs actual tracking, bias adjustment
```

### Pattern 1: Extension Module (established -- follow exactly)
**What:** New intelligence modules add RwLock-guarded fields to AgentEngine, with behavior in `impl AgentEngine` blocks in separate files.
**When:** Always. This is the only pattern used in the codebase.
**Example:**
```rust
// engine.rs -- add new fields
pub(super) awareness: RwLock<awareness::AwarenessMonitor>,
pub(super) uncertainty_config: RwLock<uncertainty::UncertaintyConfig>,

// awareness/tracker.rs -- implement via AgentEngine
impl AgentEngine {
    pub(crate) async fn record_awareness_outcome(
        &self,
        entity_id: &str,
        entity_type: &str,
        tool_name: &str,
        success: bool,
    ) {
        let now = super::now_millis();
        let mut monitor = self.awareness.write().await;
        monitor.record_outcome(entity_id, entity_type, tool_name, success, now);
    }
}
```

### Pattern 2: In-Memory Sliding Windows (new for awareness, follows VecDeque pattern)
**What:** Ring-buffered outcome tracking with three time horizons. No SQLite persistence needed -- awareness state is transient by design.
**When:** AWAR-01, AWAR-05 sliding window analysis.
**Example:**
```rust
pub struct OutcomeWindow {
    pub entity_id: String,
    pub entity_type: String,
    /// Ring buffer of recent outcomes, capped at max_entries.
    pub recent_outcomes: VecDeque<OutcomeEntry>,
    /// Computed from recent_outcomes on each insert.
    pub short_term_success_rate: f64,   // last 5 actions
    pub medium_term_success_rate: f64,  // last 30 minutes
    pub long_term_success_rate: f64,    // full session
    pub consecutive_same_pattern: u32,
    pub last_progress_at: u64,
}

pub struct OutcomeEntry {
    pub timestamp: u64,
    pub tool_name: String,
    pub args_hash: String,  // SHA-256 first 16 chars (reuse counter_who pattern)
    pub success: bool,
    pub is_progress: bool,  // did this action produce new information?
}
```

### Pattern 3: Pure Computation Functions (for embodied + uncertainty)
**What:** Confidence and embodied metadata are computed by pure functions that take structural signals as input. No LLM calls.
**When:** EMBD-01..04, UNCR-01..06.
**Example:**
```rust
pub fn compute_step_confidence(signals: &ConfidenceSignals) -> ConfidenceBand {
    // Combine structural signals into a single probability
    let score = 0.3 * signals.tool_success_rate
        + 0.25 * signals.episodic_familiarity
        + 0.25 * (1.0 - signals.blast_radius_score)
        + 0.2 * signals.approach_novelty_score;
    confidence_band(score)  // reuse existing explanation.rs function
}
```

### Pattern 4: AgentEvent Broadcasting (established)
**What:** New intelligence events use the existing event_tx broadcast channel.
**When:** Trajectory updates, mode shift notifications, confidence warnings.
**Example (new variants to add to AgentEvent enum):**
```rust
TrajectoryUpdate {
    goal_run_id: String,
    direction: String,    // "converging", "diverging", "stalled"
    progress_ratio: f64,  // 0.0-1.0
    message: String,
},
ModeShift {
    thread_id: String,
    reason: String,
    previous_strategy: String,
    new_strategy: String,
},
ConfidenceWarning {
    thread_id: String,
    action_type: String,  // "plan_step", "tool_call"
    band: String,         // "high", "medium", "low"
    evidence: String,
    domain: String,
    blocked: bool,
},
```

### Anti-Patterns to Avoid
- **SQLite for awareness state:** Awareness windows are transient in-memory state. Writing every outcome to SQLite would add write contention (Pitfall 2 from PITFALLS.md) for no benefit. Awareness state is cheap to recompute on restart.
- **LLM-based confidence scoring:** Confidence MUST derive from structural signals (tool success rates, episodic familiarity, blast radius). LLMs are systematically overconfident (Pitfall 4). Never ask the LLM "how confident are you?"
- **Single global sliding window:** Per-entity windows prevent a stuck thread from poisoning healthy threads' signals (Architecture anti-pattern 5).
- **Blocking mode shifts without counter-who check:** Locked decision from CONTEXT.md -- counter-who is ALWAYS consulted before any mode shift fires.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Approach hashing | New hash function | `counter_who::compute_approach_hash` (SHA-256) | Already exists, already wired into agent_loop.rs |
| Confidence band mapping | New enum/thresholds | `explanation::ConfidenceBand` + `confidence_band()` | Already used by causal_traces.rs, heartbeat.rs, audit trail |
| Escalation pathways | New escalation system | `metacognitive::escalation::EscalationState` | Full L0-L3 escalation already exists with audit integration |
| Stuck detection patterns | New pattern detector | `liveness::stuck_detection::StuckDetector` | Proven patterns with configurable thresholds, 19+ tests |
| Health assessment | New assessment framework | `metacognitive::self_assessment::SelfAssessor` | Already computes progress, efficiency, quality metrics |
| Momentum computation | New momentum function | `self_assessment::compute_momentum()` | Already exists with interval-based acceleration/deceleration |
| Event broadcasting | New notification system | `event_tx.send(AgentEvent::...)` | Established pattern used by 20+ event types |
| Blast radius assessment | New risk classifier | `causal_traces::command_blast_radius_advisory()` | Already classifies commands by impact family |

**Key insight:** Phase 2 is primarily a _composition_ effort -- assembling existing primitives (approach hashing, confidence bands, stuck detection, blast radius, self-assessment) into new higher-level abstractions (awareness monitor, confidence pipeline, embodied metadata). The primitives are already tested and wired.

## Common Pitfalls

### Pitfall 1: False Positive Mode Shifts on Legitimate Difficult Work
**What goes wrong:** Awareness monitor detects "diminishing returns" and triggers mode shift during legitimately difficult refactoring tasks that involve many similar-looking file reads.
**Why it happens:** Tool-name repetition (the existing StuckDetector pattern) conflates difficulty with being stuck. Complex tasks legitimately involve repeated tool calls with different arguments.
**How to avoid:** The locked decision to consult counter-who before all mode shifts is the primary defense. Counter-who tracks argument hashes, not just tool names. If arguments differ, the agent is exploring, not stuck. Additionally, the approach_hash in counter_who already uses SHA-256 of "tool_name:args" -- reuse this exact mechanism in awareness tracking.
**Warning signs:** Mode shifts fire during productive goal runs where each action reads a different file.

### Pitfall 2: Confidence Labels Become Theatrical Without Structural Grounding
**What goes wrong:** The uncertainty engine reports HIGH confidence on steps it should flag as uncertain, because structural signals are not properly computed or weighted.
**Why it happens:** It's tempting to short-circuit confidence computation to just return HIGH/MEDIUM/LOW based on simplistic heuristics. Without proper signal computation (episodic familiarity lookup, tool success rate calculation, blast radius assessment), confidence labels degrade to random noise.
**How to avoid:** Each of the 4 structural signals (tool_success_rate, episodic_familiarity, blast_radius, approach_novelty) must be computed from real data, not estimated. Tool success rate comes from awareness sliding windows. Episodic familiarity comes from FTS5 hit count. Blast radius comes from existing causal_traces.rs. Approach novelty comes from counter-who's tried_approaches list.
**Warning signs:** >70% of steps labeled HIGH; escalation triggers based on LOW fire <5% of the time.

### Pitfall 3: Sliding Window Memory Leak
**What goes wrong:** Per-entity OutcomeWindows accumulate for every thread/goal/session that has ever existed, growing unbounded in memory.
**Why it happens:** Windows are created when an entity starts activity but never cleaned up when the entity completes or is abandoned.
**How to avoid:** Cap the total number of tracked entities (e.g., 100 active windows). Prune windows for completed/abandoned goal runs on a periodic basis (reuse the consolidation.rs idle-time pattern). Each window's VecDeque should be capped at a fixed max_entries (e.g., 200 entries).
**Warning signs:** Memory growth correlating with number of historical goal runs.

### Pitfall 4: Embodied Metadata Dimensions Are Not Independent
**What goes wrong:** Difficulty, familiarity, and trajectory all correlate strongly. A novel approach (low familiarity) that encounters errors (high difficulty) on a diverging trajectory produces three signals that all say the same thing. The confidence computation over-penalizes because it triple-counts the same underlying situation.
**How to avoid:** Use orthogonal weighting: difficulty and familiarity contribute to confidence, but trajectory is used as a rate-of-change signal (is the situation getting better or worse?), not a direct confidence input. Temperature and weight are context modifiers that adjust the threshold, not the confidence score itself.
**Warning signs:** Confidence is consistently LOW for new tasks even when the agent is making progress.

### Pitfall 5: Calibration Cold Start
**What goes wrong:** The calibration feedback loop (UNCR-07) has no historical data when first deployed. Without data points, the calibration tracker cannot compute bias adjustments, making the feature useless until sufficient data accumulates.
**How to avoid:** Pre-populate with conservative defaults: treat all confidence bands as one step more cautious until 50+ observations per band are collected. HIGH is treated as MEDIUM, MEDIUM as LOW, LOW as approval-required. This matches the project-level pitfall analysis (PITFALLS.md Pitfall 4: "pre-populate with conservative defaults").
**Warning signs:** Calibration tracker reports <10 observations per band after 100+ goal runs.

## Code Examples

### Awareness: Three-Tier Sliding Window
```rust
// awareness/tracker.rs
use std::collections::{HashMap, VecDeque};

const MAX_OUTCOMES_PER_WINDOW: usize = 200;
const SHORT_TERM_COUNT: usize = 5;
const MEDIUM_TERM_SECS: u64 = 30 * 60; // 30 minutes

pub struct AwarenessMonitor {
    windows: HashMap<String, OutcomeWindow>,
}

impl AwarenessMonitor {
    pub fn record_outcome(
        &mut self,
        entity_id: &str,
        entity_type: &str,
        tool_name: &str,
        args_hash: &str,
        success: bool,
        is_progress: bool,
        now_ms: u64,
    ) {
        let window = self.windows.entry(entity_id.to_string()).or_insert_with(|| {
            OutcomeWindow::new(entity_id.to_string(), entity_type.to_string())
        });
        window.push(OutcomeEntry {
            timestamp: now_ms,
            tool_name: tool_name.to_string(),
            args_hash: args_hash.to_string(),
            success,
            is_progress,
        });
        window.recompute_rates(now_ms);
    }

    /// Check if entity shows diminishing returns.
    /// Returns Some(reason) if mode shift should be considered.
    pub fn check_diminishing_returns(&self, entity_id: &str) -> Option<String> {
        let window = self.windows.get(entity_id)?;
        // Short-term failure rate above threshold AND no progress recently
        if window.short_term_success_rate < 0.3
            && window.consecutive_same_pattern >= 3
        {
            Some(format!(
                "Diminishing returns: {:.0}% success in last {} actions, same pattern {} times",
                window.short_term_success_rate * 100.0,
                SHORT_TERM_COUNT,
                window.consecutive_same_pattern,
            ))
        } else {
            None
        }
    }
}
```

### Embodied: Familiarity From Episodic Memory
```rust
// embodied/dimensions.rs

/// Compute familiarity (0.0-1.0) from episodic memory match count.
/// 0 matches = 0.0 (novel), 5+ matches = 1.0 (very familiar).
pub fn compute_familiarity(episodic_hit_count: usize) -> f64 {
    let capped = episodic_hit_count.min(5) as f64;
    capped / 5.0
}

/// Compute difficulty (0.0-1.0) from error rate and retry count.
pub fn compute_difficulty(error_rate: f64, retry_count: u32) -> f64 {
    let retry_factor = (retry_count as f64 / 5.0).min(1.0);
    (0.6 * error_rate + 0.4 * retry_factor).clamp(0.0, 1.0)
}

/// Compute trajectory (-1.0 to 1.0) from progress/failure ratio in window.
/// Positive = converging, negative = diverging, 0 = stalled.
pub fn compute_trajectory(progress_count: u32, failure_count: u32) -> f64 {
    let total = progress_count + failure_count;
    if total == 0 {
        return 0.0;
    }
    let ratio = progress_count as f64 / total as f64;
    // Map 0.0-1.0 to -1.0..1.0
    (ratio * 2.0 - 1.0).clamp(-1.0, 1.0)
}
```

### Uncertainty: Structural Confidence Scoring
```rust
// uncertainty/confidence.rs
use crate::agent::explanation::{confidence_band, ConfidenceBand};

pub struct ConfidenceSignals {
    pub tool_success_rate: f64,     // from awareness window
    pub episodic_familiarity: f64,  // from embodied dimensions
    pub blast_radius_score: f64,    // 0.0 = safe, 1.0 = destructive
    pub approach_novelty: f64,      // 1.0 = never tried, 0.0 = proven pattern
}

pub struct ConfidenceAssessment {
    pub band: ConfidenceBand,
    pub evidence: Vec<String>,
    pub domain: DomainClassification,
    pub should_block: bool,
}

/// Compute confidence from structural signals only (UNCR-06: no LLM input).
pub fn compute_step_confidence(
    signals: &ConfidenceSignals,
    domain: DomainClassification,
    thresholds: &DomainThresholds,
) -> ConfidenceAssessment {
    // Weighted combination of structural signals
    let score = 0.30 * signals.tool_success_rate
        + 0.25 * signals.episodic_familiarity
        + 0.25 * (1.0 - signals.blast_radius_score)
        + 0.20 * (1.0 - signals.approach_novelty);

    let band = confidence_band(score);

    let mut evidence = Vec::new();
    if signals.tool_success_rate < 0.5 {
        evidence.push(format!("Low tool success rate: {:.0}%", signals.tool_success_rate * 100.0));
    }
    if signals.episodic_familiarity < 0.3 {
        evidence.push("Unfamiliar pattern (few similar past episodes)".to_string());
    }
    if signals.blast_radius_score > 0.7 {
        evidence.push("High blast radius: action could have wide impact".to_string());
    }

    let threshold = thresholds.threshold_for(domain);
    let should_block = band < threshold;

    ConfidenceAssessment {
        band,
        evidence,
        domain,
        should_block,
    }
}
```

### Plan Step Annotation (UNCR-01 + UNCR-08)
```rust
// In goal_llm.rs, after plan generation
// Annotate each step with confidence label
for step in &mut plan.steps {
    let signals = self.compute_signals_for_step(step, &goal_run).await;
    let domain = classify_step_domain(&step.kind, &step.instructions);
    let assessment = compute_step_confidence(&signals, domain, &config.uncertainty);
    // Prepend confidence label: "[HIGH] Step title"
    step.title = format!("[{}] {}", assessment.band.as_str().to_uppercase(), step.title);
    if !assessment.evidence.is_empty() {
        step.instructions = format!(
            "{}\n\nConfidence note: {}",
            step.instructions,
            assessment.evidence.join("; ")
        );
    }
}

// UNCR-08: Route based on overall plan confidence
let min_band = plan.steps.iter()
    .map(|s| /* extract band from title prefix */)
    .min();
match min_band {
    Some(ConfidenceBand::Confident) => { /* proceed autonomously */ }
    Some(ConfidenceBand::Likely) => { /* inform operator */ }
    _ => { /* require approval */ }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| LLM self-reported confidence | Structural signal-based confidence | 2025 (KDD survey, ICLR 2025) | LLMs are systematically overconfident; structural signals are calibratable |
| Single stuck detector per entity | Multi-tier sliding windows | Phase 2 design | Broader awareness across all agent activity, not just goal runs |
| ConfidenceBand as display-only | ConfidenceBand as decision gate | Phase 2 design | Confidence labels now drive approval routing (UNCR-08) |

**Deprecated/outdated:**
- Vector embeddings for confidence familiarity: FTS5 hit count is sufficient and provider-agnostic (confirmed in Phase 1 research)
- Numeric confidence scores (0.0-1.0) as primary display: Labels (HIGH/MEDIUM/LOW) are more actionable per project decision

## Open Questions

1. **Calibration feedback granularity**
   - What we know: UNCR-07 requires operator corrections to adjust confidence. The existing counter_who correction_patterns provide a mechanism.
   - What's unclear: How many data points per confidence band are needed before calibration adjustments are meaningful? 50 is the research suggestion, but tamux usage patterns may differ.
   - Recommendation: Start with 50 as threshold; expose calibration stats in explainability mode for operator review. Use conservative bias until threshold is met.

2. **Temperature dimension data source**
   - What we know: EMBD-02 requires urgency signals from operator messages.
   - What's unclear: How to reliably detect "urgency" from message text without an LLM call. Message frequency (multiple messages in quick succession) is a structural signal; message content analysis is not.
   - Recommendation: Use structural signals only: message frequency in last 5 minutes + explicit priority flags on tasks/goals. Do not parse message sentiment.

3. **Source authority classification for UNCR-03**
   - What we know: Output confidence should label source authority (official/community/unknown).
   - What's unclear: How to classify sources when tool outputs don't include metadata about their origin. Web search results have URLs; file reads have paths; but LLM reasoning has no source.
   - Recommendation: Only classify sources where structural metadata exists (URLs, file paths). LLM reasoning outputs get "unverified" label by default. This keeps UNCR-03 honest rather than theatrical.

## Project Constraints (from CLAUDE.md)

- **Rust stable channel, edition 2021** -- all new code must compile on stable
- **Local-first** -- no cloud calls for confidence computation
- **Provider-agnostic** -- confidence must not depend on LLM-specific features (logits, token probabilities)
- **Daemon-first architecture** -- all new modules are in-process within AgentEngine, not separate services
- **Backward compatibility** -- new config fields must have `#[serde(default)]` to not break existing configs
- **Naming conventions** -- snake_case modules, PascalCase types, pub(super) for module internals, pub(crate) for cross-module APIs
- **Error handling** -- use `anyhow` for error propagation, `tracing` for logging
- **Type patterns** -- `#[derive(Debug, Clone, Serialize, Deserialize)]` for wire types; `#[serde(rename_all = "snake_case")]` for enums

## Integration Map

### Where New Code Hooks Into Existing Code

| Integration Point | Existing Code | New Code | How |
|-------------------|---------------|----------|-----|
| Tool result tracking | agent_loop.rs L1138 (update_counter_who) | awareness.record_outcome() | Call immediately after counter_who update |
| Goal plan generation | goal_llm.rs L6 (request_goal_plan) | uncertainty.annotate_plan_steps() | Call after plan generated, before returning |
| Plan approval routing | goal_planner.rs (plan execution) | uncertainty.plan_confidence_gate() | Check min confidence band, route to approval if needed |
| Heartbeat integration | heartbeat.rs (run_structured_heartbeat) | awareness.trajectory_summary() | Include trajectory in heartbeat check results |
| System prompt injection | system_prompt.rs L16-17 (episodic/constraints params) | awareness.trajectory_context() | New optional parameter for trajectory status |
| AgentEngine construction | engine.rs L132 (episodic_store field) | awareness + uncertainty_config fields | Add 2 new RwLock fields |
| AgentEvent enum | types.rs L2057 (AgentEvent) | TrajectoryUpdate, ModeShift, ConfidenceWarning | Add 3 new variants |
| AgentConfig | types.rs L1248 (episodic field) | uncertainty: UncertaintyConfig | Add new config field with #[serde(default)] |
| Blast radius | causal_traces.rs L461 | uncertainty.blast_radius_to_score() | Convert existing advisory to 0.0-1.0 score |
| Episodic familiarity | episodic/retrieval.rs (retrieve_relevant_episodes) | embodied.compute_familiarity(hit_count) | Count episodes matching current step context |

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `liveness/stuck_detection.rs` -- DetectionSnapshot, StuckDetector, has_tool_call_loop (357 lines, 19 tests)
- Codebase analysis: `metacognitive/self_assessment.rs` -- SelfAssessor, ProgressMetrics, EfficiencyMetrics, compute_momentum (449 lines, 13 tests)
- Codebase analysis: `metacognitive/escalation.rs` -- EscalationLevel, EscalationState, EscalationCriteria (825 lines, 24 tests)
- Codebase analysis: `explanation.rs` -- ConfidenceBand enum, confidence_band(), format_confidence_text() (428 lines, 16 tests)
- Codebase analysis: `episodic/counter_who.rs` -- compute_approach_hash, detect_repeated_approaches, record_correction (500 lines, 10 tests)
- Codebase analysis: `causal_traces.rs` -- command_blast_radius_advisory, CommandBlastRadiusAdvisory
- Codebase analysis: `engine.rs` -- AgentEngine struct with 30+ fields, RwLock/Mutex pattern
- Codebase analysis: `agent_loop.rs` L1138 -- counter_who integration point after tool execution
- Codebase analysis: `goal_llm.rs` L6-55 -- episodic context + negative constraints injection into planning prompt

### Secondary (MEDIUM confidence)
- Project research: PITFALLS.md Pitfall 4 (LLM confidence miscalibration) and Pitfall 6 (false positive mode shifts) -- backed by academic literature
- Project research: ARCHITECTURE.md awareness monitor design (AwarenessMonitor struct, OutcomeWindow, TrajectoryState)
- [KDD 2025 Survey: Uncertainty Quantification and Confidence Calibration in LLMs](https://arxiv.org/abs/2503.15850)
- [ICLR 2025: Do LLMs Estimate Uncertainty Well](https://proceedings.iclr.cc/paper_files/paper/2025/file/ef472869c217bf693f2d9bbde66a6b07-Paper-Conference.pdf)
- [Cursor Forum: False Positive Loop Detection](https://forum.cursor.com/t/false-positive-loop-detection-when-using-custom-model-qwen3-coder-plus-with-repetitive-reasoning-text-before-different-tool-calls/145252)

### Tertiary (LOW confidence)
- Calibration cold-start threshold of 50 observations: reasonable heuristic from statistics but not validated for agent-specific workloads

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- zero new dependencies; all primitives already exist in codebase
- Architecture: HIGH -- follows established extension module pattern with 8 proven examples; 10+ integration points mapped to exact lines
- Pitfalls: HIGH -- 5 pitfalls identified, all backed by project-level research or codebase analysis

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable -- no external dependencies that could change)
