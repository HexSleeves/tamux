import { Badge, Button, cn } from "../ui";

export function SidebarActions({
  workspacesCount,
  toggleAgentPanel,
  toggleSystemMonitor,
  toggleCommandPalette,
  toggleSearch,
  toggleCommandHistory,
  toggleCommandLog,
  toggleFileManager,
  toggleSessionVault,
  toggleSettings,
}: {
  workspacesCount: number;
  toggleAgentPanel: () => void;
  toggleSystemMonitor: () => void;
  toggleCommandPalette: () => void;
  toggleSearch: () => void;
  toggleCommandHistory: () => void;
  toggleCommandLog: () => void;
  toggleFileManager: () => void;
  toggleSessionVault: () => void;
  toggleSettings: () => void;
}) {
  const actions = [
    { label: "Mission", onClick: toggleAgentPanel, accent: true },
    { label: "Monitor", onClick: toggleSystemMonitor },
    { label: "Palette", onClick: toggleCommandPalette },
    { label: "Search", onClick: toggleSearch },
    { label: "History", onClick: toggleCommandHistory },
    { label: "Logs", onClick: toggleCommandLog },
    { label: "Files", onClick: toggleFileManager, accent: true },
    { label: "Vault", onClick: toggleSessionVault },
    { label: "Settings", onClick: toggleSettings },
  ];

  return (
    <div className="border-t border-[var(--border)] bg-[var(--bg-secondary)]">
      <div className="grid grid-cols-3 gap-[var(--space-2)] p-[var(--space-3)]">
        {actions.map((item) => (
          <Button
            key={item.label}
            variant={item.accent ? "agent" : "secondary"}
            size="sm"
            className={cn(
              "h-auto min-h-[2.5rem] flex-col gap-[2px] px-[var(--space-2)] py-[var(--space-2)] text-center text-[9px] font-semibold uppercase tracking-[0.08em]",
              !item.accent && "text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            )}
            onClick={(event) => {
              event.stopPropagation();
              item.onClick();
            }}
            title={item.label}
          >
            {item.label}
          </Button>
        ))}
      </div>
      <div className="flex justify-center px-[var(--space-4)] pb-[var(--space-3)]">
        <Badge variant="default" className="text-[var(--text-muted)]">
          {workspacesCount} workspace{workspacesCount !== 1 ? "s" : ""}
        </Badge>
      </div>
    </div>
  );
}
