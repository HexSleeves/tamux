import type { CSSProperties } from "react";

export type WebBrowserPanelProps = {
    style?: CSSProperties;
    className?: string;
};

export function getBrowserContainerStyle(webBrowserFullscreen: boolean): CSSProperties {
    if (webBrowserFullscreen) {
        return {
            position: "fixed",
            inset: 0,
            zIndex: 3500,
            background: "var(--bg-primary)",
            borderLeft: "1px solid var(--border)",
            display: "flex",
            flexDirection: "column",
        };
    }

    return {
        width: 620,
        minWidth: 360,
        maxWidth: 1200,
        height: "100%",
        display: "flex",
        flexDirection: "column",
        background: "var(--bg-primary)",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-xl)",
        resize: "horizontal",
        overflow: "hidden",
    };
}

export const navBtnStyle: CSSProperties = {
    height: 26,
    minWidth: 28,
    border: "1px solid var(--border)",
    borderRadius: "var(--radius-sm)",
    background: "var(--bg-primary)",
    color: "var(--text-primary)",
    fontSize: 12,
    cursor: "pointer",
    padding: "0 8px",
};
