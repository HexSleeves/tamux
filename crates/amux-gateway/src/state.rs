use std::collections::HashMap;

use amux_protocol::{
    GatewayBootstrapPayload, GatewayCursorState, GatewayProviderBootstrap, GatewayRouteMode,
    GatewayRouteModeState, GatewayThreadBindingState,
};

#[derive(Debug, Clone, Default)]
pub struct GatewayRuntimeState {
    bootstrap_correlation_id: String,
    feature_flags: Vec<String>,
    providers: HashMap<String, GatewayProviderBootstrap>,
    cursors: HashMap<String, GatewayCursorState>,
    thread_bindings: HashMap<String, GatewayThreadBindingState>,
    route_modes: HashMap<String, GatewayRouteModeState>,
}

impl GatewayRuntimeState {
    pub fn from_bootstrap(payload: &GatewayBootstrapPayload) -> Self {
        let providers = payload
            .providers
            .iter()
            .cloned()
            .map(|provider| (provider.platform.to_ascii_lowercase(), provider))
            .collect::<HashMap<_, _>>();

        let cursors = payload
            .continuity
            .cursors
            .iter()
            .cloned()
            .map(|cursor| (cursor_key(&cursor.platform, &cursor.channel_id), cursor))
            .collect::<HashMap<_, _>>();

        let thread_bindings = payload
            .continuity
            .thread_bindings
            .iter()
            .cloned()
            .map(|binding| (binding.channel_key.to_ascii_lowercase(), binding))
            .collect::<HashMap<_, _>>();

        let route_modes = payload
            .continuity
            .route_modes
            .iter()
            .cloned()
            .map(|mode| (mode.channel_key.to_ascii_lowercase(), mode))
            .collect::<HashMap<_, _>>();

        Self {
            bootstrap_correlation_id: payload.bootstrap_correlation_id.clone(),
            feature_flags: payload.feature_flags.clone(),
            providers,
            cursors,
            thread_bindings,
            route_modes,
        }
    }

    pub fn bootstrap_correlation_id(&self) -> &str {
        &self.bootstrap_correlation_id
    }

    pub fn feature_flags(&self) -> &[String] {
        &self.feature_flags
    }

    pub fn provider(&self, platform: &str) -> Option<&GatewayProviderBootstrap> {
        self.providers.get(&platform.to_ascii_lowercase())
    }

    pub fn thread_binding(&self, channel_key: &str) -> Option<String> {
        self.thread_bindings
            .get(&channel_key.to_ascii_lowercase())
            .and_then(|binding| binding.thread_id.clone())
    }

    pub fn route_mode(&self, channel_key: &str) -> Option<GatewayRouteMode> {
        self.route_modes
            .get(&channel_key.to_ascii_lowercase())
            .map(|mode| mode.route_mode)
    }

    pub fn cursor(&self, platform: &str, channel_id: &str) -> Option<&GatewayCursorState> {
        self.cursors.get(&cursor_key(platform, channel_id))
    }

    pub fn apply_cursor_update(&mut self, update: GatewayCursorState) {
        self.cursors
            .insert(cursor_key(&update.platform, &update.channel_id), update);
    }

    pub fn apply_thread_binding_update(&mut self, update: GatewayThreadBindingState) {
        self.thread_bindings
            .insert(update.channel_key.to_ascii_lowercase(), update);
    }

    pub fn apply_route_mode_update(&mut self, update: GatewayRouteModeState) {
        self.route_modes
            .insert(update.channel_key.to_ascii_lowercase(), update);
    }
}

fn cursor_key(platform: &str, channel_id: &str) -> String {
    format!(
        "{}:{}",
        platform.to_ascii_lowercase(),
        channel_id.to_ascii_lowercase()
    )
}
