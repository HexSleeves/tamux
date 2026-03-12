import { ReasoningStream } from "../ReasoningStream";
import { useAgentMissionStore } from "../../lib/agentMissionStore";
import { EmptyPanel, MetricRibbon, SectionTitle } from "./shared";

export function TraceView({
    operationalEvents,
    cognitiveEvents,
    pendingApprovals,
}: {
    operationalEvents: ReturnType<typeof useAgentMissionStore.getState>["operationalEvents"];
    cognitiveEvents: ReturnType<typeof useAgentMissionStore.getState>["cognitiveEvents"];
    pendingApprovals: ReturnType<typeof useAgentMissionStore.getState>["approvals"];
}) {
    return (
        <div style={{ padding: "var(--space-4)", height: "100%", overflow: "auto" }}>
            <MetricRibbon
                items={[
                    { label: "Ops", value: String(operationalEvents.length) },
                    { label: "Reasoning", value: String(cognitiveEvents.length) },
                    { label: "Pending", value: String(pendingApprovals.length) },
                ]}
            />

            <SectionTitle title="Reasoning Trace" subtitle="Parsed cognitive events" />
            <ReasoningStream events={cognitiveEvents} />

            <SectionTitle title="Operational Timeline" subtitle="Execution events" />

            {operationalEvents.length === 0 ? (
                <EmptyPanel message="No operational events captured yet." />
            ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                    {operationalEvents.map((event) => (
                        <TimelineRow key={event.id} event={event} />
                    ))}
                </div>
            )}
        </div>
    );
}

function TimelineRow({ event }: { event: ReturnType<typeof useAgentMissionStore.getState>["operationalEvents"][number] }) {
    return (
        <div
            style={{
                display: "flex",
                gap: "var(--space-3)",
                padding: "var(--space-3)",
                borderRadius: "var(--radius-lg)",
                border: "1px solid var(--glass-border)",
                background: "var(--bg-secondary)",
                alignItems: "flex-start",
            }}
        >
            <div style={{ minWidth: 72, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                {new Date(event.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
            </div>

            <div style={{ flex: 1 }}>
                <div style={{ fontSize: "var(--text-sm)", fontWeight: 600 }}>{event.kind}</div>

                {event.command && (
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", marginTop: "var(--space-1)" }}>
                        {event.command}
                    </div>
                )}
            </div>

            {event.exitCode !== null && (
                <div
                    style={{
                        fontSize: "var(--text-xs)",
                        color: event.exitCode === 0 ? "var(--success)" : "var(--danger)",
                    }}
                >
                    {event.exitCode === 0 ? "✓" : `✗ ${event.exitCode}`}
                </div>
            )}
        </div>
    );
}