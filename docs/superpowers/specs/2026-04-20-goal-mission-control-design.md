# Goal Mission Control Design

Date: 2026-04-20
Status: Proposed
Scope: `crates/amux-tui`

## Summary

The current goal experience is a decorated task/detail view. It is useful for passive inspection, but it breaks down as soon as the operator needs to steer work, inspect the exact execution thread, change agent assignments, or trust the header metadata. The UI currently mixes goal orchestration with inline chat assumptions, starts goals without an explicit agent configuration step, and shows header context values that do not always reflect the thread actually doing the work.

This design replaces the patch-oriented goal pane with a dedicated `Goal Mission Control` workflow. Mission Control becomes the operator-facing surface for starting, observing, steering, and reconfiguring a goal run. Threads remain a separate surface for message-level intervention. Mission Control owns orchestration state; Threads owns conversation state.

## Product Goals

1. Separate Goals from Threads as first-class surfaces with clear responsibilities.
2. Let the operator jump from a goal into the exact active execution thread and return safely.
3. Require an explicit preflight agent/model configuration step before starting a goal.
4. Reuse the previous goal's agent settings as defaults when available.
5. Allow agent/provider/model/role edits during a running goal with operator-safe guardrails.
6. Bind the header provider/model/context window to the active execution thread while Mission Control is focused.
7. Create an architecture that can absorb future operator controls without collapsing into ad hoc patches.

## Non-Goals

1. Replacing the general `/threads` conversation experience.
2. Redesigning the daemon scheduling model in this phase.
3. Building a fully new backend orchestration protocol before the TUI surface is clarified.

## Core Interaction Model

### Surfaces

- `Goal Mission Control`
  - Orchestration view for a single goal run.
  - Shows step board, live execution feed, agent roster, runtime controls, thread routing, approvals, and dossier.
- `Threads`
  - Message-level intervention surface.
  - Used when the operator wants to steer a specific execution thread directly.

### Contract Between Surfaces

- Goals do not implicitly collapse back into inline chat.
- Mission Control can open the exact current execution thread in `/threads`.
- `/threads` shows a `Return to goal` affordance when entered from Mission Control.
- Returning to the goal restores the same goal run and, when possible, the same step/selection anchor.

## Mission Control Layout

Mission Control should stop behaving like a long, plain text transcript. It should be organized into stable operator regions:

1. `Goal Overview`
   - Goal title, status, current step, approval state, primary controls.
2. `Agent Roster`
   - Main agent and role assignments.
   - Provider, model, reasoning effort, enabled state, inheritance markers.
3. `Execution Feed`
   - Live tool calls, file changes, thread switches, handoffs, messages, and errors.
4. `Thread Router`
   - Root thread, active execution thread, known worker threads, and `Open active thread`.
5. `Step Board`
   - Ordered steps, ownership, state, retry/rerun/reassign controls.
6. `Dossier / Evidence`
   - Current planning notes, reports, checkpoints, and artifacts.

These areas may share vertical space in the TUI, but the information architecture should remain stable so operators know where to look.

## Goal Start: Preflight Launch Flow

### Problem

`GoalComposer` currently accepts text and immediately sends `StartGoalRun`. That is too thin for an operator workflow where model choice and role assignment materially affect cost, speed, and quality.

### New Flow

Before launching a goal, the operator enters a Mission Control preflight screen.

Sections:

1. `Goal Prompt`
   - Editable goal text.
2. `Primary Agent`
   - Provider, model, reasoning effort.
3. `Role Assignments`
   - Planner, executor, reviewer, researcher, compactor, and other enabled workers/subagents.
4. `Preset Source`
   - Indicates whether defaults came from:
     - previous goal snapshot,
     - main-agent inheritance,
     - or a saved default preset.
5. `Launch Actions`
   - Start now.
   - Reset all roles to main agent.
   - Save current assignment as future default.

### Default Resolution

Default order:

1. Most recent goal-run assignment snapshot, if available.
2. Saved goal-launch preset, if introduced later.
3. Main-agent inheritance for all roles.

When previous-goal defaults are loaded, the UI must say so explicitly and allow one action to reset to simple inheritance.

## Runtime Editing

### Goal

The operator must be able to change agent assignments during a run, not only before it starts.

### Editable Fields

- provider
- model
- reasoning effort
- role assignment
- enabled/disabled state
- inherit-from-main toggle

### Safety Rules

- Default behavior: changes affect future work only.
- If the edit impacts the currently running step or active thread, show a confirmation with choices:
  - apply on next turn,
  - reassign active step,
  - restart/requeue active step on the new agent/model.
- In-flight tool execution must never be silently mutated.
- Mission Control must differentiate:
  - `live now`
  - `pending next turn`
  - `stale / fallback`

## Thread Steering

### Primary Operator Action

Mission Control adds `Open active thread` as a first-class control.

Behavior:

1. Resolve the exact active execution thread.
2. Switch to `/threads`.
3. Select that thread immediately.
4. Display `Return to goal`.

If no active execution thread is known:

1. Fall back to the goal root thread.
2. Tell the operator that a fallback was used.

### Return Path

When Threads was opened from Mission Control:

- Preserve the source goal run ID.
- Preserve step selection if available.
- Preserve useful scroll/selection anchors when practical.

## Header Binding

### Problem

The header currently uses a mixture of active conversation state, goal owner profile fallback, and generic config defaults. In Mission Control this can result in a context window that does not represent the thread actually consuming tokens.

### New Resolution Order

When Mission Control is focused, the header should resolve in this order:

1. active execution thread runtime metadata
2. goal root thread runtime metadata
3. launch assignment snapshot
4. generic config defaults

Current-step or planner owner-profile hints may still be used to keep the header label legible when thread metadata is partial, but they must not displace the thread-first provider/model/context-window resolution above.

Values bound through this path:

- provider
- model
- reasoning effort
- total tokens
- current active-window tokens
- context window
- compaction threshold / target display

If the header is using fallback values instead of live runtime values, the UI should indicate that fact rather than pretending the values are live.

## Data Model Changes

### Goal Run State

Extend the goal-facing state to track:

- `root_thread_id`
- `active_thread_id`
- `execution_thread_ids`
- `launch_agent_assignment_snapshot`
- `runtime_agent_assignments`
- `pending_runtime_assignment_changes`

### Agent Assignment Shape

Each role assignment should include:

- role ID
- enabled flag
- provider
- model
- reasoning effort
- inherit-from-main flag
- last-updated source (`launch`, `runtime edit`, `fallback`)

### Return Navigation State

Add a lightweight navigation memory for:

- source goal run ID
- source step ID or selection anchor
- source pane context

This lets `/threads` provide a reliable `Return to goal` behavior.

## Failure Handling

1. If the active execution thread disappears or is not hydrated yet:
   - show `thread unavailable`,
   - fall back to root thread,
   - avoid presenting stale metadata as live.
2. If a previous-goal preset references a missing provider/model:
   - degrade only that broken field,
   - keep the rest of the preset,
   - mark the broken entries visually.
3. If a runtime edit conflicts with current execution:
   - require explicit operator choice.
4. If no live runtime metadata exists yet:
   - use launch snapshot,
   - mark header as fallback.

## Implementation Slices

### Slice 1: Mission Control Foundation

- Replace the thin goal composer/start flow with a dedicated Mission Control preflight.
- Introduce goal launch assignment snapshot state.
- Keep current goal detail rendering working while the layout is migrated.

### Slice 2: Thread Routing

- Track active/root/execution thread IDs for goals.
- Add `Open active thread`.
- Add `Return to goal` from `/threads`.

### Slice 3: Runtime Agent Roster

- Render editable agent assignments in Mission Control.
- Support future-turn edits.
- Add confirmation flow for active-step reassignment/restart.

### Slice 4: Header Rebinding

- Resolve header metadata from active execution thread for Mission Control.
- Add fallback-state indication.

### Slice 5: Layout Upgrade

- Promote Mission Control regions from plain stacked text to stable operator panels.
- Ensure liveness for tool calls, file changes, handoffs, and execution ownership.

## Verification Plan

Tests should cover:

1. Goal preflight loads previous-goal defaults when available.
2. Goal preflight falls back to main-agent inheritance when no snapshot exists.
3. Starting a goal records launch assignment snapshot and opens Mission Control.
4. Runtime edit updates future work only by default.
5. Active-step reassignment/restart requires confirmation.
6. `Open active thread` switches to `/threads` and selects the exact active execution thread.
7. `Return to goal` restores the originating goal run and step selection.
8. Mission Control header follows active execution thread metadata.
9. Header fallback order behaves correctly when live thread data is missing.
10. Missing providers/models in reused presets degrade gracefully and stay editable.

## Open Questions

1. Whether role taxonomy should be fixed (`planner`, `executor`, `reviewer`, etc.) or partially dynamic from daemon capabilities.
2. Whether previous-goal defaults should be global, per-operator, or per-goal-family.
3. Whether runtime agent edits should be applied directly through daemon commands or staged locally until confirmed.

## Recommendation

Proceed with the full Mission Control architecture, not an incremental patch series. The current issues all stem from the same structural problem: goal orchestration is being forced into a pane model built around conversations. The long-term fix is to make goal control a first-class workflow with explicit preflight, runtime roster editing, thread routing, and active-thread header binding.
