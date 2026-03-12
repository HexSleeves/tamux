import { closeBtnStyle } from "./shared";

export function TimeTravelHeader({
    snapshotCount,
    toggle,
}: {
    snapshotCount: number;
    toggle: () => void;
}) {
    return (
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
            <div style={{ display: "grid", gap: 2 }}>
                <span className="amux-panel-title" style={{ color: "var(--timeline)" }}>Time Travel</span>
                <span style={{ fontSize: 13, fontWeight: 700 }}>Snapshot Scrubber</span>
            </div>
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                <span style={{ fontSize: 11, color: "var(--text-secondary)" }}>
                    {snapshotCount} snapshot{snapshotCount !== 1 ? "s" : ""}
                </span>
                <button onClick={toggle} style={closeBtnStyle} title="Close (Esc)">
                    ✕
                </button>
            </div>
        </div>
    );
}
