import {
  PersistedSession,
  PersistedWorkspace,
  PersistedSurface,
  PersistedPane,
} from "./types";
import { allLeafIds } from "./bspTree";
import { useWorkspaceStore } from "./workspaceStore";
import {
  deletePersistedPath,
  readLegacyLocalStorageJson,
  readPersistedJson,
  scheduleJsonWrite,
  writeLegacyLocalStorageJson,
} from "./persistence";

const STORAGE_KEY = "amux_session";
const SESSION_FILE = "session.json";
const VERSION = 1;

/** Serialize the current workspace state into a PersistedSession blob. */
export function captureSession(): PersistedSession {
  const state = useWorkspaceStore.getState();

  return {
    version: VERSION,
    windowState: {
      x: window.screenX,
      y: window.screenY,
      width: window.innerWidth,
      height: window.innerHeight,
      maximized: false,
    },
    sidebarVisible: state.sidebarVisible,
    sidebarWidth: state.sidebarWidth,
    workspaces: state.workspaces.map(
      (ws): PersistedWorkspace => ({
        id: ws.id,
        name: ws.name,
        icon: ws.icon,
        accentColor: ws.accentColor,
        cwd: ws.cwd,
        browser: (() => {
          const defaultUrl = "https://example.com";
          const fallbackUrl = ws.id === state.activeWorkspaceId ? state.webBrowserUrl : defaultUrl;
          const fallbackHistory = ws.id === state.activeWorkspaceId
            ? state.webBrowserHistory
            : [fallbackUrl];
          const fallbackHistoryIndex = ws.id === state.activeWorkspaceId
            ? state.webBrowserHistoryIndex
            : 0;
          const browser = state.workspaceBrowserState[ws.id];
          const normalizedUrl = browser?.url ?? fallbackUrl;
          const normalizedHistory = Array.isArray(browser?.history) && browser.history.length > 0
            ? browser.history
            : fallbackHistory.length > 0
              ? fallbackHistory
              : [normalizedUrl];
          const historyIndex = browser
            ? Math.max(0, Math.min(normalizedHistory.length - 1, browser.historyIndex))
            : Math.max(0, Math.min(normalizedHistory.length - 1, fallbackHistoryIndex));

          return {
            open: browser?.open ?? (ws.id === state.activeWorkspaceId ? state.webBrowserOpen : false),
            fullscreen: browser?.fullscreen ?? (ws.id === state.activeWorkspaceId ? state.webBrowserFullscreen : false),
            url: normalizedHistory[historyIndex] ?? normalizedUrl,
            history: normalizedHistory,
            historyIndex,
          };
        })(),
        surfaces: ws.surfaces.map(
          (sf): PersistedSurface => ({
            id: sf.id,
            name: sf.name,
            icon: sf.icon,
            layout: sf.layout,
            activePaneId: sf.activePaneId,
            paneNames: sf.paneNames,
            panes: allLeafIds(sf.layout).map(
              (paneId): PersistedPane => ({
                id: paneId,
                cwd: null,
                scrollback: null,
                commandHistory: [],
              })
            ),
          })
        ),
        activeSurfaceId: ws.activeSurfaceId,
      })
    ),
    activeWorkspaceId: state.activeWorkspaceId,
  };
}

/** Save the current session to localStorage. */
export function saveSession(): void {
  const data = captureSession();
  writeLegacyLocalStorageJson(STORAGE_KEY, data);
  scheduleJsonWrite(SESSION_FILE, data, 300);
}

/** Load a persisted session from localStorage. Returns null if nothing stored. */
export async function loadSession(): Promise<PersistedSession | null> {
  const diskSession = await readPersistedJson<PersistedSession>(SESSION_FILE);
  if (diskSession?.version === VERSION) {
    writeLegacyLocalStorageJson(STORAGE_KEY, diskSession);
    return diskSession;
  }

  const legacySession = readLegacyLocalStorageJson<PersistedSession>(STORAGE_KEY);
  if (legacySession?.version === VERSION) {
    scheduleJsonWrite(SESSION_FILE, legacySession, 0);
    return legacySession;
  }

  return null;
}

/** Clear the persisted session. */
export function clearSession(): void {
  localStorage.removeItem(STORAGE_KEY);
  void deletePersistedPath(SESSION_FILE);
}

/** Start auto-saving the session at a regular interval. Returns cleanup fn. */
export function startAutoSave(intervalMs: number = 30_000): () => void {
  const id = setInterval(saveSession, intervalMs);
  // Also save on unload.
  const onBeforeUnload = () => saveSession();
  window.addEventListener("beforeunload", onBeforeUnload);
  return () => {
    clearInterval(id);
    window.removeEventListener("beforeunload", onBeforeUnload);
  };
}
