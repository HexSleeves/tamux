import { useMemo } from "react";
import { compileViewDocument } from "../../lib/cduiLoader";
import { isCDUIViewVisible, useCDUIVisibilityFlags } from "../../lib/cduiVisibility";
import { useViewBuilderStore } from "../../lib/viewBuilderStore";
import DynamicRenderer from "../../renderers/DynamicRenderer";
import { ViewErrorBoundary } from "../../renderers/ViewErrorBoundary";
import type { ViewMountProps } from "./shared";

export const ViewMount: React.FC<ViewMountProps> = ({ targetViewId }) => {
    const visibilityFlags = useCDUIVisibilityFlags();
    const activeViewId = useViewBuilderStore((state) => state.activeViewId);
    const isEditMode = useViewBuilderStore((state) => state.isEditMode);
    const draftDocument = useViewBuilderStore((state) =>
        targetViewId ? state.draftDocuments[targetViewId] : undefined,
    );
    const originalDocument = useViewBuilderStore((state) =>
        targetViewId ? state.originalDocuments[targetViewId] : undefined,
    );

    const compiledView = useMemo(() => {
        if (!targetViewId) {
            return null;
        }

        const document = isEditMode && activeViewId === targetViewId && draftDocument
            ? draftDocument
            : originalDocument;

        if (!document) {
            return null;
        }

        try {
            return compileViewDocument(document, `embedded:${targetViewId}`);
        } catch (error) {
            console.warn(`Failed to compile embedded view '${targetViewId}'.`, error);
            if (document !== originalDocument && originalDocument) {
                try {
                    return compileViewDocument(originalDocument, `embedded:${targetViewId}:fallback`);
                } catch (fallbackError) {
                    console.warn(`Failed to compile fallback embedded view '${targetViewId}'.`, fallbackError);
                }
            }

            return null;
        }
    }, [activeViewId, draftDocument, isEditMode, originalDocument, targetViewId]);

    if (!targetViewId || !compiledView) {
        return null;
    }

    if (!isCDUIViewVisible(visibilityFlags, targetViewId, compiledView.when)) {
        return null;
    }

    return (
        <ViewErrorBoundary
            viewId={targetViewId}
            resetKey={`embedded:${targetViewId}`}
            onCrash={(viewId, error) => {
                console.error(`Embedded view '${viewId}' crashed.`, error);
            }}
        >
            <DynamicRenderer
                viewId={targetViewId}
                config={compiledView.layout}
                fallback={compiledView.fallback}
            />
        </ViewErrorBoundary>
    );
};