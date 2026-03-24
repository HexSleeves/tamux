import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { useAgentMissionStore } from "../lib/agentMissionStore";
import { useAgentStore } from "../lib/agentStore";
import { useKeybindStore } from "../lib/keybindStore";
import { useNotificationStore } from "../lib/notificationStore";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { Badge, Button, Separator, cn } from "./ui";

type TitleMenuItem = {
  id: string;
  label: string;
  shortcut?: string;
  tone?: "default" | "agent";
  onSelect: () => void;
};

type TitleMenuGroup = {
  id: string;
  label: string;
  items: TitleMenuItem[];
};

export function TitleBar() {
  const workspace = useWorkspaceStore((s) => s.activeWorkspace());
  const surface = useWorkspaceStore((s) => s.activeSurface());
  const createWorkspace = useWorkspaceStore((s) => s.createWorkspace);
  const createSurface = useWorkspaceStore((s) => s.createSurface);
  const splitActive = useWorkspaceStore((s) => s.splitActive);
  const toggleZoom = useWorkspaceStore((s) => s.toggleZoom);
  const toggleSidebar = useWorkspaceStore((s) => s.toggleSidebar);
  const toggleSettings = useWorkspaceStore((s) => s.toggleSettings);
  const settingsOpen = useWorkspaceStore((s) => s.settingsOpen);
  const toggleFileManager = useWorkspaceStore((s) => s.toggleFileManager);
  const toggleSessionVault = useWorkspaceStore((s) => s.toggleSessionVault);
  const toggleCommandPalette = useWorkspaceStore((s) => s.toggleCommandPalette);
  const toggleCommandHistory = useWorkspaceStore((s) => s.toggleCommandHistory);
  const toggleCommandLog = useWorkspaceStore((s) => s.toggleCommandLog);
  const toggleSnippets = useWorkspaceStore((s) => s.toggleSnippetPicker);
  const toggleAgentPanel = useWorkspaceStore((s) => s.toggleAgentPanel);
  const toggleTimeTravel = useWorkspaceStore((s) => s.toggleTimeTravel);
  const toggleSystemMonitor = useWorkspaceStore((s) => s.toggleSystemMonitor);
  const toggleSearch = useWorkspaceStore((s) => s.toggleSearch);
  const toggleNotificationPanel = useWorkspaceStore((s) => s.toggleNotificationPanel);
  const notificationPanelOpen = useWorkspaceStore((s) => s.notificationPanelOpen);
  const approvals = useAgentMissionStore((s) => s.approvals);
  const cognitiveEvents = useAgentMissionStore((s) => s.cognitiveEvents);
  const notifications = useNotificationStore((s) => s.notifications);
  const active_provider = useAgentStore((s) => s.agentSettings.active_provider);
  const bindings = useKeybindStore((s) => s.bindings);
  const [platform, setPlatform] = useState<string | null>(null);
  const [maximized, setMaximized] = useState(false);
  const [openMenuId, setOpenMenuId] = useState<string | null>(null);
  const menuBarRef = useRef<HTMLDivElement | null>(null);
  const approvalCount = useMemo(
    () => approvals.filter((entry) => entry.status === "pending").length,
    [approvals],
  );
  const traceCount = cognitiveEvents.length;
  const unreadNotifications = useMemo(
    () => notifications.filter((entry) => !entry.isRead).length,
    [notifications],
  );

  const shortcutFor = useCallback(
    (action: string): string | undefined => bindings.find((binding) => binding.action === action)?.combo,
    [bindings],
  );

  const openAbout = useCallback(() => {
    if (!settingsOpen) {
      toggleSettings();
    }
    window.setTimeout(() => {
      window.dispatchEvent(new CustomEvent("tamux-open-settings-tab", {
        detail: { tab: "about" },
      }));
      window.dispatchEvent(new CustomEvent("amux-open-settings-tab", {
        detail: { tab: "about" },
      }));
    }, 50);
  }, [settingsOpen, toggleSettings]);

  const linuxMenus = useMemo<TitleMenuGroup[]>(() => [
    {
      id: "workspace",
      label: "Workspace",
      items: [
        { id: "new-workspace", label: "New Workspace", shortcut: shortcutFor("newWorkspace"), onSelect: () => createWorkspace() },
        { id: "new-surface", label: "New Surface", shortcut: shortcutFor("newSurface"), onSelect: () => createSurface() },
        { id: "split-horizontal", label: "Split Horizontal", shortcut: shortcutFor("splitHorizontal"), onSelect: () => splitActive("horizontal") },
        { id: "split-vertical", label: "Split Vertical", shortcut: shortcutFor("splitVertical"), onSelect: () => splitActive("vertical") },
        { id: "toggle-zoom", label: "Zoom Pane", shortcut: shortcutFor("toggleZoom"), onSelect: toggleZoom },
      ],
    },
    {
      id: "panels",
      label: "Panels",
      items: [
        { id: "mission", label: "Mission Console", shortcut: shortcutFor("toggleAgentPanel"), tone: "agent", onSelect: toggleAgentPanel },
        { id: "notifications", label: "Notifications", shortcut: shortcutFor("toggleNotifications"), onSelect: toggleNotificationPanel },
        { id: "monitor", label: "System Monitor", shortcut: shortcutFor("toggleSystemMonitor"), onSelect: toggleSystemMonitor },
        { id: "files", label: "File Manager", shortcut: shortcutFor("toggleFileManager"), onSelect: toggleFileManager },
        { id: "vault", label: "Session Vault", shortcut: shortcutFor("toggleSessionVault"), onSelect: toggleSessionVault },
        { id: "settings", label: "Settings", shortcut: shortcutFor("toggleSettings"), onSelect: toggleSettings },
        { id: "sidebar", label: "Toggle Sidebar", shortcut: shortcutFor("toggleSidebar"), onSelect: toggleSidebar },
      ],
    },
    {
      id: "tools",
      label: "Tools",
      items: [
        { id: "palette", label: "Command Palette", shortcut: shortcutFor("toggleCommandPalette"), onSelect: toggleCommandPalette },
        { id: "search", label: "Search", shortcut: shortcutFor("toggleSearch"), onSelect: toggleSearch },
        { id: "history", label: "Command History", shortcut: shortcutFor("toggleCommandHistory"), onSelect: toggleCommandHistory },
        { id: "logs", label: "Command Log", shortcut: shortcutFor("toggleCommandLog"), onSelect: toggleCommandLog },
        { id: "time-travel", label: "Time Travel", shortcut: shortcutFor("toggleTimeTravel"), onSelect: toggleTimeTravel },
        { id: "snippets", label: "Snippets", shortcut: shortcutFor("toggleSnippets"), onSelect: toggleSnippets },
        { id: "runtime", label: "Runtime Settings", onSelect: openAbout },
      ],
    },
  ], [
    createSurface,
    createWorkspace,
    openAbout,
    shortcutFor,
    splitActive,
    toggleAgentPanel,
    toggleCommandHistory,
    toggleCommandLog,
    toggleNotificationPanel,
    toggleSnippets,
    toggleTimeTravel,

    toggleCommandPalette,
    toggleFileManager,
    toggleSearch,
    toggleSessionVault,
    toggleSettings,
    toggleSidebar,
    toggleSystemMonitor,
    toggleZoom,
  ]);

  useEffect(() => {
    const amux = (window as any).tamux ?? (window as any).amux;
    if (!amux?.onWindowState) return;

    amux.getPlatform?.().then((value: string) => setPlatform(value));

    const cleanup = amux.onWindowState((state: string) => {
      setMaximized(state === "maximized");
    });

    amux.windowIsMaximized?.().then((m: boolean) => setMaximized(m));

    return cleanup;
  }, []);

  useEffect(() => {
    if (!openMenuId) {
      return;
    }

    const handlePointerDown = (event: MouseEvent) => {
      if (!menuBarRef.current?.contains(event.target as Node)) {
        setOpenMenuId(null);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpenMenuId(null);
      }
    };

    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleEscape);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleEscape);
    };
  }, [openMenuId]);

  const hasAmux = typeof window !== "undefined" && ("tamux" in window || "amux" in window);
  if (!hasAmux) return null;
  if (platform === null) return null;
  if (platform === "win32") return null;

  const amux = (window as any).tamux ?? (window as any).amux;

  return (
    <div
      className="flex h-[var(--title-bar-height)] shrink-0 items-center justify-between border-b border-[var(--border)] bg-[var(--bg-secondary)] px-[var(--space-3)] pl-[var(--space-4)] select-none"
      style={{ WebkitAppRegion: "drag" } as CSSProperties}
    >
      <div className="flex items-center gap-[var(--space-3)] text-[var(--text-sm)] font-semibold">
        <div className="flex flex-col gap-px">
          <span className="text-[var(--text-xs)] font-bold uppercase tracking-[0.15em] text-[var(--mission)]">Tamux</span>
          <span className="text-[var(--text-xs)] text-[var(--text-muted)]">agentic runtime</span>
        </div>

        {workspace && (
          <div className="flex items-center gap-[var(--space-2)] text-[var(--text-xs)] text-[var(--text-secondary)]">
            <Badge variant="default" className="max-w-[14rem] truncate" style={{ color: workspace.accentColor }}>
              {workspace.name}
              {surface && <span className="text-[var(--text-muted)]">/{surface.name}</span>}
            </Badge>

            <Badge variant="default">provider {active_provider}</Badge>

            <Badge variant={approvalCount > 0 ? "approval" : "success"}>
              {approvalCount > 0 ? `${approvalCount} approvals` : "safe lane"}
            </Badge>

            <Badge variant="default">trace {traceCount}</Badge>
          </div>
        )}
      </div>

      {platform === "linux" ? (
        <div
          ref={menuBarRef}
          className="relative flex items-stretch gap-[2px]"
          style={{ WebkitAppRegion: "no-drag" } as CSSProperties}
        >
          {linuxMenus.map((menu) => (
            <div key={menu.id} className="relative">
              <Button
                type="button"
                onClick={() => setOpenMenuId((current) => (current === menu.id ? null : menu.id))}
                variant={openMenuId === menu.id ? "secondary" : "ghost"}
                size="sm"
                className={cn(
                  "mt-[6px] h-7 rounded-[var(--radius-md)] px-[var(--space-3)] text-[var(--text-xs)] font-semibold tracking-[0.03em]",
                  openMenuId === menu.id && "border-[var(--mission-border)] bg-[var(--mission-soft)] text-[var(--text-primary)]"
                )}
              >
                {menu.label}
              </Button>

              {openMenuId === menu.id ? (
                <div
                  className="absolute left-0 top-[calc(100%+8px)] z-40 grid min-w-[15rem] gap-[2px] rounded-[var(--radius-lg)] border border-[var(--glass-border)] bg-[rgba(15,18,32,0.98)] p-[var(--space-2)] shadow-[0_18px_48px_rgba(0,0,0,0.35)]"
                >
                  {menu.items.map((item) => (
                    <Button
                      key={item.id}
                      type="button"
                      onClick={() => {
                        setOpenMenuId(null);
                        item.onSelect();
                      }}
                      variant={item.tone === "agent" ? "agent" : "ghost"}
                      size="sm"
                      className="w-full justify-between rounded-[var(--radius-md)] px-[var(--space-3)] py-[var(--space-2)] text-left text-[var(--text-sm)]"
                    >
                      <span>{item.label}</span>
                      <span className="whitespace-nowrap text-[var(--text-xs)] text-[var(--text-muted)]">
                        {item.shortcut ?? ""}
                      </span>
                    </Button>
                  ))}
                </div>
              ) : null}
            </div>
          ))}
        </div>
      ) : (
        <div className="flex items-center gap-[var(--space-1)]" style={{ WebkitAppRegion: "no-drag" } as CSSProperties}>
          <ActionPill label="Mission" onClick={toggleAgentPanel} tone="agent" />
          <ActionPill label="Monitor" onClick={toggleSystemMonitor} />
          <ActionPill label="Palette" onClick={toggleCommandPalette} />
          <ActionPill label="Search" onClick={toggleSearch} />
          <ActionPill label="History" onClick={toggleCommandHistory} />
          <ActionPill label="Logs" onClick={toggleCommandLog} />
        </div>
      )}

      <div className="flex items-center gap-[var(--space-2)]" style={{ WebkitAppRegion: "no-drag" } as CSSProperties}>
        <Button
          type="button"
          onClick={toggleNotificationPanel}
          title={unreadNotifications > 0 ? `${unreadNotifications} unread notification(s)` : "Open notifications"}
          variant={notificationPanelOpen || unreadNotifications > 0 ? "outline" : "ghost"}
          size="sm"
          className={cn(
            "h-7 gap-[6px] rounded-[var(--radius-md)] px-2 text-[var(--text-xs)] font-bold",
            notificationPanelOpen && "border-[var(--mission-border)] bg-[var(--mission-soft)] text-[var(--text-primary)]",
            !notificationPanelOpen && unreadNotifications > 0 && "border-[var(--approval-border)] bg-[var(--approval-soft)] text-[var(--approval)]"
          )}
        >
          <span className="tracking-[0.06em]">NTF</span>
          {unreadNotifications > 0 ? (
            <Badge variant="approval" className="min-w-4 px-1 py-0 text-[10px] font-extrabold leading-4 text-[var(--bg-primary)]">
              {unreadNotifications > 99 ? "99+" : unreadNotifications}
            </Badge>
          ) : null}
        </Button>
        <Separator orientation="vertical" className="h-4 bg-[var(--border)]" />
        <WindowButton label="─" onClick={() => amux.windowMinimize()} />
        <WindowButton label={maximized ? "❐" : "□"} onClick={() => amux.windowMaximize()} />
        <WindowButton label="✕" onClick={() => amux.windowClose()} isClose />
      </div>
    </div>
  );
}

function ActionPill({ label, onClick, tone = "default" }: { label: string; onClick: () => void; tone?: "default" | "agent" }) {
  return (
    <Button
      onClick={onClick}
      variant={tone === "agent" ? "agent" : "ghost"}
      size="sm"
      className="h-7 rounded-full px-[var(--space-2)] text-[var(--text-xs)]"
    >
      {label}
    </Button>
  );
}

function WindowButton({ label, onClick, isClose }: { label: string; onClick: () => void; isClose?: boolean }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex h-[var(--title-bar-height)] w-11 items-center justify-center border-none bg-transparent font-inherit text-[var(--text-sm)] text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]",
        isClose && "hover:bg-[var(--danger)] hover:text-white"
      )}
    >
      {label}
    </button>
  );
}
