import { useEffect, type CSSProperties } from "react";
import { CommandLogPanel } from "@/components/CommandLogPanel";
import { FileManagerPanel } from "@/components/FileManagerPanel";
import { SessionVaultPanel } from "@/components/SessionVaultPanel";
import { SystemMonitorPanel } from "@/components/SystemMonitorPanel";
import { TerminalPane } from "@/components/TerminalPane";
import { WebBrowserPanel } from "@/components/WebBrowserPanel";
import { useWorkspaceStore } from "@/lib/workspaceStore";
import { zoraiTools, type ZoraiToolId } from "./tools";

type ToolsProps = {
  activeTool: ZoraiToolId;
  onSelectTool: (toolId: ZoraiToolId) => void;
};

const embeddedPanelStyle: CSSProperties = {
  position: "relative",
  inset: "auto",
  zIndex: "auto",
  width: "100%",
  minWidth: 0,
  maxWidth: "none",
  height: "100%",
  minHeight: 0,
  maxHeight: "none",
  padding: 0,
  background: "transparent",
  border: "1px solid var(--zorai-border)",
  borderRadius: 7,
  overflow: "hidden",
};

export function ToolsRail({ activeTool, onSelectTool }: ToolsProps) {
  return (
    <div className="zorai-rail-stack">
      <div className="zorai-section-label">Tools</div>
      {zoraiTools.map((tool) => (
        <button
          type="button"
          key={tool.id}
          className={[
            "zorai-rail-card",
            "zorai-rail-card--button",
            tool.id === activeTool ? "zorai-rail-card--active" : "",
          ].filter(Boolean).join(" ")}
          onClick={() => onSelectTool(tool.id)}
        >
          <strong>{tool.title}</strong>
          <span>{tool.description}</span>
        </button>
      ))}
    </div>
  );
}

export function ToolsView({ activeTool, onSelectTool }: ToolsProps) {
  const selectedTool = zoraiTools.find((tool) => tool.id === activeTool) ?? zoraiTools[0];

  return (
    <section className="zorai-feature-surface zorai-tools-surface">
      <div className="zorai-view-header">
        <div>
          <div className="zorai-kicker">Tools</div>
          <h1>{selectedTool.title}</h1>
          <p>
            Zorai keeps terminal multiplexing as a useful capability, while the default
            shell remains centered on threads, goals, and workspace orchestration.
          </p>
        </div>
      </div>

      <div className="zorai-tool-layout">
        <div className="zorai-tool-picker" aria-label="Tool picker">
          {zoraiTools.map((tool) => (
            <button
              type="button"
              key={tool.id}
              className={[
                "zorai-tool-card",
                tool.id === activeTool ? "zorai-tool-card--active" : "",
              ].filter(Boolean).join(" ")}
              onClick={() => onSelectTool(tool.id)}
            >
              <strong>{tool.title}</strong>
              <span>{tool.description}</span>
            </button>
          ))}
        </div>

        <div className="zorai-tool-frame">
          <ToolSurface activeTool={activeTool} />
        </div>
      </div>
    </section>
  );
}

function ToolSurface({ activeTool }: { activeTool: ZoraiToolId }) {
  useEmbeddedToolState(activeTool);
  const activePaneId = useWorkspaceStore((state) => state.activePaneId());

  if (activeTool === "terminal") {
    if (!activePaneId) {
      return (
        <div className="zorai-tool-empty">
          <strong>No active terminal pane</strong>
          <span>Create or select a workspace surface to attach the terminal tool.</span>
        </div>
      );
    }

    return (
      <div className="zorai-terminal-tool">
        <TerminalPane paneId={activePaneId} hideHeader />
      </div>
    );
  }

  if (activeTool === "files") {
    return <FileManagerPanel style={embeddedPanelStyle} className="zorai-embedded-tool-panel" />;
  }

  if (activeTool === "browser") {
    return <WebBrowserPanel style={embeddedPanelStyle} className="zorai-embedded-tool-panel" />;
  }

  if (activeTool === "history") {
    return <CommandLogPanel style={embeddedPanelStyle} className="zorai-embedded-tool-panel" />;
  }

  if (activeTool === "system") {
    return <SystemMonitorPanel style={embeddedPanelStyle} className="zorai-embedded-tool-panel" />;
  }

  return <SessionVaultPanel style={embeddedPanelStyle} className="zorai-embedded-tool-panel" />;
}

function useEmbeddedToolState(activeTool: ZoraiToolId) {
  const fileManagerOpen = useWorkspaceStore((state) => state.fileManagerOpen);
  const commandLogOpen = useWorkspaceStore((state) => state.commandLogOpen);
  const sessionVaultOpen = useWorkspaceStore((state) => state.sessionVaultOpen);
  const systemMonitorOpen = useWorkspaceStore((state) => state.systemMonitorOpen);
  const webBrowserOpen = useWorkspaceStore((state) => state.webBrowserOpen);
  const setWebBrowserOpen = useWorkspaceStore((state) => state.setWebBrowserOpen);
  const setWebBrowserFullscreen = useWorkspaceStore((state) => state.setWebBrowserFullscreen);

  useEffect(() => {
    useWorkspaceStore.setState({
      fileManagerOpen: activeTool === "files",
      commandLogOpen: activeTool === "history",
      sessionVaultOpen: activeTool === "vault",
      systemMonitorOpen: activeTool === "system",
    });
    setWebBrowserOpen(activeTool === "browser");
    if (activeTool === "browser") setWebBrowserFullscreen(false);

    return () => {
      useWorkspaceStore.setState({
        fileManagerOpen: false,
        commandLogOpen: false,
        sessionVaultOpen: false,
        systemMonitorOpen: false,
      });
      setWebBrowserOpen(false);
    };
  }, [activeTool, setWebBrowserFullscreen, setWebBrowserOpen]);

  useEffect(() => {
    if (activeTool === "files" && !fileManagerOpen) {
      useWorkspaceStore.setState({ fileManagerOpen: true });
    }
    if (activeTool === "history" && !commandLogOpen) {
      useWorkspaceStore.setState({ commandLogOpen: true });
    }
    if (activeTool === "vault" && !sessionVaultOpen) {
      useWorkspaceStore.setState({ sessionVaultOpen: true });
    }
    if (activeTool === "system" && !systemMonitorOpen) {
      useWorkspaceStore.setState({ systemMonitorOpen: true });
    }
    if (activeTool === "browser" && !webBrowserOpen) {
      setWebBrowserOpen(true);
    }
  }, [
    activeTool,
    commandLogOpen,
    fileManagerOpen,
    sessionVaultOpen,
    setWebBrowserOpen,
    systemMonitorOpen,
    webBrowserOpen,
  ]);
}
