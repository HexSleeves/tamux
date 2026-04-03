# TUI Approval Center Design

## Goal

Make approvals impossible to miss in the TUI by adding a first-class Approval Center, while preserving an immediate blocking path for approvals that stop work in the currently active thread.

## Scope

- Add a first-class Approval Center view to the TUI.
- Keep a blocking approval modal for approvals that originate from the current thread.
- Add a non-blocking arrival banner for approvals that originate from other threads.
- Add mouse interactions for approval list rows, filters, detail actions, and arrival banners.
- Hydrate unresolved approvals from existing task state so reconnecting or restarting the TUI does not hide pending approvals.
- Expose a global pending queue with filters for `current thread`, `current workspace`, and `all pending`.

## Non-Goals

- Do not change daemon approval semantics or policy decisions.
- Do not merge approvals into the generic notifications inbox as the canonical workflow.
- Do not redesign the full TUI layout or notification system beyond approval-related entry points.
- Do not change frontend/Electron approval UX in this design.
- Do not introduce approval batching or bulk approve/deny actions.

## Problem Statement

The current TUI receives approval events and already has an approval overlay, but the operator can still miss approvals in practice. A blocked task may remain in `awaiting_approval` with no reliable, persistent operator-facing queue to discover and resolve it later.

This causes two concrete failures:

- a current session can stall behind an approval if the modal is not seen at the right moment
- background-thread approvals can block lane availability and pile up queued work without a dedicated operator work queue

The WELES governance case that motivated this work demonstrates the failure mode clearly: one approval-gated WELES task blocked the `daemon-main` lane, and dozens of additional WELES tasks accumulated behind it waiting for lane availability. The daemon and task state were correct; the operator affordance was insufficient.

## Current State

The TUI already has partial approval handling:

- [events_connection.rs](/home/mkurman/gitlab/it/cmux-next/crates/amux-tui/src/app/events/events_connection.rs) listens for `ApprovalRequired` and `ApprovalResolved` events.
- On `ApprovalRequired`, it upserts a `PendingApproval` into local approval state and always pushes `ModalKind::ApprovalOverlay`.
- [state/approval.rs](/home/mkurman/gitlab/it/cmux-next/crates/amux-tui/src/state/approval.rs) stores pending approvals as a simple FIFO vector and exposes only `current_approval()`.
- [widgets/approval.rs](/home/mkurman/gitlab/it/cmux-next/crates/amux-tui/src/widgets/approval.rs) renders a single approval overlay.
- [modal_handlers.rs](/home/mkurman/gitlab/it/cmux-next/crates/amux-tui/src/app/modal_handlers.rs) resolves only the current approval from that overlay.
- The TUI already has a notifications modal and local notification state, but approvals are not modeled as a first-class queue in that system.

Current gaps:

- approval visibility is modal-only, not queue-based
- there is no persistent approval inbox/work queue
- all approvals behave the same, regardless of whether they come from the current thread or another thread
- reconnecting depends on whether the live event was seen; unresolved approvals are not reconstructed into a dedicated operator UI
- there is no first-class mouseable approval workflow

## Design Principles

- Approvals are workflow state, not passive notifications.
- The daemon task state remains canonical; the TUI derives approval visibility from events plus task hydration.
- Current-thread approvals should interrupt because the operator is already blocked on them.
- Cross-thread approvals should not steal focus, but they must remain visible until resolved.
- The operator should be able to resolve any pending approval from a single durable queue.

## Proposed UX Model

Approvals will surface through three related interfaces:

1. Approval Center
2. Current-thread blocking approval modal
3. Cross-thread non-blocking arrival banner

These surfaces share the same approval payload and resolution actions. The Approval Center is the durable home; the modal and banner are just entry points optimized for urgency and context.

## Approval Center

The Approval Center is a first-class TUI view dedicated to pending approvals.

### Core behavior

- Open and close with a dedicated shortcut such as `Ctrl+A`.
- Show a global badge/count in existing chrome when any approvals are pending.
- Default to the `all pending` filter.
- Support quick filters:
  - `current thread`
  - `current workspace`
  - `all pending`
- Display pending approvals in a list with a detail pane.

### Queue row contents

Each row should show:

- task title or approval title
- source thread title or thread id fallback
- workspace label when known
- risk level
- short reason summary
- relative arrival time

Rows must visually distinguish unseen items and current-thread items.

### Detail pane contents

Selecting a row opens a detail pane that shows:

- approval id
- thread title and jump target
- task title
- command when the approval is command-backed
- rationale when available
- blast radius
- reasons list
- risk level
- received timestamp
- action buttons:
  - `Approve once`
  - `Approve session` when supported
  - `Deny`

### Recent resolution feedback

After a decision, remove the approval from pending immediately and optionally keep a short-lived resolved section or status flash so the operator receives visible confirmation.

## Interrupt Behavior

### Current-thread approvals

If the approval belongs to the currently active thread, the TUI should open a blocking modal immediately.

Rules:

- the modal takes focus
- the current-thread workflow cannot continue until the modal is resolved or intentionally dismissed into the Approval Center only if the action remains clearly blocked
- the same approval is still present in the Approval Center

This keeps the existing strong interrupt behavior for the thread the operator is actively driving, but grounds it in a durable queue.

### Cross-thread approvals

If the approval belongs to a different thread, the TUI should not steal focus.

Instead:

- show a non-blocking arrival banner
- include thread title, task title, risk, and a shortcut hint to open the Approval Center
- allow click-to-open behavior that focuses the Approval Center on the relevant approval
- keep the approval badge highlighted until the operator has viewed or resolved it

The banner is informational, not modal. Losing the banner must not lose the approval.

## Mouse Interaction

Approval Center and approval entry points must support mouse usage in addition to keyboard shortcuts.

### Banner

- click banner to open the Approval Center focused on that approval
- click close affordance to dismiss only the banner presentation, not the pending approval itself

### Approval Center

- click filter chips to switch filters
- click queue rows to select an approval
- click thread target to jump to the source thread
- click action buttons to approve once, approve session, or deny
- scroll queue and detail panes independently using wheel/trackpad

### Current-thread modal

- click buttons for `Approve once`, `Approve session`, and `Deny`
- click thread/context affordance only if it does not undermine the blocking nature of the modal

## State Model

The existing approval state is too narrow because it only models a FIFO list plus allowlist behavior. It needs to become a queue-oriented view model.

### Extend approval state

Replace the current single-purpose structure with approval entries that include:

- `approval_id`
- `task_id`
- `task_title`
- `thread_id`
- `thread_title`
- `workspace_id` or workspace label when known
- `command`
- `rationale`
- `risk_level`
- `blast_radius`
- `reasons`
- `received_at`
- `seen_at`
- `origin_scope`
  - current thread
  - current workspace
  - other

The state should support:

- list all pending approvals
- get current-thread pending approval
- filter by thread/workspace/global scope
- mark seen
- resolve by id
- optionally keep a small resolved-history ring buffer for confirmation UX

### Canonical source of truth

Pending task state remains canonical. The TUI approval store is a projection that must be reconstructable from task snapshots.

That means:

- live `ApprovalRequired` events improve immediacy
- `TaskList` and `TaskUpdate` hydration repairs missed events and reconnect scenarios
- `ApprovalResolved` and task status updates clear the same approval from projection state

## Hydration And Reconnect Behavior

On connect and on task snapshot updates, the TUI should scan tasks for:

- `status == awaiting_approval`
- `awaiting_approval_id != null`

For each such task, create or refresh the corresponding approval entry in Approval Center state, even if no live approval event was seen by the current UI session.

This is required to fix the exact failure mode the operator reported: a valid approval existed in daemon/task state but there was no reliable way to discover and act on it in the TUI.

### Hydration detail

Hydration should populate as much approval metadata as possible from:

- approval event payloads when present
- task title and thread linkage from task state
- fallback risk parsing from command text only when richer data is absent

If the daemon task snapshot does not contain full reasons/rationale, the Approval Center should still render a usable fallback entry rather than omitting the approval.

## Keyboard Interaction

Recommended bindings:

- `Ctrl+A`: open or close Approval Center
- `j/k` or arrows: move through approval queue
- `Enter`: open detail or focus action group
- `a`: approve once
- `s`: approve session when available
- `d`: deny
- `Esc`: close in priority order:
  - current blocking modal
  - cross-thread banner
  - Approval Center

If the final keybinding set conflicts with existing bindings, follow current TUI conventions, but preserve the dedicated global shortcut requirement.

## Rendering And Layout

Approval Center should be a first-class view, not a nested section of notifications.

Suggested structure:

- header:
  - title
  - pending count
  - filters
- left pane:
  - queue list
- right pane:
  - approval details and actions

Visual priorities:

- high-contrast risk markers
- clear differentiation of current-thread approvals
- obvious primary actions
- visible empty state with “no pending approvals”

The existing single-approval overlay renderer in [widgets/approval.rs](/home/mkurman/gitlab/it/cmux-next/crates/amux-tui/src/widgets/approval.rs) should be refactored into shared approval detail rendering primitives so the modal and Approval Center do not diverge.

## Event Handling Changes

### ApprovalRequired

Current behavior always pushes the approval modal. Replace it with conditional routing:

- upsert approval into Approval Center state
- determine whether the approval belongs to the active thread
- if active thread:
  - push blocking approval modal
- else:
  - show cross-thread arrival banner
  - increment badge/highlight

### ApprovalResolved

On resolution:

- remove approval from Approval Center pending state
- clear current-thread modal if it is showing the resolved approval
- clear or collapse any matching banner
- update badge/count

### Task hydration

Task list and task update handlers should also synchronize Approval Center state from `awaiting_approval` tasks. This should happen even if the explicit approval event was missed.

## Notification Integration

Notifications remain separate from approvals.

They may still provide optional entry points, for example:

- an approval-related notification that links to Approval Center
- a status line or notifications badge that reflects pending approvals

But they must not become the canonical place where approvals live. The Approval Center owns approval workflow.

## Data And Thread Matching

Approval entries need better context than the current state holds.

The TUI should enrich approval entries by matching the approval id against:

- tasks with matching `awaiting_approval_id`
- known thread ids and titles
- current active thread
- workspace associations when available in the task or thread state

This enables:

- current-thread vs cross-thread behavior
- thread jump links
- useful queue labels
- meaningful filters

## Error Handling And Fallbacks

- If an approval event arrives before the corresponding task snapshot, show the approval with partial information and enrich it later.
- If thread title is unavailable, show thread id.
- If command text is unavailable, render rationale/reasons only.
- If approval resolution fails at daemon level, keep the approval pending and surface an error in status/badge context.
- If the TUI reconnects and finds pending approvals with no prior local state, hydrate them without requiring any special operator action.

## Testing Strategy

Follow TDD for each behavior slice.

### State tests

Extend approval-state tests to cover:

- upsert and deduplication by `approval_id`
- filtering by current thread, current workspace, and all pending
- hydration from `awaiting_approval` task snapshots
- resolving from event and from task update convergence
- seen/unseen transitions

### Event handling tests

Add tests proving:

- current-thread approval opens blocking modal
- cross-thread approval does not open blocking modal
- cross-thread approval creates a banner and queue entry
- reconnect or refresh with `awaiting_approval` task snapshot reconstructs missing approvals

### Rendering tests

Add rendering coverage for:

- Approval Center empty state
- Approval Center list with multiple approvals
- detail pane risk and action rendering
- badge/count state
- cross-thread banner
- current-thread blocking modal

### Keyboard tests

Add tests for:

- Approval Center toggle shortcut
- queue navigation
- approve once / approve session / deny
- close precedence for modal, banner, and view

### Mouse tests

Add tests for:

- clicking banner opens the Approval Center
- clicking filter chips changes scope
- clicking list rows changes selection
- clicking action buttons dispatches resolution commands
- clicking thread link jumps to related thread

### Manual smoke tests

- trigger approval in active thread and confirm blocking modal
- trigger approval in background thread and confirm non-blocking banner plus queue entry
- restart or reconnect the TUI and confirm unresolved approvals are still visible
- resolve approval and confirm the queue, modal, and badge all clear immediately

## Expected TUI File Changes

The exact split may move during implementation, but the design is expected to touch at least:

- `crates/amux-tui/src/state/approval.rs`
- `crates/amux-tui/src/app/events/events_connection.rs`
- `crates/amux-tui/src/app/model_impl_part2.rs`
- `crates/amux-tui/src/app/modal_handlers.rs`
- `crates/amux-tui/src/app/keyboard.rs`
- `crates/amux-tui/src/app/mouse.rs`
- `crates/amux-tui/src/widgets/approval.rs`
- one or more new approval-center widget/view files under `crates/amux-tui/src/widgets/`
- modal/view state definitions that route first-class panels and overlays
- existing TUI tests covering state, modal handling, keyboard, and mouse interaction

Additional files may be required if the current view-routing or chrome/badge rendering is centralized elsewhere.

## Recommendation

Implement Approval Center as a first-class TUI view backed by approval queue state hydrated from both live approval events and `awaiting_approval` task snapshots. Keep a blocking modal only for approvals from the active thread, add a non-blocking cross-thread banner for everything else, and make all approval actions available through both keyboard and mouse so approvals become durable, discoverable operator work instead of transient modal interruptions.
