import { useEffect, useMemo, useRef, useState } from "react";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { BrowserChrome } from "./web-browser-panel/BrowserChrome";
import { getBrowserContainerStyle, type WebBrowserPanelProps } from "./web-browser-panel/shared";
import { useWebBrowserController } from "./web-browser-panel/useWebBrowserController";
import { WebviewFrame } from "./web-browser-panel/WebviewFrame";

export function WebBrowserPanel({ style, className }: WebBrowserPanelProps = {}) {
    const activeWorkspaceId = useWorkspaceStore((s) => s.activeWorkspaceId);
    const webBrowserOpen = useWorkspaceStore((s) => s.webBrowserOpen);
    const webBrowserUrl = useWorkspaceStore((s) => s.webBrowserUrl);
    const webBrowserReloadToken = useWorkspaceStore((s) => s.webBrowserReloadToken);
    const webBrowserFullscreen = useWorkspaceStore((s) => s.webBrowserFullscreen);
    const setWebBrowserOpen = useWorkspaceStore((s) => s.setWebBrowserOpen);
    const navigateWebBrowser = useWorkspaceStore((s) => s.navigateWebBrowser);
    const webBrowserBack = useWorkspaceStore((s) => s.webBrowserBack);
    const webBrowserForward = useWorkspaceStore((s) => s.webBrowserForward);
    const webBrowserReload = useWorkspaceStore((s) => s.webBrowserReload);
    const toggleWebBrowserFullscreen = useWorkspaceStore((s) => s.toggleWebBrowserFullscreen);

    const webviewRef = useRef<any>(null);
    const [address, setAddress] = useState(webBrowserUrl);
    const [pageTitle, setPageTitle] = useState("Browser");
    const [isDomReady, setIsDomReady] = useState(false);

    useEffect(() => {
        setAddress(webBrowserUrl);
    }, [webBrowserUrl]);

    useEffect(() => {
        const webview = webviewRef.current;
        if (!webview || !isDomReady || typeof webview.reload !== "function") return;
        if (webBrowserReloadToken <= 0) return;
        webview.reload();
    }, [isDomReady, webBrowserReloadToken]);

    useEffect(() => {
        const webview = webviewRef.current;
        if (!webview) return;

        const handleDomReady = () => {
            setIsDomReady(true);
        };

        const handleNavigate = (event: any) => {
            const nextUrl = String(event?.url || "");
            if (nextUrl) {
                navigateWebBrowser(nextUrl);
            }
            if (typeof webview.getTitle === "function") {
                const title = String(webview.getTitle() || "").trim();
                if (title) setPageTitle(title);
            }
        };

        const handlePageTitle = (event: any) => {
            const title = String(event?.title || "").trim();
            if (title) setPageTitle(title);
        };

        const handleDidFailLoad = (event: any) => {
            const errorCode = Number(event?.errorCode);
            if (errorCode === -3) {
                // ERR_ABORTED is expected when a navigation gets superseded.
                return;
            }

            const description = String(event?.errorDescription || "unknown error");
            const failedUrl = String(event?.validatedURL || event?.url || "");
            console.warn("[WebBrowserPanel] failed to load URL", { errorCode, description, failedUrl });
        };

        webview.addEventListener("dom-ready", handleDomReady);
        webview.addEventListener("did-navigate", handleNavigate);
        webview.addEventListener("did-navigate-in-page", handleNavigate);
        webview.addEventListener("page-title-updated", handlePageTitle);
        webview.addEventListener("did-fail-load", handleDidFailLoad);

        return () => {
            setIsDomReady(false);
            webview.removeEventListener("dom-ready", handleDomReady);
            webview.removeEventListener("did-navigate", handleNavigate);
            webview.removeEventListener("did-navigate-in-page", handleNavigate);
            webview.removeEventListener("page-title-updated", handlePageTitle);
            webview.removeEventListener("did-fail-load", handleDidFailLoad);
        };
    }, [navigateWebBrowser]);

    useWebBrowserController({
        webviewRef,
        isDomReady,
        navigateWebBrowser,
        webBrowserBack,
        webBrowserForward,
        webBrowserReload,
        webBrowserUrl,
        pageTitle,
    });

    const containerStyle = useMemo(() => {
        return getBrowserContainerStyle(webBrowserFullscreen);
    }, [webBrowserFullscreen]);

    if (!webBrowserOpen) return null;

    return (
        <div style={{ ...containerStyle, ...(style ?? {}) }} className={className}>
            <BrowserChrome
                address={address}
                setAddress={setAddress}
                pageTitle={pageTitle}
                back={webBrowserBack}
                forward={webBrowserForward}
                reload={webBrowserReload}
                navigate={navigateWebBrowser}
                toggleFullscreen={toggleWebBrowserFullscreen}
                close={() => setWebBrowserOpen(false)}
            />

            <WebviewFrame
                activeWorkspaceId={activeWorkspaceId}
                webviewRef={webviewRef}
                webBrowserUrl={webBrowserUrl}
            />
        </div>
    );
}
