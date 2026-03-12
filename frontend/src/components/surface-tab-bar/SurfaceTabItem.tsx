import { useEffect, useRef, useState } from "react";
import { SURFACE_ICONS, type SurfaceRecord } from "./shared";

export function SurfaceTabItem({
    surface,
    isActive,
    accentColor,
    approvalCount,
    paneCount,
    onSelect,
    onClose,
    onRename,
    onSetIcon,
}: {
    surface: SurfaceRecord;
    isActive: boolean;
    accentColor: string;
    approvalCount: number;
    paneCount: number;
    onSelect: () => void;
    onClose: () => void;
    onRename: (name: string) => void;
    onSetIcon: (icon: string) => void;
}) {
    const [editing, setEditing] = useState(false);
    const [draftName, setDraftName] = useState(surface.name);
    const [draftIcon, setDraftIcon] = useState(surface.icon);
    const commitTimeoutRef = useRef<number | null>(null);

    const cancelScheduledCommit = () => {
        if (commitTimeoutRef.current !== null) {
            window.clearTimeout(commitTimeoutRef.current);
            commitTimeoutRef.current = null;
        }
    };

    const commit = () => {
        cancelScheduledCommit();
        onRename(draftName.trim() || surface.name);
        onSetIcon(draftIcon.trim() || surface.icon);
        setEditing(false);
    };

    const scheduleCommit = () => {
        commitTimeoutRef.current = window.setTimeout(() => {
            commit();
        }, 150);
    };

    useEffect(() => () => cancelScheduledCommit(), []);

    return (
        <div
            onClick={onSelect}
            onDoubleClick={() => setEditing(true)}
            style={{
                display: "flex",
                alignItems: "center",
                gap: "var(--space-2)",
                padding: "0 var(--space-3)",
                height: 28,
                fontSize: "var(--text-xs)",
                cursor: "pointer",
                background: isActive ? "var(--bg-tertiary)" : "transparent",
                color: isActive ? "var(--text-primary)" : "var(--text-muted)",
                border: "1px solid",
                borderColor: isActive ? accentColor : "transparent",
                borderRadius: "var(--radius-md)",
                whiteSpace: "nowrap",
                transition: "all var(--transition-fast)",
            }}
        >
            <span style={{ fontSize: "var(--text-xs)", textTransform: "uppercase", opacity: 0.8 }}>{surface.icon}</span>

            {editing ? (
                <div style={{ display: "flex", gap: "var(--space-1)" }} onClick={(event) => event.stopPropagation()}>
                    <input
                        type="text"
                        value={draftName}
                        onChange={(event) => setDraftName(event.target.value)}
                        onBlur={scheduleCommit}
                        onKeyDown={(event) => {
                            if (event.key === "Enter") commit();
                            if (event.key === "Escape") {
                                setDraftName(surface.name);
                                setDraftIcon(surface.icon);
                                setEditing(false);
                            }
                        }}
                        autoFocus
                        style={{
                            background: "var(--bg-surface)",
                            border: "1px solid var(--glass-border)",
                            color: "var(--text-primary)",
                            borderRadius: "var(--radius-sm)",
                            padding: "2px 6px",
                            fontSize: "var(--text-xs)",
                            outline: "none",
                            width: 100,
                        }}
                    />
                    <select
                        value={draftIcon}
                        onChange={(event) => setDraftIcon(event.target.value)}
                        onFocus={cancelScheduledCommit}
                        onBlur={scheduleCommit}
                        style={{
                            background: "var(--bg-surface)",
                            border: "1px solid var(--glass-border)",
                            color: "var(--text-primary)",
                            borderRadius: "var(--radius-sm)",
                            padding: "2px 6px",
                            fontSize: "var(--text-xs)",
                            outline: "none",
                        }}
                    >
                        {SURFACE_ICONS.map((icon) => (
                            <option key={icon} value={icon}>{icon}</option>
                        ))}
                    </select>
                </div>
            ) : (
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                    <span style={{ fontWeight: isActive ? 600 : 400 }}>{surface.name}</span>
                    <span style={{ opacity: 0.6 }}>
                        {paneCount}
                        {approvalCount > 0 ? <span style={{ color: "var(--approval)", marginLeft: "var(--space-1)" }}>· {approvalCount}</span> : null}
                    </span>
                </div>
            )}

            <button
                onClick={(event) => {
                    event.stopPropagation();
                    if (editing) {
                        commit();
                    } else {
                        setDraftName(surface.name);
                        setDraftIcon(surface.icon);
                        setEditing(true);
                    }
                }}
                style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    fontSize: "var(--text-xs)",
                    padding: "0 var(--space-1)",
                    opacity: 0,
                    transition: "opacity var(--transition-fast)",
                }}
                onMouseEnter={(event) => {
                    event.currentTarget.style.opacity = "1";
                }}
            >
                ✎
            </button>

            <button
                onClick={(event) => {
                    event.stopPropagation();
                    onClose();
                }}
                style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    fontSize: "var(--text-sm)",
                    padding: "0 var(--space-1)",
                    opacity: 0,
                    transition: "opacity var(--transition-fast)",
                }}
                onMouseEnter={(event) => {
                    event.currentTarget.style.opacity = "1";
                }}
            >
                ×
            </button>
        </div>
    );
}
