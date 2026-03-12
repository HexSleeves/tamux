import { useEffect } from "react";
import { registerBrowserController } from "../../lib/browserRegistry";

export function useWebBrowserController({
    webviewRef,
    isDomReady,
    navigateWebBrowser,
    webBrowserBack,
    webBrowserForward,
    webBrowserReload,
    webBrowserUrl,
    pageTitle,
}: {
    webviewRef: React.RefObject<any>;
    isDomReady: boolean;
    navigateWebBrowser: (url: string) => void;
    webBrowserBack: () => void;
    webBrowserForward: () => void;
    webBrowserReload: () => void;
    webBrowserUrl: string;
    pageTitle: string;
}) {
    useEffect(() => {
        const unregister = registerBrowserController({
            navigate: async (url: string) => {
                navigateWebBrowser(url);
                return true;
            },
            back: async () => {
                webBrowserBack();
                return true;
            },
            forward: async () => {
                webBrowserForward();
                return true;
            },
            reload: async () => {
                webBrowserReload();
                return true;
            },
            getDomSnapshot: async () => {
                const webview = webviewRef.current;
                if (!webview || !isDomReady || typeof webview.executeJavaScript !== "function") {
                    throw new Error("Browser not ready");
                }

                const result = await webview.executeJavaScript(`(() => {
  const text = (document?.body?.innerText || "").trim();
  return {
    title: document?.title || "",
    url: location?.href || "",
    text: text.length > 20000 ? text.slice(0, 20000) : text,
  };
})();`);

                return {
                    title: String(result?.title || ""),
                    url: String(result?.url || ""),
                    text: String(result?.text || ""),
                };
            },
            captureScreenshot: async () => {
                const webview = webviewRef.current;
                if (!webview || !isDomReady || typeof webview.capturePage !== "function") {
                    throw new Error("Browser not ready");
                }

                const image = await webview.capturePage();
                const dataUrl = image.toDataURL();
                const currentUrl = webBrowserUrl;
                const title = typeof webview.getTitle === "function" ? String(webview.getTitle() || "") : pageTitle;

                return {
                    dataUrl,
                    url: currentUrl,
                    title,
                };
            },
        });

        return unregister;
    }, [isDomReady, navigateWebBrowser, pageTitle, webBrowserBack, webBrowserForward, webBrowserReload, webBrowserUrl, webviewRef]);
}
