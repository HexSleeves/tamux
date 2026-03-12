export function SidebarHeader({
    workspacesCount,
    approvalsCount,
    reasoningCount,
    createWorkspace,
    query,
    setQuery,
}: {
    workspacesCount: number;
    approvalsCount: number;
    reasoningCount: number;
    createWorkspace: () => void;
    query: string;
    setQuery: (value: string) => void;
}) {
    return (
        <>
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "var(--space-3)",
                    padding: "var(--space-4)",
                    borderBottom: "1px solid var(--border)",
                }}
            >
                <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: "var(--space-3)" }}>
                    <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                        <span className="amux-panel-title">Runtime Environments</span>
                        <div style={{ fontSize: "var(--text-lg)", fontWeight: 700 }}>Workspace Fleet</div>
                        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", lineHeight: 1.5 }}>
                            Grouped environments for code, approvals, and telemetry
                        </div>
                    </div>

                    <button
                        onClick={createWorkspace}
                        style={createButtonStyle}
                        onMouseEnter={(e) => {
                            e.currentTarget.style.background = "rgba(94, 231, 223, 0.2)";
                            e.currentTarget.style.borderColor = "var(--accent)";
                        }}
                        onMouseLeave={(e) => {
                            e.currentTarget.style.background = "var(--accent-soft)";
                            e.currentTarget.style.borderColor = "var(--accent-soft)";
                        }}
                        title="New workspace"
                    >
                        +
                    </button>
                </div>

                <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "var(--space-2)" }}>
                    <SidebarMetric label="Workspaces" value={String(workspacesCount)} accent="var(--mission)" />
                    <SidebarMetric label="Approvals" value={String(approvalsCount)} accent="var(--approval)" />
                    <SidebarMetric label="Reasoning" value={String(reasoningCount)} accent="var(--reasoning)" />
                </div>
            </div>

            <div style={{ padding: "var(--space-3) var(--space-3) 0" }}>
                <input
                    type="text"
                    value={query}
                    onChange={(event) => setQuery(event.target.value)}
                    placeholder="Search workspaces..."
                    style={searchInputStyle}
                />
            </div>
        </>
    );
}

function SidebarMetric({ label, value, accent }: { label: string; value: string; accent: string }) {
    return (
        <div
            style={{
                padding: "var(--space-2)",
                background: "var(--bg-secondary)",
                border: "1px solid var(--border)",
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
            }}
        >
            <span className="amux-panel-title">{label}</span>
            <span style={{ color: accent, fontWeight: 700, fontSize: "var(--text-md)" }}>{value}</span>
        </div>
    );
}

const createButtonStyle: React.CSSProperties = {
    background: "var(--accent-soft)",
    border: "1px solid var(--accent-soft)",
    color: "var(--accent)",
    cursor: "pointer",
    fontSize: "var(--text-lg)",
    lineHeight: 1,
    padding: "var(--space-1) var(--space-2)",
    borderRadius: "var(--radius-md)",
    fontWeight: 600,
    transition: "all var(--transition-fast)",
};

const searchInputStyle: React.CSSProperties = {
    width: "100%",
    background: "var(--bg-secondary)",
    border: "1px solid var(--glass-border)",
    color: "var(--text-primary)",
    borderRadius: "var(--radius-md)",
    padding: "var(--space-2) var(--space-3)",
    fontSize: "var(--text-sm)",
    outline: "none",
};
