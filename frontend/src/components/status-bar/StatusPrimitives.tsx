import { Badge, Button, cn } from "../ui";

export function StatusIndicator({ label, status }: { label: string; status: "success" | "warning" | "neutral" }) {
  const colors = {
    success: "bg-[var(--success)]",
    warning: "bg-[var(--warning)]",
    neutral: "bg-[var(--text-muted)]",
  }[status];

  return (
    <div className="inline-flex items-center gap-[var(--space-2)] text-[var(--text-xs)] text-[var(--text-secondary)]">
      <span className={cn("h-1.5 w-1.5 rounded-full", colors)} />
      <span>{label}</span>
    </div>
  );
}

export function StatusBadge({
  label,
  tone,
  onClick,
}: {
  label: string;
  tone: "success" | "warning" | "agent" | "neutral";
  onClick?: () => void;
}) {
  const variant = {
    success: "success",
    warning: "warning",
    agent: "agent",
    neutral: "default",
  } as const;

  if (onClick) {
    return (
      <Button
        variant={tone === "agent" ? "agent" : tone === "warning" ? "outline" : "secondary"}
        size="sm"
        className="h-6 rounded-full px-[var(--space-2)] text-[var(--text-xs)]"
        onClick={onClick}
      >
        {label}
      </Button>
    );
  }

  return (
    <Badge variant={variant[tone]} className="px-[var(--space-2)] py-[2px] text-[var(--text-xs)]">
      {label}
    </Badge>
  );
}
