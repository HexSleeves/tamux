import { useMemo } from "react";
import { useAgentMissionStore } from "./agentMissionStore";
import { useWorkspaceStore } from "./workspaceStore";

export const DEFAULT_VIEW_WHEN_BY_ID: Record<string, string> = {
    "search-overlay": "searchOpen",
    "time-travel-slider": "timeTravelOpen",
    "web-browser-panel": "webBrowserOpen",
    "agent-chat-panel": "agentPanelOpen",
    "settings-panel": "settingsOpen",
    "session-vault-panel": "sessionVaultOpen",
    "command-log-panel": "commandLogOpen",
    "system-monitor-panel": "systemMonitorOpen",
    "file-manager-panel": "fileManagerOpen",
    "command-palette": "commandPaletteOpen",
    "notification-panel": "notificationPanelOpen",
    "command-history-picker": "commandHistoryOpen",
    "snippet-picker": "snippetPickerOpen",
    "execution-canvas": "canvasOpen",
    "agent-approval-overlay": "hasPendingApproval",
};

export type CDUIVisibilityFlags = Record<string, unknown>;

export const useCDUIVisibilityFlags = (): CDUIVisibilityFlags => {
    const commandPaletteOpen = useWorkspaceStore((state) => state.commandPaletteOpen);
    const notificationPanelOpen = useWorkspaceStore((state) => state.notificationPanelOpen);
    const settingsOpen = useWorkspaceStore((state) => state.settingsOpen);
    const sessionVaultOpen = useWorkspaceStore((state) => state.sessionVaultOpen);
    const commandLogOpen = useWorkspaceStore((state) => state.commandLogOpen);
    const commandHistoryOpen = useWorkspaceStore((state) => state.commandHistoryOpen);
    const searchOpen = useWorkspaceStore((state) => state.searchOpen);
    const snippetPickerOpen = useWorkspaceStore((state) => state.snippetPickerOpen);
    const agentPanelOpen = useWorkspaceStore((state) => state.agentPanelOpen);
    const systemMonitorOpen = useWorkspaceStore((state) => state.systemMonitorOpen);
    const fileManagerOpen = useWorkspaceStore((state) => state.fileManagerOpen);
    const canvasOpen = useWorkspaceStore((state) => state.canvasOpen);
    const timeTravelOpen = useWorkspaceStore((state) => state.timeTravelOpen);
    const webBrowserOpen = useWorkspaceStore((state) => state.webBrowserOpen);
    const hasPendingApproval = useAgentMissionStore((state) =>
        state.approvals.some((entry) => entry.status === "pending" && entry.handledAt === null),
    );

    return useMemo(() => ({
        commandPaletteOpen,
        notificationPanelOpen,
        settingsOpen,
        sessionVaultOpen,
        commandLogOpen,
        commandHistoryOpen,
        searchOpen,
        snippetPickerOpen,
        agentPanelOpen,
        systemMonitorOpen,
        fileManagerOpen,
        canvasOpen,
        timeTravelOpen,
        webBrowserOpen,
        hasPendingApproval,
    }), [
        agentPanelOpen,
        canvasOpen,
        commandHistoryOpen,
        commandLogOpen,
        commandPaletteOpen,
        fileManagerOpen,
        hasPendingApproval,
        notificationPanelOpen,
        searchOpen,
        sessionVaultOpen,
        settingsOpen,
        snippetPickerOpen,
        systemMonitorOpen,
        timeTravelOpen,
        webBrowserOpen,
    ]);
};

export const isCDUIViewVisible = (
    flags: CDUIVisibilityFlags,
    viewId: string,
    when?: string,
): boolean => {
    const normalizedWhen = typeof when === "string" ? when.trim() : when;
    const effectiveWhen = normalizedWhen || DEFAULT_VIEW_WHEN_BY_ID[viewId];
    if (!effectiveWhen) {
        return true;
    }

    return Boolean(flags[effectiveWhen]);
};