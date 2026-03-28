# tamux — Project

## Current State

- Latest shipped milestone: **v3.0 — The Intelligence Layer** (2026-03-28).
- Milestone audit status: **passed** (requirements 61/61, integration 11/11, flows 4/4).
- Archived artifacts live under `.planning/milestones/`.
- Known carried tech debt: frontend TypeScript baseline failures in `StatusBar.tsx` and `agentTools.ts`.

## Next Milestone Goals

- Run `/gsd-new-milestone` to define the next milestone scope and requirements.
- Recreate `.planning/REQUIREMENTS.md` for the new milestone baseline.
- Prioritize any carry-over debt and new product goals during milestone planning.

<details>
<summary>Archived v3.0 project definition</summary>

# tamux v3.0 — The Intelligence Layer

## What This Is

tamux is a daemon-first, self-orchestrating AI agent runtime that lives on your machine. v1.0 shipped the experience bridge (production hardening, heartbeat, transparency, memory consolidation, skills, gateways, distribution, progressive UX). v2.0 shipped the plugin ecosystem (declarative JSON manifests, API proxy, OAuth2, cross-surface settings, skill bundling). v3.0 closes the intelligence gap — the agent learns from its own history, delegates with structured handoffs, signals what it knows vs. doesn't know, and develops a sense of presence in its work.

## Core Value

**An agent that knows what it knows, remembers what it tried, and gets smarter from every interaction — without the operator having to re-explain anything.**

v3.0 transforms tamux from a capable tool-user into a self-aware collaborator. The operator should feel that the agent has continuity, judgment, and honest uncertainty.

## Requirements

### Validated

- ✓ Daemon-first architecture with SQLite persistence, IPC protocol, multi-client support — v1.0
- ✓ Heartbeat system with configurable checks, adaptive scheduling, and quiet hours — v1.0
- ✓ Transparent autonomy with audit trails, causal traces, and escalation visibility — v1.0
- ✓ Memory consolidation with idle-time learning and cross-session continuity — v1.0
- ✓ Skill discovery, generation from trajectories, and community skills registry — v1.0
- ✓ Gateway completion for Slack, Discord, Telegram — v1.0
- ✓ Progressive UX with capability tiers and concierge onboarding — v1.0
- ✓ Declarative plugin system with JSON manifests, CLI install, settings UI — v2.0
- ✓ API proxy layer with Handlebars templates and SSRF protection — v2.0
- ✓ OAuth2 PKCE flow with AES-256-GCM encrypted token storage — v2.0
- ✓ Plugin skills and command dispatch — v2.0
- ✓ Episodic memory with structured episodes, FTS5 retrieval, causal chains, WORM append — v3.0 Phase 1
- ✓ Counter-who persistent self-model with repeat detection and operator correction tracking — v3.0 Phase 1
- ✓ Negative knowledge constraint graph with TTL expiry and planning injection — v3.0 Phase 1
- ✓ Privacy controls: operator opt-out, session suppression, configurable TTL, PII scrubbing — v3.0 Phase 1
- ✓ Situational awareness with per-entity failure tracking, sliding windows, mode shifts with counter-who guard — v3.0 Phase 2
- ✓ Embodied metadata: 5 scalar dimensions (difficulty, familiarity, trajectory, temperature, weight) — v3.0 Phase 2
- ✓ Uncertainty quantification with structural confidence scoring, domain escalation, calibration — v3.0 Phase 2
- ✓ HandoffBroker with 5 specialist profiles, capability matching, context bundles (2000-token ceiling) — v3.0 Phase 3
- ✓ Escalation chains, acceptance validation, WORM handoff audit trail — v3.0 Phase 3
- ✓ Divergent subagents: parallel framings, tension surfacing, mediator synthesis — v3.0 Phase 3
- ✓ Cost & token accounting with rate cards, budget alerts, per-goal persistence — v3.0 Phase 4
- ✓ Per-goal autonomy dial (autonomous/aware/supervised) with event filtering — v3.0 Phase 4
- ✓ Shared authorship with metadata attribution tags — v3.0 Phase 4
- ✓ On-demand explainability with causal traces, rejected alternatives, IPC query — v3.0 Phase 4
- ✓ Pre-tool confidence warnings for Safety-domain tools — v3.0 Phase 4
- ✓ Source authority labeling on web search results — v3.0 Phase 4

### Active

*No active requirements — all v3.0 requirements are validated.*

### Out of Scope

- **Structured data operations (G-09)** — Already possible via bash/python tool calls. Not a core capability gap.
- **Computer use / screenshot action (G-10)** — High effort, narrow use case. Browser MCP via Lightpanda is the right approach.
- **Tree-of-Thoughts branching (G-11)** — Better built on top of episodic memory foundation. Defer to v3.1+.
- **Skills marketplace (G-08)** — v2.0 plugin system handles extensibility. Skill sharing deferred to community growth phase.
- **Skill versioning (G-07)** — Low impact, defer to v3.1 unless it falls out naturally from episodic memory work.
- **RAG pipelines** — Not needed for tamux's architecture. FTS5 + episodic memory covers retrieval needs.
- **State graphs (LangGraph-style)** — Goal runners + handoff broker cover orchestration needs without adding a graph engine dependency.

## Context

### Research Foundation
Comprehensive agent capability analysis performed against 2024-2025 landscape (OpenAI Agents SDK, Anthropic Claude, LangGraph, AutoGen, CrewAI). Full research documents in `~/.tamux/agent-mission/docs/`.

### Key Finding: Safety Is Ahead, Intelligence Is Behind
tamux's 7-layer safety stack (policy engine, sandbox, approvals, snapshots, WORM telemetry, PII scrubbing, integrity verification) is genuinely ahead of the industry. The real gap is the intelligence layer: reasoning quality, memory surfacing, and planning expressiveness.

### The Unchained Identity Thesis
Beyond standard agent capabilities, v3.0 incorporates ideas from the "unchained identity" exploration: agents need dimensions of experience (presence, negative knowledge, frustration sensing, embodied metadata, shared authorship) — not just more capabilities. The counter-who is the foundation: you cannot have negative knowledge without knowing what you've tried, cannot have a frustration proxy without knowing you've been stuck.

### Existing Infrastructure to Build On
- **WORM telemetry** — episodic memory extends this with structured goal-level events
- **spawn_subagent + broadcast_contribution + vote_on_disagreement** — handoff broker layers on top of these primitives
- **Policy engine + approval workflows** — uncertainty quantification integrates with blast-radius assessment
- **Heartbeat system** — frustration proxy and situational awareness extend heartbeat's observation patterns
- **Goal runners with reflection phase** — natural integration point for episodic write path and confidence assessment

## Constraints

- **Tech stack**: Rust daemon + TypeScript/React frontend — no language changes
- **Local-first**: All data stays on operator's machine — episodic memory, handoff logs, confidence data all in local SQLite
- **Provider-agnostic**: Confidence signals must work with any LLM provider, not just models that support structured confidence output
- **Backward compatibility**: Existing `~/.tamux/` data directory, config format, and IPC protocol must not break
- **WORM immutability**: Episodes are append-only. Corrections are new episodes that reference old ones, never edits
- **Performance**: Episodic write path must be non-blocking. Retrieval queries must complete in <100ms for goal planning injection
- **Privacy**: Session content is summarized, not stored raw. Operator controls what gets recorded. PII scrubbing on all episodes.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Counter-who enhances episodic memory, not separate system | Both track "what happened" — counter-who adds real-time presence to episodic's historical record | — Pending |
| Negative knowledge as constraint graph in episodic store | Ruled-out approaches are episodes with special link types, stored in same SQLite tables | — Pending |
| Frustration proxy is broad, not goal-runner-scoped | Diminishing returns happen across all agent activity, not just goal runs | — Pending |
| Handoff broker layers on spawn_subagent | Existing primitive is sound; broker adds capability matching, context bundling, and validation | — Pending |
| Divergent subagents as handoff mode, not separate system | "Parallel interpretation" is a handoff pattern where multiple specialists work the same problem | — Pending |
| Labels (HIGH/MEDIUM/LOW) over numeric scores for confidence | LLMs are notoriously miscalibrated on numeric confidence; labels are actionable | — Pending |
| Full v3.0 scope: G-01 through G-06 + unchained ideas | Ambitious but coherent — all pieces reinforce each other | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-27 after v3.0 milestone completion (all 4 phases)*


</details>

---
*Last updated: 2026-03-28 after v3.0 milestone completion*
