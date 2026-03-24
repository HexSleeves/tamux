import { useMemo } from "react";
import { ComponentRegistryAPI } from "../registry/componentRegistry";
import { VIEW_BUILDER_PRIMITIVE_PALETTE } from "../lib/viewBuilderPrimitives";
import { useViewBuilderStore } from "../lib/viewBuilderStore";
import { Badge } from "./ui/Badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/Card";
import { BuilderDocumentTree } from "./view-builder-overlay/BuilderDocumentTree";
import { BuilderHeader } from "./view-builder-overlay/BuilderHeader";
import { BuilderInspector } from "./view-builder-overlay/BuilderInspector";
import { BuilderPaletteSection } from "./view-builder-overlay/BuilderPaletteSection";
import { BuilderSelectionPanel } from "./view-builder-overlay/BuilderSelectionPanel";
import { BUILDER_PRIMITIVE_COMPONENTS, findNodeById, findNodeEditable, overlayShellStyle } from "./view-builder-overlay/shared";

export function ViewBuilderOverlay() {
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const activeViewId = useViewBuilderStore((state) => state.activeViewId);
  const selectedNode = useViewBuilderStore((state) => state.selectedNode);
  const selectNode = useViewBuilderStore((state) => state.selectNode);
  const stopEditing = useViewBuilderStore((state) => state.stopEditing);
  const dirtyViewIds = useViewBuilderStore((state) => state.dirtyViewIds);
  const draftDocuments = useViewBuilderStore((state) => state.draftDocuments);

  const draftDocument = activeViewId ? draftDocuments[activeViewId] : null;
  const selectedNodeDocument = useMemo(
    () => (draftDocument && selectedNode?.nodeId ? findNodeById(draftDocument, selectedNode.nodeId) : null),
    [draftDocument, selectedNode?.nodeId]
  );
  const isDirty = activeViewId ? Boolean(dirtyViewIds[activeViewId]) : false;
  const selectedEditable = useMemo(() => {
    if (!draftDocument || !selectedNode?.nodeId) {
      return null;
    }

    return findNodeEditable(draftDocument, selectedNode.nodeId);
  }, [draftDocument, selectedNode?.nodeId]);

  const registeredComponents = useMemo(
    () =>
      ComponentRegistryAPI.list()
        .filter((name) => name !== "Unknown" && !BUILDER_PRIMITIVE_COMPONENTS.has(name))
        .sort((left, right) => left.localeCompare(right)),
    []
  );

  if (!isEditMode) {
    return null;
  }

  return (
    <Card
      style={overlayShellStyle}
      className="overflow-auto text-[var(--text-primary)]"
    >
      <BuilderHeader
        activeViewId={activeViewId}
        isDirty={isDirty}
        selectedEditable={selectedEditable}
        stopEditing={stopEditing}
      />

      <CardContent className="grid gap-[var(--space-4)] p-[var(--space-4)]">
        <BuilderSelectionPanel
          nodeId={selectedNode?.nodeId ?? null}
          componentType={selectedNode?.componentType ?? null}
          selectedEditable={selectedEditable}
        />

        <BuilderInspector selectedNodeDocument={selectedNodeDocument} />

        <BuilderDocumentTree
          activeViewId={activeViewId}
          draftDocument={draftDocument}
          selectedNodeId={selectedNode?.nodeId ?? null}
          onSelect={selectNode}
        />

        <BuilderPaletteSection
          title="Primitive Palette"
          items={VIEW_BUILDER_PRIMITIVE_PALETTE.map((item) => ({
            key: item.id,
            label: item.label,
            payload: { blockId: item.blockId, componentType: item.componentType },
          }))}
        />

        <BuilderPaletteSection
          title="Runtime Components"
          items={registeredComponents.map((name) => ({
            key: name,
            label: name,
            payload: { componentType: name },
          }))}
        />

        <Card className="border-dashed bg-[var(--surface)]/80">
          <CardHeader className="gap-[var(--space-2)] p-[var(--space-3)]">
            <Badge variant="accent" className="w-fit">
              Builder status
            </Badge>
            <CardTitle className="text-[var(--text-sm)]">Next Interaction Targets</CardTitle>
            <CardDescription className="text-[13px] leading-6">
              This first builder slice supports edit mode entry, node targeting, and a live component palette.
              Drag, resize, align, and YAML mutation can now build on stable node ids instead of anonymous tree positions.
            </CardDescription>
          </CardHeader>
        </Card>
      </CardContent>
    </Card>
  );
}
