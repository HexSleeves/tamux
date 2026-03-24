import { useEffect, useState } from "react";
import { Badge } from "../ui";

export function InlineSystemMonitor() {
  const [stats, setStats] = useState<{
    cpu: number;
    memUsed: number;
    memTotal: number;
    vram: number | null;
  } | null>(null);

  useEffect(() => {
    let active = true;
    const amux = (window as any).tamux ?? (window as any).amux;
    if (!amux?.getSystemMonitorSnapshot) return;

    const fetchStats = async () => {
      try {
        const snap = await amux.getSystemMonitorSnapshot({ processLimit: 0 });
        if (!active) return;
        setStats({
          cpu: snap.cpu?.usagePercent ?? 0,
          memUsed: snap.memory?.usedBytes ?? 0,
          memTotal: snap.memory?.totalBytes ?? 1,
          vram: snap.gpus?.[0]
            ? (snap.gpus[0].memoryUsedMB / snap.gpus[0].memoryTotalMB) * 100
            : null,
        });
      } catch {
        // silent
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 3000);
    return () => {
      active = false;
      clearInterval(interval);
    };
  }, []);

  if (!stats) return null;

  const memPercent = (stats.memUsed / stats.memTotal) * 100;
  const memGB = (stats.memUsed / (1024 * 1024 * 1024)).toFixed(1);
  const memTotalGB = (stats.memTotal / (1024 * 1024 * 1024)).toFixed(0);

  return (
    <div className="flex items-center gap-[var(--space-2)]" title="System health">
      <MiniMeter label="CPU" value={stats.cpu} />
      <MiniMeter label="RAM" value={memPercent} suffix={`${memGB}/${memTotalGB}G`} />
      {stats.vram !== null ? <MiniMeter label="VRAM" value={stats.vram} /> : null}
    </div>
  );
}

function MiniMeter({ label, value, suffix }: { label: string; value: number; suffix?: string }) {
  const variant = value > 90 ? "danger" : value > 70 ? "warning" : "success";
  return (
    <Badge variant={variant} className="gap-[var(--space-1)] px-[var(--space-2)] py-[2px]">
      <span className="text-[10px] uppercase tracking-[0.08em] text-[var(--text-muted)]">{label}</span>
      <span className="text-[11px] font-semibold tabular-nums">{suffix || `${value.toFixed(0)}%`}</span>
    </Badge>
  );
}
