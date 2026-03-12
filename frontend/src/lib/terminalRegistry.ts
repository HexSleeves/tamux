export type TerminalSendOptions = {
    execute?: boolean;
    bracketed?: boolean;
    trackHistory?: boolean;
    managed?: boolean;
    rationale?: string;
    allowNetwork?: boolean;
    sandboxEnabled?: boolean;
    languageHint?: string;
    source?: "human" | "agent" | "replay" | "gateway";
};

export type TerminalSearchOptions = {
    regex?: boolean;
    caseSensitive?: boolean;
};

export type TerminalSearchResult = {
    query: string;
    matchCount: number;
    currentIndex: number;
};

export type TerminalController = {
    sendText: (text: string, options?: TerminalSendOptions) => Promise<boolean>;
    getSnapshot: () => string;
    search: (query: string, direction?: "next" | "prev", reset?: boolean, options?: TerminalSearchOptions) => TerminalSearchResult;
    clearSearch: () => void;
    searchHistory: (query: string, limit?: number) => Promise<boolean>;
    generateSkill: (query?: string, title?: string) => Promise<boolean>;
    findSymbol: (workspaceRoot: string, symbol: string, limit?: number) => Promise<boolean>;
    listSnapshots: (workspaceId?: string | null) => Promise<boolean>;
    restoreSnapshot: (snapshotId: string) => Promise<boolean>;
};

const controllers = new Map<string, TerminalController>();

export function registerTerminalController(
    paneId: string,
    controller: TerminalController,
): () => void {
    controllers.set(paneId, controller);
    return () => {
        const current = controllers.get(paneId);
        if (current === controller) {
            controllers.delete(paneId);
        }
    };
}

export function getTerminalController(paneId: string | null | undefined): TerminalController | undefined {
    if (!paneId) return undefined;
    return controllers.get(paneId);
}

export function getTerminalSnapshot(paneId: string | null | undefined): string {
    return getTerminalController(paneId)?.getSnapshot() ?? "";
}

export function searchTerminal(
    paneId: string | null | undefined,
    query: string,
    direction: "next" | "prev" = "next",
    reset = false,
    options?: TerminalSearchOptions,
): TerminalSearchResult {
    return getTerminalController(paneId)?.search(query, direction, reset, options) ?? {
        query,
        matchCount: 0,
        currentIndex: 0,
    };
}

export function clearTerminalSearch(paneId: string | null | undefined): void {
    getTerminalController(paneId)?.clearSearch();
}

export function hasTerminalController(paneId: string | null | undefined): boolean {
    if (!paneId) return false;
    return controllers.has(paneId);
}