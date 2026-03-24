import type { ReactNode } from "react";
import { Button } from "../ui";

export function ActionButton({
  title,
  onClick,
  children,
}: {
  title: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Button
      onClick={onClick}
      title={title}
      variant="ghost"
      size="sm"
      className="h-7 min-w-7 rounded-[var(--radius-sm)] px-[var(--space-2)] text-[var(--text-xs)] text-[var(--text-muted)] hover:bg-[var(--muted)] hover:text-[var(--text-primary)]"
    >
      {children}
    </Button>
  );
}
