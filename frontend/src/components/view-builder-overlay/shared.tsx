import type { CSSProperties } from "react";
import type { ViewDocument, UIViewNode } from "../../schemas/uiSchema";

export const BUILDER_PRIMITIVE_COMPONENTS = new Set(["Container", "Header", "Text", "Button", "Input", "TextArea", "Select", "Divider", "Spacer"]);

export const overlayShellStyle: CSSProperties = {
    position: "fixed",
    top: 20,
    right: 20,
    width: 320,
    maxHeight: "calc(100vh - 40px)",
    overflow: "auto",
    zIndex: 5000,
    borderRadius: 16,
    border: "1px solid var(--border-strong)",
    background: "var(--surface-overlay-panel)",
    boxShadow: "var(--shadow-xl)",
    backdropFilter: "blur(var(--panel-blur))",
    color: "var(--text-primary)",
};

export const sectionTitleStyle: CSSProperties = {
    fontSize: 12,
    fontWeight: 700,
    marginBottom: 8,
    color: "var(--text-secondary)",
};

export const sectionCardStyle: CSSProperties = {
    border: "1px solid var(--border)",
    borderRadius: 12,
    padding: 12,
    background: "var(--surface-muted)",
};

export function findNodeById(document: ViewDocument, nodeId: string): UIViewNode | null {
    const visit = (node: UIViewNode): UIViewNode | null => {
        if (node.id === nodeId) {
            return node;
        }

        for (const child of node.children ?? []) {
            const match = visit(child);
            if (match) {
                return match;
            }
        }

        return null;
    };

    return visit(document.layout)
        ?? (document.fallback ? visit(document.fallback) : null)
        ?? Object.values(document.blocks ?? {}).map((block) => visit(block.layout)).find(Boolean)
        ?? null;
}

export function findNodeEditable(document: ViewDocument, nodeId: string): boolean | null {
    const visit = (node: UIViewNode): boolean | null => {
        if (node.id === nodeId) {
            return node.builder?.editable ?? null;
        }

        for (const child of node.children ?? []) {
            const match = visit(child);
            if (match !== null) {
                return match;
            }
        }

        return null;
    };

    const layoutMatch = visit(document.layout);
    if (layoutMatch !== null) {
        return layoutMatch;
    }

    if (document.fallback) {
        const fallbackMatch = visit(document.fallback);
        if (fallbackMatch !== null) {
            return fallbackMatch;
        }
    }

    for (const block of Object.values(document.blocks ?? {})) {
        const blockMatch = visit(block.layout);
        if (blockMatch !== null) {
            return blockMatch;
        }
    }

    return null;
}

export function styleValue(node: UIViewNode, key: string): unknown {
    const style = node.props?.style;
    if (!style || typeof style !== "object" || Array.isArray(style)) {
        return undefined;
    }

    return (style as Record<string, unknown>)[key];
}

export function stringValue(value: unknown): string {
    return typeof value === "string" ? value : value == null ? "" : String(value);
}

export function chipButtonStyle(active: boolean) {
    return {
        border: active ? "1px solid var(--surface-highlight-accent-border)" : "1px solid var(--border-strong)",
        background: active ? "var(--surface-highlight-accent)" : "var(--surface-soft)",
        color: "var(--text-primary)",
        borderRadius: 999,
        padding: "6px 10px",
        cursor: "pointer",
        textTransform: "capitalize",
    } satisfies CSSProperties;
}

export function actionButtonStyle(variant: "primary" | "secondary") {
    return {
        border: `1px solid ${variant === "primary" ? "var(--accent-border)" : "var(--border-strong)"}`,
        background: variant === "primary" ? "var(--accent-soft)" : "var(--surface-soft)",
        color: "var(--text-primary)",
        borderRadius: 10,
        padding: "10px 12px",
        cursor: "pointer",
        fontWeight: 600,
    } satisfies CSSProperties;
}
