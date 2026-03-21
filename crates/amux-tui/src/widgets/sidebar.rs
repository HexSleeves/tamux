use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

use crate::state::sidebar::{SidebarState, SidebarTab};
use crate::state::task::TaskState;
use crate::theme::ThemeTokens;

const TAB_LABELS: [&str; 2] = ["Files", "Todos"];
const TAB_DIVIDER: &str = " | ";

#[derive(Debug, Clone, Copy)]
struct SidebarTabLayout {
    tab: SidebarTab,
    start_x: u16,
    end_x: u16,
}

#[derive(Debug, Clone)]
struct SidebarRow {
    line: Line<'static>,
    file_path: Option<String>,
}

pub enum SidebarHitTarget {
    Tab(SidebarTab),
    File(String),
    Todo(usize),
}

fn tab_hit_test(tab_area: Rect, mouse_x: u16) -> Option<SidebarTab> {
    tab_layouts(tab_area)
        .into_iter()
        .find(|layout| mouse_x >= layout.start_x && mouse_x < layout.end_x)
        .map(|layout| layout.tab)
}

fn tab_layouts(tab_area: Rect) -> Vec<SidebarTabLayout> {
    let tabs = [SidebarTab::Files, SidebarTab::Todos];
    let divider_width = UnicodeWidthStr::width(TAB_DIVIDER) as u16;
    let padded_labels = TAB_LABELS
        .iter()
        .map(|label| format!(" {label} "))
        .collect::<Vec<_>>();
    let total_width = padded_labels
        .iter()
        .map(|label| UnicodeWidthStr::width(label.as_str()) as u16)
        .sum::<u16>()
        .saturating_add(divider_width);
    let mut x = tab_area
        .x
        .saturating_add(tab_area.width.saturating_sub(total_width) / 2);
    let mut layouts = Vec::with_capacity(tabs.len());

    for (idx, tab) in tabs.into_iter().enumerate() {
        let width = UnicodeWidthStr::width(padded_labels[idx].as_str()) as u16;
        layouts.push(SidebarTabLayout {
            tab,
            start_x: x,
            end_x: x.saturating_add(width),
        });
        x = x.saturating_add(width);
        if idx + 1 < padded_labels.len() {
            x = x.saturating_add(divider_width);
        }
    }

    layouts
}

fn tab_line(sidebar: &SidebarState, theme: &ThemeTokens) -> Line<'static> {
    let mut spans = Vec::new();
    let layouts = [
        (SidebarTab::Files, " Files "),
        (SidebarTab::Todos, " Todos "),
    ];

    for (idx, (tab, label)) in layouts.into_iter().enumerate() {
        let style = if sidebar.active_tab() == tab {
            theme.fg_active.bg(Color::Indexed(236))
        } else {
            theme.fg_dim
        };
        spans.push(Span::styled(label, style));
        if idx + 1 < layouts.len() {
            spans.push(Span::styled(TAB_DIVIDER, theme.fg_dim));
        }
    }

    Line::from(spans).alignment(Alignment::Center)
}

fn rows_for_thread(
    tasks: &TaskState,
    sidebar: &SidebarState,
    thread_id: Option<&str>,
    theme: &ThemeTokens,
    width: usize,
) -> Vec<SidebarRow> {
    let Some(thread_id) = thread_id else {
        return vec![SidebarRow {
            line: Line::from(Span::styled(" No thread selected", theme.fg_dim)),
            file_path: None,
        }];
    };

    let selected = sidebar.selected_item();
    let selected_style = Style::default().bg(Color::Indexed(236));

    match sidebar.active_tab() {
        SidebarTab::Files => {
            let Some(context) = tasks.work_context_for_thread(thread_id) else {
                return vec![SidebarRow {
                    line: Line::from(Span::styled(" No files", theme.fg_dim)),
                    file_path: None,
                }];
            };
            if context.entries.is_empty() {
                return vec![SidebarRow {
                    line: Line::from(Span::styled(" No files", theme.fg_dim)),
                    file_path: None,
                }];
            }

            context
                .entries
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    let label = entry.change_kind.as_deref().unwrap_or_else(|| {
                        entry.kind
                            .map(|kind| match kind {
                                crate::state::task::WorkContextEntryKind::RepoChange => "diff",
                                crate::state::task::WorkContextEntryKind::Artifact => "file",
                                crate::state::task::WorkContextEntryKind::GeneratedSkill => "skill",
                            })
                            .unwrap_or("file")
                    });
                    let mut path = entry.path.clone();
                    let max_len = width.saturating_sub(12).max(8);
                    if path.chars().count() > max_len {
                        let tail: String = path
                            .chars()
                            .rev()
                            .take(max_len.saturating_sub(1))
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect();
                        path = format!("…{tail}");
                    }

                    let line = Line::from(vec![
                        Span::styled(if idx == selected { "> " } else { "  " }, theme.accent_primary),
                        Span::styled(format!("[{}]", label), theme.fg_dim),
                        Span::raw(" "),
                        Span::styled(path, theme.fg_active),
                    ]);

                    SidebarRow {
                        line: if idx == selected {
                            line.style(selected_style)
                        } else {
                            line
                        },
                        file_path: Some(entry.path.clone()),
                    }
                })
                .collect()
        }
        SidebarTab::Todos => {
            let todos = tasks.todos_for_thread(thread_id);
            if todos.is_empty() {
                return vec![SidebarRow {
                    line: Line::from(Span::styled(" No todos", theme.fg_dim)),
                    file_path: None,
                }];
            }

            todos.iter()
                .enumerate()
                .map(|(idx, todo)| {
                    let marker = match todo.status {
                        Some(crate::state::task::TodoStatus::Completed) => "[x]",
                        Some(crate::state::task::TodoStatus::InProgress) => "[~]",
                        Some(crate::state::task::TodoStatus::Blocked) => "[!]",
                        _ => "[ ]",
                    };
                    let mut text = todo.content.clone();
                    let max_len = width.saturating_sub(8).max(8);
                    if text.chars().count() > max_len {
                        text = format!(
                            "{}…",
                            text.chars().take(max_len.saturating_sub(1)).collect::<String>()
                        );
                    }
                    let line = Line::from(vec![
                        Span::styled(if idx == selected { "> " } else { "  " }, theme.accent_primary),
                        Span::styled(marker, theme.fg_dim),
                        Span::raw(" "),
                        Span::styled(text, theme.fg_active),
                    ]);
                    SidebarRow {
                        line: if idx == selected {
                            line.style(selected_style)
                        } else {
                            line
                        },
                        file_path: None,
                    }
                })
                .collect()
        }
    }
}

fn resolved_scroll(rows: &[SidebarRow], sidebar: &SidebarState, body_height: usize) -> usize {
    let max_scroll = rows.len().saturating_sub(body_height);
    let mut scroll = sidebar.scroll_offset().min(max_scroll);
    let selected = sidebar.selected_item().min(rows.len().saturating_sub(1));
    if selected < scroll {
        scroll = selected;
    } else if selected >= scroll.saturating_add(body_height) {
        scroll = selected.saturating_add(1).saturating_sub(body_height);
    }
    scroll.min(max_scroll)
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
    theme: &ThemeTokens,
    _focused: bool,
) {
    if area.height < 2 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    frame.render_widget(Paragraph::new(tab_line(sidebar, theme)), chunks[0]);

    let rows = rows_for_thread(tasks, sidebar, thread_id, theme, chunks[1].width as usize);
    let scroll = resolved_scroll(&rows, sidebar, chunks[1].height as usize);
    let paragraph = Paragraph::new(rows.into_iter().map(|row| row.line).collect::<Vec<_>>())
        .scroll((scroll as u16, 0));
    frame.render_widget(paragraph, chunks[1]);
}

pub fn body_item_count(tasks: &TaskState, sidebar: &SidebarState, thread_id: Option<&str>) -> usize {
    match (sidebar.active_tab(), thread_id) {
        (SidebarTab::Files, Some(thread_id)) => tasks
            .work_context_for_thread(thread_id)
            .map(|ctx| ctx.entries.len().max(1))
            .unwrap_or(1),
        (SidebarTab::Todos, Some(thread_id)) => tasks.todos_for_thread(thread_id).len().max(1),
        _ => 1,
    }
}

pub fn hit_test(
    area: Rect,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
    mouse: Position,
) -> Option<SidebarHitTarget> {
    if area.height < 2
        || mouse.x < area.x
        || mouse.x >= area.x.saturating_add(area.width)
        || mouse.y < area.y
        || mouse.y >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    if mouse.y == chunks[0].y {
        return tab_hit_test(chunks[0], mouse.x).map(SidebarHitTarget::Tab);
    }

    let rows = rows_for_thread(tasks, sidebar, thread_id, &ThemeTokens::default(), chunks[1].width as usize);
    let scroll = resolved_scroll(&rows, sidebar, chunks[1].height as usize);
    let row_idx = scroll + mouse.y.saturating_sub(chunks[1].y) as usize;
    let row = rows.get(row_idx)?;
    if let Some(path) = &row.file_path {
        Some(SidebarHitTarget::File(path.clone()))
    } else {
        Some(SidebarHitTarget::Todo(row_idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::sidebar::SidebarState;
    use crate::state::task::TaskState;

    #[test]
    fn sidebar_handles_empty_state() {
        let sidebar = SidebarState::new();
        let tasks = TaskState::new();
        let _theme = ThemeTokens::default();
        assert_eq!(sidebar.active_tab(), crate::state::sidebar::SidebarTab::Files);
        assert_eq!(body_item_count(&tasks, &sidebar, None), 1);
    }

    #[test]
    fn tab_hit_test_uses_rendered_label_positions() {
        let area = Rect::new(10, 3, 30, 1);
        let layouts = tab_layouts(area);
        assert_eq!(layouts.len(), 2);
        assert_eq!(
            tab_hit_test(area, layouts[0].start_x + 1),
            Some(SidebarTab::Files)
        );
        assert_eq!(
            tab_hit_test(area, layouts[1].start_x + 1),
            Some(SidebarTab::Todos)
        );
        let divider_x = layouts[0].end_x;
        assert_eq!(tab_hit_test(area, divider_x + 1), None);
    }
}
