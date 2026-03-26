use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::theme::ThemeTokens;

fn content_width(line: &Line<'_>) -> u16 {
    let width = line
        .spans
        .iter()
        .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
        .sum::<usize>();
    width.min(u16::MAX as usize) as u16
}

fn centered_content_rect(area: Rect, lines: &[Line<'_>]) -> Rect {
    let content_width = lines.iter().map(content_width).max().unwrap_or(1);
    let content_height = lines.len().min(area.height as usize) as u16;
    let width = content_width.min(area.width).max(1);
    let height = content_height.min(area.height).max(1);
    let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(height) / 2);
    Rect::new(x, y, width, height)
}

fn render_line_clipped(frame: &mut Frame, area: Rect, row: u16, line: &Line<'_>) {
    if row >= area.height {
        return;
    }

    let y = area.y.saturating_add(row);
    let mut x = area.x;
    let max_x = area.x.saturating_add(area.width);
    let line_style = line.style;

    for span in &line.spans {
        let style = line_style.patch(span.style);
        for ch in span.content.chars() {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if width == 0 {
                continue;
            }
            if x >= max_x {
                return;
            }

            if let Some(cell) = frame.buffer_mut().cell_mut((x, y)) {
                cell.set_symbol(&ch.to_string());
                cell.set_style(style);
                cell.skip = false;
            }

            if width > 1 {
                for offset in 1..width {
                    let continuation_x = x.saturating_add(offset);
                    if continuation_x >= max_x {
                        break;
                    }
                    if let Some(cell) = frame.buffer_mut().cell_mut((continuation_x, y)) {
                        cell.reset();
                        cell.set_style(style);
                        cell.skip = true;
                    }
                }
            }

            x = x.saturating_add(width);
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, theme: &ThemeTokens) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let height = area.height as usize;
    let mut lines: Vec<Line<'static>> = Vec::new();

    let pad_top = height.saturating_div(4);
    for _ in 0..pad_top {
        lines.push(Line::raw(""));
    }

    lines.push(Line::from(vec![
        Span::styled("\u{2591}", Style::default().fg(Color::Indexed(24))),
        Span::styled("\u{2592}", Style::default().fg(Color::Indexed(31))),
        Span::styled("\u{2593}", Style::default().fg(Color::Indexed(38))),
        Span::styled("\u{2588}", Style::default().fg(Color::Indexed(75))),
        Span::styled(" T A M U X ", theme.accent_primary),
        Span::styled("\u{2588}", Style::default().fg(Color::Indexed(75))),
        Span::styled("\u{2593}", Style::default().fg(Color::Indexed(38))),
        Span::styled("\u{2592}", Style::default().fg(Color::Indexed(31))),
        Span::styled("\u{2591}", Style::default().fg(Color::Indexed(24))),
    ]));
    lines.push(Line::from(Span::styled(
        "think \u{00b7} plan \u{00b7} ship",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "Clean thread. No concierge noise. Type to begin.",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("[ Think ]", theme.accent_primary),
        Span::raw("  "),
        Span::styled("[ Plan ]", theme.accent_secondary),
        Span::raw("  "),
        Span::styled("[ Ship ]", theme.fg_active),
    ]));
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("Ctrl+P", theme.accent_primary),
        Span::styled(" command palette  ", theme.fg_dim),
        Span::styled("Ctrl+T", theme.accent_primary),
        Span::styled(" threads", theme.fg_dim),
    ]));

    while lines.len() < height {
        lines.push(Line::raw(""));
    }
    lines.truncate(height);

    let content_area = centered_content_rect(area, &lines);
    for (row, line) in lines.iter().enumerate() {
        if row >= content_area.height as usize {
            break;
        }
        render_line_clipped(frame, content_area, row as u16, line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centered_content_rect_stays_inside_target_area() {
        let area = Rect::new(0, 0, 80, 24);
        let lines = vec![
            Line::from("short"),
            Line::from("a much wider centered line"),
        ];

        let rect = centered_content_rect(area, &lines);

        assert!(rect.x >= area.x);
        assert!(rect.y >= area.y);
        assert!(rect.x.saturating_add(rect.width) <= area.x.saturating_add(area.width));
        assert!(rect.y.saturating_add(rect.height) <= area.y.saturating_add(area.height));
    }

    #[test]
    fn content_width_counts_styled_spans_without_overflow() {
        let line = Line::from(vec![
            Span::styled("abc", Style::default()),
            Span::styled("def", Style::default()),
        ]);

        assert_eq!(content_width(&line), 6);
    }
}
