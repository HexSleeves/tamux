export function SidebarResizeHandle() {
  return (
    <div
      data-sidebar-resize-handle="true"
      className="absolute right-0 top-0 z-20 h-full w-2 cursor-col-resize bg-[linear-gradient(90deg,transparent,rgba(148,163,184,0.3),transparent)] transition-opacity hover:opacity-100"
    />
  );
}
