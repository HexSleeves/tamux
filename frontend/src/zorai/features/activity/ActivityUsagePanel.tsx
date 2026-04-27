import {
  formatCost,
  formatCount,
  formatDate,
  formatDuration,
  type GoalUsageRow,
  type SessionUsageRow,
  type UsageRow,
  type UsageStats,
} from "./ActivityUsageStats";

export function UsagePanel({ stats }: { stats: UsageStats }) {
  const topProviders = stats.providerRows.slice(0, 6);
  const tokenMax = Math.max(...topProviders.map((row) => row.totalTokens), 1);
  const costMax = Math.max(...topProviders.map((row) => row.cost), 0.000001);

  return (
    <div className="zorai-usage-stack">
      <div className="zorai-metric-grid">
        <UsageMetric label="Requests" value={formatCount(stats.totals.requests)} />
        <UsageMetric label="Prompt tokens" value={formatCount(stats.totals.promptTokens)} />
        <UsageMetric label="Completion tokens" value={formatCount(stats.totals.completionTokens)} />
        <UsageMetric label="Estimated cost" value={formatCost(stats.totals.cost)} />
        <UsageMetric label="Reasoning tokens" value={formatCount(stats.totals.reasoningTokens)} />
        <UsageMetric label="Avg TPS" value={stats.totals.avgTps.toFixed(1)} />
      </div>

      <div className="zorai-usage-grid">
        <div className="zorai-panel">
          <div className="zorai-section-label">Token Usage</div>
          <UsageBars rows={topProviders} valueKey="totalTokens" max={tokenMax} formatter={formatCount} />
        </div>
        <div className="zorai-panel">
          <div className="zorai-section-label">Cost Usage</div>
          <UsageBars rows={topProviders} valueKey="cost" max={costMax} formatter={formatCost} />
        </div>
        <ProviderUsageTable rows={stats.providerRows} />
        <SessionUsageTable rows={stats.sessionRows} />
        <GoalUsageTable rows={stats.goalRows} />
      </div>
    </div>
  );
}

function ProviderUsageTable({ rows }: { rows: UsageRow[] }) {
  return (
    <div className="zorai-panel zorai-usage-panel--wide">
      <div className="zorai-section-label">Provider / Model</div>
      <div className="zorai-usage-table-wrap">
        <table className="zorai-usage-table">
          <thead>
            <tr><th>Provider / Model</th><th>Req</th><th>Prompt</th><th>Completion</th><th>Total</th><th>Reasoning</th><th>Cost</th><th>TPS</th></tr>
          </thead>
          <tbody>
            {rows.length === 0 ? <UsageEmptyRow colSpan={8} /> : rows.map((row) => (
              <tr key={row.key}>
                <td>{row.provider} / {row.model}</td>
                <td>{row.requests}</td>
                <td>{formatCount(row.promptTokens)}</td>
                <td>{formatCount(row.completionTokens)}</td>
                <td>{formatCount(row.totalTokens)}</td>
                <td>{formatCount(row.reasoningTokens)}</td>
                <td>{formatCost(row.cost)}</td>
                <td>{row.avgTps.toFixed(1)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function SessionUsageTable({ rows }: { rows: SessionUsageRow[] }) {
  return (
    <div className="zorai-panel zorai-usage-panel--wide">
      <div className="zorai-section-label">Sessions</div>
      <div className="zorai-usage-table-wrap">
        <table className="zorai-usage-table">
          <thead>
            <tr><th>Thread</th><th>Provider models</th><th>Req</th><th>Total</th><th>Audio</th><th>Video</th><th>Cost</th><th>Updated</th></tr>
          </thead>
          <tbody>
            {rows.length === 0 ? <UsageEmptyRow colSpan={8} /> : rows.map((row) => (
              <tr key={row.threadId}>
                <td>{row.title}</td>
                <td>{Array.from(row.providerModels).join(", ") || "unknown"}</td>
                <td>{row.requests}</td>
                <td>{formatCount(row.totalTokens)}</td>
                <td>{formatCount(row.audioTokens)}</td>
                <td>{formatCount(row.videoTokens)}</td>
                <td>{formatCost(row.cost)}</td>
                <td>{formatDate(row.updatedAt)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function GoalUsageTable({ rows }: { rows: GoalUsageRow[] }) {
  return (
    <div className="zorai-panel zorai-usage-panel--wide">
      <div className="zorai-section-label">Goal Usage</div>
      <div className="zorai-usage-table-wrap">
        <table className="zorai-usage-table">
          <thead>
            <tr><th>Goal</th><th>Status</th><th>Provider / Model</th><th>Req</th><th>Prompt</th><th>Completion</th><th>Cost</th><th>Duration</th></tr>
          </thead>
          <tbody>
            {rows.length === 0 ? <UsageEmptyRow colSpan={8} /> : rows.map((row) => (
              <tr key={row.key}>
                <td>{row.goal}</td>
                <td>{row.status}</td>
                <td>{row.provider} / {row.model}</td>
                <td>{row.requests}</td>
                <td>{formatCount(row.promptTokens)}</td>
                <td>{formatCount(row.completionTokens)}</td>
                <td>{formatCost(row.cost)}</td>
                <td>{formatDuration(row.durationMs)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function UsageMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="zorai-metric-card">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function UsageBars({
  rows,
  valueKey,
  max,
  formatter,
}: {
  rows: UsageRow[];
  valueKey: "totalTokens" | "cost";
  max: number;
  formatter: (value: number) => string;
}) {
  if (rows.length === 0) {
    return <div className="zorai-empty-state">No usage has been recorded yet.</div>;
  }

  return (
    <div className="zorai-usage-bars">
      {rows.map((row) => {
        const value = row[valueKey];
        const width = `${Math.max(4, Math.round((value / max) * 100))}%`;
        return (
          <div className="zorai-usage-bar" key={`${valueKey}-${row.key}`}>
            <span>{row.provider}</span>
            <div><i style={{ width }} /></div>
            <strong>{formatter(value)}</strong>
          </div>
        );
      })}
    </div>
  );
}

function UsageEmptyRow({ colSpan }: { colSpan: number }) {
  return (
    <tr>
      <td colSpan={colSpan}>No usage has been recorded yet.</td>
    </tr>
  );
}
