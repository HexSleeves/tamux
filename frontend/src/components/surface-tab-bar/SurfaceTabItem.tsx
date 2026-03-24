import { useEffect, useRef, useState } from "react";
import { iconChoices, iconGlyph, iconLabel, normalizeIconId } from "../../lib/iconRegistry";
import { Badge, Button, DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, Input, cn } from "../ui";
import { SURFACE_ICONS, type SurfaceRecord } from "./shared";

export function SurfaceTabItem({
  surface,
  isActive,
  accentColor,
  approvalCount,
  paneCount,
  onSelect,
  onClose,
  onRename,
  onSetIcon,
}: {
  surface: SurfaceRecord;
  isActive: boolean;
  accentColor: string;
  approvalCount: number;
  paneCount: number;
  onSelect: () => void;
  onClose: () => void;
  onRename: (name: string) => void;
  onSetIcon: (icon: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(surface.name);
  const commitTimeoutRef = useRef<number | null>(null);

  const cancelScheduledCommit = () => {
    if (commitTimeoutRef.current !== null) {
      window.clearTimeout(commitTimeoutRef.current);
      commitTimeoutRef.current = null;
    }
  };

  const commit = () => {
    cancelScheduledCommit();
    onRename(draftName.trim() || surface.name);
    setEditing(false);
  };

  const scheduleCommit = () => {
    commitTimeoutRef.current = window.setTimeout(() => {
      commit();
    }, 150);
  };

  useEffect(() => () => cancelScheduledCommit(), []);

  return (
    <div
      role="tab"
      tabIndex={0}
      aria-selected={isActive}
      onClick={onSelect}
      onDoubleClick={() => setEditing(true)}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        "group relative flex h-8 items-center gap-[var(--space-2)] rounded-[var(--radius-md)] border px-[var(--space-3)] text-[var(--text-xs)] transition-colors",
        isActive
          ? "bg-[var(--card)] text-[var(--text-primary)] shadow-[var(--shadow-sm)]"
          : "border-transparent bg-transparent text-[var(--text-muted)] hover:bg-[var(--muted)] hover:text-[var(--text-primary)]"
      )}
      style={{ borderColor: isActive ? accentColor : undefined }}
    >
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-5 min-w-5 px-0 text-[var(--text-sm)]"
            onClick={(event) => event.stopPropagation()}
            title={`Icon: ${iconLabel(surface.icon)}`}
          >
            {iconGlyph(surface.icon)}
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="min-w-[10rem]">
          {iconChoices(SURFACE_ICONS).map((icon) => (
            <DropdownMenuItem
              key={icon.id}
              onSelect={() => onSetIcon(normalizeIconId(icon.id))}
              className="gap-[var(--space-2)]"
            >
              <span className="min-w-6 text-center font-mono">{icon.glyph}</span>
              <span>{icon.label}</span>
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      {editing ? (
        <div className="flex items-center gap-[var(--space-1)]" onClick={(event) => event.stopPropagation()}>
          <Input
            type="text"
            value={draftName}
            onChange={(event) => setDraftName(event.target.value)}
            onBlur={scheduleCommit}
            onKeyDown={(event) => {
              if (event.key === "Enter") commit();
              if (event.key === "Escape") {
                setDraftName(surface.name);
                setEditing(false);
              }
            }}
            autoFocus
            className="h-6 w-[7rem] px-[var(--space-2)] py-0 text-[var(--text-xs)]"
          />
        </div>
      ) : (
        <div className="flex items-center gap-[var(--space-2)] overflow-hidden">
          <span className={cn("truncate", isActive ? "font-semibold" : "font-medium")}>{surface.name}</span>
          <Badge variant="default" className="px-[var(--space-2)] py-[1px] opacity-80">
            {paneCount}
            {approvalCount > 0 ? (
              <span className="ml-[var(--space-1)] text-[var(--approval)]">· {approvalCount}</span>
            ) : null}
          </Badge>
        </div>
      )}

      <Button
        onClick={(event) => {
          event.stopPropagation();
          if (editing) {
            commit();
          } else {
            setDraftName(surface.name);
            setEditing(true);
          }
        }}
        variant="ghost"
        size="sm"
        className="h-5 min-w-5 px-[var(--space-1)] opacity-0 group-hover:opacity-100"
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
        className="h-5 min-w-5 px-[var(--space-1)] opacity-0 group-hover:opacity-100"
      >
        ×
      </Button>
    </div>
  );
}
