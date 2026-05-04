fn subagents_hit_test(
    content_area: Rect,
    subagents: &SubAgentsState,
    scroll: usize,
    mouse: Position,
) -> Option<SettingsHitTarget> {
    let row = mouse.y.saturating_sub(content_area.y) as usize + scroll;
    let list_len = subagents.entries.len();
    if list_len > 0 && (4..4 + list_len).contains(&row) {
        let index = row - 4;
        if let Some(entry) = subagents.entries.get(index) {
            let (edit_start, delete_start, toggle_start, _, toggle_end) =
                subagent_row_action_offsets(content_area, entry);
            if mouse.x >= edit_start && mouse.x < delete_start.saturating_sub(1) {
                return Some(SettingsHitTarget::SubAgentRowAction {
                    index,
                    action: SubAgentTabAction::Edit,
                });
            }
            if mouse.x >= delete_start && mouse.x < toggle_start.saturating_sub(1) {
                return Some(SettingsHitTarget::SubAgentRowAction {
                    index,
                    action: SubAgentTabAction::Delete,
                });
            }
            if mouse.x >= toggle_start && mouse.x < toggle_end {
                return Some(SettingsHitTarget::SubAgentRowAction {
                    index,
                    action: SubAgentTabAction::Toggle,
                });
            }
        }
        return Some(SettingsHitTarget::SubAgentListItem(index));
    }
    match row {
        r if r == 5 + list_len => Some(SettingsHitTarget::SubAgentAction(SubAgentTabAction::Add)),
        _ => None,
    }
}

fn render_tab_content<'a>(
    content_width: u16,
    settings: &'a SettingsState,
    config: &'a ConfigState,
    modal: &'a ModalState,
    auth: &'a crate::state::auth::AuthState,
    subagents: &'a SubAgentsState,
    concierge: &'a ConciergeState,
    tier: &crate::state::tier::TierState,
    plugin_settings: &PluginSettingsState,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    match settings.active_tab() {
        SettingsTab::Provider => render_provider_tab(settings, config, theme),
        SettingsTab::Tools => render_tools_tab(settings, config, theme),
        SettingsTab::WebSearch => render_websearch_tab(settings, config, theme),
        SettingsTab::Chat => render_chat_tab(settings, config, theme),
        SettingsTab::Gateway => render_gateway_tab(settings, config, modal, theme),
        SettingsTab::Auth => render_auth_tab(content_width, auth, config, theme),
        SettingsTab::Agent => render_agent_tab(settings, config, theme),
        SettingsTab::SubAgents => render_subagents_tab(content_width, settings, subagents, theme),
        SettingsTab::Concierge => render_concierge_tab(settings, concierge, theme),
        SettingsTab::Features => render_features_tab(settings, config, tier, theme),
        SettingsTab::Advanced => render_advanced_tab(settings, config, theme),
        SettingsTab::Plugins => render_plugins_tab(settings, plugin_settings, content_width, theme),
        SettingsTab::About => render_about_tab(theme),
    }
}
