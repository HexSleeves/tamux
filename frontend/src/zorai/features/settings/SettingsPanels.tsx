import { useEffect, useMemo, type ReactNode } from "react";
import { useAgentStore, type AgentSettings } from "@/lib/agentStore";
import type { AuthSource } from "@/lib/agentStore/types";
import { DEFAULT_KEYBINDINGS, useKeybindStore } from "@/lib/keybindStore";
import { usePluginStore } from "@/lib/pluginStore";
import { useSettingsStore } from "@/lib/settingsStore";
import { BUILTIN_THEMES } from "@/lib/themes";
import { ZORAI_APP_NAME } from "@/zorai/branding";
import type { ZoraiSettingsTabId } from "./settingsTabs";

type ToggleSetting = {
  key: keyof AgentSettings;
  label: string;
  description: string;
};

const authSources: AuthSource[] = ["api_key", "chatgpt_subscription", "github_copilot"];

const toolToggles: ToggleSetting[] = [
  { key: "enable_bash_tool", label: "Terminal tool", description: "Allow agents to execute managed shell commands." },
  { key: "enable_vision_tool", label: "Vision tool", description: "Allow screenshot and image inspection workflows." },
  { key: "enable_web_browsing_tool", label: "Browser tool", description: "Allow browser-backed research actions." },
  { key: "enable_web_search_tool", label: "Web search", description: "Allow configured search provider access." },
  { key: "enable_streaming", label: "Streaming", description: "Stream assistant output as it arrives." },
  { key: "enable_conversation_memory", label: "Conversation memory", description: "Keep durable context across agent sessions." },
  { key: "auto_retry", label: "Auto retry", description: "Retry recoverable provider and tool failures." },
];

export function SettingsTabPanel({ activeTab }: { activeTab: ZoraiSettingsTabId }) {
  if (activeTab === "model") return <ModelPanel />;
  if (activeTab === "auth") return <AuthPanel />;
  if (activeTab === "interface") return <InterfacePanel />;
  if (activeTab === "tools") return <ToolsPanel />;
  if (activeTab === "concierge") return <ConciergePanel />;
  if (activeTab === "subagents") return <SubAgentsPanel />;
  if (activeTab === "gateway") return <GatewayPanel />;
  if (activeTab === "keyboard") return <KeyboardPanel />;
  if (activeTab === "plugins") return <PluginsPanel />;
  if (activeTab === "about") return <AboutPanel />;
  return <RuntimePanel />;
}

function RuntimePanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);
  const resetAgentSettings = useAgentStore((state) => state.resetAgentSettings);

  return (
    <SettingsGrid>
      <Panel section="Runtime" title="Agent engine">
        <SettingRow label="Enable agent runtime" description="Turns the primary Zorai agent runtime on or off.">
          <Switch checked={agentSettings.enabled} onChange={(checked) => updateAgentSetting("enabled", checked)} />
        </SettingRow>
        <SettingRow label="Backend" description="Choose the agent execution backend.">
          <select className="zorai-input" value={agentSettings.agent_backend} onChange={(event) => updateAgentSetting("agent_backend", event.target.value as AgentSettings["agent_backend"])}>
            <option value="daemon">{ZORAI_APP_NAME}</option>
            <option value="openclaw">OpenClaw</option>
            <option value="hermes">Hermes</option>
            <option value="legacy">Legacy fallback</option>
          </select>
        </SettingRow>
        <SettingRow label="Reasoning effort" description="Default reasoning budget for agent responses.">
          <select className="zorai-input" value={agentSettings.reasoning_effort} onChange={(event) => updateAgentSetting("reasoning_effort", event.target.value as AgentSettings["reasoning_effort"])}>
            {["none", "minimal", "low", "medium", "high", "xhigh"].map((value) => <option key={value} value={value}>{value}</option>)}
          </select>
        </SettingRow>
        <button type="button" className="zorai-ghost-button" onClick={resetAgentSettings}>Reset agent defaults</button>
      </Panel>
      <Panel section="Context" title="Loop controls">
        <NumberRow label="Max tool loops" description="Upper bound for tool-call cycles in one response." value={agentSettings.max_tool_loops} onChange={(value) => updateAgentSetting("max_tool_loops", value)} min={1} max={50} />
        <NumberRow label="Max retries" description="Provider and tool retry attempts." value={agentSettings.max_retries} onChange={(value) => updateAgentSetting("max_retries", value)} min={0} max={10} />
        <NumberRow label="Context messages" description="Conversation messages kept before compaction." value={agentSettings.max_context_messages} onChange={(value) => updateAgentSetting("max_context_messages", value)} min={10} max={500} />
        <SettingRow label="Auto compact context" description="Compress older conversation context automatically.">
          <Switch checked={agentSettings.auto_compact_context} onChange={(checked) => updateAgentSetting("auto_compact_context", checked)} />
        </SettingRow>
      </Panel>
    </SettingsGrid>
  );
}

function ModelPanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);
  const providerIds = useProviderIds(agentSettings);
  const activeProvider = agentSettings.active_provider;
  const activeProviderConfig = agentSettings[activeProvider] ?? {};
  const updateProviderConfig = (patch: Record<string, unknown>) => updateAgentSetting(activeProvider, { ...activeProviderConfig, ...patch });

  return (
    <SettingsGrid>
      <Panel section="Model" title="Provider selection">
        <SettingRow label="Active provider" description="Provider used by the primary Zorai agent.">
          <select className="zorai-input" value={activeProvider} onChange={(event) => updateAgentSetting("active_provider", event.target.value as AgentSettings["active_provider"])}>
            {providerIds.map((providerId) => <option key={providerId} value={providerId}>{providerId}</option>)}
          </select>
        </SettingRow>
        <SettingRow label="Model" description="Default model for this provider.">
          <input className="zorai-input" value={String(activeProviderConfig.model ?? "")} onChange={(event) => updateProviderConfig({ model: event.target.value })} />
        </SettingRow>
        <SettingRow label="Transport" description="Provider API transport mode.">
          <select className="zorai-input" value={String(activeProviderConfig.api_transport ?? "responses")} onChange={(event) => updateProviderConfig({ api_transport: event.target.value })}>
            <option value="responses">responses</option>
            <option value="chat_completions">chat completions</option>
            <option value="anthropic_messages">anthropic messages</option>
            <option value="native_assistant">native assistant</option>
          </select>
        </SettingRow>
        <SettingRow label="Base URL" description="Optional OpenAI-compatible endpoint override.">
          <input className="zorai-input" value={String(activeProviderConfig.base_url ?? "")} onChange={(event) => updateProviderConfig({ base_url: event.target.value })} />
        </SettingRow>
      </Panel>
    </SettingsGrid>
  );
}

function AuthPanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);
  const authStates = useAgentStore((state) => state.providerAuthStates);
  const refreshAuth = useAgentStore((state) => state.refreshProviderAuthStates);
  const activeProvider = agentSettings.active_provider;
  const activeProviderConfig = agentSettings[activeProvider] ?? {};
  const updateProviderConfig = (patch: Record<string, unknown>) => updateAgentSetting(activeProvider, { ...activeProviderConfig, ...patch });

  return (
    <SettingsGrid>
      <Panel section="Auth" title="Provider credentials">
        <SettingRow label="Auth source" description="Credential mode for the active provider.">
          <select className="zorai-input" value={String(activeProviderConfig.auth_source ?? "api_key")} onChange={(event) => updateProviderConfig({ auth_source: event.target.value })}>
            {authSources.map((source) => <option key={source} value={source}>{source}</option>)}
          </select>
        </SettingRow>
        <SettingRow label="API key" description="Stored locally through the agent settings bridge.">
          <input className="zorai-input" type="password" value={String(activeProviderConfig.api_key ?? "")} onChange={(event) => updateProviderConfig({ api_key: event.target.value })} />
        </SettingRow>
        <button type="button" className="zorai-ghost-button" onClick={() => void refreshAuth()}>Refresh auth status</button>
      </Panel>
      <Panel section="Status" title="Provider status">
        {authStates.length === 0 ? <p className="zorai-empty-state">No provider auth status has been reported by the daemon yet.</p> : authStates.map((state) => (
          <div key={`${state.provider_id}-${state.auth_source}`} className="zorai-setting-row">
            <div><strong>{state.provider_name}</strong><span>{state.model} via {state.auth_source}</span></div>
            <span className="zorai-status-pill">{state.authenticated ? "authenticated" : "not authenticated"}</span>
          </div>
        ))}
      </Panel>
    </SettingsGrid>
  );
}

function InterfacePanel() {
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);

  return (
    <SettingsGrid>
      <Panel section="Interface" title="Shell presentation">
        <SettingRow label="Theme" description="Terminal palette used by embedded runtime tools.">
          <select className="zorai-input" value={settings.themeName} onChange={(event) => updateSetting("themeName", event.target.value)}>
            {BUILTIN_THEMES.map((theme) => <option key={theme.name} value={theme.name}>{theme.name}</option>)}
          </select>
        </SettingRow>
        <SettingRow label="Chat font size" description="Message text size for agent conversations.">
          <input className="zorai-input" type="number" min={11} max={22} value={agentSettings.chatFontSize} onChange={(event) => updateAgentSetting("chatFontSize", Number(event.target.value))} />
        </SettingRow>
        <SettingRow label="Chat font family" description="Font stack used in conversation surfaces.">
          <input className="zorai-input" value={agentSettings.chatFontFamily} onChange={(event) => updateAgentSetting("chatFontFamily", event.target.value)} />
        </SettingRow>
      </Panel>
    </SettingsGrid>
  );
}

function ToolsPanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);

  return (
    <SettingsGrid>
      <Panel section="Tools" title="Agent capabilities" extraClassName="zorai-settings-tools">
        {toolToggles.map((toggle) => (
          <SettingRow key={toggle.key} label={toggle.label} description={toggle.description}>
            <Switch checked={Boolean(agentSettings[toggle.key])} onChange={(checked) => updateAgentSetting(toggle.key, checked as never)} />
          </SettingRow>
        ))}
      </Panel>
    </SettingsGrid>
  );
}

function ConciergePanel() {
  const config = useAgentStore((state) => state.conciergeConfig);
  const updateConfig = useAgentStore((state) => state.updateConciergeConfig);
  const patchConfig = (patch: Record<string, unknown>) => void updateConfig({ ...config, ...patch });

  return (
    <SettingsGrid>
      <Panel section="Concierge" title="Briefing behavior">
        <SettingRow label="Enabled" description="Allow the concierge to brief and guide operator sessions.">
          <Switch checked={config.enabled} onChange={(checked) => patchConfig({ enabled: checked })} />
        </SettingRow>
        <SettingRow label="Detail level" description="Default depth for concierge guidance.">
          <select className="zorai-input" value={config.detail_level} onChange={(event) => patchConfig({ detail_level: event.target.value })}>
            <option value="brief">brief</option>
            <option value="standard">standard</option>
            <option value="detailed">detailed</option>
          </select>
        </SettingRow>
        <SettingRow label="Auto cleanup" description="Dismiss stale concierge context when navigating.">
          <Switch checked={config.auto_cleanup_on_navigate} onChange={(checked) => patchConfig({ auto_cleanup_on_navigate: checked })} />
        </SettingRow>
      </Panel>
    </SettingsGrid>
  );
}

function SubAgentsPanel() {
  const subAgents = useAgentStore((state) => state.subAgents);
  const refreshSubAgents = useAgentStore((state) => state.refreshSubAgents);
  const updateSubAgent = useAgentStore((state) => state.updateSubAgent);

  return (
    <SettingsGrid>
      <Panel section="Sub-agents" title="Delegated roles">
        <button type="button" className="zorai-ghost-button" onClick={() => void refreshSubAgents()}>Refresh sub-agents</button>
        {subAgents.length === 0 ? <p className="zorai-empty-state">No sub-agents are currently registered with the daemon.</p> : subAgents.map((agent) => (
          <SettingRow key={agent.id} label={agent.name} description={`${agent.provider} / ${agent.model}${agent.role ? ` / ${agent.role}` : ""}`}>
            <Switch checked={agent.enabled} onChange={(checked) => void updateSubAgent({ ...agent, enabled: checked })} />
          </SettingRow>
        ))}
      </Panel>
    </SettingsGrid>
  );
}

function GatewayPanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const updateAgentSetting = useAgentStore((state) => state.updateAgentSetting);

  return (
    <SettingsGrid>
      <Panel section="Gateway" title="Messaging bridge">
        <SettingRow label="Gateway enabled" description="Bridge external chat platforms into Zorai.">
          <Switch checked={agentSettings.gateway_enabled} onChange={(checked) => updateAgentSetting("gateway_enabled", checked)} />
        </SettingRow>
        <SettingRow label="Command prefix" description="Prefix used for external platform commands.">
          <input className="zorai-input" value={agentSettings.gateway_command_prefix} onChange={(event) => updateAgentSetting("gateway_command_prefix", event.target.value)} />
        </SettingRow>
        <SecretRow label="Slack token" value={agentSettings.slack_token} onChange={(value) => updateAgentSetting("slack_token", value)} />
        <SecretRow label="Discord token" value={agentSettings.discord_token} onChange={(value) => updateAgentSetting("discord_token", value)} />
        <SecretRow label="Telegram token" value={agentSettings.telegram_token} onChange={(value) => updateAgentSetting("telegram_token", value)} />
        <SecretRow label="WhatsApp token" value={agentSettings.whatsapp_token} onChange={(value) => updateAgentSetting("whatsapp_token", value)} />
      </Panel>
    </SettingsGrid>
  );
}

function KeyboardPanel() {
  const bindings = useKeybindStore((state) => state.bindings);
  const setBinding = useKeybindStore((state) => state.setBinding);
  const resetBindings = useKeybindStore((state) => state.resetBindings);

  return (
    <SettingsGrid>
      <Panel section="Keyboard" title="Operator shortcuts">
        <button type="button" className="zorai-ghost-button" onClick={resetBindings}>Reset keybindings</button>
        {bindings.map((binding) => (
          <SettingRow key={binding.action} label={binding.description} description={binding.action}>
            <input className="zorai-input" value={binding.combo} placeholder={DEFAULT_KEYBINDINGS.find((item) => item.action === binding.action)?.combo} onChange={(event) => setBinding(binding.action, event.target.value)} />
          </SettingRow>
        ))}
      </Panel>
    </SettingsGrid>
  );
}

function PluginsPanel() {
  const plugins = usePluginStore((state) => state.plugins);
  const loading = usePluginStore((state) => state.loading);
  const error = usePluginStore((state) => state.error);
  const fetchPlugins = usePluginStore((state) => state.fetchPlugins);
  const toggleEnabled = usePluginStore((state) => state.toggleEnabled);

  useEffect(() => {
    if (plugins.length === 0 && !loading) void fetchPlugins();
  }, [fetchPlugins, loading, plugins.length]);

  return (
    <SettingsGrid>
      <Panel section="Plugins" title="Installed extensions">
        <button type="button" className="zorai-ghost-button" onClick={() => void fetchPlugins()}>{loading ? "Refreshing..." : "Refresh plugins"}</button>
        {error ? <p className="zorai-empty-state">{error}</p> : null}
        {plugins.length === 0 && !loading ? <p className="zorai-empty-state">No plugins are currently reported by the plugin daemon.</p> : plugins.map((plugin) => (
          <SettingRow key={plugin.name} label={plugin.name} description={`${plugin.version} / ${plugin.endpoint_count} endpoints / auth ${plugin.auth_status}`}>
            <Switch checked={plugin.enabled} onChange={(checked) => void toggleEnabled(plugin.name, checked)} />
          </SettingRow>
        ))}
      </Panel>
    </SettingsGrid>
  );
}

function AboutPanel() {
  const agentSettings = useAgentStore((state) => state.agentSettings);
  const settings = useSettingsStore((state) => state.settings);

  return (
    <SettingsGrid>
      <Panel section="About" title={`${ZORAI_APP_NAME} shell`}>
        <Metric label="Active provider" value={agentSettings.active_provider} />
        <Metric label="Backend" value={agentSettings.agent_backend} />
        <Metric label="Theme" value={settings.themeName} />
        <Metric label="Chat page size" value={String(agentSettings.react_chat_history_page_size)} />
      </Panel>
    </SettingsGrid>
  );
}

function useProviderIds(agentSettings: AgentSettings) {
  return useMemo(() => Object.keys(agentSettings).filter((key) => {
    const value = agentSettings[key];
    return value && typeof value === "object" && "model" in value && "base_url" in value;
  }).sort(), [agentSettings]);
}

function SettingsGrid({ children }: { children: ReactNode }) {
  return <div className="zorai-settings-grid">{children}</div>;
}

function Panel({ section, title, children, extraClassName }: { section: string; title: string; children: ReactNode; extraClassName?: string }) {
  return (
    <div className={["zorai-panel", extraClassName ?? ""].filter(Boolean).join(" ")}>
      <div><div className="zorai-section-label">{section}</div><h2>{title}</h2></div>
      {children}
    </div>
  );
}

function SettingRow({ label, description, children }: { label: string; description: string; children: ReactNode }) {
  return (
    <div className="zorai-setting-row">
      <div><strong>{label}</strong><span>{description}</span></div>
      {children}
    </div>
  );
}

function SecretRow({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <SettingRow label={label} description="Stored as a local gateway credential.">
      <input className="zorai-input" type="password" value={value} onChange={(event) => onChange(event.target.value)} />
    </SettingRow>
  );
}

function NumberRow({ label, description, value, onChange, min, max }: { label: string; description: string; value: number; onChange: (value: number) => void; min: number; max: number }) {
  return (
    <SettingRow label={label} description={description}>
      <input className="zorai-input" type="number" min={min} max={max} value={value} onChange={(event) => onChange(Number(event.target.value))} />
    </SettingRow>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="zorai-setting-row"><div><strong>{label}</strong><span>{value}</span></div></div>;
}

function Switch({ checked, onChange }: { checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <button type="button" className={["zorai-switch", checked ? "zorai-switch--on" : ""].filter(Boolean).join(" ")} aria-pressed={checked} onClick={() => onChange(!checked)}>
      <span />
    </button>
  );
}
