import { useEffect } from "react";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { matchesKeybinding, useKeybindStore } from "../lib/keybindStore";

/**
 * Global keyboard shortcuts for the application.
 *
 * Keybindings:
 *   Ctrl+D           Split horizontal
 *   Ctrl+Shift+D     Split vertical
 *   Ctrl+Shift+W     Close active pane
 *   Ctrl+Shift+Z     Toggle zoom pane
 *   Ctrl+Alt+Arrow   Focus directional (left/right/up/down)
 *   Ctrl+T           New surface (tab)
 *   Ctrl+W           Close surface (tab)
 *   Ctrl+Tab          Next surface
 *   Ctrl+Shift+Tab   Prev surface
 *   Ctrl+Shift+N     New workspace
 *   Ctrl+1..9        Switch workspace by index
 *   Ctrl+Shift+P     Command palette
 *   Ctrl+B           Toggle sidebar
 *   Ctrl+I           Notifications panel
 *   Ctrl+,           Settings
 *   Ctrl+Shift+V     Session vault
 *   Ctrl+Shift+L     Command log
 *   Ctrl+Shift+F     Search in buffer
 *   Ctrl+Alt+H       Command history picker
 *   Ctrl+Shift+M     System monitor
 */
export function useHotkeys() {
  const store = useWorkspaceStore;
  const bindings = useKeybindStore((s) => s.bindings);

  useEffect(() => {
    function runAction(action: string) {
      const s = store.getState();
      switch (action) {
        case "splitHorizontal":
          s.splitActive("horizontal");
          return;
        case "splitVertical":
          s.splitActive("vertical");
          return;
        case "closePane": {
          const paneId = s.activePaneId();
          if (paneId) s.closePane(paneId);
          return;
        }
        case "toggleZoom":
          s.toggleZoom();
          return;
        case "focusLeft":
          s.focusDirection("left");
          return;
        case "focusRight":
          s.focusDirection("right");
          return;
        case "focusUp":
          s.focusDirection("up");
          return;
        case "focusDown":
          s.focusDirection("down");
          return;
        case "newSurface":
          s.createSurface();
          return;
        case "closeSurface": {
          const surface = s.activeSurface();
          if (surface) s.closeSurface(surface.id);
          return;
        }
        case "nextSurface":
          s.nextSurface();
          return;
        case "prevSurface":
          s.prevSurface();
          return;
        case "newWorkspace":
          s.createWorkspace();
          return;
        case "toggleCommandPalette":
          s.toggleCommandPalette();
          return;
        case "toggleSidebar":
          s.toggleSidebar();
          return;
        case "toggleNotifications":
          s.toggleNotificationPanel();
          return;
        case "toggleSettings":
          s.toggleSettings();
          return;
        case "toggleSessionVault":
          s.toggleSessionVault();
          return;
        case "toggleCommandLog":
          s.toggleCommandLog();
          return;
        case "toggleSearch":
          s.toggleSearch();
          return;
        case "toggleCommandHistory":
          s.toggleCommandHistory();
          return;
        case "toggleSnippets":
          s.toggleSnippetPicker();
          return;
        case "toggleAgentPanel":
          s.toggleAgentPanel();
          return;
        case "toggleSystemMonitor":
          s.toggleSystemMonitor();
          return;
        case "toggleFileManager":
          s.toggleFileManager();
          return;
        case "toggleCanvas":
          s.toggleCanvas();
          return;
        case "toggleTimeTravel":
          s.toggleTimeTravel();
          return;
        case "switchWorkspace1":
        case "switchWorkspace2":
        case "switchWorkspace3":
        case "switchWorkspace4":
        case "switchWorkspace5":
        case "switchWorkspace6":
        case "switchWorkspace7":
        case "switchWorkspace8":
        case "switchWorkspace9":
          s.switchWorkspaceByIndex(Number(action.slice(-1)));
          return;
        case "nextWorkspace":
          s.nextWorkspace();
          return;
        case "prevWorkspace":
          s.prevWorkspace();
          return;
      }
    }

    function handleKeyDown(e: KeyboardEvent) {
      for (const binding of bindings) {
        if (binding.combo && matchesKeybinding(binding.combo, e)) {
          e.preventDefault();
          runAction(binding.action);
          return;
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [bindings]);
}
