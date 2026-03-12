import { useState } from "react";
import { executeCommand } from "../../registry/commandRegistry";

interface UseEditableShellDragDropInput {
    dropEnabled: boolean;
    builderNodeId?: string;
}

export function useEditableShellDragDrop({
    dropEnabled,
    builderNodeId,
}: UseEditableShellDragDropInput) {
    const [dropActive, setDropActive] = useState(false);

    return {
        dropActive,
        onDragOver: (event: React.DragEvent<HTMLDivElement>) => {
            if (!dropEnabled) {
                return;
            }

            event.preventDefault();
            event.dataTransfer.dropEffect = "move";
            setDropActive(true);
        },
        onDragLeave: () => {
            if (dropActive) {
                setDropActive(false);
            }
        },
        onDrop: (event: React.DragEvent<HTMLDivElement>) => {
            if (!dropEnabled || !builderNodeId) {
                return;
            }

            event.preventDefault();
            event.stopPropagation();
            setDropActive(false);

            const paletteRaw = event.dataTransfer.getData("text/amux-palette-item");
            if (paletteRaw) {
                try {
                    const paletteItem = JSON.parse(paletteRaw) as { componentType?: string; blockId?: string };
                    void executeCommand("builder.insertChild", {
                        targetNodeId: builderNodeId,
                        componentType: paletteItem.componentType,
                        blockId: paletteItem.blockId,
                    });
                    return;
                } catch (error) {
                    console.warn("Invalid palette drag payload", error);
                }
            }

            const draggedNodeId = event.dataTransfer.getData("text/amux-node-id");
            if (draggedNodeId) {
                void executeCommand("builder.moveNodeToTarget", { draggedNodeId, targetNodeId: builderNodeId });
            }
        },
    };
}