/**
 * BSP (Binary Space Partitioning) Tree layout model.
 *
 * This is the data structure that powers dynamic pane splitting, exactly
 * like i3wm, tmux, and Zellij.
 *
 * - A **Leaf** is a terminal pane with a unique ID.
 * - A **Node** is a split: it has a direction (horizontal/vertical),
 *   a split ratio, and two children (which can each be Leaves or Nodes).
 *
 * When the user splits a pane, the targeted Leaf transforms into a Node
 * and two new Leaves are created as its children.
 */

export type SplitDirection = "horizontal" | "vertical";

export interface BspLeaf {
  type: "leaf";
  id: string;
  /** Session ID from the daemon (assigned after spawn). */
  sessionId?: string;
}

export interface BspNode {
  type: "node";
  direction: SplitDirection;
  /** 0..1 — fraction of space allocated to the first child. */
  ratio: number;
  first: BspTree;
  second: BspTree;
}

export type BspTree = BspLeaf | BspNode;

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function normalizeDirection(value: unknown): SplitDirection | null {
  if (value === "horizontal" || value === "vertical") {
    return value;
  }

  if (value === "row") {
    return "horizontal";
  }

  if (value === "column") {
    return "vertical";
  }

  return null;
}

function createFallbackLeaf(fallbackPaneIds: string[]): BspLeaf {
  const fallbackId = fallbackPaneIds.shift();
  return createLeaf(typeof fallbackId === "string" && fallbackId ? fallbackId : undefined);
}

export function normalizeBspTree(
  tree: unknown,
  fallbackPaneIds: string[] = []
): BspTree {
  const remainingPaneIds = [...fallbackPaneIds];

  const visit = (node: unknown): BspTree => {
    if (!isObjectRecord(node)) {
      return createFallbackLeaf(remainingPaneIds);
    }

    if (node.type === "leaf" || typeof node.id === "string") {
      return {
        type: "leaf",
        id: typeof node.id === "string" && node.id ? node.id : createFallbackLeaf(remainingPaneIds).id,
        ...(typeof node.sessionId === "string" && node.sessionId ? { sessionId: node.sessionId } : {}),
      };
    }

    const direction = normalizeDirection(node.direction);
    const first = node.first ?? node.left;
    const second = node.second ?? node.right;

    if (direction && first !== undefined && second !== undefined) {
      const ratio = typeof node.ratio === "number"
        ? Math.max(0.1, Math.min(0.9, node.ratio))
        : 0.5;

      return {
        type: "node",
        direction,
        ratio,
        first: visit(first),
        second: visit(second),
      };
    }

    return createFallbackLeaf(remainingPaneIds);
  };

  return visit(tree);
}

// ---------------------------------------------------------------------------
// Tree operations
// ---------------------------------------------------------------------------

let _nextId = 1;
export function generatePaneId(): string {
  return `pane_${_nextId++}`;
}

export function syncPaneIdCounter(tree: BspTree): void {
  let maxId = 0;

  const visit = (node: BspTree | null | undefined) => {
    if (!node) {
      return;
    }

    if (node.type === "leaf") {
      const match = /^pane_(\d+)$/.exec(node.id);
      if (match) {
        maxId = Math.max(maxId, Number(match[1]));
      }
      return;
    }

    visit(node.first);
    visit(node.second);
  };

  visit(tree);
  _nextId = Math.max(_nextId, maxId + 1);
}

/** Create a fresh leaf node. */
export function createLeaf(id?: string): BspLeaf {
  return { type: "leaf", id: id ?? generatePaneId() };
}

/**
 * Split a target leaf in the tree, replacing it with a node containing
 * the original leaf and a new sibling.
 */
export function splitPane(
  tree: BspTree,
  targetId: string,
  direction: SplitDirection
): { tree: BspTree; newPaneId: string } {
  const newLeaf = createLeaf();

  return {
    tree: splitInTree(tree, targetId, direction, newLeaf),
    newPaneId: newLeaf.id,
  };
}

function splitInTree(
  node: BspTree,
  targetId: string,
  direction: SplitDirection,
  newLeaf: BspLeaf
): BspTree {
  if (node.type === "leaf") {
    if (node.id === targetId) {
      return {
        type: "node",
        direction,
        ratio: 0.5,
        first: node,
        second: newLeaf,
      };
    }
    return node;
  }

  return {
    ...node,
    first: splitInTree(node.first, targetId, direction, newLeaf),
    second: splitInTree(node.second, targetId, direction, newLeaf),
  };
}

/**
 * Remove a pane from the tree. Its sibling takes over the parent's space.
 * If the tree contains only one leaf, returns `null`.
 */
export function removePane(
  tree: BspTree,
  targetId: string
): BspTree | null {
  if (tree.type === "leaf") {
    return tree.id === targetId ? null : tree;
  }

  // Check if either direct child is the target leaf.
  if (tree.first.type === "leaf" && tree.first.id === targetId) {
    return tree.second;
  }
  if (tree.second.type === "leaf" && tree.second.id === targetId) {
    return tree.first;
  }

  // Recurse into children.
  const newFirst = removePane(tree.first, targetId);
  if (newFirst === null) return tree.second;

  const newSecond = removePane(tree.second, targetId);
  if (newSecond === null) return tree.first;

  return { ...tree, first: newFirst, second: newSecond };
}

/**
 * Update the split ratio of a node that directly contains `paneId`
 * as one of its children.
 */
export function updateRatio(
  tree: BspTree,
  paneId: string,
  newRatio: number
): BspTree {
  if (tree.type === "leaf") return tree;

  const isFirst =
    tree.first.type === "leaf" && tree.first.id === paneId;
  const isSecond =
    tree.second.type === "leaf" && tree.second.id === paneId;

  if (isFirst || isSecond) {
    return { ...tree, ratio: Math.max(0.1, Math.min(0.9, newRatio)) };
  }

  return {
    ...tree,
    first: updateRatio(tree.first, paneId, newRatio),
    second: updateRatio(tree.second, paneId, newRatio),
  };
}

/** Collect all leaf IDs in the tree. */
export function allLeafIds(tree: BspTree): string[] {
  if (tree.type === "leaf") return [tree.id];
  return [...allLeafIds(tree.first), ...allLeafIds(tree.second)];
}

/** Find a leaf by ID. */
export function findLeaf(
  tree: BspTree,
  id: string
): BspLeaf | undefined {
  if (tree.type === "leaf") {
    return tree.id === id ? tree : undefined;
  }
  return findLeaf(tree.first, id) ?? findLeaf(tree.second, id);
}

/** Set the sessionId on a leaf (immutable update). */
export function setSessionId(
  tree: BspTree,
  paneId: string,
  sessionId: string
): BspTree {
  if (tree.type === "leaf") {
    if (tree.id === paneId) {
      return { ...tree, sessionId };
    }
    return tree;
  }
  return {
    ...tree,
    first: setSessionId(tree.first, paneId, sessionId),
    second: setSessionId(tree.second, paneId, sessionId),
  };
}

/** Reset every split ratio to an even 50/50 balance. */
export function equalizeLayoutRatios(tree: BspTree): BspTree {
  if (tree.type === "leaf") {
    return tree;
  }

  return {
    ...tree,
    ratio: 0.5,
    first: equalizeLayoutRatios(tree.first),
    second: equalizeLayoutRatios(tree.second),
  };
}

// ---------------------------------------------------------------------------
// Preset layouts
// ---------------------------------------------------------------------------

export type PresetLayout =
  | "single"
  | "2-columns"
  | "3-columns"
  | "grid-2x2"
  | "main-stack";

/** Build a preset layout from scratch with fresh panes. */
export function buildPresetLayout(preset: PresetLayout): BspTree {
  switch (preset) {
    case "single":
      return createLeaf();

    case "2-columns": {
      return {
        type: "node",
        direction: "horizontal",
        ratio: 0.5,
        first: createLeaf(),
        second: createLeaf(),
      };
    }

    case "3-columns": {
      return {
        type: "node",
        direction: "horizontal",
        ratio: 0.33,
        first: createLeaf(),
        second: {
          type: "node",
          direction: "horizontal",
          ratio: 0.5,
          first: createLeaf(),
          second: createLeaf(),
        },
      };
    }

    case "grid-2x2": {
      return {
        type: "node",
        direction: "vertical",
        ratio: 0.5,
        first: {
          type: "node",
          direction: "horizontal",
          ratio: 0.5,
          first: createLeaf(),
          second: createLeaf(),
        },
        second: {
          type: "node",
          direction: "horizontal",
          ratio: 0.5,
          first: createLeaf(),
          second: createLeaf(),
        },
      };
    }

    case "main-stack": {
      return {
        type: "node",
        direction: "horizontal",
        ratio: 0.6,
        first: createLeaf(),
        second: {
          type: "node",
          direction: "vertical",
          ratio: 0.5,
          first: createLeaf(),
          second: createLeaf(),
        },
      };
    }
  }
}

// ---------------------------------------------------------------------------
// Split boundary computation (for draggable splitter handles)
// ---------------------------------------------------------------------------

export interface SplitBoundary {
  /** Direction of the split (horizontal = vertical bar, vertical = horizontal bar). */
  nodeDirection: SplitDirection;
  /** Position of the split line in normalized 0..1 coords. */
  position: number;
  /** Perpendicular span start (0..1). */
  spanStart: number;
  /** Perpendicular span end (0..1). */
  spanEnd: number;
  /** A leaf ID in the first child subtree — pass to updateRatio(). */
  firstChildLeafId: string;
  /** Parent rect for computing new ratio from mouse position. */
  parentRect: { x: number; y: number; w: number; h: number };
}

/** Walk the BSP tree and compute split line boundaries for each BspNode. */
export function computeSplitBoundaries(
  tree: BspTree,
  rect: { x: number; y: number; w: number; h: number } = { x: 0, y: 0, w: 1, h: 1 },
): SplitBoundary[] {
  const result: SplitBoundary[] = [];

  function walk(node: BspTree, r: { x: number; y: number; w: number; h: number }) {
    if (node.type === "leaf") return;

    const firstChildLeafId = allLeafIds(node.first)[0];

    if (node.direction === "horizontal") {
      const splitX = r.x + r.w * node.ratio;
      result.push({
        nodeDirection: "horizontal",
        position: splitX,
        spanStart: r.y,
        spanEnd: r.y + r.h,
        firstChildLeafId,
        parentRect: { ...r },
      });
      const w1 = r.w * node.ratio;
      walk(node.first, { x: r.x, y: r.y, w: w1, h: r.h });
      walk(node.second, { x: r.x + w1, y: r.y, w: r.w - w1, h: r.h });
    } else {
      const splitY = r.y + r.h * node.ratio;
      result.push({
        nodeDirection: "vertical",
        position: splitY,
        spanStart: r.x,
        spanEnd: r.x + r.w,
        firstChildLeafId,
        parentRect: { ...r },
      });
      const h1 = r.h * node.ratio;
      walk(node.first, { x: r.x, y: r.y, w: r.w, h: h1 });
      walk(node.second, { x: r.x, y: r.y + h1, w: r.w, h: r.h - h1 });
    }
  }

  walk(tree, rect);
  return result;
}

// ---------------------------------------------------------------------------
// Focus navigation (directional)
// ---------------------------------------------------------------------------

interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Compute the bounding rectangle for each leaf in the tree. */
export function computeLeafRects(
  tree: BspTree,
  rect: Rect = { x: 0, y: 0, w: 1, h: 1 }
): Map<string, Rect> {
  const result = new Map<string, Rect>();

  function walk(node: BspTree, r: Rect) {
    if (node.type === "leaf") {
      result.set(node.id, r);
      return;
    }
    if (node.direction === "horizontal") {
      const w1 = r.w * node.ratio;
      walk(node.first, { x: r.x, y: r.y, w: w1, h: r.h });
      walk(node.second, { x: r.x + w1, y: r.y, w: r.w - w1, h: r.h });
    } else {
      const h1 = r.h * node.ratio;
      walk(node.first, { x: r.x, y: r.y, w: r.w, h: h1 });
      walk(node.second, { x: r.x, y: r.y + h1, w: r.w, h: r.h - h1 });
    }
  }

  walk(tree, rect);
  return result;
}

export type Direction = "left" | "right" | "up" | "down";

/** Find the next pane in a given direction from the current pane. */
export function findAdjacentPane(
  tree: BspTree,
  currentId: string,
  direction: Direction
): string | null {
  const rects = computeLeafRects(tree);
  const current = rects.get(currentId);
  if (!current) return null;

  const cx = current.x + current.w / 2;
  const cy = current.y + current.h / 2;

  let bestId: string | null = null;
  let bestDist = Infinity;

  for (const [id, rect] of rects) {
    if (id === currentId) continue;

    const rx = rect.x + rect.w / 2;
    const ry = rect.y + rect.h / 2;

    let valid = false;
    switch (direction) {
      case "left":
        valid = rx < cx;
        break;
      case "right":
        valid = rx > cx;
        break;
      case "up":
        valid = ry < cy;
        break;
      case "down":
        valid = ry > cy;
        break;
    }

    if (valid) {
      const dist = Math.abs(rx - cx) + Math.abs(ry - cy);
      if (dist < bestDist) {
        bestDist = dist;
        bestId = id;
      }
    }
  }

  return bestId;
}
