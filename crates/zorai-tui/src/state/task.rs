// Local wire type copies (will be replaced by crate::wire imports in Task 9)
// These mirror the types in state.rs
#![allow(dead_code)]

include!("task_parts/task_status_to_task_state.rs");
include!("task_parts/new_to_reduce.rs");
include!("task_parts/goal_step_todo_thread_ids_to_merge_usize_field.rs");
include!("task_parts/merge_goal_run_dossier.rs");
#[cfg(test)]
#[path = "tests/task.rs"]
mod tests;
