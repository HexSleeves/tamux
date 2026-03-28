# Phase 2: Awareness and Judgment - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

The agent senses when it is stuck, tracks the texture of its own activity, and expresses honest confidence grounded in structural evidence. This phase builds situational awareness (empirical failure tracking, mode shifts, sliding window analysis), embodied metadata (5 scalar dimensions per action), and uncertainty quantification (confidence labels on plan steps, domain-specific escalation, calibration feedback). Builds on Phase 1's episodic memory for familiarity signals and counter-who for false-positive guards.

</domain>

<decisions>
## Implementation Decisions

### Awareness & Mode Shifts
- Stuck detection triggers after 3+ same tool+args pattern with no new information gained (SHA-256 approach hashing from counter-who)
- When stuck is detected: auto-shift strategy (try different approach) + notify operator with "diminishing returns detected"
- Sliding window sizes: short-term 5 actions, medium-term 30 minutes, long-term full session
- Counter-who is consulted before ALL mode shifts fire (prevents false positives from productive repetition)

### Confidence Signals
- Confidence labels displayed inline with plan steps: `[HIGH] Step 1: ...` — compact, scannable
- Structural signals feeding confidence: tool success rate + episodic familiarity + blast radius + approach novelty
- Default domain escalation: Safety domains block on LOW, Business domains warn on LOW, Research domains surface all levels
- Confidence visible in goal planning only for v3.0 — expand to chat/tasks in v3.1
- Provider-agnostic: confidence derives from structural signals, not LLM logits or model-specific features

### Embodied Metadata & Trajectory
- 5 core scalar dimensions: difficulty (retries, error rate), familiarity (episodic match), trajectory (converging/diverging), temperature (operator urgency), weight (conceptual mass)
- Trajectory calculated as ratio of "progress events" vs "retry/failure events" in sliding window
- Trajectory displayed in goal run status updates to operator + available in heartbeat
- Heartbeat uses trajectory and difficulty to adjust check frequency (stuck + hard = more frequent checks)

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions and are at Claude's discretion.

</decisions>

<code_context>
## Existing Code Insights

### Phase 1 Deliverables (Available)
- `episodic/counter_who.rs` — approach hashing with SHA-256, repeat detection (threshold 3), already wired into agent_loop.rs
- `episodic/retrieval.rs` — FTS5 retrieval with BM25+recency ranking, `retrieve_relevant_episodes`
- `episodic/store.rs` — `record_goal_episode`, `record_session_end_episode`, `expire_old_episodes`
- `episodic/negative_knowledge.rs` — constraint graph, `query_active_constraints`
- EpisodicConfig on AgentConfig — global toggle, TTL, max retrieval episodes

### Reusable Assets
- `stuck_detection.rs` — existing stuck detection (limited to goal runner scope, needs broadening)
- `self_assessment.rs` — existing self-assessment infrastructure in metacognitive/
- `heartbeat` system in gateway_loop — adaptive scheduling, quiet hours, check frequency
- `ConfidenceBand` enum — may already exist in agent types
- `policy.rs` — blast radius assessment for tool calls
- `liveness/` module — existing liveness monitoring patterns

### Established Patterns
- Extension modules: `learning/`, `liveness/`, `metacognitive/` — each is a subdirectory under agent/
- Heartbeat integration: hooks in gateway_loop.rs
- System prompt injection: `build_system_prompt()` parameter pattern from Phase 1

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches following established patterns.

</specifics>

<deferred>
## Deferred Ideas

- Confidence in non-goal contexts (chat, tasks) — v3.1
- Momentum and coherence dimensions — v3.1 if needed
- Dashboard widget for trajectory visualization — future UI work

</deferred>
