export function SharedCursor({ mode }: { mode: "idle" | "human" | "agent" | "approval" }) {
    const palette = {
        idle: { label: "Idle", color: "var(--text-secondary)" },
        human: { label: "Human", color: "var(--success)" },
        agent: { label: "Agent", color: "var(--accent)" },
        approval: { label: "Approval", color: "var(--warning)" },
    }[mode];

    return (
        <div
            style={{
                position: "absolute",
                top: 14,
                right: 14,
                zIndex: 4,
                display: "flex",
                alignItems: "center",
                gap: 8,
                padding: "6px 10px",
                borderRadius: 0,
                border: "1px solid var(--border)",
                background: "var(--surface-overlay-panel)",
                backdropFilter: "none",
                boxShadow: "none",
            }}
        >
            <span
                style={{
                    width: 8,
                    height: 8,
                    borderRadius: "50%",
                    background: palette.color,
                    boxShadow: "none",
                }}
            />
            <span style={{ fontSize: 11, color: palette.color, letterSpacing: "0.08em", textTransform: "uppercase" }}>
                {palette.label} Cursor
            </span>
        </div>
    );
}
