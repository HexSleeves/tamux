import { Badge } from "./ui/Badge";
import { cn } from "./ui/shared";

interface SpinnerProps {
  variant?: "spinner";
  size?: number;
  label?: string;
}

interface SkeletonProps {
  variant: "skeleton";
  width?: string | number;
  height?: string | number;
  lines?: number;
}

interface ProgressProps {
  variant: "progress";
  value: number;
  label?: string;
}

type LoadingStateProps = SpinnerProps | SkeletonProps | ProgressProps;

export function LoadingState(props: LoadingStateProps) {
  const variant = props.variant ?? "spinner";

  if (variant === "skeleton") {
    const { width = "100%", height = 14, lines = 3 } = props as SkeletonProps;
    return (
      <div className="grid gap-[var(--space-2)]">
        {Array.from({ length: lines }, (_, i) => (
          <div
            key={i}
            className={cn(
              "rounded-[var(--radius-md)] bg-[var(--bg-secondary)] bg-[length:200%_100%] animate-[shimmer_1.5s_ease-in-out_infinite]",
              i === lines - 1 && "max-w-[60%]"
            )}
            style={{ width: i === lines - 1 ? "60%" : width, height }}
          />
        ))}
      </div>
    );
  }

  if (variant === "progress") {
    const { value, label } = props as ProgressProps;
    const clamped = Math.max(0, Math.min(100, value));
    return (
      <div className="grid gap-[var(--space-2)]">
        {label ? (
          <div className="flex items-center justify-between gap-[var(--space-2)] text-[var(--text-xs)] text-[var(--text-secondary)]">
            <span>{label}</span>
            <Badge variant="accent">{Math.round(clamped)}%</Badge>
          </div>
        ) : null}
        <div className="h-[6px] overflow-hidden rounded-[var(--radius-full)] bg-[var(--bg-tertiary)]">
          <div
            className="h-full rounded-[var(--radius-full)] bg-[var(--accent)] transition-[width] duration-300 ease-out"
            style={{ width: `${clamped}%` }}
          />
        </div>
      </div>
    );
  }

  const { size = 20, label } = props as SpinnerProps;
  return (
    <div className="flex items-center gap-[var(--space-2)] text-[var(--text-secondary)]">
      <div
        className="shrink-0 rounded-full border-2 border-[var(--bg-tertiary)] border-t-[var(--accent)] animate-[spin_0.7s_linear_infinite]"
        style={{ width: size, height: size }}
      />
      {label ? <span className="text-[var(--text-xs)]">{label}</span> : null}
    </div>
  );
}
