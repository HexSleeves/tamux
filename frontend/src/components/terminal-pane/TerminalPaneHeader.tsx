import { Badge, Input } from "../ui";

export function TerminalPaneHeader({
  paneId,
  paneName,
  paneNameDraft,
  setPaneNameDraft,
  setPaneName,
}: {
  paneId: string;
  paneName: string;
  paneNameDraft: string;
  setPaneNameDraft: (value: string) => void;
  setPaneName: (paneId: string, name: string) => void;
}) {
  return (
    <div className="mb-[var(--space-2)] flex h-8 max-w-[22rem] items-center justify-between gap-[var(--space-2)] px-[2px] text-[11px] text-[var(--text-secondary)]">
      <div className="flex min-w-0 flex-1 items-center gap-[var(--space-2)]">
        <Badge variant="default" className="shrink-0 px-[var(--space-2)] py-[2px] text-[10px] uppercase tracking-[0.08em]">
          Pane
        </Badge>
        <Input
          value={paneNameDraft}
          onChange={(event) => setPaneNameDraft(event.target.value)}
          onMouseDown={(event) => event.stopPropagation()}
          onClick={(event) => event.stopPropagation()}
          onBlur={() => setPaneName(paneId, paneNameDraft || paneName)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              setPaneName(paneId, paneNameDraft || paneName);
              (event.currentTarget as HTMLInputElement).blur();
            } else if (event.key === "Escape") {
              event.preventDefault();
              setPaneNameDraft(paneName);
              (event.currentTarget as HTMLInputElement).blur();
            }
          }}
          className="h-7 min-w-[5rem] max-w-[15rem] bg-[var(--bg-secondary)] px-[var(--space-2)] py-0 font-[var(--font-mono)] text-[11px]"
        />
      </div>
      <span className="truncate font-[var(--font-mono)] text-[var(--text-muted)]">{paneId}</span>
    </div>
  );
}
