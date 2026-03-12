import { useEffect, useMemo, useState } from "react";
import { useAgentMissionStore } from "../lib/agentMissionStore";
import { useAgentStore } from "../lib/agentStore";
import { useWorkspaceStore } from "../lib/workspaceStore";

export function TitleBar() {
  const workspace = useWorkspaceStore((s) => s.activeWorkspace());
  const surface = useWorkspaceStore((s) => s.activeSurface());
  const toggleCommandPalette = useWorkspaceStore((s) => s.toggleCommandPalette);
  const toggleCommandHistory = useWorkspaceStore((s) => s.toggleCommandHistory);
  const toggleCommandLog = useWorkspaceStore((s) => s.toggleCommandLog);
  const toggleAgentPanel = useWorkspaceStore((s) => s.toggleAgentPanel);
  const toggleSystemMonitor = useWorkspaceStore((s) => s.toggleSystemMonitor);
  const toggleSearch = useWorkspaceStore((s) => s.toggleSearch);
  const approvals = useAgentMissionStore((s) => s.approvals);
  const cognitiveEvents = useAgentMissionStore((s) => s.cognitiveEvents);
  const activeProvider = useAgentStore((s) => s.agentSettings.activeProvider);
  const [platform, setPlatform] = useState<string | null>(null);
  const [maximized, setMaximized] = useState(false);
  const approvalCount = useMemo(
    () => approvals.filter((entry) => entry.status === "pending").length,
    [approvals],
  );
  const traceCount = cognitiveEvents.length;

  useEffect(() => {
    const amux = (window as any).amux;
    if (!amux?.onWindowState) return;

    amux.getPlatform?.().then((value: string) => setPlatform(value));

    const cleanup = amux.onWindowState((state: string) => {
      setMaximized(state === "maximized");
    });

    amux.windowIsMaximized?.().then((m: boolean) => setMaximized(m));

    return cleanup;
  }, []);

  const hasAmux = typeof window !== "undefined" && "amux" in window;
  if (!hasAmux) return null;
  if (platform === null) return null;
  if (platform === "win32") return null;

  const amux = (window as any).amux;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        height: "var(--title-bar-height)",
        background: "var(--bg-secondary)",
        borderBottom: "1px solid var(--border)",
        WebkitAppRegion: "drag",
        flexShrink: 0,
        padding: "0 var(--space-3) 0 var(--space-4)",
        userSelect: "none",
      } as React.CSSProperties}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", fontSize: "var(--text-sm)", fontWeight: 600 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 1 }}>
          <span
            style={{
              color: "var(--mission)",
              letterSpacing: "0.15em",
              textTransform: "uppercase",
              fontSize: "var(--text-xs)",
              fontWeight: 700,
            }}
          >
            amux
          </span>
          <span style={{ color: "var(--text-muted)", fontSize: "var(--text-xs)" }}>agentic runtime</span>
        </div>

        {workspace && (
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", fontSize: "var(--text-xs)", color: "var(--text-secondary)" }}>
            <span className="amux-chip" style={{ color: workspace.accentColor }}>
              {workspace.name}
              {surface && <span style={{ color: "var(--text-muted)" }}>/{surface.name}</span>}
            </span>

            <span className="amux-chip">provider {activeProvider}</span>

            <span
              className="amux-chip"
              style={{
                color: approvalCount > 0 ? "var(--approval)" : "var(--success)",
                background: approvalCount > 0 ? "var(--approval-soft)" : "var(--success-soft)",
              }}
            >
              {approvalCount > 0 ? `${approvalCount} approvals` : "safe lane"}
            </span>

            <span className="amux-chip">trace {traceCount}</span>
          </div>
        )}
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-1)", WebkitAppRegion: "no-drag" } as React.CSSProperties}>
        <ActionPill label="Mission" onClick={toggleAgentPanel} tone="agent" />
        <ActionPill label="Monitor" onClick={toggleSystemMonitor} />
        <ActionPill label="Palette" onClick={toggleCommandPalette} />
        <ActionPill label="Search" onClick={toggleSearch} />
        <ActionPill label="History" onClick={toggleCommandHistory} />
        <ActionPill label="Logs" onClick={toggleCommandLog} />
      </div>

      <div style={{ display: "flex", WebkitAppRegion: "no-drag" } as React.CSSProperties}>
        <WindowButton label="─" onClick={() => amux.windowMinimize()} />
        <WindowButton label={maximized ? "❐" : "□"} onClick={() => amux.windowMaximize()} />
        <WindowButton label="✕" onClick={() => amux.windowClose()} isClose />
      </div>
    </div>
  );
}

function ActionPill({ label, onClick, tone = "default" }: { label: string; onClick: () => void; tone?: "default" | "agent" }) {
  return (
    <button
      onClick={onClick}
      style={{
        border: "1px solid",
        borderColor: tone === "agent" ? "var(--agent-soft)" : "var(--glass-border)",
        background: tone === "agent" ? "var(--agent-soft)" : "transparent",
        color: tone === "agent" ? "var(--agent)" : "var(--text-secondary)",
        borderRadius: "var(--radius-full)",
        padding: "var(--space-1) var(--space-2)",
        fontSize: "var(--text-xs)",
        cursor: "pointer",
        transition: "all var(--transition-fast)",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = tone === "agent" ? "rgba(130, 170, 255, 0.2)" : "var(--bg-tertiary)";
        e.currentTarget.style.color = "var(--text-primary)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = tone === "agent" ? "var(--agent-soft)" : "transparent";
        e.currentTarget.style.color = tone === "agent" ? "var(--agent)" : "var(--text-secondary)";
      }}
    >
      {label}
    </button>
  );
}

function WindowButton({ label, onClick, isClose }: { label: string; onClick: () => void; isClose?: boolean }) {
  return (
    <button
      onClick={onClick}
      style={{
        width: 44,
        height: "var(--title-bar-height)",
        border: "none",
        background: "transparent",
        color: "var(--text-muted)",
        cursor: "pointer",
        fontSize: "var(--text-sm)",
        fontFamily: "inherit",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        borderRadius: 0,
        transition: "all var(--transition-fast)",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = isClose ? "var(--danger)" : "var(--bg-tertiary)";
        e.currentTarget.style.color = isClose ? "#fff" : "var(--text-primary)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
        e.currentTarget.style.color = "var(--text-muted)";
      }}
    >
      {label}
    </button>
  );
}
