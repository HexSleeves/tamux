#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingWorkspaceActorPicker {
    target: PendingWorkspaceActorPickerTarget,
    task_id: String,
    mode: workspace_actor_picker::WorkspaceActorPickerMode,
}

