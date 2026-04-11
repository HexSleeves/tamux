import { createRef } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { AgentThread } from "@/lib/agentStore";
import { ChatView } from "./ChatView";

describe("ChatView participant suggestions", () => {
  it("renders queued participant suggestions with badges", () => {
    const activeThread: AgentThread = {
      id: "thread-1",
      daemonThreadId: "daemon-thread-1",
      workspaceId: null,
      surfaceId: null,
      paneId: null,
      agent_name: "Svarog",
      title: "Conversation",
      createdAt: 1,
      updatedAt: 1,
      messageCount: 0,
      totalInputTokens: 0,
      totalOutputTokens: 0,
      totalTokens: 0,
      compactionCount: 0,
      lastMessagePreview: "",
      threadParticipants: [],
      queuedParticipantSuggestions: [
        {
          id: "sugg-1",
          targetAgentId: "weles",
          targetAgentName: "Weles",
          instruction: "verify claims",
          forceSend: true,
          status: "failed",
          createdAt: 1,
          updatedAt: 1,
          error: "provider unavailable",
        },
      ],
    };

    const html = renderToStaticMarkup(
      <ChatView
        messages={[]}
        todos={[]}
        input=""
        setInput={() => {}}
        inputRef={createRef<HTMLTextAreaElement>()}
        onKeyDown={() => {}}
        agentSettings={{ enabled: true, chatFontFamily: "monospace", reasoning_effort: "high" }}
        isStreamingResponse={false}
        activeThread={activeThread}
        messagesEndRef={createRef<HTMLDivElement>()}
        onSendMessage={() => {}}
        onSendParticipantSuggestion={() => {}}
        onDismissParticipantSuggestion={() => {}}
        onStopStreaming={() => {}}
        onUpdateReasoningEffort={() => {}}
        canStartGoalRun={false}
        onStartGoalRun={async () => false}
      />,
    );

    expect(html).toContain("Participant Suggestions");
    expect(html).toContain("Weles");
    expect(html).toContain("Force Send");
    expect(html).toContain("Failed");
    expect(html).toContain("provider unavailable");
  });
});