import { useState } from "react";
import type { Workspace } from "../../lib/types";
import { Badge, Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, cn } from "../ui";
import { ICON_CHOICES } from "./shared";

export function WorkspaceItem({
  workspace,
  index,
  isActive,
  unreadCount,
  onSelect,
  onClose,
  onRename,
  onSetIcon,
  children,
}: {
  workspace: Workspace;
  index: number;
  isActive: boolean;
  unreadCount: number;
  onSelect: () => void;
  onClose: () => void;
  onRename: (name: string) => void;
  onSetIcon: (icon: string) => void;
  children?: React.ReactNode;
}) {
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(workspace.name);
  const [draftIcon, setDraftIcon] = useState(workspace.icon);

  const resetDraft = () => {
    setDraftName(workspace.name);
    setDraftIcon(workspace.icon);
  };

  const commit = () => {
    onRename(draftName.trim() || workspace.name);
    onSetIcon(draftIcon.trim() || workspace.icon);
    setEditing(false);
  };

  return (
    <div className="flex flex-col gap-[var(--space-1)]">
      <div
        onClick={onSelect}
        className={cn(
          "flex cursor-pointer items-center gap-[var(--space-3)] rounded-[var(--radius-md)] border border-transparent px-[var(--space-3)] py-[var(--space-3)] transition-colors",
          isActive
            ? "bg-[var(--bg-tertiary)] text-[var(--text-primary)]"
            : "hover:bg-[var(--bg-secondary)] text-[var(--text-secondary)]"
        )}
        style={{
          borderColor: isActive ? "var(--glass-border)" : undefined,
          borderLeft: `2px solid ${isActive ? workspace.accentColor : "transparent"}`,
        }}
      >
        <div className="flex shrink-0 flex-col items-center gap-[var(--space-1)]">
          <Badge variant="default" className="min-w-8 justify-center px-[var(--space-2)] py-[var(--space-1)] uppercase">
            {workspace.icon}
          </Badge>
          <span
            className="h-2 w-2 rounded-full border"
            style={{ background: workspace.accentColor, borderColor: workspace.accentColor }}
          />
        </div>

        <div className="min-w-0 flex-1 overflow-hidden">
          {editing ? (
            <div className="flex flex-col gap-[var(--space-2)]" onClick={(event) => event.stopPropagation()}>
              <Input
                type="text"
                value={draftName}
                onChange={(event) => setDraftName(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") commit();
                  if (event.key === "Escape") {
                    resetDraft();
                    setEditing(false);
                  }
                }}
                autoFocus
                className="h-8"
              />
              <Select value={draftIcon} onValueChange={setDraftIcon}>
                <SelectTrigger className="h-8">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {ICON_CHOICES.map((icon) => (
                    <SelectItem key={icon} value={icon}>
                      {icon}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <div className="flex justify-end gap-[var(--space-2)]">
                <Button
                  type="button"
                  variant="secondary"
                  size="sm"
                  onClick={() => {
                    resetDraft();
                    setEditing(false);
                  }}
                >
                  Cancel
                </Button>
                <Button type="button" variant="primary" size="sm" onClick={commit}>
                  Save
                </Button>
              </div>
            </div>
          ) : (
            <>
              <div className={cn("truncate text-[var(--text-sm)]", isActive ? "font-semibold" : "font-normal")}>
                {workspace.name}
              </div>

              {workspace.gitBranch ? (
                <div className="mt-[2px] truncate text-[var(--text-xs)] text-[var(--text-muted)]">
                  <span className="opacity-60">⎇</span> {workspace.gitBranch}
                  {workspace.gitDirty ? <span className="ml-1 text-[var(--warning)]">●</span> : null}
                </div>
              ) : null}

              <div className="mt-[var(--space-2)] flex flex-wrap gap-[var(--space-1)]">
                {workspace.cwd ? (
                  <Badge variant="default" className="amux-code max-w-40 truncate">
                    {workspace.cwd}
                  </Badge>
                ) : null}

                {workspace.listeningPorts.length > 0 ? (
                  <Badge variant="default">:{workspace.listeningPorts.join(",")}</Badge>
                ) : null}
              </div>
            </>
          )}
        </div>

        {index <= 9 ? <Badge variant="default">{index}</Badge> : null}

        {unreadCount > 0 ? <Badge variant="accent">{unreadCount > 99 ? "99+" : unreadCount}</Badge> : null}

        <div className="flex items-center gap-[var(--space-1)]">
          <Button
            onClick={(event) => {
              event.stopPropagation();
              setEditing((current) => !current);
            }}
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0 text-[var(--text-muted)]"
            title="Rename workspace"
          >
            ✎
          </Button>

          <Button
            onClick={(event) => {
              event.stopPropagation();
              onClose();
            }}
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0 text-[var(--text-muted)]"
            title="Close workspace"
          >
            ×
          </Button>
        </div>
      </div>
      {children}
    </div>
  );
}
