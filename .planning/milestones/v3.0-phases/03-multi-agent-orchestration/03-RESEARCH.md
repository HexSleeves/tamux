# Phase 3: Multi-Agent Orchestration - Research

**Researched:** 2026-03-27
**Domain:** Multi-agent handoff orchestration, specialist profiles, context bundling, divergent subagents
**Confidence:** HIGH

## Summary

Phase 3 builds a HandoffBroker that layers on top of the existing `spawn_subagent()` primitive, adding capability-based specialist matching, structured context bundles, output validation, escalation chains, WORM audit trails, and a divergent subagent mode. The codebase already has substantial multi-agent infrastructure: `spawn_subagent()` tool, `SubAgentDefinition` with provider/model overrides, `CollaborationSession` with `broadcast_contribution()`/`vote_on_disagreement()`, `SubagentLifecycle` state machine, `ContextBudget` enforcement, `ToolFilter` for per-agent tool restrictions, `SupervisorAction` health monitoring, and `TerminationCondition` DSL. The handoff broker wraps and extends this infrastructure rather than replacing it.

The key architectural insight is that most of the mechanical plumbing already exists. What Phase 3 adds is the **intelligence layer on top**: deciding *which* specialist to route to (based on capability tags and confidence signals from Phase 2), *what context* to send (assembling bundles from Phase 1 episodic refs + negative constraints + partial outputs), *when to escalate* (structured triggers tied to the existing escalation framework), and *how to validate output* (acceptance criteria per handoff). The divergent subagent mode extends the existing `CollaborationSession` by spawning 2-3 framings of the same problem and using the existing disagreement detection + voting mechanism to surface tensions.

**Primary recommendation:** Build the handoff broker as a new `handoff/` module under `crates/amux-daemon/src/agent/` following the AgentEngine extension pattern. Start with specialist profiles and capability matching, then context bundle assembly, then escalation chains, then output validation, and finally divergent mode. The existing collaboration infrastructure handles 60-70% of the divergent subagent requirements.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- HandoffBroker layers on existing `spawn_subagent` primitive -- not a separate orchestration engine
- 5 default specialist profiles ship out of the box: researcher, backend-developer, frontend-developer, reviewer, generalist
- Proficiency levels: expert, advanced, competent, familiar -- used for capability matching
- Context bundles carry typed references (memory refs, episodic refs, document refs, partial outputs) with strict 2000-token ceiling per bundle
- Context bundles are summarized, not forwarded raw -- prevents exponential growth through handoff chains
- Handoff depth limit: max 3 hops, then escalate to operator with full chain context
- Structured triggers: ConfidenceBelow(band), ToolFails(count), TimeExceeds(duration)
- Structured actions: HandBack, RetryWithNewContext, EscalateTo(profile), AbortWithReport
- Escalation chains are configurable per specialist profile
- Orchestrator validates specialist output against acceptance criteria before accepting
- Acceptance criteria defined per handoff task (not per profile)
- 2-3 parallel framings work the same problem simultaneously
- Disagreement between framings is surfaced as the valuable output (tensions, not forced consensus)
- Mediator synthesizes tensions into a recommendation that acknowledges tradeoffs
- Divergent mode is a handoff mode, not a separate system
- Every handoff logged to WORM audit trail (from, to, task, outcome, duration, confidence, audit_hash)
- Uses existing WORM ledger infrastructure from Phase 1

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions and are at Claude's discretion.

### Deferred Ideas (OUT OF SCOPE)
- Async handoff task queue (specialist doesn't need to be online) -- v3.1
- Custom specialist profiles defined by operator -- v3.1
- Shared semantic memory layer between specialists -- v3.1
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HAND-01 | HandoffBroker matches tasks to specialist profiles by capability tags (proficiency levels: expert, advanced, competent, familiar) | Specialist profiles with capability_tags + proficiency_level fields; matching algorithm scores tag overlap weighted by proficiency |
| HAND-02 | Context bundles carry typed references: memory refs, episodic refs, document refs, partial outputs | ContextBundle struct with typed fields; leverages Phase 1 `retrieve_relevant_episodes()` and `query_active_constraints()` |
| HAND-03 | Context bundles are summarized with strict token ceiling (not forwarded raw) to prevent exponential growth | 2000-token ceiling enforced at bundle assembly; `summarize_text()` helper already exists in codebase |
| HAND-04 | Escalation chains with structured triggers (ConfidenceBelow, ToolFails, TimeExceeds) and actions (HandBack, RetryWithNewContext, EscalateTo, AbortWithReport) | EscalationTrigger + EscalationAction enums; configurable per specialist profile; integrates with existing `metacognitive/escalation.rs` |
| HAND-05 | Orchestrator validates specialist output against acceptance criteria before accepting | AcceptanceCriteria struct with validation method; checked after subagent completion in task lifecycle |
| HAND-06 | Every handoff logged to WORM audit trail (from, to, task, outcome, duration, confidence, audit_hash) | New "handoff" WORM ledger kind using existing `record_telemetry_event` pattern from Phase 1 |
| HAND-07 | Default specialist profiles ship out of the box (researcher, backend-developer, frontend-developer, reviewer, generalist) | 5 built-in profiles with capability tags, tool filters, and system prompt snippets; loaded at init_schema time |
| HAND-08 | Handoff depth limit (max 3 hops) to prevent handoff loops, then escalate to operator | Depth counter on ContextBundle incremented at each hop; broker refuses handoff at depth >= 3 |
| HAND-09 | HandoffBroker layers on existing spawn_subagent primitive (not a separate orchestration engine) | Broker calls into existing `enqueue_task()` with source="subagent" + handoff metadata; no new orchestration loop |
| DIVR-01 | Parallel interpretation mode where multiple framings work the same problem simultaneously | DivergentSession spawns 2-3 subagents via existing `spawn_subagent()` with different system prompt framings |
| DIVR-02 | Disagreement between framings is surfaced as the valuable output (tensions, not consensus) | Extends existing `detect_disagreements()` from collaboration.rs; formats tensions with evidence per position |
| DIVR-03 | Mediator synthesizes tensions into a recommendation that acknowledges tradeoffs | Mediator uses LLM call to synthesize from structured disagreement data; produces recommendation with explicit tradeoff acknowledgment |
</phase_requirements>

## Standard Stack

### Core

No new crate dependencies required. Phase 3 builds entirely on existing infrastructure.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.x | Async runtime for parallel subagent spawning | Already in workspace |
| serde + serde_json | 1.x | Handoff log serialization, context bundle JSON | Already in workspace |
| rusqlite | 0.32 | Specialist profiles + handoff_log persistence | Already in workspace (bundled) |
| uuid | 1.x | Handoff IDs, specialist profile IDs | Already in workspace |
| sha2 | 0.10 | WORM audit hash chain for handoff ledger | Already in workspace |

### Supporting (Existing Infrastructure Used)

| Module | Location | Purpose | How Phase 3 Uses It |
|--------|----------|---------|---------------------|
| `spawn_subagent()` | tool_executor.rs L4275 | Subagent spawning primitive | HandoffBroker wraps this with specialist context injection |
| `CollaborationSession` | collaboration.rs | Multi-agent coordination | Divergent mode extends with structured framing metadata |
| `broadcast_contribution()` | collaboration.rs L138 | Inter-agent communication | Divergent framings publish positions through this |
| `vote_on_disagreement()` | collaboration.rs L225 | Disagreement resolution | Divergent mode uses for structural voting |
| `detect_disagreements()` | collaboration.rs L364 | Automatic disagreement detection | Extended for divergent tension surfacing |
| `SubAgentDefinition` | types.rs L1843 | Specialist config with provider/model overrides | Specialist profiles extend this pattern |
| `ContextBudget` | subagent/context_budget.rs | Token budget enforcement | Context bundle ceiling uses same tracking |
| `ToolFilter` | subagent/tool_filter.rs | Per-agent tool restrictions | Specialist profiles carry tool filters |
| `SubagentLifecycle` | subagent/lifecycle.rs | State machine (Queued->Running->Completed) | Handoff tracking uses lifecycle transitions |
| `SupervisorAction` | subagent/supervisor.rs | Health monitoring | Handoff specialist health monitoring |
| `TerminationCondition` | subagent/termination.rs | Auto-stop DSL | TimeExceeds trigger maps to timeout condition |
| `EscalationLevel` | metacognitive/escalation.rs | L0-L3 escalation levels | Handoff escalation integrates at L1 (SubAgent) |
| `ConfidenceBand` | explanation.rs | Confidence classification | ConfidenceBelow trigger uses this |
| `retrieve_relevant_episodes()` | episodic/retrieval.rs | Episode retrieval | Context bundle assembly pulls episodic refs |
| `query_active_constraints()` | episodic/negative_knowledge.rs | Negative knowledge | Context bundle includes constraint refs |
| `AwarenessMonitor` | awareness/mod.rs | Trajectory tracking | Handoff decisions factor in trajectory signals |
| `compute_step_confidence()` | uncertainty/confidence.rs | Structural confidence scoring | ConfidenceBelow trigger threshold evaluation |
| `record_telemetry_event()` | history.rs L3104 | WORM ledger append | Handoff audit trail appends here |
| `summarize_text()` | agent mod.rs | Text compression | Context bundle summarization |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| In-process broker | External orchestration service | Violates daemon-first architecture, adds IPC latency |
| Static specialist profiles | LLM-inferred profiles at runtime | Non-deterministic routing, harder to debug |
| Token-counted bundle ceiling | Character-counted ceiling | Token counting is more accurate but requires tiktoken-rs dependency; chars/4 approximation is sufficient and free |

## Architecture Patterns

### Recommended Module Structure

```
crates/amux-daemon/src/agent/handoff/
    mod.rs              -- public API, HandoffBroker struct, re-exports
    broker.rs           -- match_specialist(), route_handoff(), core logic
    profiles.rs         -- SpecialistProfile type, default profiles, capability matching
    context_bundle.rs   -- ContextBundle assembly, summarization, token ceiling
    escalation.rs       -- HandoffEscalationTrigger, HandoffEscalationAction, chain evaluation
    acceptance.rs       -- AcceptanceCriteria, validate_output()
    audit.rs            -- WORM handoff ledger, handoff_log SQLite persistence
    divergent.rs        -- DivergentSession, spawn parallel framings, mediator synthesis
    schema.rs           -- SQLite table definitions (specialist_profiles, handoff_log)
```

### Pattern 1: HandoffBroker as AgentEngine Extension

**What:** HandoffBroker state lives as an `RwLock<HandoffBroker>` field on AgentEngine. Behavior is implemented via `impl AgentEngine` blocks in handoff module files. This follows the exact pattern of episodic_store, awareness, and calibration_tracker from Phases 1-2.

**When to use:** Always -- this is the only pattern for adding subsystems to the daemon.

**Example:**
```rust
// In engine.rs -- new field
pub(super) handoff_broker: RwLock<handoff::HandoffBroker>,

// In handoff/broker.rs
impl AgentEngine {
    pub(super) async fn route_handoff(
        &self,
        task_description: &str,
        capability_tags: &[String],
        parent_task_id: Option<&str>,
        goal_run_id: Option<&str>,
        thread_id: &str,
        acceptance_criteria: &str,
    ) -> Result<HandoffResult> {
        let broker = self.handoff_broker.read().await;
        let matched = broker.match_specialist(capability_tags)?;
        let bundle = self.assemble_context_bundle(
            task_description, parent_task_id, goal_run_id, thread_id
        ).await?;
        // Validate bundle size
        if bundle.estimated_tokens() > 2000 {
            bundle.summarize_to_ceiling(2000);
        }
        // Spawn via existing primitive
        let task = self.enqueue_task(/* ... */).await;
        // Record to WORM audit
        self.record_handoff_event(/* ... */).await?;
        Ok(HandoffResult { task_id: task.id, specialist: matched })
    }
}
```

### Pattern 2: Context Bundle Assembly with Token Ceiling

**What:** Assemble a ContextBundle from multiple sources (episodic refs, negative constraints, partial outputs, parent thread summary) with a strict 2000-token ceiling enforced by summarization.

**When to use:** Every handoff. The bundle is the primary communication channel between orchestrator and specialist.

**Example:**
```rust
pub struct ContextBundle {
    pub task_spec: String,                     // what to do
    pub acceptance_criteria: String,           // how to validate success
    pub episodic_refs: Vec<EpisodeRef>,        // relevant past experience (IDs + summaries)
    pub negative_constraints: Vec<String>,     // what NOT to try
    pub partial_outputs: Vec<PartialOutput>,   // work done so far
    pub parent_context_summary: String,        // compressed parent thread context
    pub handoff_depth: u8,                     // 0-indexed, max 2 (3 hops)
    pub estimated_tokens: u32,                 // tracked during assembly
}

impl ContextBundle {
    /// Compress the bundle to fit within the token ceiling.
    /// Uses summarize_text on the largest field first (parent_context_summary),
    /// then partial_outputs, until under ceiling.
    pub fn enforce_token_ceiling(&mut self, max_tokens: u32) {
        while self.estimated_tokens > max_tokens {
            // Compress the largest field
            // ...
        }
    }
}
```

### Pattern 3: Specialist Profile Matching with Proficiency Scoring

**What:** Match a task's required capability tags against specialist profiles. Score = sum of (tag match * proficiency weight). Proficiency weights: expert=1.0, advanced=0.75, competent=0.5, familiar=0.25.

**When to use:** Every handoff routing decision.

**Example:**
```rust
pub struct SpecialistProfile {
    pub id: String,
    pub name: String,
    pub role: String,                          // "researcher", "backend-developer", etc.
    pub capabilities: Vec<CapabilityTag>,      // (tag, proficiency)
    pub tool_filter: Option<Vec<String>>,      // allowed tools
    pub system_prompt_snippet: Option<String>,  // appended to subagent system prompt
    pub escalation_chain: Vec<HandoffEscalationRule>,
    pub is_builtin: bool,
}

pub struct CapabilityTag {
    pub tag: String,                           // e.g., "code-review", "rust", "api-design"
    pub proficiency: Proficiency,              // Expert, Advanced, Competent, Familiar
}

pub fn match_specialist(
    profiles: &[SpecialistProfile],
    required_tags: &[String],
) -> Option<(SpecialistProfile, f64)> {
    // Score each profile by tag overlap * proficiency weight
    // Return highest-scoring profile above threshold
    // If top-2 within 10%, flag as ambiguous -> decompose or escalate
}
```

### Pattern 4: Divergent Mode as Extended Collaboration

**What:** Divergent mode spawns 2-3 subagents with different system prompt framings ("frame this as a performance problem", "frame this as an API design problem"), registered in a CollaborationSession. The existing `detect_disagreements()` surfaces tensions. A mediator LLM call synthesizes.

**When to use:** When the orchestrator/goal planner detects ambiguity that would benefit from multiple perspectives.

**Example:**
```rust
pub struct DivergentSession {
    pub collaboration_session_id: String,
    pub framings: Vec<Framing>,
    pub mediator_prompt: String,
    pub status: DivergentStatus,               // Spawning, Running, Mediating, Complete
}

pub struct Framing {
    pub label: String,                         // "performance-lens", "design-lens"
    pub system_prompt_override: String,
    pub task_id: Option<String>,               // after spawn
}
```

### Anti-Patterns to Avoid

- **Building a new orchestration loop:** The HandoffBroker must NOT run its own event loop or scheduler. It wraps `enqueue_task()` / `spawn_subagent()` and hooks into existing task lifecycle events. The task_scheduler already handles dispatch.

- **Forwarding raw conversation history in bundles:** Every Anthropic, OpenAI, and Google reference confirms this is the #1 cause of degraded specialist performance. Always summarize. The 2000-token ceiling is not optional.

- **Starting with all 5 profiles active:** Even though 5 profiles are defined, the matching algorithm should be conservative. If no profile scores above a threshold (e.g., 0.3), the generalist handles it. Profile proliferation causes routing ambiguity (Pitfall 8).

- **LLM-based capability matching:** Do not use an LLM call to decide which specialist to route to. Use structured tag matching. LLM routing adds latency and non-determinism. Save LLM calls for the actual specialist work.

- **Synchronous mediator in divergent mode:** The mediator synthesis should happen after all framings complete, not between them. Do not have the mediator interrupt framings mid-execution.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Subagent spawning | Custom spawn mechanism | Existing `enqueue_task()` with source="subagent" | Already handles terminal allocation, lifecycle tracking, supervision |
| Inter-agent communication | Message bus / channel system | Existing `CollaborationSession` + `broadcast_contribution()` | Already has disagreement detection, voting, persistence |
| Health monitoring | Custom health checker | Existing `supervisor::check_health()` + `SupervisorAction` | Already handles timeout, error loops, resource exhaustion |
| Tool restriction | Custom tool gating | Existing `ToolFilter` with whitelist/blacklist | Already has conflict detection and per-tool checking |
| State machine tracking | Custom state tracker | Existing `SubagentLifecycle` | Already has valid transition enforcement and history |
| WORM audit trail | Custom ledger | Existing `record_telemetry_event()` + new "handoff" kind | SHA-256 hash chain, chain tip persistence already built |
| Token budget enforcement | Custom counter | Existing `ContextBudget` for subagent runtime | Warning at 90%, exceeded handling with overflow actions |
| Text summarization | Custom summarizer | Existing `summarize_text()` in agent mod.rs | Already used throughout codebase for context compression |

**Key insight:** The existing subagent infrastructure is approximately 70% of what Phase 3 needs. The HandoffBroker is primarily a routing and policy layer on top of proven primitives, not a new execution engine.

## Common Pitfalls

### Pitfall 1: Context Bundle Exponential Growth Through Handoff Chains

**What goes wrong:** Agent A hands to Agent B with context. B hands to C with A's context + B's output. Context doubles per hop.
**Why it happens:** The natural instinct is "give the next agent everything." The MAST study found coordination breakdowns at 36.9% of multi-agent failures, with context loss during handoffs as primary contributor.
**How to avoid:** Enforce the 2000-token ceiling strictly. Bundle carries episodic IDs (not content) that specialists can optionally retrieve. `handoff_depth` counter incremented per hop. Broker refuses at depth >= 3.
**Warning signs:** Handoff bundle estimated_tokens growing >2x per hop. Third-hop agents producing lower quality than first-hop.

### Pitfall 2: Specialist Profile Routing Ambiguity

**What goes wrong:** "Backend refactoring" matches researcher, backend-developer, and reviewer simultaneously. Non-deterministic routing.
**Why it happens:** Capability tags defined by what specialists CAN do, not what they SHOULD own. Overlapping tag spaces.
**How to avoid:** Include negative constraints per profile (what they must NOT do). If top-2 scores within 10%, flag as ambiguous -- decompose into sub-tasks or escalate. Start with 3 tightly-scoped profiles before enabling all 5.
**Warning signs:** Routing confidence below 0.3 frequently. Same task type routed to different specialists non-deterministically.

### Pitfall 3: Divergent Consensus Defaults to Longest Response

**What goes wrong:** The mediator picks the most detailed/confident response rather than the most correct one. LLM overconfidence bias (Pitfall 4 from Phase 2 research) compounds here.
**Why it happens:** Without structural signals, the mediator uses text quality as a proxy for correctness.
**How to avoid:** Mediator receives structural metadata per framing: tool call count, error count, execution duration, sources cited. "Disagreement is the product" -- surface tensions to operator when automated resolution would lose nuance.
**Warning signs:** Same framing wins >80% of mediations. Operator reports mediator summaries miss important dissent.

### Pitfall 4: Acceptance Criteria Too Vague to Validate

**What goes wrong:** Acceptance criteria like "implement the feature correctly" cannot be mechanically validated. Output validation becomes a rubber stamp.
**Why it happens:** Acceptance criteria are written by the orchestrator LLM, which tends toward vague language.
**How to avoid:** Define acceptance criteria templates per step kind: code steps must include "compiles without errors" and "tests pass"; research steps must include "at least N sources cited"; review steps must include "all concerns addressed with rationale." The orchestrator fills in specifics but the template provides structure.
**Warning signs:** Validation always passes. Specialist outputs accepted without meaningful checking.

### Pitfall 5: Handoff Audit Trail Missing Critical Context

**What goes wrong:** The WORM handoff entry records from/to/outcome but not the context bundle or acceptance criteria. Post-hoc debugging is impossible.
**Why it happens:** Audit entries are kept small for performance. Bundle content is "too large" for the ledger.
**How to avoid:** Store the full context bundle and acceptance criteria in the handoff_log SQLite table (not size-constrained like WORM). The WORM entry carries a reference (handoff_log_id) to the detailed record. Audit hash covers the reference, not the payload.
**Warning signs:** Debugging handoff failures requires reconstructing context from thread history rather than reading the handoff log.

## Code Examples

### Specialist Profile Definition (Default Built-Ins)

```rust
// Source: derived from CONTEXT.md locked decisions + existing SubAgentDefinition pattern

pub fn default_specialist_profiles() -> Vec<SpecialistProfile> {
    vec![
        SpecialistProfile {
            id: "builtin-researcher".to_string(),
            name: "researcher".to_string(),
            role: "researcher".to_string(),
            capabilities: vec![
                CapabilityTag { tag: "research".into(), proficiency: Proficiency::Expert },
                CapabilityTag { tag: "analysis".into(), proficiency: Proficiency::Expert },
                CapabilityTag { tag: "documentation".into(), proficiency: Proficiency::Advanced },
            ],
            tool_filter: Some(vec![
                "web_search".into(), "read_file".into(), "list_directory".into(),
                "search_codebase".into(), "broadcast_contribution".into(),
            ]),
            system_prompt_snippet: Some(
                "You are a research specialist. Focus on gathering information, \
                 analyzing sources, and producing structured findings. \
                 Do NOT modify code or execute commands.".to_string()
            ),
            escalation_chain: vec![],
            is_builtin: true,
        },
        SpecialistProfile {
            id: "builtin-backend-developer".to_string(),
            name: "backend-developer".to_string(),
            role: "backend-developer".to_string(),
            capabilities: vec![
                CapabilityTag { tag: "rust".into(), proficiency: Proficiency::Expert },
                CapabilityTag { tag: "backend".into(), proficiency: Proficiency::Expert },
                CapabilityTag { tag: "api-design".into(), proficiency: Proficiency::Advanced },
                CapabilityTag { tag: "testing".into(), proficiency: Proficiency::Advanced },
                CapabilityTag { tag: "database".into(), proficiency: Proficiency::Competent },
            ],
            tool_filter: None, // full tool access
            system_prompt_snippet: Some(
                "You are a backend development specialist. Focus on Rust code, \
                 daemon internals, API design, and testing. Stay within your \
                 assigned scope -- do NOT modify frontend code.".to_string()
            ),
            escalation_chain: vec![],
            is_builtin: true,
        },
        // frontend-developer, reviewer, generalist follow same pattern
    ]
}
```

### Handoff Escalation Trigger Evaluation

```rust
// Source: CONTEXT.md locked decisions + existing escalation.rs pattern

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffEscalationTrigger {
    ConfidenceBelow(String),      // band name: "low", "uncertain"
    ToolFails(u32),               // consecutive failures
    TimeExceeds(u64),             // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffEscalationAction {
    HandBack,                     // return to parent with partial output
    RetryWithNewContext,          // retry same specialist with fresh context
    EscalateTo(String),           // route to different specialist profile
    AbortWithReport,              // terminate with failure report
}

pub fn evaluate_escalation_triggers(
    triggers: &[(HandoffEscalationTrigger, HandoffEscalationAction)],
    consecutive_failures: u32,
    elapsed_secs: u64,
    confidence_band: &str,
) -> Option<HandoffEscalationAction> {
    for (trigger, action) in triggers {
        let fired = match trigger {
            HandoffEscalationTrigger::ConfidenceBelow(band) => {
                confidence_band_order(confidence_band) < confidence_band_order(band)
            }
            HandoffEscalationTrigger::ToolFails(threshold) => {
                consecutive_failures >= *threshold
            }
            HandoffEscalationTrigger::TimeExceeds(max_secs) => {
                elapsed_secs >= *max_secs
            }
        };
        if fired {
            return Some(action.clone());
        }
    }
    None
}
```

### WORM Handoff Audit Entry

```rust
// Source: existing record_telemetry_event pattern from history.rs

pub async fn record_handoff_audit(
    &self,
    from_task_id: &str,
    to_specialist: &str,
    to_task_id: &str,
    task_description: &str,
    outcome: &str,          // "dispatched", "accepted", "rejected", "timeout", "error"
    duration_ms: Option<u64>,
    confidence_band: Option<&str>,
    handoff_log_id: &str,   // reference to detailed handoff_log row
) -> Result<()> {
    let payload = serde_json::json!({
        "from": from_task_id,
        "to_specialist": to_specialist,
        "to_task_id": to_task_id,
        "task": task_description,
        "outcome": outcome,
        "duration_ms": duration_ms,
        "confidence": confidence_band,
        "handoff_log_id": handoff_log_id,
    });
    self.history
        .record_telemetry_event("handoff", &payload)
        .await
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Vague task descriptions to subagents | Structured task specs with acceptance criteria | Anthropic multi-agent system (2025) | Prevents duplicate work and scope drift |
| Full conversation forwarding at handoff | Structured context objects (200-500 tokens) vs raw (5-20K tokens) | Industry consensus mid-2025 | 70-90% token reduction, better specialist focus |
| Consensus-seeking in multi-agent | Disagreement as valuable output | MAD framework (EMNLP 2024), Anthropic research | Moderate disagreement outperforms forced consensus |
| Majority voting for resolution | Structural signal weighting (error rate, sources cited) | Multi-agent resilience guide (2026) | Prevents loudest-voice-wins bias |
| Static agent routing | Confidence-based dynamic routing | OpenAI Agents SDK (Mar 2025) | Routes to specialists only when confidence warrants delegation |

**Deprecated/outdated:**
- **Swarm framework**: Replaced by OpenAI Agents SDK (March 2025). The pattern is the same but the SDK is production-ready.
- **Linear handoff chains**: Industry moved to DAG-based task decomposition. tamux's goal planner already produces step graphs, which is the right approach.

## Open Questions

1. **Specialist profile system prompt injection method**
   - What we know: Existing `SubAgentDefinition` has `system_prompt` override. The `task_prompt.rs` `append_sub_agent_registry()` lists available sub-agents.
   - What's unclear: Should the specialist profile system prompt replace or augment the default system prompt? The existing `SubAgentDefinition.system_prompt` is a full override.
   - Recommendation: Augment (append a `## Specialist Role` section) rather than replace. The base system prompt contains critical safety and tool usage instructions that specialists need.

2. **How the orchestrator decides when to hand off**
   - What we know: ConfidenceBelow trigger from escalation chain can trigger handoff. Goal planner steps can specify specialist requirements.
   - What's unclear: Should the handoff broker be invoked automatically by the goal planner, or only when explicitly requested via a tool call?
   - Recommendation: Both. Goal planner can specify `kind: GoalRunStepKind::Specialist("backend-developer")` to force routing, AND a new `route_to_specialist` tool allows the agent to trigger handoff mid-task when it detects it needs specialist help.

3. **Acceptance criteria validation mechanism**
   - What we know: Output must be validated against criteria before accepting.
   - What's unclear: Is validation structural (regex/format checks) or LLM-based (ask the LLM "does this output meet the criteria")?
   - Recommendation: Structural checks first (non-empty output, expected format), then a brief LLM validation call if structural checks pass. This avoids expensive LLM calls for obviously-bad output while catching subtle quality issues.

## Project Constraints (from CLAUDE.md)

- **Rust daemon-first architecture**: All handoff logic runs in-process within AgentEngine. No separate services.
- **Local-first**: Specialist profiles stored in SQLite. No cloud profile registry.
- **Provider-agnostic**: Specialist profiles can override provider/model but must work with any configured provider.
- **Backward compatibility**: New fields on AgentConfig must use `#[serde(default)]`. Existing `sub_agents` config field and `CollaborationConfig` must continue working.
- **Single process**: HandoffBroker is an in-process module, not a sidecar.
- **Naming conventions**: `snake_case` for files and functions, `PascalCase` for types, `#[derive(Debug, Clone, Serialize, Deserialize)]` for wire types.
- **Error handling**: `anyhow` for error propagation, `?` with `.context()` for error context.
- **Testing**: Rust built-in `#[test]` framework, inline unit tests, `make_` prefix for test builders, `sample_` prefix for factory helpers.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `crates/amux-daemon/src/agent/subagent/` -- lifecycle.rs, supervisor.rs, context_budget.rs, tool_filter.rs, termination.rs, tool_graph.rs
- Codebase analysis: `crates/amux-daemon/src/agent/collaboration.rs` -- CollaborationSession, broadcast_contribution, vote_on_disagreement, detect_disagreements
- Codebase analysis: `crates/amux-daemon/src/agent/engine.rs` -- AgentEngine struct fields and extension pattern
- Codebase analysis: `crates/amux-daemon/src/agent/tool_executor.rs` L4275 -- execute_spawn_subagent implementation
- Codebase analysis: `crates/amux-daemon/src/agent/types.rs` -- SubAgentDefinition, AgentTask, AgentEvent variants, GoalRun
- Codebase analysis: `crates/amux-daemon/src/agent/metacognitive/escalation.rs` -- EscalationLevel, EscalationCriteria, EscalationDecision
- Codebase analysis: `crates/amux-daemon/src/agent/episodic/` -- retrieve_relevant_episodes, query_active_constraints (Phase 1 deliverables)
- Codebase analysis: `crates/amux-daemon/src/agent/uncertainty/` -- compute_step_confidence, ConfidenceBand (Phase 2 deliverables)
- Phase 1 Verification: 18/18 truths verified, all episodic infrastructure operational
- Phase 2 Verification: 20/20 truths verified, all awareness + uncertainty infrastructure operational

### Secondary (MEDIUM confidence)
- [Anthropic: How we built our multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) -- Structured task specs, artifact bypass (lightweight refs not content), external memory pattern
- [Azure AI Agent Design Patterns](https://learn.microsoft.com/en-us/azure/architecture/ai-ml/guide/ai-agent-design-patterns) -- Orchestrator-worker pattern reference
- [OpenAI Agents SDK Multi-Agent](https://openai.github.io/openai-agents-python/multi_agent/) -- Handoff pattern, AgentWorkflow vs Orchestrator
- [How Agent Handoffs Work (Towards Data Science)](https://towardsdatascience.com/how-agent-handoffs-work-in-multi-agent-systems/) -- Context transfer best practices
- [Skywork: Best Practices for Multi-Agent Orchestration](https://skywork.ai/blog/ai-agent-orchestration-best-practices-handoffs/) -- Observability, quality assurance, structured communication
- [Google: Architecting efficient context-aware multi-agent framework](https://developers.googleblog.com/architecting-efficient-context-aware-multi-agent-framework-for-production/) -- Context window management, summarization at handoffs
- [Encouraging Divergent Thinking in LLMs through Multi-Agent Debate (EMNLP 2024)](https://arxiv.org/abs/2305.19118) -- MAD framework, moderate disagreement outperforms forced consensus
- [MAST: Why Do Multi-Agent LLM Systems Fail? (March 2025)](https://arxiv.org/pdf/2503.13657) -- Coordination breakdowns at 36.9%, role confusion failure mode

### Tertiary (LOW confidence)
- [Two Paradigms of Multi-Agent AI: Rust Parallel Agents vs Claude Code Agent Teams](https://vadim.blog/two-paradigms-multi-agent-ai-rust-vs-claude-teams) -- team.rs pattern with TaskQueue + Mailbox for Rust agents. Interesting but tamux has its own established patterns.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Zero new dependencies. All existing infrastructure verified through Phase 1+2 verification reports.
- Architecture: HIGH -- Follows established AgentEngine extension pattern with extensive precedent in episodic/, awareness/, uncertainty/ modules. Existing subagent + collaboration infrastructure covers 70% of requirements.
- Pitfalls: HIGH -- Five pitfalls backed by MAST study data, Anthropic production experience, and Phase 2 research findings on LLM confidence miscalibration.

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable domain, patterns well-established)
