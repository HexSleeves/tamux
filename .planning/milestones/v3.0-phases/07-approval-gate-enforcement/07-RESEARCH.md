# Phase 7: Approval Gate Enforcement - Research

**Researched:** 2026-03-27  
**Domain:** Agent runtime execution gating (autonomy + uncertainty) across dispatcher/scheduler paths  
**Confidence:** HIGH

## User Constraints

No `*-CONTEXT.md` exists for this phase (`.planning/phases/07-approval-gate-enforcement`), so there are no additional locked user decisions beyond roadmap/requirements scope.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UNCR-08 | If all plan steps are HIGH -> proceed autonomously. Any MEDIUM -> inform operator. Any LOW -> require approval. | Existing gate exists in `goal_planner.rs` (`plan_confidence_gate`) but is bypassable from background dispatcher path; enforcement must move to shared runtime gate and fail-closed checks in dispatcher/scheduler. |
| AUTO-04 | Supervised: agent reports every significant step and waits for acknowledgment. | Current step-boundary gate is set in `enqueue_goal_run_step` but task is already queued and scheduler can still execute; require pre-dispatch acknowledgment checks and explicit resume semantics that only clear gate after acknowledgment. |
</phase_requirements>

## Summary

Phase 7 is a runtime state-machine hardening phase, not a new capability phase. The bypass root cause is architectural: approval/autonomy gates are partially implemented at planning/enqueue time, but the background loop (`gateway_loop -> dispatch_goal_runs + dispatch_ready_tasks`) continues advancing runs/tasks based on queue state without a centralized gate check. As a result, supervised and low-confidence steps can still execute via dispatcher/scheduler paths.

The highest-risk surface is the interaction between `goal_planner::enqueue_goal_run_step` and `dispatcher::dispatch_ready_tasks`. In supervised mode, the goal is set to `AwaitingApproval` **after** the child task is enqueued, so the task remains `Queued` and can be dispatched by scheduler. Additionally, `dispatch_goal_runs` processes all non-terminal runs, including `GoalRunStatus::AwaitingApproval`, which allows advancement attempts while awaiting operator acknowledgment.

Primary implementation strategy: introduce a shared, fail-closed gating function used by **all** runtime entrypoints (planning completion, goal advancement, task dispatch, and resume/control actions), and make scheduler selection explicitly skip tasks whose parent goal run is awaiting approval for autonomy/confidence reasons.

**Primary recommendation:** Centralize approval-gate policy into one runtime guard (`can_advance_goal_run` / `can_dispatch_task`) and require all dispatcher/scheduler transitions to call it before mutating status.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust | 1.94.0 | Runtime and state machine implementation | Existing codebase and all agent runtime modules are Rust-native |
| tokio | 1.50.0 | Async scheduling/dispatch loop | Background dispatch path already built on tokio intervals/spawn |
| serde | 1.0.228 | State serialization for GoalRun/Task state | Existing protocol and persistence model already serde-based |
| amux-protocol (workspace) | 0.1.10 | IPC contract for approval/control messages | Existing client/server approval plumbing depends on this |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tracing | 0.1.44 | Gate decision observability | Add structured logs at each gate deny/allow branch |
| anyhow | 1.0.102 | Error propagation in dispatcher/goal planner | Return actionable errors on illegal transitions |
| uuid | 1.22.0 | Approval/event correlation ids | If adding explicit acknowledgment tokens/events |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Runtime central gate helper | Ad-hoc checks per module | Faster initially, but regressions likely and bypasses reappear |
| Fail-closed scheduler filtering | UI-only “pause” semantics | UI can drift; scheduler still executes unless backend enforces |

**Installation:**
```bash
# Existing workspace uses Cargo; no new package needed for Phase 7.
cargo test -p tamux-daemon --no-run
```

## Architecture Patterns

### Recommended Project Structure
```text
crates/amux-daemon/src/agent/
├── goal_planner.rs      # plan + step enqueue (existing confidence/autonomy gating)
├── dispatcher.rs        # background advancement + task dispatch (bypass surface)
├── task_scheduler.rs    # queue-state and ready-task selection (gate enforcement point)
├── task_crud.rs         # control/resume/approval resolution transitions
└── work_context.rs      # event emission/autonomy filtering visibility
```

### Pattern 1: Centralized Runtime Gate Evaluation
**What:** A single helper determines whether goal/task transitions are legal under current approval/autonomy/confidence state.  
**When to use:** Before every transition from `Queued/Running` to execution in dispatcher/scheduler and goal advancement.  
**Example:**
```rust
// Source: crates/amux-daemon/src/agent/dispatcher.rs
if current_step.task_id.is_none() {
    self.enqueue_goal_run_step(goal_run_id).await?;
    return Ok(());
}
```
This must be guarded by a shared `can_advance_goal_run(...)` check that fails closed when goal run is awaiting acknowledgment.

### Pattern 2: Fail-Closed Status Machine
**What:** Treat `GoalRunStatus::AwaitingApproval` as a hard blocker for advancement and task dispatch unless a specific approval-resolution event clears it.  
**When to use:** In `dispatch_goal_runs`, `dispatch_ready_tasks`, and `select_ready_task_indices`.  
**Example:**
```rust
// Source: crates/amux-daemon/src/agent/dispatcher.rs
goal_runs.iter().filter(|goal_run| {
    !matches!(goal_run.status, GoalRunStatus::Paused | GoalRunStatus::Completed | GoalRunStatus::Failed | GoalRunStatus::Cancelled)
})
```
Current filter includes `AwaitingApproval`; Phase 7 should exclude it by default unless resumed/approved.

### Anti-Patterns to Avoid
- **Gate logic only in planner:** Planner-only checks do not protect scheduler/dispatcher entrypoints.
- **Status-only UI gating:** Client pause indicators are insufficient; backend must block state transitions.
- **Clearing approval fields opportunistically:** Avoid resetting `awaiting_approval_id` except in explicit approval/ack handlers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Distributed gate checks | Repeated `if status == ...` snippets in each function | Shared gate helper(s) consumed by planner + dispatcher + scheduler | Prevents drift and future bypass regressions |
| Approval correlation | New ad-hoc string flags in logs only | Existing `awaiting_approval_id` + structured goal/task events | Already integrated with server/client approval resolution |
| Verification strategy | Manual “click test” only | Deterministic unit/integration tests around dispatcher ticks and control actions | Regression-proof for blocker path |

**Key insight:** This phase is primarily about making illegal transitions impossible, not about adding new UI prompts.

## Common Pitfalls

### Pitfall 1: Supervised gate set too late
**What goes wrong:** Task is enqueued first, goal marked `AwaitingApproval` second, scheduler dispatches queued task before acknowledgment.  
**Why it happens:** `enqueue_goal_run_step` currently queues child task before supervised gate event.  
**How to avoid:** Block enqueue or keep child task blocked until acknowledgment is recorded.  
**Warning signs:** Supervised goal run status shows awaiting approval while child task status is `InProgress`.

### Pitfall 2: Dispatcher advances awaiting-approval runs
**What goes wrong:** `dispatch_goal_runs` continues processing runs that should be stopped.  
**Why it happens:** status filter excludes paused/completed/failed/cancelled, but not awaiting-approval.  
**How to avoid:** Exclude `GoalRunStatus::AwaitingApproval` from active dispatch set unless approval cleared.  
**Warning signs:** Goal events continue while awaiting approval remains set.

### Pitfall 3: Resume action bypasses acknowledgment intent
**What goes wrong:** Generic `"resume"` moves paused runs to running without proving acknowledgment for supervised step gates.  
**Why it happens:** `control_goal_run` currently has coarse action semantics.  
**How to avoid:** Distinguish acknowledgment resume from general pause/resume, or validate gate preconditions before setting running.  
**Warning signs:** Supervised runs can proceed with no associated approval/ack event.

## Code Examples

Verified patterns from repository sources:

### Existing plan confidence gate (partial UNCR-08 implementation)
```rust
// Source: crates/amux-daemon/src/agent/goal_planner.rs
let gate_action = self.plan_confidence_gate(&updated).await;
if gate_action == super::uncertainty::PlanConfidenceAction::RequireApproval {
    current.status = GoalRunStatus::AwaitingApproval;
}
```

### Existing supervised acknowledgment marker (AUTO-04 intent)
```rust
// Source: crates/amux-daemon/src/agent/goal_planner.rs
if super::autonomy::requires_acknowledgment(updated.autonomy_level) {
    goal_run.status = GoalRunStatus::AwaitingApproval;
    goal_run.events.push(make_goal_run_event("autonomy_acknowledgment", ...));
}
```

### Bypass surface in background dispatcher
```rust
// Source: crates/amux-daemon/src/agent/dispatcher.rs
if current_step.task_id.is_none() {
    self.enqueue_goal_run_step(goal_run_id).await?;
    return Ok(());
}
```
No centralized gate check exists here today.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Planner-local gating assumptions | Runtime-wide fail-closed gating across all loops | Phase 7 target | Prevents dispatcher/scheduler bypass |
| “Awaiting approval” as mostly informational state | “Awaiting approval” as strict transition blocker | Phase 7 target | Enforces UNCR-08/AUTO-04 guarantees |

**Deprecated/outdated:**
- Planner-only enforcement for approval/autonomy guarantees.

## Open Questions

1. **What exact operator action is canonical for supervised step acknowledgment?**
   - What we know: `AgentControlGoalRun(action="resume")` exists; no dedicated ack message type for goal-step acknowledgment.
   - What's unclear: Whether resume should count as acknowledgment or whether a distinct action/event is required.
   - Recommendation: Define explicit semantics in backend tests first (resume-as-ack or new action), then align clients.

2. **Should supervised step gates use `awaiting_approval_id` tokenization like managed commands?**
   - What we know: Managed command approval uses `awaiting_approval_id` and `AgentResolveTaskApproval`.
   - What's unclear: Whether supervised acknowledgment should be represented with same token mechanism.
   - Recommendation: Reuse same token model if possible to avoid a second gating mechanism.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | Build/test validation | ✓ | 1.94.0 | — |
| rustc | Compile phase changes | ✓ | 1.94.0 | — |
| node | Frontend/client contract checks | ✓ | v24.13.1 | — |
| npm | Frontend type/bridge checks | ✓ | 11.8.0 | — |

**Missing dependencies with no fallback:**
- None identified.

**Missing dependencies with fallback:**
- None identified.

## Sources

### Primary (HIGH confidence)
- `crates/amux-daemon/src/agent/dispatcher.rs` — background goal/task dispatch behavior and bypass surface.
- `crates/amux-daemon/src/agent/goal_planner.rs` — existing UNCR-08/AUTO-04 gating logic and event emission.
- `crates/amux-daemon/src/agent/task_crud.rs` — goal control/resume and approval resolution transitions.
- `crates/amux-daemon/src/agent/task_scheduler.rs` — scheduler selection and queue-state transitions.
- `crates/amux-daemon/src/agent/work_context.rs` — autonomy event filtering behavior.
- `crates/amux-daemon/src/agent/uncertainty/{mod.rs,domains.rs,confidence.rs}` — domain thresholds and confidence model.
- `.planning/REQUIREMENTS.md` — requirement text for UNCR-08 and AUTO-04.
- `.planning/ROADMAP.md` — phase 7 success criteria and blocker statement.
- `.planning/v3.0-MILESTONE-AUDIT.md` — explicit blocker evidence.

### Secondary (MEDIUM confidence)
- `crates/amux-cli/src/client.rs`, `crates/amux-tui/src/client.rs`, `frontend/src/lib/goalRuns.ts` — client control pathways for goal-run resume/approval interactions.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - versions and tooling validated from `Cargo.lock` + local environment.
- Architecture: HIGH - bypass path confirmed directly in dispatcher/goal_planner/task_scheduler code.
- Pitfalls: HIGH - derived from concrete current state transitions and roadmap blocker evidence.

**Research date:** 2026-03-27  
**Valid until:** 2026-04-03 (runtime behavior can change quickly across active gap-closure phases)

