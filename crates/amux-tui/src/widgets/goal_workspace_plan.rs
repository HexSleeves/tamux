use crate::state::goal_workspace::GoalWorkspaceState;
use crate::state::task::{TaskState, TodoStatus};
use ratatui::text::{Line, Span};

use super::GoalWorkspaceHitTarget;

pub(crate) struct GoalWorkspacePlanRow {
    pub(crate) line: Line<'static>,
    pub(crate) target: Option<GoalWorkspaceHitTarget>,
}

pub(crate) fn build_rows(
    tasks: &TaskState,
    goal_run_id: &str,
    state: &GoalWorkspaceState,
) -> Vec<GoalWorkspacePlanRow> {
    let mut rows = Vec::new();
    for step in tasks.goal_steps_in_display_order(goal_run_id) {
        let expanded = state.is_step_expanded(&step.id);
        rows.push(GoalWorkspacePlanRow {
            line: Line::from(vec![
                Span::raw(if expanded { "▾ " } else { "▸ " }),
                Span::raw(format!("{}. {}", step.order + 1, step.title)),
            ]),
            target: Some(GoalWorkspaceHitTarget::PlanStep(step.id.clone())),
        });

        if expanded {
            for todo in tasks.goal_step_todos(goal_run_id, step.order as usize) {
                rows.push(GoalWorkspacePlanRow {
                    line: Line::from(vec![
                        Span::raw("  "),
                        Span::raw(todo_status_chip(todo.status)),
                        Span::raw(" "),
                        Span::raw(todo.content),
                    ]),
                    target: Some(GoalWorkspaceHitTarget::PlanTodo {
                        step_id: step.id.clone(),
                        todo_id: todo.id,
                    }),
                });
            }
        }
    }

    if rows.is_empty() {
        rows.push(GoalWorkspacePlanRow {
            line: Line::from("No plan yet"),
            target: None,
        });
    }

    rows
}

fn todo_status_chip(status: Option<TodoStatus>) -> &'static str {
    match status {
        Some(TodoStatus::InProgress) => "[~]",
        Some(TodoStatus::Completed) => "[x]",
        Some(TodoStatus::Blocked) => "[!]",
        _ => "[ ]",
    }
}
