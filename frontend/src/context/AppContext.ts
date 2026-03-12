import { createContext } from "react";

export interface AppState {
  workspace: unknown;
  panels: unknown[];
  settings: Record<string, unknown>;
}

export const defaultAppState: AppState = {
  workspace: null,
  panels: [],
  settings: {},
};

export const AppContext = createContext<AppState>(defaultAppState);
