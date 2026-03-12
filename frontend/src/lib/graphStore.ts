import { create } from "zustand";
import type { Node, Edge } from "@xyflow/react";
import type { CommandLogEntry } from "./types";

interface GraphState {
  nodes: Node[];
  edges: Edge[];
  selectedNodeId: string | null;
  selectNode: (id: string | null) => void;
  buildFromEntries: (entries: CommandLogEntry[]) => void;
  clear: () => void;
}

type GraphStep = {
  label: string;
  connectorBefore: string | null;
};

function splitCommand(command: string): GraphStep[] {
  const parts = command
    .split(/(\|\||&&|\||;)/g)
    .map((part) => part.trim())
    .filter(Boolean);
  const steps: GraphStep[] = [];
  let connector: string | null = null;

  for (const part of parts) {
    if (["|", "&&", "||", ";"].includes(part)) {
      connector = part;
      continue;
    }
    steps.push({ label: part, connectorBefore: connector });
    connector = null;
  }

  return steps.length > 0
    ? steps
    : [{ label: command, connectorBefore: null }];
}

const CONNECTOR_LABELS: Record<string, string> = {
  "|": "pipe",
  "&&": "and",
  "||": "or",
  ";": "seq",
};

const NODE_WIDTH = 200;
const X_GAP = 60;
const Y_GAP = 120;

export const useGraphStore = create<GraphState>((set) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,

  selectNode: (id) => set({ selectedNodeId: id }),

  buildFromEntries: (entries) => {
    const nodes: Node[] = [];
    const edges: Edge[] = [];

    // Process entries in chronological order (oldest first)
    const sorted = [...entries].sort((a, b) => a.timestamp - b.timestamp);

    sorted.forEach((entry, rowIndex) => {
      const steps = splitCommand(entry.command);
      const yOffset = rowIndex * Y_GAP;

      steps.forEach((step, colIndex) => {
        const nodeId = `${entry.id}-${colIndex}`;
        nodes.push({
          id: nodeId,
          type: "toolNode",
          position: { x: colIndex * (NODE_WIDTH + X_GAP), y: yOffset },
          data: {
            label: step.label,
            exitCode: colIndex === steps.length - 1 ? entry.exitCode : null,
            durationMs: colIndex === steps.length - 1 ? entry.durationMs : null,
            timestamp: entry.timestamp,
            isRunning: entry.exitCode === null,
            entryId: entry.id,
          },
        });

        // Intra-command edges (pipes, &&, etc.)
        if (colIndex > 0) {
          const prevNodeId = `${entry.id}-${colIndex - 1}`;
          const connector = step.connectorBefore ?? "|";
          edges.push({
            id: `${prevNodeId}-${nodeId}`,
            source: prevNodeId,
            target: nodeId,
            type: "dataFlowEdge",
            data: {
              connectorType: connector,
              label: CONNECTOR_LABELS[connector] ?? connector,
            },
            animated: entry.exitCode === null,
          });
        }
      });

      // Inter-command edges (sequential execution flow)
      if (rowIndex > 0) {
        const prevEntry = sorted[rowIndex - 1];
        const prevSteps = splitCommand(prevEntry.command);
        const prevLastNodeId = `${prevEntry.id}-${prevSteps.length - 1}`;
        const currentFirstNodeId = `${entry.id}-0`;

        edges.push({
          id: `seq-${prevLastNodeId}-${currentFirstNodeId}`,
          source: prevLastNodeId,
          target: currentFirstNodeId,
          type: "dataFlowEdge",
          data: {
            connectorType: "seq",
            label: "then",
          },
          style: { strokeDasharray: "6 3" },
          animated: false,
        });
      }
    });

    set({ nodes, edges });
  },

  clear: () => set({ nodes: [], edges: [], selectedNodeId: null }),
}));
