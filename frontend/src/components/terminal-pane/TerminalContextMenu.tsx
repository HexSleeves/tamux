export interface TerminalContextMenuItem {
    label?: string;
    shortcut?: string;
    disabled?: boolean;
    danger?: boolean;
    separator?: boolean;
    action?: () => void;
}

export function TerminalContextMenu({
    visible,
    x,
    y,
    items,
    hideContextMenu,
}: {
    visible: boolean;
    x: number;
    y: number;
    items: TerminalContextMenuItem[];
    hideContextMenu: () => void;
}) {
    if (!visible) {
        return null;
    }

    return (
        <div
            onPointerDown={(event) => event.stopPropagation()}
            style={{
                position: "fixed",
                top: y,
                left: x,
                width: 220,
                background: "#141420",
                border: "1px solid #2a2a3c",
                borderRadius: 0,
                boxShadow: "none",
                padding: 6,
                zIndex: 4000,
            }}
        >
            {items.map((item, index) => {
                if (item.separator) {
                    return (
                        <div
                            key={`separator-${index}`}
                            style={{
                                height: 1,
                                margin: "6px 4px",
                                background: "#2a2a3c",
                            }}
                        />
                    );
                }

                return (
                    <button
                        key={item.label}
                        type="button"
                        disabled={item.disabled}
                        onClick={() => {
                            hideContextMenu();
                            item.action?.();
                        }}
                        style={{
                            width: "100%",
                            border: 0,
                            background: "transparent",
                            color: item.danger ? "#ef4444" : item.disabled ? "#6b6b85" : "#e2e2e9",
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            borderRadius: 0,
                            padding: "8px 10px",
                            fontSize: 12,
                            cursor: item.disabled ? "not-allowed" : "pointer",
                        }}
                    >
                        <span>{item.label}</span>
                        <span style={{ color: "#8f8fa8", fontSize: 11 }}>{item.shortcut ?? ""}</span>
                    </button>
                );
            })}
        </div>
    );
}