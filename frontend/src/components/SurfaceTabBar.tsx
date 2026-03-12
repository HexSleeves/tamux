import { allLeafIds } from "../lib/bspTree";
import { useAgentMissionStore } from "../lib/agentMissionStore";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { SurfaceTabActions } from "./surface-tab-bar/SurfaceTabActions";
import { SurfaceCreateButton } from "./surface-tab-bar/SurfaceCreateButton";
import { SurfaceTabItem } from "./surface-tab-bar/SurfaceTabItem";
import { dividerStyle } from "./surface-tab-bar/shared";

export function SurfaceTabBar() {
  const ws = useWorkspaceStore((s) => s.activeWorkspace());
  const createSurface = useWorkspaceStore((s) => s.createSurface);
  const closeSurface = useWorkspaceStore((s) => s.closeSurface);
  const setActiveSurface = useWorkspaceStore((s) => s.setActiveSurface);
  const renameSurface = useWorkspaceStore((s) => s.renameSurface);
  const setSurfaceIcon = useWorkspaceStore((s) => s.setSurfaceIcon);
  const toggleSidebar = useWorkspaceStore((s) => s.toggleSidebar);
  const sidebarVisible = useWorkspaceStore((s) => s.sidebarVisible);
  const splitActive = useWorkspaceStore((s) => s.splitActive);
  const applyPresetLayout = useWorkspaceStore((s) => s.applyPresetLayout);
  const equalizeLayout = useWorkspaceStore((s) => s.equalizeLayout);
  const toggleZoom = useWorkspaceStore((s) => s.toggleZoom);
  const toggleWebBrowser = useWorkspaceStore((s) => s.toggleWebBrowser);
  const approvals = useAgentMissionStore((s) => s.approvals);

  const surfaces = ws?.surfaces ?? [];
  const activeSurfaceId = ws?.activeSurfaceId ?? null;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        background: "var(--bg-secondary)",
        borderBottom: "1px solid var(--border)",
        height: "var(--tab-height)",
        overflow: "hidden",
        padding: "0 var(--space-2)",
        gap: "var(--space-2)",
        flexShrink: 0,
      }}
    >
      <button
        onClick={toggleSidebar}
        style={{
          background: sidebarVisible ? "var(--accent-soft)" : "transparent",
          border: "1px solid",
          borderColor: sidebarVisible ? "var(--accent-soft)" : "var(--glass-border)",
          color: sidebarVisible ? "var(--accent)" : "var(--text-muted)",
          cursor: "pointer",
          fontSize: "var(--text-sm)",
          padding: "0 var(--space-2)",
          height: 26,
          minWidth: 28,
          borderRadius: "var(--radius-md)",
          transition: "all var(--transition-fast)",
        }}
        title="Toggle sidebar"
      >
        ☰
      </button>

      <div style={dividerStyle} />

      <SurfaceTabActions
        splitActive={splitActive}
        applyPresetLayout={applyPresetLayout}
        equalizeLayout={equalizeLayout}
        toggleZoom={toggleZoom}
        toggleWebBrowser={toggleWebBrowser}
      />

      <div style={dividerStyle} />

      <div
        style={{
          display: "flex",
          alignItems: "center",
          flex: 1,
          overflow: "auto",
          gap: "var(--space-1)",
        }}
      >
        {surfaces.map((sf) => (
          <SurfaceTabItem
            key={sf.id}
            surface={sf}
            isActive={sf.id === activeSurfaceId}
            accentColor={ws?.accentColor ?? "var(--accent)"}
            approvalCount={approvals.filter((entry) => entry.surfaceId === sf.id && entry.status === "pending").length}
            paneCount={allLeafIds(sf.layout).length}
            onSelect={() => setActiveSurface(sf.id)}
            onClose={() => closeSurface(sf.id)}
            onRename={(name) => renameSurface(sf.id, name)}
            onSetIcon={(icon) => setSurfaceIcon(sf.id, icon)}
          />
        ))}
      </div>

      <SurfaceCreateButton createSurface={createSurface} />
    </div>
  );
}
