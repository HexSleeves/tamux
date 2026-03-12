export interface CodingAgentDefinition {
    id: string;
    label: string;
    description: string;
    executables: string[];
    versionArgs?: string[];
    launchArgs?: string[];
}

export interface DiscoveredCodingAgent extends CodingAgentDefinition {
    available: boolean;
    executable: string | null;
    path: string | null;
    version: string | null;
    error?: string | null;
}

export type CodingAgentsDiscoveryStatus = "idle" | "loading" | "ready" | "error";