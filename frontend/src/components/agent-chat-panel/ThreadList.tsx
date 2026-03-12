import type { AgentThread } from "../../lib/agentStore";
import { iconButtonStyle } from "./shared";

export function ThreadList({
    threads,
    searchQuery,
    onSearch,
    onSelect,
    onDelete,
}: {
    threads: AgentThread[];
    searchQuery: string;
    onSearch: (q: string) => void;
    onSelect: (t: AgentThread) => void;
    onDelete: (id: string) => void;
}) {
    return (
        <div style={{ height: "100%", overflow: "auto", padding: "var(--space-3)" }}>
            <div style={{ marginBottom: "var(--space-3)" }}>
                <input
                    type="text"
                    value={searchQuery}
                    onChange={(e) => onSearch(e.target.value)}
                    placeholder="Search threads..."
                    style={{
                        width: "100%",
                        background: "var(--bg-secondary)",
                        border: "1px solid var(--border)",
                        borderRadius: "var(--radius-md)",
                        color: "var(--text-primary)",
                        fontSize: "var(--text-sm)",
                        padding: "var(--space-2) var(--space-3)",
                        outline: "none",
                    }}
                />
            </div>

            {threads.length === 0 && (
                <div className="amux-empty-state">
                    <div className="amux-empty-state__icon">💬</div>
                    <div className="amux-empty-state__title">No conversations yet</div>
                    <div className="amux-empty-state__description">Create a new thread to start collaborating with the agent</div>
                </div>
            )}

            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                {threads.map((t) => (
                    <div
                        key={t.id}
                        onClick={() => onSelect(t)}
                        style={{
                            padding: "var(--space-3)",
                            borderRadius: "var(--radius-lg)",
                            border: "1px solid var(--glass-border)",
                            background: "var(--bg-secondary)",
                            cursor: "pointer",
                            transition: "all var(--transition-fast)",
                        }}
                        onMouseEnter={(e) => {
                            e.currentTarget.style.borderColor = "var(--border-strong)";
                            e.currentTarget.style.background = "var(--bg-tertiary)";
                        }}
                        onMouseLeave={(e) => {
                            e.currentTarget.style.borderColor = "var(--glass-border)";
                            e.currentTarget.style.background = "var(--bg-secondary)";
                        }}
                    >
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <div
                                    style={{
                                        fontSize: "var(--text-sm)",
                                        fontWeight: 600,
                                        overflow: "hidden",
                                        textOverflow: "ellipsis",
                                        whiteSpace: "nowrap",
                                    }}
                                >
                                    {t.title}
                                </div>

                                {t.lastMessagePreview && (
                                    <div
                                        style={{
                                            fontSize: "var(--text-xs)",
                                            color: "var(--text-muted)",
                                            overflow: "hidden",
                                            textOverflow: "ellipsis",
                                            whiteSpace: "nowrap",
                                            marginTop: "var(--space-1)",
                                        }}
                                    >
                                        {t.lastMessagePreview}
                                    </div>
                                )}

                                <div
                                    style={{
                                        fontSize: "var(--text-xs)",
                                        color: "var(--text-muted)",
                                        marginTop: "var(--space-2)",
                                    }}
                                >
                                    {t.messageCount} msgs · {new Date(t.updatedAt).toLocaleDateString()}
                                </div>
                            </div>

                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDelete(t.id);
                                }}
                                style={{
                                    ...iconButtonStyle,
                                    color: "var(--danger)",
                                    flexShrink: 0,
                                }}
                                title="Delete thread"
                            >
                                ✕
                            </button>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}