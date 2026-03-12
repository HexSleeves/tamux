import { createElement } from "react";

export function WebviewFrame({
    activeWorkspaceId,
    webviewRef,
    webBrowserUrl,
}: {
    activeWorkspaceId: string | null;
    webviewRef: React.RefObject<any>;
    webBrowserUrl: string;
}) {
    return (
        <div style={{ flex: 1, minHeight: 0 }}>
            {createElement("webview" as any, {
                key: activeWorkspaceId ?? "default-workspace",
                ref: webviewRef,
                src: webBrowserUrl,
                style: {
                    display: "inline-flex",
                    width: "100%",
                    height: "100%",
                    border: "none",
                    background: "#ffffff",
                },
                allowpopups: "true",
                partition: "persist:amux-browser",
            })}
        </div>
    );
}
