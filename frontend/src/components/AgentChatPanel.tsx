import type { CSSProperties } from "react";
import { AgentChatPanelProvider, AgentChatPanelScaffold } from "./agent-chat-panel/runtime";

type AgentChatPanelProps = {
    style?: CSSProperties;
    className?: string;
};

export function AgentChatPanel({ style, className }: AgentChatPanelProps = {}) {
    return (
        <AgentChatPanelProvider>
            <AgentChatPanelScaffold style={style} className={className} />
        </AgentChatPanelProvider>
    );
}
