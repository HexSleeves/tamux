# Phase 1: Memory Foundation - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

The agent remembers what it tried, what failed, and what approaches are ruled out -- and uses that memory to avoid repeating mistakes. This phase builds the episodic memory system (structured episode records, FTS5 retrieval, causal chains), the counter-who persistent self-model (repeat detection, correction tracking), and the negative knowledge constraint graph (ruled-out approaches with TTL expiry). All data stays in local SQLite, append-only (WORM), with PII scrubbing.

</domain>

<decisions>
## Implementation Decisions

### Episode Recording Behavior
- Episodes are recorded on goal completion/failure only -- not every session end or tool call (minimal noise, high signal)
- Episode summaries are LLM-generated 2-3 sentence summaries, piggy-backing on existing GoalReflectionResponse (no separate LLM call)
- Episodic context injection is compact: outcome + root_cause + 1-line summary per episode, max 5 episodes, max 1500 tokens
- Agent always announces "I found N relevant past experiences" before planning when episodic memory is used -- transparency over stealth

### Counter-Who and Negative Knowledge
- Repeat detection threshold is 3 -- suggest pivot after 3 similar failed attempts (balanced between aggressive and patient)
- Negative knowledge is warn-only -- inject "previously ruled out" context, let agent decide (not hard blocks)
- Negative constraint default TTL is 30 days -- long enough to be useful, short enough to not stale
- Counter-who state is visible to operator in goal planning output (transparency principle)

### Privacy and Operator Control
- Episode recording is opt-in by default (enabled unless operator explicitly disables)
- Privacy controls: per-session suppression + global toggle + configurable TTL
- PII scrubbing uses existing scrub_sensitive patterns (API keys, tokens, secrets) plus home directory path sanitization
- No episode export in v3.0 -- SQLite is directly queryable if needed, defer export tooling to v3.1

### Claude's Discretion
All implementation-level decisions (exact SQL schema, FTS5 detail level, internal module structure, error handling patterns) follow established codebase conventions and are at Claude's discretion.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scrub_sensitive()` in persistence.rs -- PII redaction for episode content
- `GoalReflectionResponse` in goal_planner.rs -- existing LLM reflection that can produce episode summaries
- `WormLedger` with `record_telemetry_event()` in history.rs -- WORM append pattern for episodes
- `context_archive_fts` FTS5 table in history.rs -- pattern for external content FTS5 tables
- `AgentConfig` in types.rs -- config struct to extend with EpisodicConfig

### Established Patterns
- Extension modules: `learning/`, `liveness/`, `metacognitive/` -- each is a subdirectory under agent/ with `mod.rs` + submodule files, behavior in `impl AgentEngine` blocks
- State fields: `RwLock<T>` or `Mutex<T>` guarded fields on AgentEngine struct
- SQLite access: `self.history.conn.call(move |conn| { ... })` via tokio-rusqlite for async writes
- Schema init: `init_schema()` in history.rs creates tables if not exist

### Integration Points
- `complete_goal_run()` and `fail_goal_run()` in goal_planner.rs -- hook for episode recording
- `request_goal_plan()` in goal_llm.rs -- hook for episodic retrieval before planning
- `build_system_prompt()` in system_prompt.rs -- hook for injecting episodic context and negative constraints
- `agent_loop.rs` main loop -- hook for counter-who updates on tool results
- `consolidation.rs` -- hook for TTL-based cleanup of episodes and constraints

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches following established codebase patterns.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>
