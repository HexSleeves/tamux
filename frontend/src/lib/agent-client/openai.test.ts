import { afterEach, describe, expect, it, vi } from "vitest";
import { sendOpenAIResponses } from "./openai.ts";

describe("sendOpenAIResponses", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("omits previous_response_id and sends session continuity headers for ChatGPT subscription", async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) =>
      new Response(
        JSON.stringify({
          id: "resp_1",
          output: [],
          usage: { input_tokens: 1, output_tokens: 1 },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      ));
    vi.stubGlobal("fetch", fetchMock);

    const iterator = sendOpenAIResponses({
      provider: "openai",
      config: {
        base_url: "https://api.openai.com/v1",
        model: "gpt-5.4",
        custom_model_name: "",
        api_key: "token",
        assistant_id: "",
        api_transport: "responses",
        auth_source: "chatgpt_subscription",
        context_window_tokens: 128_000,
      },
      system_prompt: "system",
      messages: [{ role: "user", content: "hello" }],
      streaming: false,
      previousResponseId: "resp_123",
      upstreamThreadId: "thread-1",
      _chatgptAccountId: "acct-1",
    });

    await iterator.next();

    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [, init] = fetchMock.mock.calls[0] as [RequestInfo | URL, RequestInit];
    const body = JSON.parse(String(init.body));
    const headers = init.headers as Record<string, string>;

    expect(body.previous_response_id).toBeUndefined();
    expect(headers["session_id"]).toBe("thread-1");
    expect(headers["x-client-request-id"]).toBe("thread-1");
    expect(headers["chatgpt-account-id"]).toBe("acct-1");
  });
});
