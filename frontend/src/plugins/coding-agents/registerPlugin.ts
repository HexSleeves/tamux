import { useCodingAgentsStore } from "./store";

let registered = false;

export function registerCodingAgentsPlugin() {
    if (registered || typeof window === "undefined" || !window.AmuxApi) {
        return;
    }

    window.AmuxApi.registerPlugin({
        id: "coding-agents",
        name: "Coding Agents",
        version: "0.1.0",
        commands: {
            refreshDiscovery: () => {
                void useCodingAgentsStore.getState().refreshAgents();
            },
            launchSelected: () => {
                void useCodingAgentsStore.getState().launchSelectedAgent();
            },
        },
        onLoad: () => {
            void useCodingAgentsStore.getState().refreshAgents();
        },
    });

    registered = true;
}