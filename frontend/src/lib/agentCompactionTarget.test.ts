import { describe, expect, it } from "vitest";
import { buildDaemonAgentConfig } from "./agentDaemonConfig.ts";
import { resolveContextCompactionTargetTokens } from "./agent-client/context.ts";
import {
  DEFAULT_AGENT_SETTINGS,
  normalizeAgentSettingsFromSource,
} from "./agentStore/settings.ts";

describe("agent compaction target", () => {
  it("does not serialize removed context budget settings", () => {
    expect(buildDaemonAgentConfig(DEFAULT_AGENT_SETTINGS)).not.toHaveProperty(
      "context_budget_tokens",
    );
  });

  it("drops legacy context budget values during frontend normalization", () => {
    const normalized = normalizeAgentSettingsFromSource({
      context_budget_tokens: 222_000,
    } as any);

    expect(normalized).not.toHaveProperty("context_budget_tokens");
  });

  it("uses the primary model threshold for heuristic compaction", () => {
    expect(
      resolveContextCompactionTargetTokens({
        auto_compact_context: true,
        max_context_messages: 100,
        context_window_tokens: 400_000,
        compact_threshold_pct: 80,
        keep_recent_on_compact: 10,
        compaction: DEFAULT_AGENT_SETTINGS.compaction,
      }),
    ).toBe(320_000);
  });

  it("caps the target by the WELES compaction window", () => {
    expect(
      resolveContextCompactionTargetTokens({
        auto_compact_context: true,
        max_context_messages: 100,
        context_window_tokens: 400_000,
        compact_threshold_pct: 80,
        keep_recent_on_compact: 10,
        compaction: {
          ...DEFAULT_AGENT_SETTINGS.compaction,
          strategy: "weles",
          weles: {
            provider: "minimax-coding-plan",
            model: "MiniMax-M2.7",
            reasoning_effort: "medium",
          },
        },
      }),
    ).toBe(164_000);
  });

  it("caps the target by the custom compaction model window", () => {
    expect(
      resolveContextCompactionTargetTokens({
        auto_compact_context: true,
        max_context_messages: 100,
        context_window_tokens: 400_000,
        compact_threshold_pct: 80,
        keep_recent_on_compact: 10,
        compaction: {
          ...DEFAULT_AGENT_SETTINGS.compaction,
          strategy: "custom_model",
          custom_model: {
            ...DEFAULT_AGENT_SETTINGS.compaction.custom_model,
            context_window_tokens: 160_000,
          },
        },
      }),
    ).toBe(128_000);
  });
});
