# Operator Profile Onboarding + Passive Learning Design

Date: 2026-03-26

Status: Approved for planning

## Problem Statement

tamux currently captures little explicit operator context during first use. The setup flows focus on provider/runtime configuration, but not on human context (preferred name, goals, painpoints, dreams, collaboration style). Existing behavior learning in the daemon is aggregate and implicit, but does not provide a transparent, consent-driven profile loop that users can shape.

We need a mechanism that:

- asks lightweight human questions during first interactive use (TUI/React),
- keeps learning passively from interactions,
- asks occasional follow-up questions (weekly and contextual),
- enables proactive, user-aligned suggestions only with explicit opt-in,
- uses SQLite as source of truth,
- keeps `USER.md` in sync as a derived profile summary agents can rely on.

## Goals and Non-Goals

### Goals

- Create a first-session concierge onboarding interview in TUI and React.
- Capture explicit profile primitives: how to address user, goals, painpoints, dreams, preferred collaboration tone.
- Add consent-gated personalization behavior:
  - proactive suggestions/news,
  - passive learning,
  - weekly check-ins.
- Keep profile state in daemon-managed SQLite tables.
- Generate and refresh `USER.md` from DB-backed profile state.
- Support contextual micro-questions when profile confidence is low or behavior shifts.

### Non-Goals

- No cloud sync or remote profile storage.
- No replacement of existing mission memory files; this extends and synchronizes them.
- No mandatory full re-onboarding loop after first completion.
- No hidden/opaque proactive behavior without explicit consent.

## High-Level Architecture

Daemon-first architecture remains unchanged:

- Daemon owns profile state, interview logic, learning signals, confidence, and scheduling.
- TUI/React clients render questions and send answers.
- Protocol exposes interview/profile RPC-style messages.
- SQLite is the canonical store.
- `USER.md` is generated from profile snapshots/signals and is not canonical.

This preserves current multi-client consistency: disconnect/reconnect across surfaces still references one profile state.

## Component Design

### 1) Daemon `OperatorProfileV2` Domain

New bounded domain in `crates/amux-daemon/src/agent/` responsible for:

- profile schema and field confidence,
- answer ingestion,
- profile completeness and missing-field detection,
- consent flags and policy checks for proactive actions.

Core profile fields:

- `preferred_name`
- `agent_call_style` (how agent should address user)
- `user_call_style` (how user prefers to address agent)
- `primary_goals[]`
- `painpoints[]`
- `aspirations[]`
- `topic_interests[]`
- `collaboration_preferences`
- `last_reviewed_at`

### 2) Daemon `ConciergeInterviewPlanner`

Responsible for selecting next question:

- first-session onboarding path (lightweight, finite),
- weekly check-in path,
- contextual check-ins triggered by confidence decay/behavior deltas.

Rules:

- one question at a time,
- avoid repeated asks when confidence is already high,
- ask only missing/uncertain fields,
- respect user fatigue caps (max questions per session/check-in).

### 3) Daemon `ProfileLearningEngine`

Consumes passive signals from existing interaction streams:

- explicit user phrasing changes,
- correction/revision patterns,
- accepted/ignored suggestion patterns,
- topic recurrence across sessions.

Outputs:

- confidence adjustments on profile fields,
- resonance scores (what tends to land well),
- contextual check-in recommendations.

### 4) Protocol Extensions (`amux-protocol`)

Add message types for:

- start onboarding/check-in session,
- fetch next concierge question,
- submit answer,
- fetch profile summary/consent status,
- acknowledge/skip/defer question.

Message design keeps clients thin and daemon logic centralized.

### 5) UI Adapters

- TUI: first-run concierge modal/panel after provider setup readiness.
- React/Electron: first-run concierge panel in app shell flow.
- Settings: “About You” section to view profile snapshot, consent toggles, and next check-in status.

All adapters call protocol APIs; no local profile business logic.

## Data Model and Persistence (SQLite First)

SQLite is source of truth. Proposed tables:

- `operator_profile_fields`
  - `field_key TEXT`
  - `field_value_json TEXT`
  - `confidence REAL`
  - `source TEXT` (`onboarding`, `passive`, `checkin`, `manual_edit`)
  - `updated_at INTEGER`

- `operator_profile_consents`
  - `consent_key TEXT` (`proactive_suggestions`, `passive_learning`, `weekly_checkins`)
  - `granted INTEGER`
  - `updated_at INTEGER`

- `operator_profile_events`
  - event log for answers, inference updates, prompt decisions, skips, deferrals.

- `operator_profile_checkins`
  - scheduling and execution metadata:
  - `kind TEXT` (`weekly`, `contextual`)
  - `scheduled_at INTEGER`
  - `shown_at INTEGER`
  - `status TEXT`

`USER.md` synchronization:

- daemon renders a compact, stable profile summary from DB,
- writes through existing memory infrastructure,
- stores sync metadata (last render hash/time) to avoid unnecessary rewrites,
- if sync fails, DB remains canonical and sync is retried.

## End-to-End Data Flow

1. First TUI/React launch triggers `StartOperatorOnboarding`.
2. Daemon checks profile completeness and consent state.
3. Daemon emits first concierge question.
4. Client renders question; user answers/skips.
5. Daemon stores answer in SQLite, updates confidence/completeness.
6. Loop until onboarding minimum completeness threshold reached.
7. Daemon updates `USER.md` derived summary.
8. Runtime passive learning updates DB over time.
9. Weekly scheduler + contextual detector trigger micro-check-ins.
10. Concierge asks short follow-up questions only when needed.
11. Proactive suggestions execute only when consented.

## Behavioral Policy

- Onboarding is one-time in full form.
- Follow-up questions are occasional:
  - weekly check-in (time-based),
  - contextual check-in (confidence gap/behavior shift).
- Proactive suggestion/news behavior is opt-in only.
- User can decline or defer any question without blocking core product use.

## Error Handling

- No silent failures.
- If DB read/write fails:
  - return protocol error,
  - surface non-blocking warning in TUI/React,
  - allow retry.
- If `USER.md` sync fails:
  - persist profile data anyway,
  - mark sync-dirty state,
  - retry on next eligible cycle,
  - show status in logs/settings diagnostics.
- If scheduling fails, default to contextual asks only until scheduler recovers.

## Migration and Compatibility

- Additive SQLite migrations only.
- Existing `~/.tamux` and protocol behavior stays compatible.
- If new tables absent/corrupt, daemon initializes safely and reports explicit errors.
- Existing operator model remains usable; new profile layer complements rather than breaks it.

## Testing Strategy

### Daemon Unit Tests

- profile field updates and confidence math,
- question selection and fatigue caps,
- weekly schedule eligibility,
- contextual trigger thresholds,
- consent gating for proactive actions,
- `USER.md` renderer determinism and truncation rules.

### Integration Tests

- protocol roundtrip for onboarding/check-in messages,
- migration tests on existing DB fixtures,
- sync-dirty recovery path for `USER.md` write failures.

### Client Smoke Tests

- first-run onboarding appears in TUI/React,
- one-question-at-a-time UX,
- skip/defer flow,
- weekly prompt visibility behavior,
- settings “About You” state reflection.

## Rollout Phasing (for planning handoff)

1. Protocol + SQLite schema + daemon domain scaffolding.
2. Onboarding interview engine + first-run triggers (TUI/React).
3. Passive learning + confidence/resonance.
4. Weekly/contextual check-ins.
5. `USER.md` sync pipeline and diagnostics.
6. Settings transparency surfaces and consent controls.

## Open Decisions Resolved in Brainstorming

- Scope: first-session concierge onboarding after first TUI/React run, then passive learning.
- Consent model: opt-in for proactive suggestions.
- Onboarding recurrence: full interview one-time; follow-up micro-questions later.
- Follow-up cadence: weekly check-in plus contextual triggers.
- Persistence: SQLite canonical; `USER.md` generated/synced derivative.
