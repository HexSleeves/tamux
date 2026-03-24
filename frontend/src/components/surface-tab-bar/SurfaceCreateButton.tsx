import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../ui";

export function SurfaceCreateButton({
  layoutMode,
  createBspTerminal,
  createCanvasSurface,
  createCanvasTerminal,
  createCanvasBrowser,
}: {
  layoutMode: "bsp" | "canvas";
  createBspTerminal: () => void;
  createCanvasSurface: () => void;
  createCanvasTerminal: () => void;
  createCanvasBrowser: () => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="primary"
          size="sm"
          className="h-7 min-w-7 px-[var(--space-2)] text-[var(--text-md)] font-semibold"
          title="Add Terminal or Canvas"
        >
          +
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="min-w-[13rem]">
        <DropdownMenuItem onSelect={createCanvasSurface}>New Infinite Canvas</DropdownMenuItem>
        {layoutMode === "canvas" ? (
          <>
            <DropdownMenuItem onSelect={createCanvasTerminal}>New Canvas Terminal</DropdownMenuItem>
            <DropdownMenuItem onSelect={createCanvasBrowser}>New Canvas Browser</DropdownMenuItem>
          </>
        ) : null}
        <DropdownMenuItem onSelect={createBspTerminal}>
          {layoutMode === "bsp" ? "New Terminal (BSP Pane)" : "New Terminal Surface (BSP)"}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
