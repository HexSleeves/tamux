import { ActionButton } from "./SurfaceTabButton";

export function SurfaceTabActions({
    splitActive,
    applyPresetLayout,
    equalizeLayout,
    toggleZoom,
    toggleWebBrowser,
}: {
    splitActive: (direction: "horizontal" | "vertical") => void;
    applyPresetLayout: (preset: "2-columns" | "grid-2x2" | "main-stack") => void;
    equalizeLayout: () => void;
    toggleZoom: () => void;
    toggleWebBrowser: () => void;
}) {
    return (
        <div style={{ display: "flex", alignItems: "center", gap: "var(--space-1)" }}>
            <ActionButton title="Split right" onClick={() => splitActive("horizontal")}>⇄</ActionButton>
            <ActionButton title="Split down" onClick={() => splitActive("vertical")}>⇅</ActionButton>
            <ActionButton title="2-column layout" onClick={() => applyPresetLayout("2-columns")}>2C</ActionButton>
            <ActionButton title="Grid layout" onClick={() => applyPresetLayout("grid-2x2")}>▦</ActionButton>
            <ActionButton title="Main + stack layout" onClick={() => applyPresetLayout("main-stack")}>◫</ActionButton>
            <ActionButton title="Equalize ratios" onClick={equalizeLayout}>═</ActionButton>
            <ActionButton title="Toggle zoom" onClick={toggleZoom}>⛶</ActionButton>
            <ActionButton title="Toggle browser" onClick={toggleWebBrowser}>WEB</ActionButton>
        </div>
    );
}
