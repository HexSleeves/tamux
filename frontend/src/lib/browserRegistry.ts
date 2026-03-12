export type BrowserDomSnapshot = {
    title: string;
    url: string;
    text: string;
};

export type BrowserScreenshot = {
    dataUrl: string;
    url: string;
    title: string;
};

export type BrowserController = {
    navigate: (url: string) => Promise<boolean>;
    back: () => Promise<boolean>;
    forward: () => Promise<boolean>;
    reload: () => Promise<boolean>;
    getDomSnapshot: () => Promise<BrowserDomSnapshot>;
    captureScreenshot: () => Promise<BrowserScreenshot>;
};

let controller: BrowserController | null = null;

export function registerBrowserController(next: BrowserController): () => void {
    controller = next;
    return () => {
        if (controller === next) {
            controller = null;
        }
    };
}

export function getBrowserController(): BrowserController | null {
    return controller;
}
