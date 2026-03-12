import { sectionCardStyle, sectionTitleStyle } from "./shared";

export function BuilderSelectionPanel({
    nodeId,
    componentType,
    selectedEditable,
}: {
    nodeId: string | null;
    componentType: string | null;
    selectedEditable: boolean | null;
}) {
    return (
        <section>
            <div style={sectionTitleStyle}>Selection</div>
            <div style={sectionCardStyle}>
                <div style={{ fontSize: 12, color: "var(--text-muted)" }}>Node</div>
                <div style={{ fontSize: 14, fontWeight: 600 }}>{nodeId ?? "Nothing selected"}</div>
                <div style={{ marginTop: 8, fontSize: 12, color: "var(--text-muted)" }}>Component</div>
                <div style={{ fontSize: 14 }}>{componentType ?? "-"}</div>
                <div style={{ marginTop: 8, fontSize: 12, color: "var(--text-muted)" }}>Editable</div>
                <div style={{ fontSize: 14 }}>{selectedEditable === null ? "-" : String(selectedEditable)}</div>
            </div>
        </section>
    );
}
