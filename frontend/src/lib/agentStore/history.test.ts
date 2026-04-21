import { describe, expect, it } from "vitest";
import {
  buildHydratedRemoteThread,
  isGatewayAgentThread,
  isInternalAgentThread,
} from "./history";

describe("agent thread classification", () => {
  it("recognizes internal daemon threads by id or title", () => {
    expect(isInternalAgentThread({ daemonThreadId: "dm:svarog:weles", title: "Review" })).toBe(true);
    expect(isInternalAgentThread({ title: "Internal DM · Swarog ↔ WELES" })).toBe(true);
    expect(isInternalAgentThread({ daemonThreadId: "thread-user-1", title: "Regular work" })).toBe(false);
  });

  it("recognizes gateway threads by daemon title", () => {
    expect(isGatewayAgentThread({ title: "slack Alice" })).toBe(true);
    expect(isGatewayAgentThread({ title: "discord Bob" })).toBe(true);
    expect(isGatewayAgentThread({ title: "Regular Conversation", lastMessagePreview: "[slack — Alice]: hello" })).toBe(true);
    expect(isGatewayAgentThread({ title: "Regular Conversation", lastMessagePreview: "plain message" })).toBe(false);
  });
});

describe("buildHydratedRemoteThread", () => {
  it("keeps internal daemon threads visible for the React thread browser", () => {
    const hydrated = buildHydratedRemoteThread(
      {
        id: "dm:svarog:weles",
        title: "Internal DM · Swarog ↔ WELES",
        messages: [
          {
            role: "assistant",
            content: "visible in internal tab",
            timestamp: 1,
          },
        ],
      },
      "Svarog",
    );

    expect(hydrated?.thread.daemonThreadId).toBe("dm:svarog:weles");
    expect(hydrated?.thread.title).toBe("Internal DM · Swarog ↔ WELES");
  });
});
