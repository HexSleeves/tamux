import type { SnapshotRecord } from "../../lib/agentMissionStore";
import { ActionButton, ContextCard, MetricRibbon, SectionTitle, inputStyle, memoryAreaStyle } from "./shared";

type ContextViewProps = {
    agentSettings: {
        activeProvider: string;
        contextWindowTokens: number;
        contextBudgetTokens: number;
    };
    snippets: Array<unknown>;
    transcripts: Array<unknown>;
    scopePaneId: string | null;
    threads: Array<unknown>;
    latestContextSnapshot?: { timestamp: number };
    memory: {
        frozenSnapshot: string;
        userProfile: string;
    };
    updateMemory: (field: "frozenSnapshot" | "userProfile", value: string) => void;
    historyQuery: string;
    setHistoryQuery: (value: string) => void;
    historySummary: unknown;
    historyHits: Array<unknown>;
    symbolQuery: string;
    setSymbolQuery: (value: string) => void;
    symbolHits: Array<unknown>;
    snapshots: SnapshotRecord[];
    scopeController?: {
        searchHistory?: (query: string, limit?: number) => Promise<unknown>;
        generateSkill?: (query?: string, title?: string) => Promise<unknown>;
        listSnapshots?: (workspaceId: string | null) => Promise<unknown>;
        restoreSnapshot?: (snapshotId: string) => Promise<unknown>;
    } | null;
    activeWorkspace?: { id?: string } | null;
};

export function ContextView(props: ContextViewProps) {
    return (
        <div style={{ padding: "var(--space-4)", height: "100%", overflow: "auto" }}>
            <MetricRibbon
                items={[
                    { label: "Provider", value: props.agentSettings.activeProvider },
                    { label: "Skills", value: String(props.snippets.length) },
                    { label: "Transcripts", value: String(props.transcripts.length) },
                ]}
            />

            <SectionTitle title="Live Context" subtitle="Current session envelope" />
            <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "var(--space-3)", marginBottom: "var(--space-4)" }}>
                <ContextCard label="Pane" value={props.scopePaneId ?? "none"} />
                <ContextCard label="Threads" value={String(props.threads.length)} />
                <ContextCard label="Context Length" value={`${props.agentSettings.contextWindowTokens.toLocaleString()} tok`} />
                <ContextCard label="Token Budget" value={`${props.agentSettings.contextBudgetTokens.toLocaleString()} tok`} />
                <ContextCard label="Snapshot Age" value={props.latestContextSnapshot ? new Date(props.latestContextSnapshot.timestamp).toLocaleTimeString() : "n/a"} />
            </div>

            <SectionTitle title="Frozen Snapshot" subtitle={`${props.memory.frozenSnapshot.length}/2200 chars`} />
            <textarea
                value={props.memory.frozenSnapshot}
                onChange={(e) => props.updateMemory("frozenSnapshot", e.target.value)}
                style={memoryAreaStyle}
                maxLength={2200}
            />

            <SectionTitle title="User Profile" subtitle={`${props.memory.userProfile.length}/1375 chars`} />
            <textarea
                value={props.memory.userProfile}
                onChange={(e) => props.updateMemory("userProfile", e.target.value)}
                style={{ ...memoryAreaStyle, minHeight: 120 }}
                maxLength={1375}
            />

            <SectionTitle title="History Recall" subtitle="Search across managed executions" />
            <div style={{ display: "flex", gap: "var(--space-2)", marginBottom: "var(--space-3)" }}>
                <input
                    value={props.historyQuery}
                    onChange={(e) => props.setHistoryQuery(e.target.value)}
                    placeholder="Search history..."
                    style={inputStyle}
                />
                <ActionButton onClick={() => void props.scopeController?.searchHistory?.(props.historyQuery, 8)}>Search</ActionButton>
                <ActionButton onClick={() => void props.scopeController?.generateSkill?.(props.historyQuery || undefined, props.historyQuery ? `${props.historyQuery} workflow` : "Recovered Workflow")}>Extract Skill</ActionButton>
            </div>

            <SectionTitle title="Snapshots" subtitle="Filesystem checkpoints" />
            <div style={{ display: "flex", gap: "var(--space-2)", marginBottom: "var(--space-3)" }}>
                <ActionButton onClick={() => void props.scopeController?.listSnapshots?.(props.activeWorkspace?.id ?? null)}>
                    Refresh
                </ActionButton>
            </div>
            {props.snapshots.length === 0 ? (
                <div
                    style={{
                        borderRadius: "var(--radius-lg)",
                        border: "1px solid var(--glass-border)",
                        background: "var(--bg-secondary)",
                        color: "var(--text-muted)",
                        fontSize: "var(--text-sm)",
                        padding: "var(--space-3)",
                    }}
                >
                    No snapshots loaded yet.
                </div>
            ) : (
                <div style={{ display: "grid", gap: "var(--space-2)" }}>
                    {props.snapshots.slice(0, 8).map((snapshot) => (
                        <div
                            key={snapshot.snapshotId}
                            style={{
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "space-between",
                                gap: "var(--space-3)",
                                borderRadius: "var(--radius-lg)",
                                border: "1px solid var(--glass-border)",
                                background: "var(--bg-secondary)",
                                padding: "var(--space-3)",
                            }}
                        >
                            <div style={{ minWidth: 0, flex: 1 }}>
                                <div style={{ fontSize: "var(--text-sm)", fontWeight: 600 }}>
                                    {snapshot.label}
                                </div>
                                <div
                                    style={{
                                        marginTop: 2,
                                        color: "var(--text-muted)",
                                        fontSize: "var(--text-xs)",
                                        wordBreak: "break-word",
                                    }}
                                >
                                    {new Date(snapshot.createdAt).toLocaleString()} · {snapshot.kind} · {snapshot.status}
                                </div>
                                {snapshot.command ? (
                                    <div
                                        style={{
                                            marginTop: "var(--space-1)",
                                            color: "var(--text-secondary)",
                                            fontSize: "var(--text-xs)",
                                            fontFamily: "var(--font-mono)",
                                            wordBreak: "break-word",
                                        }}
                                    >
                                        {snapshot.command}
                                    </div>
                                ) : null}
                            </div>
                            <ActionButton
                                onClick={() => void props.scopeController?.restoreSnapshot?.(snapshot.snapshotId)}
                                disabled={snapshot.status !== "ready" || !props.scopeController?.restoreSnapshot}
                            >
                                Restore
                            </ActionButton>
                        </div>
                    ))}
                </div>
            )}
            {props.snapshots.length > 8 ? (
                <div style={{ marginTop: "var(--space-2)", color: "var(--text-muted)", fontSize: "var(--text-xs)" }}>
                    Showing 8 of {props.snapshots.length} snapshots. Use Time Travel for the full list.
                </div>
            ) : null}
        </div>
    );
}
