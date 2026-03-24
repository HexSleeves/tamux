import { useMemo } from "react";
import { useAgentMissionStore } from "../../lib/agentMissionStore";
import { useAgentStore } from "../../lib/agentStore";
import { useWorkspaceStore } from "../../lib/workspaceStore";
import { executeCommand } from "../../registry/commandRegistry";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Separator } from "../ui/Separator";
import { cn } from "../ui/shared";
import type { MissionDeckProps } from "./shared";

export const MissionDeck: React.FC<MissionDeckProps> = ({
  style,
  className,
  children,
  missionTagLabel = "Mission",
  missionButtonLabel = "Mission",
  vaultButtonLabel = "Vault",
  providerLabelPrefix = "provider",
  approvalsLabel = "approvals",
  traceLabel = "trace",
  opsLabel = "ops",
  recallLabel = "recall",
  snapshotsLabel = "snapshots",
  missionCommand = "view.toggleMission",
  vaultCommand = "view.toggleSessionVault",
}) => {
  const asText = (value: unknown, fallback: string): string => {
    if (typeof value === "string") {
      const trimmed = value.trim();
      return trimmed.length > 0 ? trimmed : fallback;
    }
    if (typeof value === "number") {
      return String(value);
    }
    return fallback;
  };

  const activeWorkspace = useWorkspaceStore((s) => s.activeWorkspace());
  const activeSurface = useWorkspaceStore((s) => s.activeSurface());
  const active_provider = useAgentStore((s) => s.agentSettings.active_provider);
  const cognitiveEvents = useAgentMissionStore((s) => s.cognitiveEvents);
  const operationalEvents = useAgentMissionStore((s) => s.operationalEvents);
  const approvals = useAgentMissionStore((s) => s.approvals);
  const snapshots = useAgentMissionStore((s) => s.snapshots);
  const historyHits = useAgentMissionStore((s) => s.historyHits);
  const symbolHits = useAgentMissionStore((s) => s.symbolHits);

  const approvalCount = useMemo(
    () => approvals.filter((entry) => entry.status === "pending").length,
    [approvals]
  );
  const workspaceName = asText(activeWorkspace?.name, "No workspace");
  const surfaceName = asText(activeSurface?.name, "No surface");
  const missionTag = asText(missionTagLabel, "Mission");
  const missionButton = asText(missionButtonLabel, "Mission");
  const vaultButton = asText(vaultButtonLabel, "Vault");
  const providerPrefix = asText(providerLabelPrefix, "provider");
  const providerText = asText(active_provider, "unknown");
  const approvalsText = asText(approvalsLabel, "approvals");
  const traceText = asText(traceLabel, "trace");
  const opsText = asText(opsLabel, "ops");
  const recallText = asText(recallLabel, "recall");
  const snapshotsText = asText(snapshotsLabel, "snapshots");

  return (
    <Card
      className={cn(
        "flex min-h-[52px] flex-wrap items-center justify-between gap-[var(--space-3)] overflow-x-auto px-[var(--space-3)] py-[var(--space-2)]",
        className
      )}
      style={{ flexShrink: 0, ...style }}
    >
      <div className="flex min-w-0 flex-1 items-center gap-[var(--space-2)]">
        <Badge variant="mission" className="uppercase tracking-[0.08em]">
          {missionTag}
        </Badge>
        <div className="min-w-0">
          <div
            className="truncate text-[var(--text-sm)] font-semibold text-[var(--text-primary)]"
            title={`${workspaceName} - ${surfaceName}`}
          >
            {workspaceName}
          </div>
          <div className="text-[var(--text-xs)] text-[var(--text-secondary)]">{surfaceName}</div>
        </div>
        <Badge variant="default">
          {providerPrefix} {providerText}
        </Badge>
      </div>

      <Separator orientation="vertical" className="hidden h-6 md:block" />

      <div className="flex flex-wrap items-center gap-[var(--space-2)] whitespace-nowrap">
        <Badge variant="approval">
          {approvalsText} {approvalCount}
        </Badge>
        <Badge variant="reasoning">
          {traceText} {cognitiveEvents.length}
        </Badge>
        <Badge variant="agent">
          {opsText} {operationalEvents.length}
        </Badge>
        <Badge variant="timeline">
          {recallText} {historyHits.length + symbolHits.length}
        </Badge>
        <Badge variant="default">
          {snapshotsText} {snapshots.length}
        </Badge>
      </div>

      <Separator orientation="vertical" className="hidden h-6 md:block" />

      <div className="flex items-center gap-[var(--space-2)] whitespace-nowrap">
        <Button size="sm" onClick={() => {
          void executeCommand(missionCommand);
        }}>
          {missionButton}
        </Button>
        <Button size="sm" variant="outline" onClick={() => {
          void executeCommand(vaultCommand);
        }}>
          {vaultButton}
        </Button>
      </div>
      {children}
    </Card>
  );
};
