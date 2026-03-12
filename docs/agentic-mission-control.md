# Agentic Mission Control

## Core User Journey

1. Session initialization
   - User opens a workspace and a daemon-backed PTY attaches to the active pane.
   - MEMORY and USER context are hydrated into the mission panel.
2. Task delegation
   - Human types directly in the REPL or triggers managed execution from Mission Control or Session Vault replay.
   - The daemon validates, snapshots, serializes, and policy-checks managed work.
3. Agent reasoning
   - INNER_MONOLOGUE and SCRATCHPAD payloads stream into the Reasoning Trace plane.
   - Operational, cognitive, and contextual telemetry are persisted.
4. HITL approval
   - High-risk managed commands pause behind the structured Security Interceptor modal.
   - Approval decisions flow back to the daemon.
5. Task completion
   - Execution completes, telemetry is written to JSONL + SQLite/FTS, and a replayable timeline entry is added.

## Low-Fidelity Wireframes

### Mission Control

```text
+----------------------------------------------------------------------------------+
| Title / Status / Workspace Context                                               |
+-------------------------+--------------------------------------+----------------+
| Sidebar / Surfaces      | Shared REPL / Terminal Pane         | Mission Panel  |
|                         |                                      |                |
|                         |  [Shared Cursor Badge]              | Threads        |
|                         |                                      | Trace          |
|                         |  live output / managed execution     | Context        |
|                         |                                      | Graph          |
+-------------------------+--------------------------------------+----------------+
| Status Bar: daemon | mission | trace | ops | logs | vault | search | settings   |
+----------------------------------------------------------------------------------+
```

### Approval Interceptor

```text
+------------------------------------------------------------------+
| Approval Required                              [critical/high]    |
| High-impact shell command intercepted                            |
|                                                                  |
| Command                                                          |
|   rm -rf ...                                                     |
|                                                                  |
| Blast Radius | Scope                                             |
|                                                                  |
| Reasons: destructive recursive delete | network access requested |
|                                                                  |
|                           [Deny] [Allow Once] [Allow For Session]|
+------------------------------------------------------------------+
```

### Time Travel Vault

```text
+----------------------------------------------------------------------------+
| Search / Filters / Timeline Mode                                           |
| Timeline Cards                                                             |
| Scrubber ----------------------------------------------------------------- |
| Target: checkpoint or command                                              |
|                                                                            |
| Left: transcript list or timeline rows  | Right: immutable StaticLog view  |
+----------------------------------------------------------------------------+
```

## Color Taxonomy

- Human input: `var(--success)`
- Agent-managed execution: `var(--accent)`
- Reasoning trace: cool blue text on translucent navy surfaces
- Security warnings: `var(--warning)`
- Critical danger: `var(--danger)`
- Background hierarchy:
  - canvas: `--bg-primary`
  - panels: `--bg-secondary`, `--bg-surface`
  - overlays: translucent glass with `--glass-border`

## Z-Axis Depth Model

- Z0: terminal canvas and static content
- Z1: split-pane chrome, tabs, context HUD, shared cursor badge
- Z2: mission side panel and graph surfaces
- Z3: modal overlays including the Security Interceptor
- Z4: future spatial export layer for detached DAG planes and sub-agent lanes

## BFO Guidance

- Live terminal remains xterm-backed and mutable.
- Historical content is cast into StaticLog to avoid deep re-render churn.
- Reasoning trace is chunked into immutable cards rather than a single giant text node.
- Session Vault timeline limits event density and exposes a scrubber rather than replaying the full corpus.

## Shared Cursor Rules

- Human cursor mode is shown for direct keyboard input in the REPL.
- Agent cursor mode is shown while daemon-managed commands are starting or running.
- Approval mode is shown while a structured approval request is pending.
- Idle mode is shown when the lane is quiescent.