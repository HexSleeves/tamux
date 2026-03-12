import type { Terminal } from "@xterm/xterm";
import { allLeafIds } from "../../lib/bspTree";
import { useWorkspaceStore } from "../../lib/workspaceStore";

export function findPaneLocationValue<T>(
    workspaces: ReturnType<typeof useWorkspaceStore.getState>["workspaces"],
    paneId: string,
    pick: (location: { workspaceId: string; surfaceId: string; cwd: string }) => T,
): T | undefined {
    for (const workspace of workspaces) {
        for (const surface of workspace.surfaces) {
            if (allLeafIds(surface.layout).includes(paneId)) {
                return pick({
                    workspaceId: workspace.id,
                    surfaceId: surface.id,
                    cwd: workspace.cwd,
                });
            }
        }
    }

    return undefined;
}

export function wrapBracketedPaste(text: string, enabled: boolean): string {
    return enabled ? `\u001b[200~${text}\u001b[201~` : text;
}

export function quotePathForShell(filePath: string, platform: string): string {
    if (platform === "win32") {
        return `"${filePath.replace(/"/g, '\\"')}"`;
    }

    return `'${filePath.replace(/'/g, `'\\''`)}'`;
}

export function encodeTextToBase64(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    for (const byte of bytes) {
        binary += String.fromCharCode(byte);
    }
    return btoa(binary);
}

export function decodeBase64ToBytes(value: string): Uint8Array {
    const binary = atob(value);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
        bytes[index] = binary.charCodeAt(index);
    }
    return bytes;
}

export function decodeBase64ToText(value: string): string {
    try {
        return new TextDecoder().decode(decodeBase64ToBytes(value));
    } catch {
        return "";
    }
}

export function stripAnsi(text: string): string {
    return text.replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "").replace(/\u001b\][^\u0007]*\u0007/g, "");
}

export function getSearchableBufferText(term: Terminal): string {
    const buffer = term.buffer.active;
    const lines: string[] = [];

    for (let index = 0; index < buffer.length; index += 1) {
        const line = buffer.getLine(index);
        if (!line) continue;
        lines.push(line.translateToString(true));
    }

    return lines.join("\n");
}

export function getRenderedTerminalText(container: HTMLElement | null): string {
    if (!container) return "";
    return (container.textContent ?? "").replace(/\s+/g, " ");
}

export function countSearchMatches(haystack: string, query: string, options?: { regex?: boolean; caseSensitive?: boolean }): number {
    if (!query) return 0;

    if (options?.regex) {
        try {
            const flags = options.caseSensitive ? "g" : "gi";
            const pattern = new RegExp(query, flags);
            const matches = haystack.match(pattern);
            return matches ? matches.length : 0;
        } catch {
            return 0;
        }
    }

    const source = options?.caseSensitive ? haystack : haystack.toLowerCase();
    const needle = options?.caseSensitive ? query : query.toLowerCase();
    if (!needle) return 0;

    let count = 0;
    let start = 0;
    while (start <= source.length) {
        const index = source.indexOf(needle, start);
        if (index === -1) break;
        count += 1;
        start = index + Math.max(needle.length, 1);
    }
    return count;
}