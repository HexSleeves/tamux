import { Button } from "../ui/Button";
import { Card } from "../ui/Card";

export function EditableShellChrome({
  menuOpen,
  onEdit,
}: {
  menuOpen: boolean;
  onEdit: () => void;
}) {
  return (
    <div className="absolute right-[var(--space-2)] top-[var(--space-2)] z-40 flex flex-col items-end gap-[var(--space-2)]">
      {menuOpen ? (
        <Card className="min-w-[9rem] overflow-hidden p-[var(--space-1)] shadow-[var(--shadow-lg)] backdrop-blur-[var(--panel-blur)]">
          <Button
            type="button"
            variant="ghost"
            className="w-full justify-start"
            onClick={(event) => {
              event.stopPropagation();
              onEdit();
            }}
          >
            Edit
          </Button>
        </Card>
      ) : null}
    </div>
  );
}
