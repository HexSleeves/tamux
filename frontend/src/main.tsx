import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import CDUIApp from "./CDUIApp";
import { loadSession } from "./lib/sessionPersistence";
import { hydrateCommandLogStore } from "./lib/commandLogStore";
import { hydrateAgentMissionStore } from "./lib/agentMissionStore";
import { hydrateKeybindStore } from "./lib/keybindStore";
import { hydrateSettingsStore } from "./lib/settingsStore";
import { hydrateAgentStore } from "./lib/agentStore";
import { hydrateTranscriptStore } from "./lib/transcriptStore";
import { hydrateFileManagerStore } from "./lib/fileManagerStore";
import { isCDUIEnabled } from "./lib/cduiMode";
import { useWorkspaceStore } from "./lib/workspaceStore";
import "./styles/global.css";

const renderRoot = (useCDUI: boolean): void => {
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      {useCDUI ? <CDUIApp /> : <App />}
    </React.StrictMode>
  );
};

async function bootstrap() {
  const useCDUI = isCDUIEnabled();

  await Promise.all([
    hydrateSettingsStore(),
    hydrateAgentStore(),
    hydrateCommandLogStore(),
    hydrateAgentMissionStore(),
    hydrateKeybindStore(),
    hydrateTranscriptStore(),
    hydrateFileManagerStore(),
  ]);

  const persistedSession = await loadSession();
  if (persistedSession) {
    useWorkspaceStore.getState().hydrateSession(persistedSession);
  }

  renderRoot(useCDUI);
}

void bootstrap();
