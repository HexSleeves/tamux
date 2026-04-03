import {
  DEFAULT_AGENT_SETTINGS,
  normalizeAgentSettingsFromSource,
} from "./settings.ts";

function assert(condition: unknown, message: string): void {
  if (!condition) {
    throw new Error(message);
  }
}

assert(
  DEFAULT_AGENT_SETTINGS.weles_max_concurrent_reviews === 2,
  "Default WELES review concurrency should be 2",
);

const normalized = normalizeAgentSettingsFromSource({
  builtin_sub_agents: {
    weles: {
      max_concurrent_reviews: 6,
    },
  },
});

assert(
  normalized.weles_max_concurrent_reviews === 6,
  "Settings normalization should read builtin WELES concurrency overrides",
);
