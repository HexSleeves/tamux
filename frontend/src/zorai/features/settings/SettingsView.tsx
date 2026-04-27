import { ZORAI_APP_NAME } from "@/zorai/branding";
import { SettingsTabPanel } from "./SettingsPanels";
import { getZoraiSettingsTab, zoraiSettingsTabs, type ZoraiSettingsTabId } from "./settingsTabs";

type SettingsProps = {
  activeTab: ZoraiSettingsTabId;
  onSelectTab: (tabId: ZoraiSettingsTabId) => void;
};

export function SettingsRail({ activeTab, onSelectTab }: SettingsProps) {
  return (
    <div className="zorai-rail-stack">
      <div className="zorai-section-label">Settings</div>
      {zoraiSettingsTabs.map((tab) => (
        <button
          type="button"
          key={tab.id}
          className={[
            "zorai-rail-card",
            "zorai-rail-card--button",
            tab.id === activeTab ? "zorai-rail-card--active" : "",
          ].filter(Boolean).join(" ")}
          onClick={() => onSelectTab(tab.id)}
        >
          <strong>{tab.title}</strong>
          <span>{tab.description}</span>
        </button>
      ))}
    </div>
  );
}

export function SettingsView({ activeTab, onSelectTab }: SettingsProps) {
  const selectedTab = getZoraiSettingsTab(activeTab);

  return (
    <section className="zorai-feature-surface zorai-settings-surface">
      <div className="zorai-view-header">
        <div>
          <div className="zorai-kicker">Settings</div>
          <h1>{selectedTab.title}</h1>
          <p>{selectedTab.description} Configure {ZORAI_APP_NAME} without leaving orchestration.</p>
        </div>
      </div>

      <div className="zorai-settings-tab-strip" aria-label="Settings sections">
        {zoraiSettingsTabs.map((tab) => (
          <button
            type="button"
            key={tab.id}
            className={["zorai-ghost-button", tab.id === activeTab ? "zorai-button--active" : ""].filter(Boolean).join(" ")}
            onClick={() => onSelectTab(tab.id)}
          >
            {tab.title}
          </button>
        ))}
      </div>

      <SettingsTabPanel activeTab={activeTab} />
    </section>
  );
}
