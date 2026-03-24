import { Badge, Button, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, Input } from "../ui";

export function SidebarHeader({
  workspacesCount,
  approvalsCount,
  reasoningCount,
  createWorkspace,
  query,
  setQuery,
}: {
  workspacesCount: number;
  approvalsCount: number;
  reasoningCount: number;
  createWorkspace: (layoutMode?: "bsp" | "canvas") => void;
  query: string;
  setQuery: (value: string) => void;
}) {
  return (
    <>
      <div className="flex flex-col gap-[var(--space-3)] border-b border-[var(--border)] bg-[var(--bg-primary)] px-[var(--space-4)] py-[var(--space-4)]">
        <div className="flex items-start justify-between gap-[var(--space-3)]">
          <div className="flex flex-col gap-[var(--space-1)]">
            <span className="amux-panel-title">Runtime Environments</span>
            <div className="text-[var(--text-lg)] font-bold text-[var(--text-primary)]">Workspace Fleet</div>
            <div className="text-[var(--text-xs)] leading-relaxed text-[var(--text-muted)]">
              Grouped environments for code, approvals, and telemetry
            </div>
          </div>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="primary" size="icon" className="h-9 w-9 text-[var(--text-lg)] font-semibold">
                +
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="min-w-[12rem]">
              <DropdownMenuItem onSelect={() => createWorkspace("bsp")}>New Workspace (BSP)</DropdownMenuItem>
              <DropdownMenuItem onSelect={() => createWorkspace("canvas")}>
                New Workspace (Infinite Canvas)
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <div className="grid grid-cols-3 gap-[var(--space-2)]">
          <SidebarMetric label="Workspaces" value={String(workspacesCount)} accent="mission" />
          <SidebarMetric label="Approvals" value={String(approvalsCount)} accent="approval" />
          <SidebarMetric label="Reasoning" value={String(reasoningCount)} accent="reasoning" />
        </div>
      </div>

      <div className="px-[var(--space-3)] pt-[var(--space-3)]">
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search workspaces..."
          className="h-9 bg-[var(--bg-secondary)]"
        />
      </div>
    </>
  );
}

function SidebarMetric({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent: "mission" | "approval" | "reasoning";
}) {
  return (
    <div className="flex flex-col gap-[var(--space-1)] rounded-[var(--radius-md)] border border-[var(--border)] bg-[var(--secondary)] px-[var(--space-3)] py-[var(--space-2)]">
      <span className="amux-panel-title">{label}</span>
      <Badge variant={accent} className="w-fit text-[var(--text-md)] font-bold">
        {value}
      </Badge>
    </div>
  );
}
