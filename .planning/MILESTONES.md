# Milestones

## v3.0 The Intelligence Layer (Shipped: 2026-03-28)

**Phases completed:** 9 phases, 23 plans, 40 tasks

**Key accomplishments:**

- SQLite-backed episodic memory module with 12+ data types, 5-table schema with FTS5, CRUD operations, WORM ledger, PII scrubbing, and episode link management wired into AgentEngine
- FTS5 retrieval engine with BM25 + recency re-ranking, goal boundary episode recording, and episodic context injection into goal planning prompts
- 1. [Rule 3 - Blocking] Fixed rusqlite stmt lifetime issue in query_active_constraints
- Per-entity 3-tier sliding window awareness with diminishing returns detection, counter-who false positive guard, trajectory computation, and mode shift notification
- 5 scalar dimension functions (difficulty, familiarity, trajectory, temperature, weight) as pure Rust computations with EmbodiedMetadata aggregate struct
- Structural confidence scoring with 4-signal pipeline, domain-specific escalation thresholds, cold-start calibration, and goal planner approval routing
- HandoffBroker with 5 specialist profiles, capability-weighted matching algorithm, and SQLite persistence schema wired into AgentEngine
- Context bundle assembly with 2000-token ceiling enforcement, escalation trigger evaluation for 3 trigger types, structural acceptance validation, and WORM handoff audit trail
- HandoffBroker route_handoff() orchestrating full match-bundle-audit-enqueue flow, route_to_specialist tool for mid-task handoffs, and GoalRunStepKind::Specialist for planned specialist routing
- Divergent parallel framing with 2-3 perspectives, automatic tension detection, and mediator prompt generation that surfaces tradeoffs without forcing consensus
- CostTracker module with provider rate cards, per-goal token accumulation in agent_loop, budget alert events, and cost persistence on GoalRun
- AutonomyLevel enum with event filtering gate, supervised-mode acknowledgment at step boundaries, and AuthorshipTag on goal completion
- On-demand "why did you do that?" query with causal trace cascade, rejected alternatives capture, and IPC dispatch
- Pre-tool ConfidenceWarning for Safety-domain tools and URL-based source authority labels on web search results (exa/tavily/ddg)
- Calibration feedback loop wired on goal completion/failure, embodied difficulty+weight in confidence pipeline, all 6 new AgentEvent variants forwarded over IPC
- Production callers wired for escalation triggers (HAND-04), specialist output validation (HAND-05), and session-end episode recording (EPIS-08) closing remaining handoff and episodic integration gaps
- Confidence scoring now consumes real operator pacing temperature, and specialist handoff completion is enforced through real log linkage with fail-closed validation gating.
- run_divergent tool and AgentStartDivergentSession IPC wiring two entry points into the existing 739-line DivergentSession infrastructure
- GoalRunStepKind::Divergent variant enabling autonomous goal decomposition to spawn divergent framings when a step benefits from multiple perspectives
- Divergent framing tasks now flow from completion events into recorded contributions, automatic tension synthesis, and operator-retrievable mediator payloads through both tool and IPC surfaces.
- Unified approval-gate enforcement now fail-closes runtime dispatch and requires explicit supervised acknowledgment before queued goal-step tasks can execute.
- Frontend explain/divergent commands now render payloads directly from IPC invoke responses, closing the renderer event-forwarding E2E gap while preserving daemon event compatibility.
- Milestone verification closure packaging completed by reconciling EPIS-08 cross-phase evidence and aligning requirement checklist/traceability statuses with existing verified artifacts.

### Known Gaps

- EPIS-01 checklist line remained unchecked in source requirements file at archive time.
- EPIS-07 checklist line remained unchecked in source requirements file at archive time.
- EPIS-09 checklist line remained unchecked in source requirements file at archive time.
- EPIS-10 checklist line remained unchecked in source requirements file at archive time.
- EPIS-11 checklist line remained unchecked in source requirements file at archive time.

---
