import { useEffect } from "react";
import { useAgentStore } from "../../lib/agentStore";
import { Section, SettingRow, inputStyle, smallBtnStyle } from "./shared";

const DETAIL_LEVELS = [
    { value: "minimal", label: "Quick Hello", desc: "Session title and date with action buttons. No AI call — instant." },
    { value: "context_summary", label: "Session Recap", desc: "AI-generated 1-2 sentence summary of your last session." },
    { value: "proactive_triage", label: "Smart Triage", desc: "Session summary plus pending tasks, alerts, and unfinished work." },
    { value: "daily_briefing", label: "Full Briefing", desc: "Complete operational briefing: sessions, tasks, health, gateways, snapshots." },
];

export function ConciergeSection() {
    const config = useAgentStore((s) => s.conciergeConfig);
    const refresh = useAgentStore((s) => s.refreshConciergeConfig);
    const update = useAgentStore((s) => s.updateConciergeConfig);

    useEffect(() => { refresh(); }, []);

    const selectedLevel = DETAIL_LEVELS.find((l) => l.value === config.detail_level) || DETAIL_LEVELS[2];

    return (
        <Section title="Concierge">
            <SettingRow label="Enabled">
                <button
                    onClick={() => update({ ...config, enabled: !config.enabled })}
                    style={{ ...smallBtnStyle, color: config.enabled ? "#4ade80" : "var(--text-secondary)" }}
                >
                    {config.enabled ? "ON" : "OFF"}
                </button>
            </SettingRow>
            <SettingRow label="Detail Level">
                <select
                    value={config.detail_level}
                    onChange={(e) => update({ ...config, detail_level: e.target.value })}
                    style={{ ...inputStyle, width: 180 }}
                >
                    {DETAIL_LEVELS.map((l) => (
                        <option key={l.value} value={l.value}>{l.label}</option>
                    ))}
                </select>
            </SettingRow>
            <div style={{ fontSize: 11, color: "var(--text-secondary)", padding: "4px 0 8px", fontStyle: "italic" }}>
                {selectedLevel.desc}
            </div>
            <SettingRow label="Provider">
                <input
                    type="text"
                    value={config.provider || ""}
                    onChange={(e) => update({ ...config, provider: e.target.value || undefined })}
                    placeholder="Use main agent"
                    style={{ ...inputStyle, width: 180 }}
                />
            </SettingRow>
            <SettingRow label="Model">
                <input
                    type="text"
                    value={config.model || ""}
                    onChange={(e) => update({ ...config, model: e.target.value || undefined })}
                    placeholder="Use main agent"
                    style={{ ...inputStyle, width: 180 }}
                />
            </SettingRow>
        </Section>
    );
}
