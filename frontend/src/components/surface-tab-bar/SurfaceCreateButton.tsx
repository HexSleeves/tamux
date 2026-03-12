export function SurfaceCreateButton({ createSurface }: { createSurface: () => void }) {
    return (
        <button
            onClick={() => createSurface()}
            style={{
                background: "var(--accent-soft)",
                border: "1px solid var(--accent-soft)",
                color: "var(--accent)",
                cursor: "pointer",
                fontSize: "var(--text-md)",
                padding: "0 var(--space-2)",
                height: 26,
                borderRadius: "var(--radius-md)",
                fontWeight: 600,
                transition: "all var(--transition-fast)",
            }}
            onMouseEnter={(event) => {
                event.currentTarget.style.background = "rgba(94, 231, 223, 0.2)";
                event.currentTarget.style.borderColor = "var(--accent)";
            }}
            onMouseLeave={(event) => {
                event.currentTarget.style.background = "var(--accent-soft)";
                event.currentTarget.style.borderColor = "var(--accent-soft)";
            }}
            title="New surface"
        >
            +
        </button>
    );
}
