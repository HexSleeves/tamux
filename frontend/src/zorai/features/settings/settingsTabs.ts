export const zoraiSettingsTabs = [
  { id: "runtime", title: "Runtime", description: "Agent engine, backend, retries, and context behavior." },
  { id: "model", title: "Model", description: "Provider routing, model, transport, and endpoint." },
  { id: "auth", title: "Auth", description: "Provider auth source, credentials, and auth status." },
  { id: "interface", title: "Interface", description: "Theme and chat presentation preferences." },
  { id: "tools", title: "Tools", description: "Agent capability toggles and tool loop limits." },
  { id: "concierge", title: "Concierge", description: "Briefing, cleanup, and guidance behavior." },
  { id: "subagents", title: "Sub-agents", description: "Delegated roles attached to the orchestration runtime." },
  { id: "gateway", title: "Gateway", description: "Slack, Discord, Telegram, and WhatsApp bridge." },
  { id: "keyboard", title: "Keyboard", description: "Operator keybinding profile." },
  { id: "plugins", title: "Plugins", description: "Installed plugin runtime and enabled extensions." },
  { id: "about", title: "About", description: "Local runtime identity and shell status." },
] as const;

export type ZoraiSettingsTabId = (typeof zoraiSettingsTabs)[number]["id"];

export function getDefaultZoraiSettingsTab(): ZoraiSettingsTabId {
  return "runtime";
}

export function getZoraiSettingsTab(tabId: ZoraiSettingsTabId) {
  return zoraiSettingsTabs.find((tab) => tab.id === tabId) ?? zoraiSettingsTabs[0];
}
