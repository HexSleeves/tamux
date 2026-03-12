import type { CSSProperties } from "react";

export interface Command {
    id: string;
    label: string;
    shortcut?: string;
    category?: string;
    action: () => void;
}

export type CommandPaletteProps = {
    style?: CSSProperties;
    className?: string;
};
