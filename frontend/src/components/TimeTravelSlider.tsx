import { useState, useEffect, useCallback } from "react";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { useAgentMissionStore } from "../lib/agentMissionStore";
import { getTerminalController } from "../lib/terminalRegistry";
import { TimeTravelContent } from "./time-travel-slider/TimeTravelContent";
import { TimeTravelHeader } from "./time-travel-slider/TimeTravelHeader";
import type { SnapshotEntry, TimeTravelSliderProps } from "./time-travel-slider/shared";

/**
 * Time-Travel Scrubbing Slider — floating toolbar for browsing
 * and restoring daemon-side workspace snapshots.
 */
export function TimeTravelSlider({ style, className }: TimeTravelSliderProps = {}) {
  const open = useWorkspaceStore((s) => s.timeTravelOpen);
  const toggle = useWorkspaceStore((s) => s.toggleTimeTravel);
  const activePaneId = useWorkspaceStore((s) => s.activePaneId());
  const activeWorkspace = useWorkspaceStore((s) => s.activeWorkspace());
  const missionSnapshots = useAgentMissionStore((s) => s.snapshots);

  const [snapshots, setSnapshots] = useState<SnapshotEntry[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isRestoring, setIsRestoring] = useState(false);
  const [confirmRestore, setConfirmRestore] = useState(false);

  // Fetch snapshots from daemon when panel opens
  useEffect(() => {
    if (!open) return;

    // Use mission store snapshots if available
    if (missionSnapshots.length > 0) {
      setSnapshots(
        missionSnapshots.map((s) => ({
          snapshot_id: s.snapshotId,
          label: s.label ?? `Snapshot ${s.snapshotId.slice(0, 8)}`,
          command: s.command ?? null,
          created_at: s.createdAt,
          status: s.status ?? "ready",
          workspace_id: s.workspaceId ?? null,
        }))
      );
      setSelectedIndex(0);
      return;
    }

    // Otherwise request from daemon
    const controller = getTerminalController(activePaneId);
    if (controller) {
      controller.listSnapshots(activeWorkspace?.id ?? null);
    }
  }, [open, activePaneId, activeWorkspace?.id, missionSnapshots]);

  const handleRestore = useCallback(async () => {
    const target = snapshots[selectedIndex];
    if (!target || isRestoring) return;

    if (!confirmRestore) {
      setConfirmRestore(true);
      return;
    }

    setIsRestoring(true);
    setConfirmRestore(false);

    const controller = getTerminalController(activePaneId);
    if (controller) {
      await controller.restoreSnapshot(target.snapshot_id);
    }

    setIsRestoring(false);
  }, [snapshots, selectedIndex, isRestoring, confirmRestore, activePaneId]);

  useEffect(() => {
    if (!open) {
      setConfirmRestore(false);
    }
  }, [open]);

  if (!open) return null;

  return (
    <div
      style={{
        position: "absolute",
        bottom: 12,
        left: "50%",
        zIndex: 200,
        minWidth: 520,
        maxWidth: 720,
        background: "var(--bg-secondary)",
        border: "1px solid var(--glass-border)",
        borderRadius: 0,
        padding: "14px 18px",
        boxShadow: "none",
        backdropFilter: "none",
        ...(style ?? {}),
      }}
      className={className ? `amux-shell-card ${className}` : "amux-shell-card"}
    >
      <TimeTravelHeader snapshotCount={snapshots.length} toggle={toggle} />
      <TimeTravelContent
        snapshots={snapshots}
        selectedIndex={selectedIndex}
        setSelectedIndex={setSelectedIndex}
        confirmRestore={confirmRestore}
        setConfirmRestore={setConfirmRestore}
        isRestoring={isRestoring}
        handleRestore={() => {
          void handleRestore();
        }}
      />
    </div>
  );
}
