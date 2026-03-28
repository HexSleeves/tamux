# Phase 8: Client Explainability & Divergent Entry - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning
**Mode:** Auto-generated (infrastructure phase — discuss skipped)

<domain>
## Phase Boundary

Close first-party client entrypoint gaps for explainability and divergent sessions so operator flows are usable end-to-end in frontend/Electron/CLI/TUI surfaces.

</domain>

<decisions>
## Implementation Decisions

### the agent's Discretion
All implementation choices are at the agent's discretion for this gap-closure phase. Preserve existing daemon/protocol contracts; focus on wiring and display parity across first-party clients.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Existing daemon/protocol support for explain/divergent flows is already implemented.
- CLI bridge already maps explain/divergent commands and events.
- Electron IPC query handlers and preload bridge methods already exist.

### Established Patterns
- Frontend runtime consumes daemon event stream and writes system messages to active thread.
- Electron bridge resolves query response types via pending-map allowlist.

### Integration Points
- `frontend/src/components/agent-chat-panel/runtime.tsx`
- `frontend/electron/main.cjs`
- `frontend/electron/preload.cjs`
- `crates/amux-cli/src/client.rs`
- `crates/amux-tui/src/*`

</code_context>

<specifics>
## Specific Ideas

No additional product-surface expansion required; close the current E2E render gap for `!explain` and `!diverge*` in frontend while preserving existing CLI/TUI parity.

</specifics>

<deferred>
## Deferred Ideas

None — scope is constrained to phase 8 gap closure and parity verification.

</deferred>
