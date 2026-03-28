# Phase 4: Operator Control and Transparency - Research

**Researched:** 2026-03-27
**Domain:** Cost tracking, autonomy controls, explainability, shared authorship, deferred confidence features -- all Rust daemon extensions
**Confidence:** HIGH

## Summary

Phase 4 implements five distinct operator-facing subsystems in the tamux daemon: (1) per-goal cost and token accounting with budget alerts, (2) a three-level autonomy dial controlling goal run reporting granularity, (3) on-demand explainability queries backed by episodic memory and causal traces, (4) shared authorship metadata on goal outputs, and (5) two deferred uncertainty features from Phase 2 (UNCR-02 pre-tool confidence warnings for Safety domain, UNCR-03 source authority labeling for web results).

The existing infrastructure provides nearly everything needed. Token counts (prompt + completion) are already tracked per-message on `AgentMessage.input_tokens`/`output_tokens` and aggregated on `AgentThread.total_input_tokens`/`total_output_tokens`. The `AgentEvent::Done` variant already carries `input_tokens`, `output_tokens`, `cost`, `provider`, and `model` fields. The `GoalRun` struct already has `events` and metadata fields that can carry cost summaries. The existing `CausalTrace` type in `learning/traces.rs` already stores `DecisionType`, `selected`, `rejected_options`, and `causal_factors` -- exactly what EXPL-01/EXPL-03 need. The `ConfidenceWarning` AgentEvent variant already supports `action_type: "tool_call"` for UNCR-02. The `DomainClassification::Safety` and `classify_domain()` function from Phase 2 already identify which tools need pre-execution warnings.

This is a "wiring and extension" phase, not a new subsystem phase. The primary work is: (a) adding accumulation and persistence for token/cost data at the goal run level, (b) adding an autonomy_level field to GoalRun with milestone-gated event filtering, (c) building a query path from episodic store + causal traces to structured explainability output, (d) tagging goal outputs with authorship source, and (e) inserting confidence checks and source labels at the right points in the tool execution and agent loop hot paths.

**Primary recommendation:** Implement as four plans -- cost accounting (COST-01 through COST-04), autonomy dial + authorship (AUTO-01 through AUTO-04, AUTH-01, AUTH-02), explainability (EXPL-01 through EXPL-03), and deferred confidence (UNCR-02, UNCR-03). The subsystems are independent and do not depend on each other.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Per-goal token counts (prompt + completion) tracked on every LLM API call
- Per-session and cumulative cost estimates using provider rate cards
- Cost displayed in goal completion reports (not real-time streaming)
- Budget alerts: configurable threshold in agent config, notification only (no auto-stop)
- Cost data persisted in goal_run metadata, queryable via CLI/observability
- Per-goal autonomy level setting: autonomous / aware / supervised
- Autonomous: agent proceeds, operator sees final report only
- Aware: agent reports on milestones (sub-task completions, handoffs)
- Supervised: agent reports on every significant step and waits for acknowledgment
- Default level: aware (balanced between autonomy and visibility)
- "Why did you do that?" query searches episodic store + causal traces for the referenced action
- Returns structured answer: decision point, alternatives considered, reasons for chosen approach
- Rejected alternatives stored alongside chosen plan in goal_run metadata during planning
- Uses existing episodic retrieval + negative knowledge for "what was tried and ruled out"
- Metadata tags on goal outputs: operator (from operator input), agent (from agent synthesis), joint (collaborative)
- Attribution is metadata, not inline commentary -- doesn't disrupt reading flow
- Tracked at goal output level, not per-message level
- UNCR-02: Pre-tool ConfidenceWarning for Safety-domain tools only (not all tools)
- UNCR-03: URL-based source authority classification (official/community/unknown) on web_search/web_read results

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions.

### Deferred Ideas (OUT OF SCOPE)
- Real-time cost streaming to UI -- v3.1
- Auto-budget enforcement (stop on threshold) -- v3.1
- Per-message attribution granularity -- v3.1
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AUTH-01 | Significant outputs attribute contributions: what came from operator input, what from agent synthesis, what's joint | Existing GoalRun has `events` and metadata fields; authorship tags added to goal_run completion payload |
| AUTH-02 | Attribution is metadata on the output, not inline commentary | Implemented as serde fields on GoalRun output struct, not embedded in text content |
| COST-01 | Per-goal token counts tracked on every LLM API call | AgentMessage already has input_tokens/output_tokens; accumulate into GoalRun-level counters |
| COST-02 | Per-session and cumulative cost estimates using provider rate cards | New rate card lookup table; compute from tracked tokens x rate per model |
| COST-03 | Budget alerts when spending exceeds operator-defined threshold | New config field + Notification event when cumulative cost crosses threshold |
| COST-04 | Cost data persisted in goal_run metadata, queryable | Extend GoalRun struct with cost fields; persisted via existing persist_goal_runs |
| AUTO-01 | Per-goal autonomy level setting: autonomous / aware / supervised | New AutonomyLevel enum on GoalRun; passed via AgentStartGoalRun IPC extension |
| AUTO-02 | Autonomous: agent proceeds, operator sees final report only | Event filtering in emit_goal_run_update based on autonomy_level |
| AUTO-03 | Aware: agent reports on milestones | Existing milestone events already emitted; aware is the current behavior |
| AUTO-04 | Supervised: agent reports on every significant step and waits for acknowledgment | Insert acknowledgment gates at step boundaries in goal_planner |
| EXPL-01 | On-demand reasoning log: why agent chose Plan A over Plan B | Extend CausalTrace recording during plan generation; query via IPC |
| EXPL-02 | "Why did you do that?" query returns causal trace for any past action | New IPC message + handler that searches episodic store + causal traces |
| EXPL-03 | Rejected alternatives stored alongside chosen plan in goal_run metadata | Store CausalTrace during request_goal_plan; persist in GoalRun |
| UNCR-02 | Tool-call confidence: pre-execution warnings with blast-radius for Safety tools | Insert confidence check before Safety-domain tool execution in tool_executor |
| UNCR-03 | Output confidence: research results labeled by source authority | URL-based classification in web_search/web_read result formatting |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.32 (bundled) | Cost data persistence in SQLite | Already in use for all daemon persistence |
| serde/serde_json | 1.x | Serialization for new config fields and cost structs | Already used throughout |
| tokio broadcast | 1.x | Event delivery for budget alerts and autonomy notifications | Already used for AgentEvent fanout |
| anyhow | 1.x | Error handling | Already the primary error crate |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| url (crate) | -- | URL domain parsing for UNCR-03 source authority | Not needed -- `str::contains` on domain patterns is sufficient for official/community/unknown classification |
| regex | 1.x | Pattern matching for source authority URL classification | Already in workspace deps |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static rate card table | External pricing API | Static is correct -- provider-agnostic means no API dependency; operator can update rates in config |
| Per-message cost tracking | Per-goal only | Per-message already exists via input_tokens/output_tokens on AgentMessage; goal-level aggregation is the new work |

**No new crate dependencies required.** All features build on existing rusqlite, serde, tokio, and the agent event infrastructure.

## Architecture Patterns

### Recommended Module Structure
```
crates/amux-daemon/src/agent/
  cost/
    mod.rs              # CostTracker, CostConfig, rate card types
    rate_cards.rs       # Provider rate card lookup (static table + config override)
  autonomy.rs           # AutonomyLevel enum, event filtering logic
  explainability.rs     # Explain query handler, trace assembly
  authorship.rs         # AuthorshipTag enum, GoalRun output tagging
```

### Pattern 1: Cost Accumulation via Existing Token Tracking
**What:** Accumulate per-goal costs by intercepting the existing token tracking in agent_loop.rs where `thread.total_input_tokens += input_tokens` already happens, and additionally updating a goal-run-level cost counter.
**When to use:** Every LLM API call within a goal run context.
**Key insight:** The `AgentEvent::Done` variant already carries `input_tokens`, `output_tokens`, `cost`, `provider`, and `model`. The agent_loop already accumulates tokens on the thread. The new work is (a) accumulating on the GoalRun and (b) checking budget thresholds after each accumulation.

```rust
// In agent_loop.rs, after existing token accumulation:
if let Some(goal_run_id) = active_goal_run_id.as_deref() {
    self.accumulate_goal_run_cost(
        goal_run_id,
        input_tokens.unwrap_or(0),
        output_tokens.unwrap_or(0),
        &config.provider,
        &provider_config.model,
    ).await;
}
```

### Pattern 2: Autonomy Level as GoalRun Field with Event Gate
**What:** Add `autonomy_level: AutonomyLevel` to GoalRun. Filter event emissions through an autonomy gate that checks whether the event type is allowed at the current autonomy level.
**When to use:** Every call to `emit_goal_run_update` and step-level event emissions.
**Key insight:** The `aware` level is the current default behavior -- milestones are already reported. `autonomous` suppresses intermediate events. `supervised` adds acknowledgment gates (using existing `AwaitingApproval` status).

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    Autonomous,
    Aware,
    Supervised,
}

impl Default for AutonomyLevel {
    fn default() -> Self { Self::Aware }
}

/// Check whether a given event type should be emitted at this autonomy level.
fn should_emit_event(level: AutonomyLevel, event_kind: &str) -> bool {
    match level {
        AutonomyLevel::Autonomous => matches!(event_kind, "completed" | "failed"),
        AutonomyLevel::Aware => !matches!(event_kind, "step_detail"),
        AutonomyLevel::Supervised => true,
    }
}
```

### Pattern 3: Explainability via Existing CausalTrace + Episodic Retrieval
**What:** For EXPL-01/02/03, combine existing `CausalTrace` recording (already used for skill selection) with episodic retrieval to answer "why did you do that?" queries.
**When to use:** During goal plan generation (store rejected alternatives) and on-demand via IPC query.
**Key insight:** `CausalTrace` already has `DecisionType::PlanSelection`, `selected`, `rejected_options`, and `causal_factors`. The plan generation in `request_goal_plan` already calls the LLM for structured JSON. Extending the planning prompt to also return rejected alternatives is the minimal change. The "why" query handler then looks up: (1) causal traces for the goal_run_id, (2) episodic context from Phase 1, (3) negative constraints from Phase 1.

### Pattern 4: Source Authority Classification (UNCR-03)
**What:** Classify web search/read result URLs into official/community/unknown based on domain patterns.
**When to use:** After web_search and web_read tool execution, before returning results to agent.
**Key insight:** A static list of official documentation domains (docs.*, github.com/*/docs, developer.*, official SDK repos) covers the 80% case. Community sources (stackoverflow, reddit, medium, dev.to) are explicitly tagged. Everything else is "unknown." The label is prepended to each result in the tool output.

```rust
fn classify_source_authority(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.contains("docs.") || lower.contains("/docs/")
        || lower.contains("developer.") || lower.contains(".readthedocs.")
        || lower.contains("man7.org") || lower.contains("cppreference.com")
    {
        "official"
    } else if lower.contains("stackoverflow.com") || lower.contains("reddit.com")
        || lower.contains("medium.com") || lower.contains("dev.to")
        || lower.contains("blog.") || lower.contains("forum.")
    {
        "community"
    } else {
        "unknown"
    }
}
```

### Anti-Patterns to Avoid
- **Tracking cost per-message instead of per-goal:** The locked decision says cost is displayed in goal completion reports. Per-message cost exists via input_tokens/output_tokens already. Don't build a second per-message accounting layer.
- **Real-time cost streaming to UI:** Explicitly deferred to v3.1. Don't build WebSocket or polling infrastructure for live cost updates.
- **Auto-stopping on budget exceeded:** Explicitly deferred. Budget alerts are notification-only.
- **Inline attribution text:** AUTH-02 explicitly requires metadata, not inline commentary. Don't insert "Agent: " or "Operator: " prefixes into output text.
- **LLM-based source authority classification:** UNCR-03 should use URL pattern matching, not an LLM call. Adding LLM latency to every web search result is unacceptable.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Token-to-cost conversion | Custom pricing API client | Static rate card table in config | Provider-agnostic means no external API; operator provides rates |
| Event filtering for autonomy | Custom pub/sub with filters | Conditional check before existing `event_tx.send()` | Already have broadcast channel; just gate the send |
| Causal trace storage | New database table | Existing `append_telemetry` WORM ledger + GoalRun metadata field | Follows established pattern, queryable |
| Source authority classification | ML classifier or embedding-based | Static URL domain pattern matching | Deterministic, zero-latency, covers 80% of cases |

**Key insight:** This phase has zero new architectural abstractions. Every feature wires into existing infrastructure.

## Common Pitfalls

### Pitfall 1: Rate Card Staleness
**What goes wrong:** Provider pricing changes faster than operator updates their config. Cost estimates silently become inaccurate.
**Why it happens:** Static rate cards require manual maintenance.
**How to avoid:** (1) Ship reasonable defaults for popular models (gpt-4o, claude-3.5-sonnet, etc.) that are clearly dated. (2) Log a warning if no rate card exists for the model being used. (3) Show "estimated" label on costs, never "actual." (4) Make rate card config easy to update.
**Warning signs:** Cost estimates that seem unreasonably low or high.

### Pitfall 2: Double-Counting Tokens in Goal Runs
**What goes wrong:** Token counts accumulated both in agent_loop per-turn tracking AND in goal_planner step completion, resulting in inflated costs.
**Why it happens:** Multiple code paths touch token accounting: agent_loop.rs (per-LLM-call), goal_planner.rs (step completion), reflection (LLM call for reflection summary).
**How to avoid:** Accumulate cost at exactly ONE point: in agent_loop.rs after each `CompletionChunk::Done`. Every other code path reads the accumulated value, never adds to it independently.
**Warning signs:** Cost for a 3-step goal run that seems 2x higher than the sum of individual LLM calls.

### Pitfall 3: Supervised Mode Creating Deadlocks
**What goes wrong:** In supervised mode (AUTO-04), the agent waits for operator acknowledgment on every step. If the operator is away, all goal runs stall indefinitely.
**Why it happens:** No timeout or fallback for acknowledgment gates.
**How to avoid:** (1) Use the existing `AwaitingApproval` status which already has timeout handling via heartbeat checks. (2) Emit a clear notification that says "waiting for your acknowledgment" with the step details. (3) Consider a config option for acknowledgment timeout (auto-proceed after N minutes).
**Warning signs:** Goal runs stuck in AwaitingApproval with no operator interaction.

### Pitfall 4: Explainability Queries Returning Stale or Empty Traces
**What goes wrong:** "Why did you do that?" returns "no explanation available" because causal traces weren't recorded for the action in question.
**Why it happens:** Not all decision points currently record causal traces. Only skill selection in `causal_traces.rs` currently persists traces.
**How to avoid:** (1) Record causal traces during plan generation (the most important decision point). (2) Gracefully degrade: if no causal trace exists, fall back to episodic retrieval + negative knowledge for the goal topic. (3) Always return SOMETHING -- even "this action was taken based on the goal plan step instructions, no alternatives were considered" is better than empty.
**Warning signs:** Frequent "no trace available" responses.

### Pitfall 5: Autonomy Level Not Propagated to Subagents
**What goes wrong:** Parent goal run is set to "supervised" but handoff subagent tasks run in default "aware" mode.
**Why it happens:** The handoff broker creates new tasks without inheriting the parent goal run's autonomy level.
**How to avoid:** When broker.rs routes a handoff, copy the parent goal run's autonomy_level to the child context. For supervised mode, this means the child task also needs acknowledgment gates.
**Warning signs:** Subagent tasks completing silently when the parent goal was supposed to be supervised.

## Code Examples

### Cost Tracking: New Fields on GoalRun
```rust
// Extension to GoalRun struct in types.rs
pub struct GoalRun {
    // ... existing fields ...

    /// Accumulated prompt tokens across all LLM calls in this goal run.
    #[serde(default)]
    pub total_prompt_tokens: u64,
    /// Accumulated completion tokens across all LLM calls in this goal run.
    #[serde(default)]
    pub total_completion_tokens: u64,
    /// Estimated cost in USD based on provider rate cards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
}
```

### Cost Config: Rate Cards in Agent Config
```rust
// New config section on AgentConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConfig {
    /// Enable cost tracking for goal runs.
    #[serde(default = "default_cost_enabled")]
    pub enabled: bool,
    /// Budget alert threshold in USD. Notification fires when cumulative cost exceeds this.
    #[serde(default)]
    pub budget_alert_threshold_usd: Option<f64>,
    /// Provider rate cards: map of "provider/model" -> RateCard.
    #[serde(default)]
    pub rate_cards: HashMap<String, RateCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateCard {
    /// Cost per 1M input/prompt tokens in USD.
    pub input_per_million: f64,
    /// Cost per 1M output/completion tokens in USD.
    pub output_per_million: f64,
}
```

### Autonomy Level: IPC Extension
```rust
// Extend AgentStartGoalRun in amux-protocol messages.rs
AgentStartGoalRun {
    goal: String,
    title: Option<String>,
    thread_id: Option<String>,
    session_id: Option<String>,
    priority: Option<String>,
    client_request_id: Option<String>,
    // NEW: autonomy level for this goal run
    #[serde(default)]
    autonomy_level: Option<String>,  // "autonomous" | "aware" | "supervised"
},
```

### Explainability: IPC Query
```rust
// New ClientMessage variant
AgentExplainAction {
    /// The goal_run_id or action_id to explain.
    action_id: String,
    /// Optional: narrow explanation to a specific step.
    step_index: Option<usize>,
},

// New DaemonMessage response
AgentExplanation {
    action_id: String,
    decision_point: String,
    chosen_approach: String,
    alternatives_considered: Vec<String>,
    reasons: Vec<String>,
    source: String,  // "causal_trace", "episodic", "negative_knowledge", "fallback"
},
```

### Source Authority Label (UNCR-03)
```rust
// In web_search result formatting, prepend authority label:
fn format_search_result_with_authority(title: &str, url: &str, snippet: &str) -> String {
    let authority = classify_source_authority(url);
    format!("- [{}] **{}**\n  {}\n  {}", authority, title, url, snippet)
}
```

### Pre-Tool Confidence Warning (UNCR-02)
```rust
// In tool_executor.rs, before executing Safety-domain tools:
let domain = super::uncertainty::domains::classify_domain(tool_name);
if domain == DomainClassification::Safety {
    let confidence = self.compute_tool_call_confidence(tool_name, &args).await;
    if confidence.should_warn {
        let _ = self.event_tx.send(AgentEvent::ConfidenceWarning {
            thread_id: thread_id.to_string(),
            action_type: "tool_call".to_string(),
            band: confidence.band.as_str().to_string(),
            evidence: confidence.evidence.join("; "),
            domain: "safety".to_string(),
            blocked: confidence.blocked,
        });
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No token tracking | Per-message input/output tokens on AgentMessage | Already in codebase | Foundation for cost tracking -- just need aggregation |
| No causal traces | CausalTrace type with DecisionType, selected, rejected | Phase 6+ (skill evolution) | Foundation for explainability -- just need plan-level traces |
| Single approval mode | AwaitingApproval status on GoalRun | Already in codebase | Foundation for supervised mode -- just need autonomy gate |
| No confidence on tool calls | ConfidenceWarning event variant + domains.rs classification | Phase 2 (v3.0) | Foundation for UNCR-02 -- just need pre-execution check |

## Open Questions

1. **Rate card defaults: which models to ship?**
   - What we know: The system is provider-agnostic with OpenAI-compatible and Anthropic-compatible providers.
   - What's unclear: Which specific models are commonly used by tamux operators. Including defaults for models that nobody uses is noise.
   - Recommendation: Ship defaults for the top 5-6 most common models (gpt-4o, gpt-4o-mini, claude-3.5-sonnet, claude-3-haiku, claude-3-opus, o1-mini). Log a warning for unknown models. Allow full override in config.

2. **Acknowledgment mechanism for supervised mode**
   - What we know: The existing `AwaitingApproval` status with approval flow exists. It's used for policy-gated commands.
   - What's unclear: Whether to reuse the same approval mechanism or create a lighter "acknowledge" variant that doesn't require a full approval payload.
   - Recommendation: Reuse existing AwaitingApproval. It already has the right status transitions and heartbeat monitoring. Add a new `approval_type` discriminator to distinguish "policy_approval" from "autonomy_acknowledgment."

3. **Causal trace storage location**
   - What we know: Causal traces are already defined in learning/traces.rs. The WORM ledger and goal_run metadata field are both available.
   - What's unclear: Whether to store plan-level causal traces in the WORM telemetry ledger (like handoff audits) or directly in the GoalRun struct (for easy queryability).
   - Recommendation: Store in both: WORM for audit trail integrity, and a summary in GoalRun metadata for fast IPC queries. This follows the existing pattern where handoff audits go to both WORM and handoff_log table.

## Project Constraints (from CLAUDE.md)

- **Rust daemon + TypeScript/React frontend** -- all Phase 4 work is Rust daemon code; no frontend changes
- **Local-first** -- cost data stays on operator machine; no external pricing API calls
- **Provider-agnostic** -- rate cards must work for any OpenAI-compatible or Anthropic-compatible provider
- **Backward compatibility** -- all new fields on GoalRun use `#[serde(default)]` to not break existing serialized data
- **GSD Workflow** -- all work executed through GSD commands
- **Rust conventions** -- `snake_case` modules, `PascalCase` types, `anyhow` error handling, `#[derive(Debug, Clone, Serialize, Deserialize)]` on wire types
- **Config via DB/IPC** -- per memory note `feedback_daemon_config_source.md`, daemon reads config from DB, NOT config.json. New cost/autonomy config fields must go through IPC write path.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `agent_loop.rs` -- existing token tracking at L905-915
- Codebase analysis: `types.rs` -- AgentEvent::Done at L2083-2096, GoalRun at L2914-2967, AgentMessage at L2365-2396
- Codebase analysis: `learning/traces.rs` -- CausalTrace at L42-56, DecisionType at L60-67
- Codebase analysis: `explanation.rs` -- ConfidenceBand at L22-27, ExplanationResult at L78-84
- Codebase analysis: `uncertainty/domains.rs` -- DomainClassification at L17-26, classify_domain at L29-37
- Codebase analysis: `goal_llm.rs` -- request_goal_plan at L6-99, annotate_plan_steps at L106-190
- Codebase analysis: `goal_planner.rs` -- complete_goal_run at L588-679
- Codebase analysis: `tool_executor.rs` -- web_search at L1121-1142, execute_web_search at L2724-2750
- Codebase analysis: `engine.rs` -- event_tx broadcast at L83, L163
- Codebase analysis: `amux-protocol/messages.rs` -- AgentStartGoalRun at L261-268
- Phase 1 Verification: All episodic memory infrastructure confirmed working (18/18 truths)
- Phase 2 Verification: All confidence/uncertainty infrastructure confirmed working (20/20 truths)
- Phase 3 Verification: All handoff/audit infrastructure confirmed working (12/12 truths)

### Secondary (MEDIUM confidence)
- Memory note `feedback_daemon_config_source.md` -- daemon reads config from DB, not config.json

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- zero new dependencies, all features use existing crates
- Architecture: HIGH -- all integration points verified in codebase, established patterns followed
- Pitfalls: HIGH -- all pitfalls derived from actual codebase analysis (double-counting, deadlock patterns, trace gaps)

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable -- this is internal codebase extension work)
