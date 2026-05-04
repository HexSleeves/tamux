fn content_area(area: Rect) -> Option<Rect> {
    let block = Block::default()
        .title(" SETTINGS ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double);
    let inner = block.inner(area);
    if inner.height < 5 {
        return None;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    Some(chunks[2])
}

pub fn max_scroll(
    area: Rect,
    settings: &SettingsState,
    config: &ConfigState,
    modal: &ModalState,
    auth: &crate::state::auth::AuthState,
    subagents: &SubAgentsState,
    concierge: &ConciergeState,
    tier: &crate::state::tier::TierState,
    plugin_settings: &PluginSettingsState,
    theme: &ThemeTokens,
) -> usize {
    let Some(content_area) = content_area(area) else {
        return 0;
    };
    let line_count = render_tab_content(
        content_area.width,
        settings,
        config,
        modal,
        auth,
        subagents,
        concierge,
        tier,
        plugin_settings,
        theme,
    )
    .len();
    line_count.saturating_sub(content_area.height as usize)
}

pub fn scroll_for_selected_field(
    area: Rect,
    settings: &SettingsState,
    config: &ConfigState,
    modal: &ModalState,
    auth: &crate::state::auth::AuthState,
    subagents: &SubAgentsState,
    concierge: &ConciergeState,
    tier: &crate::state::tier::TierState,
    plugin_settings: &PluginSettingsState,
    current_scroll: usize,
    theme: &ThemeTokens,
) -> usize {
    let Some(content_area) = content_area(area) else {
        return 0;
    };
    let content_lines = render_tab_content(
        content_area.width,
        settings,
        config,
        modal,
        auth,
        subagents,
        concierge,
        tier,
        plugin_settings,
        theme,
    );
    let line_count = content_lines.len();
    let max_scroll = line_count.saturating_sub(content_area.height as usize);
    let Some(selected_row) = selected_content_row(&content_lines) else {
        return current_scroll.min(max_scroll);
    };

    let mut scroll = current_scroll.min(max_scroll);
    let viewport_height = content_area.height as usize;
    if selected_row < scroll {
        scroll = selected_row;
    } else if selected_row >= scroll.saturating_add(viewport_height) {
        scroll = selected_row.saturating_add(1).saturating_sub(viewport_height);
    }
    scroll.min(max_scroll)
}

fn selected_content_row(lines: &[Line<'_>]) -> Option<usize> {
    lines.iter().position(|line| {
        let text = line.to_string();
        let trimmed = text.trim_start_matches(' ');
        let indent = text.len().saturating_sub(trimmed.len());
        indent <= 4 && trimmed.starts_with("> ")
    })
}
