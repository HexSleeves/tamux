import { cn, popoverSurfaceClassName } from "../ui";

export interface TerminalContextMenuItem {
  label?: string;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  separator?: boolean;
  action?: () => void;
}

export function TerminalContextMenu({
  visible,
  x,
  y,
  items,
  hideContextMenu,
}: {
  visible: boolean;
  x: number;
  y: number;
  items: TerminalContextMenuItem[];
  hideContextMenu: () => void;
}) {
  if (!visible) {
    return null;
  }

  return (
    <div
      onPointerDown={(event) => event.stopPropagation()}
      className={cn(popoverSurfaceClassName, "absolute z-[4000] w-[220px] p-[var(--space-1)]")}
      style={{ top: y, left: x }}
    >
      {items.map((item, index) => {
        if (item.separator) {
          return <div key={`separator-${index}`} className="my-[var(--space-1)] h-px bg-[var(--border-subtle)]" />;
        }

        return (
          <button
            key={item.label}
            type="button"
            disabled={item.disabled}
            onClick={() => {
              hideContextMenu();
              item.action?.();
            }}
            className={cn(
              "flex w-full items-center justify-between rounded-[var(--radius-sm)] px-[var(--space-3)] py-[var(--space-2)] text-left text-[9px] transition-colors",
              item.disabled
                ? "cursor-not-allowed text-[var(--text-muted)] opacity-50"
                : item.danger
                  ? "text-[var(--danger)] hover:bg-[var(--danger-soft)]"
                  : "text-[var(--text-primary)] hover:bg-[var(--muted)]"
            )}
          >
            <span>{item.label}</span>
            <span className="text-[9px] text-[var(--text-muted)]">{item.shortcut ?? ""}</span>
          </button>
        );
      })}
    </div>
  );
}
