import { describe, expect, it } from "vitest";
import { prepareOpenAIRequest } from "./context.ts";
import type { AgentMessage, AgentThread } from "../agentStore/types.ts";

const settings = {
  auto_compact_context: true,
  max_context_messages: 100,
  context_window_tokens: 128_000,
  compact_threshold_pct: 80,
  keep_recent_on_compact: 10,
} as const;

function message(
  partial: Partial<AgentMessage> & Pick<AgentMessage, "id" | "role" | "content" | "createdAt">,
): AgentMessage {
  return {
    threadId: "thread-1",
    provider: undefined,
    model: undefined,
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    reasoning: undefined,
    isCompactionSummary: false,
    isStreaming: false,
    ...partial,
  };
}

describe("prepareOpenAIRequest", () => {
  it("uses the local thread id for ChatGPT subscription responses continuity", () => {
    const prepared = prepareOpenAIRequest(
      [
        message({ id: "u1", role: "user", content: "first question", createdAt: 1 }),
        message({
          id: "a1",
          role: "assistant",
          content: "answer",
          createdAt: 2,
          provider: "openai",
          model: "gpt-5.4",
          api_transport: "responses",
          responseId: "resp_123",
        }),
        message({ id: "u2", role: "user", content: "continue", createdAt: 3 }),
      ],
      settings,
      "openai",
      "gpt-5.4",
      "responses",
      "chatgpt_subscription",
      "",
      { id: "thread-1" } as Pick<
        AgentThread,
        | "id"
        | "upstreamThreadId"
        | "upstreamTransport"
        | "upstreamProvider"
        | "upstreamModel"
        | "upstreamAssistantId"
      >,
    );

    expect(prepared.transport).toBe("responses");
    expect(prepared.previousResponseId).toBeUndefined();
    expect(prepared.upstreamThreadId).toBe("thread-1");
    expect(prepared.messages).toHaveLength(3);
  });
});
