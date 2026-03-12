export function SidebarResizeHandle() {
    return (
        <div
            data-sidebar-resize-handle="true"
            style={{
                position: "absolute",
                top: 0,
                right: 0,
                width: 6,
                height: "100%",
                cursor: "col-resize",
                background: "transparent",
            }}
        />
    );
}
