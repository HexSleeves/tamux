# Thread Agent Handoff Design

## Goal

Allow any agent persona to become the active responder for an existing operator thread through an explicit daemon-managed handoff flow, while keeping the operator experience as one continuous conversation.

## Scope

- Add first-class daemon state for active responder ownership on a thread.
- Support handoff across all agent personas, not only the built-in main and concierge agents.
- Support stacked handoffs such as `Svarog -> Radogost -> Weles -> Radogost -> Svarog`.
- Require a structured handoff summary on every switch.
- Route future operator messages to the current active responder instead of always to the thread originator.
- Show operator-visible system events for handoffs with collapsed, expandable summaries.
- Require approval for autonomous agent-initiated handoffs outside yolo mode.

## Non-Goals

- Do not split the operator-visible conversation into separate top-level threads per agent.
- Do not redesign the broader task approval system beyond what is needed for handoff approval.
- Do not replace existing internal DM or spawned-subagent mechanisms.
- Do not make handoff summaries the only source of truth for thread history; raw message history remains authoritative.
- Do not force every agent interaction to become a handoff. Internal coordination can still use the existing direct agent messaging path.

## Problem Statement

The current system already has:

- canonical identities for built-in and specialist personas
- internal agent-to-agent messaging
- visible thread metadata such as `agent_name`
- daemon-managed thread and message persistence

Current gap:

- a thread does not have a first-class active responder model
- there is no durable responder stack for nested handoffs
- there is no built-in tool for one agent to transfer ownership of a user conversation to another
- there is no return path that summarizes delegated work back to the previous responder
- the operator cannot see structured handoff events inside the conversation

As a result, switching who is "responsible to answer" is currently implicit, lossy, or purely cosmetic. The requested behavior needs explicit daemon-owned routing state and structured summaries so handoffs are reversible and resumable.

## Design Principles

- Keep one operator-facing conversation.
- Make responder ownership explicit and durable.
- Require summaries at every ownership boundary.
- Prefer reversible stack-based routing over ad hoc agent swaps.
- Preserve operator visibility with concise system events.
- Treat autonomous handoff as a governed action, not a free side effect.

## User Experience

The operator still sees one thread. What changes is:

- the thread shows the current active responder
- a handoff inserts a system event such as `Svarog handed this thread to Weles`
- the event contains a collapsed summary block that can be expanded
- future operator messages are answered by the new active responder
- when control returns, the conversation shows a matching return event with a return summary

The visible thread title remains stable. Agent ownership changes through metadata and system events, not by fragmenting the conversation into separate visible threads.

## Core Model

Each operator-visible thread becomes a conversation anchor with explicit responder routing.

### Thread-level metadata

Add durable metadata for:

- `origin_agent_id`: agent that originally owned the thread
- `active_agent_id`: agent currently responsible for replying
- `handoff_stack`: ordered stack of responder frames
- `linked_handoff_thread_ids`: internal thread ids created for handoff contexts
- `pending_handoff_approval_id`: optional approval gate for an in-flight autonomous switch

### Responder frame

Each frame in `handoff_stack` should include:

- `agent_id`
- `agent_name`
- `entered_at`
- `entered_via_handoff_event_id`
- `linked_thread_id`

The top of the stack is always the active responder.

### Handoff event

Every push or return creates a structured event record:

- `id`
- `thread_id`
- `kind`: `push_handoff` or `return_handoff`
- `from_agent_id`
- `to_agent_id`
- `requested_by`: `user` or `agent`
- `reason`
- `summary`
- `linked_thread_id`
- `approval_id`
- `stack_depth_before`
- `stack_depth_after`
- `created_at`
- `approved_at`
- `completed_at`
- `failed_at`
- `failure_reason`

This data should live as structured thread metadata and also be projected into the visible timeline as a system event.

## Linked Handoff Threads

The system should not route the target agent directly from the visible thread transcript alone. Each ownership change creates or updates a linked internal handoff thread for the target agent.

That internal thread receives a synthesized handoff package containing:

- current task summary
- operator intent
- unresolved questions
- open todos or work context when available
- important recent decisions
- minimal recent transcript needed for continuity

When control returns, the current active agent must send a return summary to the previous frame's linked thread before the stack is popped. This gives the previous responder a concise resume package instead of forcing full transcript replay.

## Routing Rules

Operator messages always enter the visible primary thread, but daemon routing sends them to `active_agent_id`.

### Push handoff

1. Current active agent decides or is asked to transfer ownership.
2. It invokes a built-in handoff tool with target agent, reason, summary, and request source.
3. If the handoff is autonomous and the runtime is not in yolo mode, the daemon creates an approval item and blocks activation.
4. After approval or auto-allow, the daemon creates or updates the linked handoff thread for the target.
5. The daemon writes the structured handoff package into that linked thread.
6. The responder stack is pushed with the target frame.
7. `active_agent_id` changes to the target.
8. The visible thread receives a system handoff event with a collapsed summary payload.

### Return handoff

1. Current active agent invokes a built-in return handoff tool.
2. The daemon verifies stack depth is greater than one.
3. The current agent provides a return summary of work completed, unresolved items, and next recommended action.
4. The daemon writes that summary to the previous responder's linked thread.
5. The current frame is popped.
6. `active_agent_id` changes to the previous frame's agent.
7. The visible thread receives a return system event with collapsed summary data.

## Built-In Tooling

Add a first-class handoff tool available to all agents. The tool contract should be explicit enough that the daemon can validate state transitions instead of inferring intent from free text.

Recommended shape:

- `action`: `push_handoff` or `return_handoff`
- `target_agent_id`: required for `push_handoff`
- `reason`
- `summary`
- `requested_by`
- `user_visible_note`: optional concise label for timeline projection

This tool is distinct from `message_agent`.

`message_agent` is for coordination without changing who owns the operator conversation.

`handoff_thread_agent` changes the active responder for future operator turns and updates the responder stack.

## Approval Model

Autonomous handoff is allowed, but governed.

### Approval behavior

- Agent-initiated `push_handoff` requires approval outside yolo mode.
- Agent-initiated `push_handoff` auto-executes in yolo mode.
- User-requested `push_handoff` can execute immediately unless another approval policy blocks it.
- `return_handoff` should execute without extra approval by default because it reduces delegation depth.

The approval payload should include:

- source agent
- target agent
- reason
- summary preview
- current stack depth

If approval is denied, the active responder does not change and the thread receives a visible system event noting the rejection.

## Failure Handling

Reject invalid transitions:

- target agent equals current active agent
- return requested when stack depth is one
- target agent is disabled or unavailable
- required summary is empty
- another pending handoff approval already exists for the thread

Failure behavior:

- If validation fails before approval, no state changes and the current agent receives a tool error.
- If approval is granted but activation fails, keep the current active responder unchanged and emit a visible failure system event.
- If activation succeeds but the target agent later fails to continue, the stack remains accurate, the linked handoff thread records the failure, and the active agent can either recover or return ownership explicitly.

## Persistence And Recovery

The handoff stack and handoff events must persist with thread state so restart or reload does not lose responder ownership.

On reload:

- reconstruct `active_agent_id` from persisted thread metadata
- restore `handoff_stack`
- restore pending approval state if still unresolved
- reconnect visible system events to structured handoff records

Compaction must preserve enough routing data to keep future handoffs and returns valid. Handoff metadata cannot live only inside raw assistant prose.

## Frontend Projection

The frontend thread model currently exposes `agent_name`, which is not enough for stacked routing. Extend the thread projection to include:

- active responder identity
- origin responder identity
- handoff history metadata for system timeline rendering
- linked handoff state when needed for debugging or inspection

The timeline system event should render:

- a one-line summary of the switch
- a collapsed summary panel
- expand/collapse interaction
- approval status when relevant

The operator should not need to navigate to separate threads just to follow ownership changes.

## Current Code Impact

This feature naturally sits across the existing daemon thread/message loop and frontend thread hydration path.

Likely touch points:

- daemon thread metadata and persistence
- send-message routing so operator turns target `active_agent_id`
- tool catalog and tool execution for the new handoff tool
- approval plumbing for autonomous non-yolo switches
- frontend thread hydration and system event rendering
- compaction so responder ownership survives summarization

## Testing Strategy

Add daemon tests for:

- creating a new thread initializes `origin_agent_id`, `active_agent_id`, and a one-frame responder stack
- `push_handoff` pushes a new frame and changes routing
- `return_handoff` pops a frame and restores previous routing
- nested transitions such as `Svarog -> Radogost -> Weles -> Radogost -> Svarog`
- autonomous non-yolo handoff creates approval instead of switching immediately
- approving a pending handoff completes activation and linked thread delivery
- denying a pending handoff preserves current ownership
- invalid transitions fail without mutating thread state
- persisted thread reload reconstructs the responder stack and active responder correctly
- compaction preserves responder ownership and handoff history

Add frontend tests for:

- hydrated remote threads expose active responder metadata correctly
- system events render handoff and return summaries in collapsed form
- approval and failure states are rendered clearly

## Rollout Notes

Implement the daemon state model first, then tooling and approval flow, then frontend projection. The visible UI should remain a thin projection over daemon-owned routing state rather than a second source of truth.
