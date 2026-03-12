import { useEffect, useMemo } from "react";
import { allLeafIds } from "../../lib/bspTree";
import { useWorkspaceStore } from "../../lib/workspaceStore";
import { buildCodingAgentLaunchCommand } from "../../plugins/coding-agents/agentDefinitions";
import { useCodingAgentsStore } from "../../plugins/coding-agents/store";
import { ActionButton, ContextCard, EmptyPanel, MetricRibbon, SectionTitle, inputStyle } from "./shared";

export function CodingAgentsView() {
    const workspaces = useWorkspaceStore((state) => state.workspaces);
    const activeWorkspaceId = useWorkspaceStore((state) => state.activeWorkspaceId);
    const activeSurface = useWorkspaceStore((state) => state.activeSurface());
    const activePaneId = useWorkspaceStore((state) => state.activePaneId());

    const {
        agents,
        status,
        error,
        selectedAgentId,
        selectedWorkspaceId,
        selectedSurfaceId,
        selectedPaneId,
        launchState,
        launchError,
        lastLaunchCommand,
        refreshAgents,
        setSelectedAgentId,
        setSelectedWorkspaceId,
        setSelectedSurfaceId,
        setSelectedPaneId,
        syncTargetSelection,
        launchSelectedAgent,
    } = useCodingAgentsStore();

    useEffect(() => {
        if (status === "idle") {
            void refreshAgents();
        }
    }, [refreshAgents, status]);

    useEffect(() => {
        syncTargetSelection(activeWorkspaceId, activeSurface?.id ?? null, activePaneId);
    }, [activePaneId, activeSurface?.id, activeWorkspaceId, syncTargetSelection]);

    const selectedWorkspace = useMemo(() => {
        return workspaces.find((workspace) => workspace.id === selectedWorkspaceId)
            ?? workspaces.find((workspace) => workspace.id === activeWorkspaceId)
            ?? workspaces[0]
            ?? null;
    }, [activeWorkspaceId, selectedWorkspaceId, workspaces]);

    const selectedSurface = useMemo(() => {
        if (!selectedWorkspace) {
            return null;
        }

        return selectedWorkspace.surfaces.find((surface) => surface.id === selectedSurfaceId)
            ?? selectedWorkspace.surfaces.find((surface) => surface.id === selectedWorkspace.activeSurfaceId)
            ?? selectedWorkspace.surfaces[0]
            ?? null;
    }, [selectedSurfaceId, selectedWorkspace]);

    const paneOptions = useMemo(() => {
        if (!selectedSurface) {
            return [] as Array<{ id: string; label: string }>;
        }

        return allLeafIds(selectedSurface.layout).map((paneId) => ({
            id: paneId,
            label: selectedSurface.paneNames[paneId] ?? paneId,
        }));
    }, [selectedSurface]);

    const selectedAgent = agents.find((agent) => agent.id === selectedAgentId) ?? null;
    const availableCount = agents.filter((agent) => agent.available).length;

    return (
        <div style={{ padding: "var(--space-4)", overflow: "auto", height: "100%" }}>
            <MetricRibbon
                items={[
                    { label: "Supported", value: String(agents.length || 0) },
                    { label: "Available", value: String(availableCount), accent: availableCount > 0 ? "var(--accent)" : "var(--text-muted)" },
                    { label: "Target", value: selectedSurface ? `${selectedSurface.name} · ${selectedPaneId ?? selectedSurface.activePaneId ?? "none"}` : "none" },
                ]}
            />

            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: "var(--space-3)", marginBottom: "var(--space-4)" }}>
                <div>
                    <div style={{ fontSize: "var(--text-lg)", fontWeight: 700 }}>Coding Agents</div>
                    <div style={{ fontSize: "var(--text-sm)", color: "var(--text-muted)", marginTop: 4 }}>
                        Discover local coding CLIs on PATH and launch them into a selected terminal pane.
                    </div>
                </div>
                <ActionButton onClick={() => void refreshAgents()}>
                    {status === "loading" ? "Scanning..." : "Refresh"}
                </ActionButton>
            </div>

            <SectionTitle title="Target Surface" subtitle="Choose where the coding agent should start." />
            <div style={{ display: "grid", gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: "var(--space-3)", marginBottom: "var(--space-4)" }}>
                <label style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                    Workspace
                    <select
                        value={selectedWorkspace?.id ?? ""}
                        onChange={(event) => setSelectedWorkspaceId(event.target.value || null)}
                        style={{ ...inputStyle, width: "100%" }}
                    >
                        {workspaces.map((workspace) => (
                            <option key={workspace.id} value={workspace.id}>{workspace.name}</option>
                        ))}
                    </select>
                </label>
                <label style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                    Surface
                    <select
                        value={selectedSurface?.id ?? ""}
                        onChange={(event) => setSelectedSurfaceId(event.target.value || null)}
                        style={{ ...inputStyle, width: "100%" }}
                    >
                        {(selectedWorkspace?.surfaces ?? []).map((surface) => (
                            <option key={surface.id} value={surface.id}>{surface.name}</option>
                        ))}
                    </select>
                </label>
                <label style={{ display: "flex", flexDirection: "column", gap: 6, fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                    Pane
                    <select
                        value={selectedPaneId ?? selectedSurface?.activePaneId ?? paneOptions[0]?.id ?? ""}
                        onChange={(event) => setSelectedPaneId(event.target.value || null)}
                        style={{ ...inputStyle, width: "100%" }}
                    >
                        {paneOptions.map((pane) => (
                            <option key={pane.id} value={pane.id}>{pane.label}</option>
                        ))}
                    </select>
                </label>
            </div>

            <SectionTitle title="Available Agents" subtitle="Detected CLIs can be launched directly into the selected pane." />
            {agents.length === 0 && status === "loading" ? (
                <EmptyPanel message="Scanning PATH for known coding-agent CLIs..." />
            ) : null}

            {agents.length === 0 && status !== "loading" ? (
                <EmptyPanel message="No coding-agent definitions are registered." />
            ) : null}

            <div style={{ display: "grid", gap: "var(--space-3)" }}>
                {agents.map((agent) => {
                    const isSelected = agent.id === selectedAgentId;
                    return (
                        <button
                            key={agent.id}
                            type="button"
                            onClick={() => setSelectedAgentId(agent.id)}
                            style={{
                                display: "grid",
                                gap: "var(--space-2)",
                                textAlign: "left",
                                padding: "var(--space-3)",
                                borderRadius: "var(--radius-lg)",
                                border: `1px solid ${isSelected ? "var(--accent)" : "var(--glass-border)"}`,
                                background: isSelected ? "var(--accent-soft)" : "var(--bg-secondary)",
                                cursor: "pointer",
                            }}
                        >
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: "var(--space-3)" }}>
                                <div>
                                    <div style={{ fontSize: "var(--text-sm)", fontWeight: 700 }}>{agent.label}</div>
                                    <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", marginTop: 4 }}>{agent.description}</div>
                                </div>
                                <span
                                    style={{
                                        padding: "4px 10px",
                                        borderRadius: 999,
                                        fontSize: "var(--text-xs)",
                                        border: "1px solid var(--border)",
                                        color: agent.available ? "var(--success, #86efac)" : "var(--text-muted)",
                                    }}
                                >
                                    {agent.available ? "Available" : "Unavailable"}
                                </span>
                            </div>
                            <div style={{ display: "grid", gridTemplateColumns: "repeat(3, minmax(0, 1fr))", gap: "var(--space-2)" }}>
                                <ContextCard label="Executable" value={agent.executable ?? agent.executables.join(", ")} />
                                <ContextCard label="Version" value={agent.version ?? "Not detected"} />
                                <ContextCard label="Path" value={agent.path ?? agent.error ?? "Not found on PATH"} />
                            </div>
                        </button>
                    );
                })}
            </div>

            <SectionTitle title="Launch" subtitle="Start the selected coding agent inside the selected pane." />
            <div style={{ display: "grid", gap: "var(--space-3)" }}>
                <ContextCard label="Command Preview" value={selectedAgent ? buildCodingAgentLaunchCommand(selectedAgent) || "Unavailable" : "Select an agent"} />
                {error ? <EmptyPanel message={error} /> : null}
                {launchError ? <EmptyPanel message={launchError} /> : null}
                {launchState === "success" && lastLaunchCommand ? <EmptyPanel message={`Launched: ${lastLaunchCommand}`} /> : null}
                <div style={{ display: "flex", gap: "var(--space-2)", justifyContent: "flex-end" }}>
                    <button
                        type="button"
                        onClick={() => void launchSelectedAgent()}
                        disabled={!selectedAgent || !selectedAgent.available || launchState === "launching"}
                        style={{
                            padding: "var(--space-2) var(--space-4)",
                            borderRadius: "var(--radius-md)",
                            border: "1px solid var(--accent)",
                            background: "var(--accent)",
                            color: "var(--bg-primary)",
                            cursor: !selectedAgent || !selectedAgent.available || launchState === "launching" ? "not-allowed" : "pointer",
                            opacity: !selectedAgent || !selectedAgent.available || launchState === "launching" ? 0.6 : 1,
                            fontWeight: 700,
                        }}
                    >
                        {launchState === "launching" ? "Launching..." : "Launch in Pane"}
                    </button>
                </div>
            </div>
        </div>
    );
}