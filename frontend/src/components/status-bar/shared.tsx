import type { CSSProperties } from "react";

export const statusBarRootStyle: CSSProperties = {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    background: "var(--bg-secondary)",
    borderTop: "1px solid var(--border)",
    height: "var(--status-bar-height)",
    padding: "0 var(--space-4)",
    fontSize: "var(--text-xs)",
    color: "var(--text-secondary)",
    flexShrink: 0,
};

export const dividerStyle: CSSProperties = {
    width: 1,
    height: 16,
    background: "var(--border)",
    margin: "0 var(--space-2)",
};
