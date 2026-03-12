import { create } from "zustand";
import { allLeafIds } from "../../lib/bspTree";
import { useWorkspaceStore } from "../../lib/workspaceStore";
import { buildCodingAgentLaunchCommand } from "./agentDefinitions";
import { discoverCodingAgents, sendCommandToPane } from "./bridge";
import type { CodingAgentsDiscoveryStatus, DiscoveredCodingAgent } from "./types";

type LaunchTarget = {
    workspaceId: string;
    surfaceId: string;
    paneId: string;
};

type CodingAgentsState = {
    agents: DiscoveredCodingAgent[];
    status: CodingAgentsDiscoveryStatus;
    error: string | null;
    selectedAgentId: string | null;
    selectedWorkspaceId: string | null;
    selectedSurfaceId: string | null;
    selectedPaneId: string | null;
    launchState: "idle" | "launching" | "success" | "error";
    launchError: string | null;
    lastLaunchCommand: string | null;
    refreshAgents: () => Promise<void>;
    setSelectedAgentId: (agentId: string | null) => void;
    setSelectedWorkspaceId: (workspaceId: string | null) => void;
    setSelectedSurfaceId: (surfaceId: string | null) => void;
    setSelectedPaneId: (paneId: string | null) => void;
    syncTargetSelection: (workspaceId: string | null, surfaceId: string | null, paneId: string | null) => void;
    launchSelectedAgent: () => Promise<boolean>;
};

function resolveLaunchTarget(
    workspaceId: string | null,
    surfaceId: string | null,
    paneId: string | null,
): LaunchTarget | null {
    const store = useWorkspaceStore.getState();
    const workspace = (workspaceId
        ? store.workspaces.find((entry) => entry.id === workspaceId)
        : undefined) ?? store.activeWorkspace();
    if (!workspace) {
        return null;
    }

    const surface = (surfaceId
        ? workspace.surfaces.find((entry) => entry.id === surfaceId)
        : undefined) ?? workspace.surfaces.find((entry) => entry.id === workspace.activeSurfaceId) ?? workspace.surfaces[0];
    if (!surface) {
        return null;
    }

    const paneIds = allLeafIds(surface.layout);
    const targetPaneId = paneId && paneIds.includes(paneId)
        ? paneId
        : surface.activePaneId && paneIds.includes(surface.activePaneId)
            ? surface.activePaneId
            : paneIds[0] ?? null;
    if (!targetPaneId) {
        return null;
    }

    return {
        workspaceId: workspace.id,
        surfaceId: surface.id,
        paneId: targetPaneId,
    };
}

function pickDefaultAgent(agents: DiscoveredCodingAgent[], currentAgentId: string | null): string | null {
    if (currentAgentId && agents.some((agent) => agent.id === currentAgentId)) {
        return currentAgentId;
    }

    return agents.find((agent) => agent.available)?.id ?? agents[0]?.id ?? null;
}

export const useCodingAgentsStore = create<CodingAgentsState>((set, get) => ({
    agents: [],
    status: "idle",
    error: null,
    selectedAgentId: null,
    selectedWorkspaceId: null,
    selectedSurfaceId: null,
    selectedPaneId: null,
    launchState: "idle",
    launchError: null,
    lastLaunchCommand: null,

    refreshAgents: async () => {
        set({ status: "loading", error: null });

        try {
            const agents = await discoverCodingAgents();
            set((state) => ({
                agents,
                status: agents.some((agent) => agent.available) ? "ready" : "error",
                error: agents.some((agent) => agent.available) ? null : agents[0]?.error ?? "No coding agents were found on PATH.",
                selectedAgentId: pickDefaultAgent(agents, state.selectedAgentId),
            }));
        } catch (error) {
            set({
                agents: [],
                status: "error",
                error: error instanceof Error ? error.message : "Failed to discover coding agents.",
            });
        }
    },

    setSelectedAgentId: (selectedAgentId) => set({ selectedAgentId, launchError: null }),
    setSelectedWorkspaceId: (selectedWorkspaceId) => set({ selectedWorkspaceId, selectedSurfaceId: null, selectedPaneId: null, launchError: null }),
    setSelectedSurfaceId: (selectedSurfaceId) => set({ selectedSurfaceId, selectedPaneId: null, launchError: null }),
    setSelectedPaneId: (selectedPaneId) => set({ selectedPaneId, launchError: null }),

    syncTargetSelection: (workspaceId, surfaceId, paneId) => {
        const target = resolveLaunchTarget(
            get().selectedWorkspaceId ?? workspaceId,
            get().selectedSurfaceId ?? surfaceId,
            get().selectedPaneId ?? paneId,
        );
        if (!target) {
            return;
        }

        set((state) => ({
            selectedWorkspaceId: state.selectedWorkspaceId && state.selectedWorkspaceId === target.workspaceId ? state.selectedWorkspaceId : target.workspaceId,
            selectedSurfaceId: state.selectedSurfaceId && state.selectedSurfaceId === target.surfaceId ? state.selectedSurfaceId : target.surfaceId,
            selectedPaneId: state.selectedPaneId && state.selectedPaneId === target.paneId ? state.selectedPaneId : target.paneId,
        }));
    },

    launchSelectedAgent: async () => {
        const state = get();
        const selectedAgent = state.agents.find((agent) => agent.id === state.selectedAgentId);
        if (!selectedAgent) {
            set({ launchState: "error", launchError: "Choose a coding agent before launching." });
            return false;
        }

        if (!selectedAgent.available) {
            set({ launchState: "error", launchError: `${selectedAgent.label} is not available on PATH.` });
            return false;
        }

        const target = resolveLaunchTarget(state.selectedWorkspaceId, state.selectedSurfaceId, state.selectedPaneId);
        if (!target) {
            set({ launchState: "error", launchError: "Choose a valid target workspace, surface, and pane." });
            return false;
        }

        const command = buildCodingAgentLaunchCommand(selectedAgent);
        if (!command) {
            set({ launchState: "error", launchError: `No launch command is defined for ${selectedAgent.label}.` });
            return false;
        }

        set({ launchState: "launching", launchError: null, lastLaunchCommand: command });

        const workspaceStore = useWorkspaceStore.getState();
        workspaceStore.setActiveWorkspace(target.workspaceId);
        workspaceStore.setActiveSurface(target.surfaceId);
        workspaceStore.setActivePaneId(target.paneId);

        try {
            await sendCommandToPane(target.paneId, command);
            set({ launchState: "success", launchError: null, selectedWorkspaceId: target.workspaceId, selectedSurfaceId: target.surfaceId, selectedPaneId: target.paneId });
            return true;
        } catch (error) {
            set({
                launchState: "error",
                launchError: error instanceof Error ? error.message : `Failed to launch ${selectedAgent.label}.`,
            });
            return false;
        }
    },
}));