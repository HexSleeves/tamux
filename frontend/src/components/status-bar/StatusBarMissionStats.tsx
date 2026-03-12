import { StatusBadge } from "./StatusPrimitives";

export function StatusBarMissionStats({
    pendingApprovals,
    traceCount,
    opsCount,
    toolCallCount,
    historyCount,
    snapshotCount,
}: {
    pendingApprovals: number;
    traceCount: number;
    opsCount: number;
    toolCallCount: number;
    historyCount: number;
    snapshotCount: number;
}) {
    return (
        <>
            {pendingApprovals > 0 ? <StatusBadge label={`approvals ${pendingApprovals}`} tone="warning" /> : null}
            <StatusBadge label={`trace ${traceCount}`} tone="agent" />
            <StatusBadge label={`ops ${opsCount}`} tone="neutral" />
            <StatusBadge label={`tools ${toolCallCount}`} tone="agent" />
            <StatusBadge label={`recall ${historyCount}`} tone="neutral" />
            <StatusBadge label={`snapshots ${snapshotCount}`} tone="neutral" />
        </>
    );
}
