#![allow(dead_code)]

use crate::state::goal_workspace::GoalWorkspaceState;
use crate::state::task::TaskState;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[path = "goal_workspace_plan.rs"]
mod plan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalWorkspaceHitTarget {
    PlanStep(String),
    PlanTodo { step_id: String, todo_id: String },
    TimelineRow(usize),
    DetailFile(String),
    DetailCheckpoint(String),
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    tasks: &TaskState,
    goal_run_id: &str,
    state: &GoalWorkspaceState,
    theme: &ThemeTokens,
) {
    if area.width < 3 || area.height < 6 {
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);
    render_summary(frame, layout[0], theme);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(32),
            Constraint::Min(24),
        ])
        .split(layout[1]);

    render_plan(frame, columns[0], tasks, goal_run_id, state, theme);
    render_placeholder(frame, columns[1], " Run timeline ", "Live run events will render here.", theme);
    render_placeholder(frame, columns[2], " Details ", "Selected step details will render here.", theme);
}

pub fn hit_test(
    area: Rect,
    tasks: &TaskState,
    goal_run_id: &str,
    state: &GoalWorkspaceState,
    mouse: Position,
) -> Option<GoalWorkspaceHitTarget> {
    if area.width < 3
        || area.height < 6
        || mouse.x < area.x
        || mouse.x >= area.x.saturating_add(area.width)
        || mouse.y < area.y
        || mouse.y >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(32),
            Constraint::Min(24),
        ])
        .split(layout[1]);

    let plan_area = columns[0];
    if mouse.x < plan_area.x || mouse.x >= plan_area.x.saturating_add(plan_area.width) {
        return None;
    }
    if mouse.y < plan_area.y || mouse.y >= plan_area.y.saturating_add(plan_area.height) {
        return None;
    }

    let inner = Block::default().borders(Borders::ALL).inner(plan_area);
    if mouse.x < inner.x
        || mouse.x >= inner.x.saturating_add(inner.width)
        || mouse.y < inner.y
        || mouse.y >= inner.y.saturating_add(inner.height)
    {
        return None;
    }

    let rows = plan::build_rows(tasks, goal_run_id, state);
    let row_index = mouse.y.saturating_sub(inner.y) as usize;
    rows.get(row_index).and_then(|row| row.target.clone())
}

fn render_summary(frame: &mut Frame, area: Rect, theme: &ThemeTokens) {
    let block = Block::default().title(" Goal Mission Control ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = Line::from(vec![
        Span::styled("Goal", theme.accent_secondary),
        Span::styled("  Progress  Active agent  Needs attention", theme.fg_dim),
    ]);
    frame.render_widget(Paragraph::new(text), inner);
}

fn render_plan(
    frame: &mut Frame,
    area: Rect,
    tasks: &TaskState,
    goal_run_id: &str,
    state: &GoalWorkspaceState,
    _theme: &ThemeTokens,
) {
    let block = Block::default().title(" Plan ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let selected_style = Style::default().bg(Color::Indexed(236));
    let lines = plan::build_rows(tasks, goal_run_id, state)
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            if index == state.selected_plan_row() {
                row.line.style(selected_style)
            } else {
                row.line
            }
        })
        .collect::<Vec<_>>();
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_placeholder(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    body: &str,
    theme: &ThemeTokens,
) {
    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(body).style(theme.fg_dim).wrap(Wrap { trim: false }),
        inner,
    );
}

#[cfg(test)]
#[path = "tests/goal_workspace.rs"]
mod tests;
