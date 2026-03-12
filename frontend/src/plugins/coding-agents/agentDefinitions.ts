import type { CodingAgentDefinition, DiscoveredCodingAgent } from "./types";

export const KNOWN_CODING_AGENT_DEFINITIONS: CodingAgentDefinition[] = [
    {
        id: "claude",
        label: "Claude Code",
        description: "Anthropic's terminal coding agent.",
        executables: ["claude"],
        versionArgs: ["--version"],
    },
    {
        id: "codex",
        label: "Codex CLI",
        description: "OpenAI Codex terminal workflow.",
        executables: ["codex"],
        versionArgs: ["--version"],
    },
    {
        id: "gemini",
        label: "Gemini CLI",
        description: "Google Gemini terminal agent.",
        executables: ["gemini"],
        versionArgs: ["--version"],
    },
    {
        id: "opencode",
        label: "OpenCode",
        description: "OpenCode terminal coding assistant.",
        executables: ["opencode"],
        versionArgs: ["--version"],
    },
    {
        id: "kimi",
        label: "Kimi CLI",
        description: "Moonshot Kimi coding assistant.",
        executables: ["kimi"],
        versionArgs: ["--version"],
    },
    {
        id: "aider",
        label: "Aider",
        description: "Aider pair-programming CLI.",
        executables: ["aider"],
        versionArgs: ["--version"],
    },
    {
        id: "goose",
        label: "Goose",
        description: "Goose local coding agent.",
        executables: ["goose"],
        versionArgs: ["--version"],
    },
    {
        id: "qwen-coder",
        label: "Qwen Coder",
        description: "Qwen coding CLI if installed locally.",
        executables: ["qwen", "qwen-coder"],
        versionArgs: ["--version"],
    },
];

export function buildCodingAgentLaunchCommand(agent: Pick<DiscoveredCodingAgent, "executable" | "launchArgs">): string {
    const executable = agent.executable?.trim();
    const args = (agent.launchArgs ?? []).map((value) => value.trim()).filter(Boolean);
    return [executable, ...args].filter(Boolean).join(" ");
}

export function createUnavailableCodingAgents(error: string): DiscoveredCodingAgent[] {
    return KNOWN_CODING_AGENT_DEFINITIONS.map((agent) => ({
        ...agent,
        available: false,
        executable: agent.executables[0] ?? null,
        path: null,
        version: null,
        error,
    }));
}