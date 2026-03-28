# Phase 4: Operator Control and Transparency - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

The operator has full visibility into cost, can tune agent autonomy per goal, can ask "why did you do that?" and get a real answer, and can see what the agent contributed vs what came from operator input. Also implements the deferred UNCR-02 (tool-call confidence for Safety tools) and UNCR-03 (source authority labeling). Builds on all previous phases.

</domain>

<decisions>
## Implementation Decisions

### Cost & Token Accounting
- Per-goal token counts (prompt + completion) tracked on every LLM API call
- Per-session and cumulative cost estimates using provider rate cards
- Cost displayed in goal completion reports (not real-time streaming)
- Budget alerts: configurable threshold in agent config, notification only (no auto-stop)
- Cost data persisted in goal_run metadata, queryable via CLI/observability

### Autonomy Dial
- Per-goal autonomy level setting: autonomous / aware / supervised
- Autonomous: agent proceeds, operator sees final report only
- Aware: agent reports on milestones (sub-task completions, handoffs)
- Supervised: agent reports on every significant step and waits for acknowledgment
- Default level: aware (balanced between autonomy and visibility)

### Explainability
- "Why did you do that?" query searches episodic store + causal traces for the referenced action
- Returns structured answer: decision point, alternatives considered, reasons for chosen approach
- Rejected alternatives stored alongside chosen plan in goal_run metadata during planning
- Uses existing episodic retrieval + negative knowledge for "what was tried and ruled out"

### Shared Authorship
- Metadata tags on goal outputs: operator (from operator input), agent (from agent synthesis), joint (collaborative)
- Attribution is metadata, not inline commentary — doesn't disrupt reading flow
- Tracked at goal output level, not per-message level

### Deferred Confidence (from Phase 2)
- UNCR-02: Pre-tool ConfidenceWarning for Safety-domain tools only (not all tools)
- UNCR-03: URL-based source authority classification (official/community/unknown) on web_search/web_read results

### Claude's Discretion
All implementation-level decisions (struct layout, integration approach, internal APIs) follow established codebase conventions.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `EfficiencyMetrics` in agent types — existing token tracking (may need extension)
- `AgentEvent` variants — for autonomy level notifications
- `ConfidenceWarning` event from Phase 2 — for pre-tool warnings
- Episodic retrieval from Phase 1 — for explainability queries
- Negative knowledge from Phase 1 — for "what was ruled out"
- `DomainClassification` from Phase 2 — for Safety-domain tool filtering

</code_context>

<specifics>
No specific requirements beyond locked decisions.
</specifics>

<deferred>
- Real-time cost streaming to UI — v3.1
- Auto-budget enforcement (stop on threshold) — v3.1
- Per-message attribution granularity — v3.1
</deferred>
