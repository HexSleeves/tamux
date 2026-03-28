# Phase 3: Multi-Agent Orchestration - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

The agent delegates tasks to specialist subagents with structured handoffs, validates their output, and can run divergent framings in parallel to surface productive disagreement. This phase builds the HandoffBroker (capability matching, context bundling, escalation chains), specialist profiles, output validation, WORM audit trail, and divergent subagent mode. Builds on Phase 1's episodic memory (context bundle refs) and Phase 2's uncertainty (confidence-based routing decisions).

</domain>

<decisions>
## Implementation Decisions

### Handoff Broker
- HandoffBroker layers on existing `spawn_subagent` primitive — not a separate orchestration engine
- 5 default specialist profiles ship out of the box: researcher, backend-developer, frontend-developer, reviewer, generalist
- Proficiency levels: expert, advanced, competent, familiar — used for capability matching
- Context bundles carry typed references (memory refs, episodic refs, document refs, partial outputs) with strict 2000-token ceiling per bundle
- Context bundles are summarized, not forwarded raw — prevents exponential growth through handoff chains
- Handoff depth limit: max 3 hops, then escalate to operator with full chain context

### Escalation
- Structured triggers: ConfidenceBelow(band), ToolFails(count), TimeExceeds(duration)
- Structured actions: HandBack, RetryWithNewContext, EscalateTo(profile), AbortWithReport
- Escalation chains are configurable per specialist profile

### Output Validation
- Orchestrator validates specialist output against acceptance criteria before accepting
- Acceptance criteria defined per handoff task (not per profile)

### Divergent Subagents
- 2-3 parallel framings work the same problem simultaneously
- Disagreement between framings is surfaced as the valuable output (tensions, not forced consensus)
- Mediator synthesizes tensions into a recommendation that acknowledges tradeoffs
- Divergent mode is a handoff mode, not a separate system

### Audit
- Every handoff logged to WORM audit trail (from, to, task, outcome, duration, confidence, audit_hash)
- Uses existing WORM ledger infrastructure from Phase 1

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions and are at Claude's discretion.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `spawn_subagent()` in subagent/ — existing subagent spawning primitive
- `broadcast_contribution()` — multi-agent communication
- `vote_on_disagreement()` — existing disagreement mechanism
- WORM ledger infrastructure from episodic module — `record_telemetry_event`
- `ConfidenceBand` from uncertainty module — confidence-based routing
- `retrieve_relevant_episodes()` from episodic/retrieval.rs — for building context bundles

### Phase 1 & 2 Deliverables
- Episodic store for context bundle episode refs
- Uncertainty engine for confidence-based handoff decisions
- Awareness monitor for trajectory signals

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond the locked decisions above.

</specifics>

<deferred>
## Deferred Ideas

- Async handoff task queue (specialist doesn't need to be online) — v3.1
- Custom specialist profiles defined by operator — v3.1
- Shared semantic memory layer between specialists — v3.1

</deferred>
