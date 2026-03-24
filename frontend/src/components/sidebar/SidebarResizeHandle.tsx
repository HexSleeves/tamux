export function SidebarResizeHandle() {
  return (
    <div
      data-sidebar-resize-handle="true"
      className="absolute right-0 top-0 z-20 h-full w-2 cursor-col-resize bg-[var(--resize-handle-gradient)] transition-opacity hover:opacity-100"
    />
  );
}
