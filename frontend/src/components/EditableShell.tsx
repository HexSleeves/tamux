import type React from "react";
import type { UINodeBuilderMeta } from "../schemas/uiSchema";
import { EditableShellChrome } from "./editable-shell/EditableShellChrome";
import { normalizeWrapperStyle } from "./editable-shell/styleNormalizers";
import { useEditableShellDragDrop } from "./editable-shell/useEditableShellDragDrop";
import { useEditableShellState } from "./editable-shell/useEditableShellState";

interface EditableShellProps {
    style?: React.CSSProperties;
    className?: string;
    children?: React.ReactNode;
    content: React.ReactNode;
    visible?: boolean;
    hidden?: boolean;
    resizable?: boolean;
    resizeAxis?: "both" | "horizontal" | "vertical";
    minWidth?: number | string;
    minHeight?: number | string;
    maxWidth?: number | string;
    maxHeight?: number | string;
    builderNodeId?: string;
    builderViewId?: string;
    builderComponentType?: string;
    builderMeta?: UINodeBuilderMeta;
}

export function EditableShell({
    style,
    className,
    children,
    content,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderNodeId,
    builderViewId,
    builderComponentType,
    builderMeta,
}: EditableShellProps) {
    if (hidden || visible === false) {
        return null;
    }

    const wrapperStyle = normalizeWrapperStyle({
        style,
        resizable,
        resizeAxis,
        minWidth,
        minHeight,
        maxWidth,
        maxHeight,
    });
    const {
        chromeEnabled,
        isSelected,
        hasWrapperStyling,
        selectionStyle,
        menuOpen,
        isEditMode,
        handleSelect,
        handleStartEditing,
    } = useEditableShellState({
        className,
        style,
        resizable,
        minWidth,
        minHeight,
        maxWidth,
        maxHeight,
        builderNodeId,
        builderViewId,
        builderComponentType,
        builderMeta,
    });

    if (!chromeEnabled && !isSelected && !hasWrapperStyling) {
        if (children) {
            return (
                <>
                    {content}
                    {children}
                </>
            );
        }

        return <>{content}</>;
    }

    if (wrapperStyle.position === undefined) {
        wrapperStyle.position = "relative";
    }

    const dropEnabled = Boolean(chromeEnabled && isEditMode && builderMeta?.locked !== true);
    const { dropActive, onDragOver, onDragLeave, onDrop } = useEditableShellDragDrop({
        dropEnabled,
        builderNodeId,
    });

    return (
        <div
            style={{
                ...wrapperStyle,
                ...selectionStyle,
                ...(dropActive ? { boxShadow: "0 0 0 2px rgba(129, 230, 217, 0.85) inset" } : {}),
            }}
            className={className}
            onClickCapture={handleSelect}
            onDragOver={onDragOver}
            onDragLeave={onDragLeave}
            onDrop={onDrop}
        >
            {chromeEnabled ? <EditableShellChrome menuOpen={menuOpen} onEdit={handleStartEditing} /> : null}
            {content}
            {children}
        </div>
    );
}
