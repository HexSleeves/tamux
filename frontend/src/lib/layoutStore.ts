import { create } from "zustand";
import {
  BspTree,
  SplitDirection,
  createLeaf,
  splitPane,
  removePane,
  allLeafIds,
  setSessionId,
} from "./bspTree";

export interface LayoutState {
  /** The BSP tree describing the pane layout. */
  tree: BspTree;
  /** Currently focused pane ID. */
  activePaneId: string | null;
  /** Command palette open state. */
  commandPaletteOpen: boolean;

  // Actions
  addPane: () => void;
  splitActive: (direction: SplitDirection) => void;
  closePane: (id: string) => void;
  setActivePaneId: (id: string) => void;
  setSessionId: (paneId: string, sessionId: string) => void;
  toggleCommandPalette: () => void;
}

export const useLayoutStore = create<LayoutState>((set, get) => ({
  tree: createLeaf(),
  activePaneId: null,
  commandPaletteOpen: false,

  addPane: () => {
    const { tree, activePaneId } = get();
    const target = activePaneId ?? allLeafIds(tree)[0];
    if (!target) return;
    const result = splitPane(tree, target, "horizontal");
    set({
      tree: result.tree,
      activePaneId: result.newPaneId,
    });
  },

  splitActive: (direction: SplitDirection) => {
    const { tree, activePaneId } = get();
    const target = activePaneId ?? allLeafIds(tree)[0];
    if (!target) return;
    const result = splitPane(tree, target, direction);
    set({
      tree: result.tree,
      activePaneId: result.newPaneId,
    });
  },

  closePane: (id: string) => {
    const { tree, activePaneId } = get();
    const newTree = removePane(tree, id);
    if (newTree === null) {
      // Last pane closed — create a fresh one.
      const leaf = createLeaf();
      set({ tree: leaf, activePaneId: leaf.id });
      return;
    }
    const remaining = allLeafIds(newTree);
    set({
      tree: newTree,
      activePaneId:
        activePaneId === id
          ? remaining[0] ?? null
          : activePaneId,
    });
  },

  setActivePaneId: (id: string) => set({ activePaneId: id }),

  setSessionId: (paneId: string, sessionId: string) => {
    const { tree } = get();
    set({ tree: setSessionId(tree, paneId, sessionId) });
  },

  toggleCommandPalette: () =>
    set((s) => ({ commandPaletteOpen: !s.commandPaletteOpen })),
}));
