import { createUnavailableCodingAgents } from "./agentDefinitions";
import type { DiscoveredCodingAgent } from "./types";

type CodingAgentsBridge = {
    discoverCodingAgents?: () => Promise<DiscoveredCodingAgent[]>;
    sendTerminalInput?: (paneId: string | null, data: string) => Promise<boolean>;
};

function getBridge(): CodingAgentsBridge | null {
    if (typeof window === "undefined") {
        return null;
    }

    return ((window as unknown as { amux?: CodingAgentsBridge }).amux ?? null);
}

export function encodeTerminalInput(text: string): string {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    for (const byte of bytes) {
        binary += String.fromCharCode(byte);
    }
    return btoa(binary);
}

export async function discoverCodingAgents(): Promise<DiscoveredCodingAgent[]> {
    const bridge = getBridge();
    if (!bridge?.discoverCodingAgents) {
        return createUnavailableCodingAgents("Coding agent discovery is only available through the Electron bridge.");
    }

    try {
        return await bridge.discoverCodingAgents();
    } catch (error) {
        const message = error instanceof Error ? error.message : "Failed to discover coding agents.";
        return createUnavailableCodingAgents(message);
    }
}

export async function sendCommandToPane(paneId: string, command: string): Promise<boolean> {
    const bridge = getBridge();
    if (!bridge?.sendTerminalInput) {
        throw new Error("Terminal bridge unavailable. Launch from the Electron app.");
    }

    return bridge.sendTerminalInput(paneId, encodeTerminalInput(`${command}\r`));
}