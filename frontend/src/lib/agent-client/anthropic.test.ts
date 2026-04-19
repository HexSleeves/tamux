import { afterEach, describe, expect, it, vi } from "vitest";

import { sendAnthropic } from "./anthropic.ts";

describe("sendAnthropic", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("uses bearer auth for GitHub Copilot Claude requests", async () => {
    const fetchMock = vi.fn(async () =>
      new Response(
        JSON.stringify({
          content: [{ type: "text", text: "ok" }],
          usage: { input_tokens: 1, output_tokens: 1 },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      ));
    vi.stubGlobal("fetch", fetchMock);

    const iterator = sendAnthropic({
      provider: "github-copilot",
      config: {
        base_url: "https://api.githubcopilot.com",
        model: "claude-sonnet-4.6",
        custom_model_name: "",
        api_key: "copilot-token",
        assistant_id: "",
        api_transport: "chat_completions",
        auth_source: "github_copilot",
        context_window_tokens: 160_000,
      },
      system_prompt: "system",
      messages: [{ role: "user", content: "hello" }],
      streaming: false,
    });

    await iterator.next();

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [, init] = fetchMock.mock.calls[0] as [RequestInfo | URL, RequestInit];
    const headers = init.headers as Record<string, string>;

    expect(headers["Authorization"]).toBe("Bearer copilot-token");
    expect(headers["x-api-key"]).toBeUndefined();
  });
});
