import { describe, expect, it } from "vitest";

import {
  getDefaultModelForProvider,
  getProviderDefinition,
  normalizeAgentProviderId,
} from "./providers.ts";

describe("frontend NVIDIA provider catalog", () => {
  it("registers NVIDIA with hosted defaults and fetch support", () => {
    const nvidia = getProviderDefinition("nvidia");

    expect(nvidia).toBeDefined();
    expect(nvidia?.defaultBaseUrl).toBe("https://integrate.api.nvidia.com/v1");
    expect(nvidia?.defaultModel).toBe("minimaxai/minimax-m2.7");
    expect(nvidia?.supportsModelFetch).toBe(true);
  });

  it("recognizes NVIDIA as a valid provider id", () => {
    expect(normalizeAgentProviderId("nvidia")).toBe("nvidia");
    expect(getDefaultModelForProvider("nvidia")).toBe("minimaxai/minimax-m2.7");
  });
});

describe("frontend Xiaomi MiMo token plan provider catalog", () => {
  it("registers Xiaomi MiMo token plan with static defaults", () => {
    const mimo = getProviderDefinition("xiaomi-mimo-token-plan" as any);

    expect(mimo).toBeDefined();
    expect(mimo?.defaultBaseUrl).toBe("https://api.xiaomimimo.com/v1");
    expect(mimo?.defaultModel).toBe("mimo-v2-pro");
    expect(mimo?.supportsModelFetch).toBe(false);
    expect(mimo?.models.map((model) => [model.id, model.contextWindow])).toEqual([
      ["mimo-v2-pro", 1_000_000],
      ["mimo-v2-omni", 256_000],
    ]);
  });

  it("recognizes Xiaomi MiMo token plan as a valid provider id", () => {
    expect(normalizeAgentProviderId("xiaomi-mimo-token-plan")).toBe("xiaomi-mimo-token-plan");
    expect(getDefaultModelForProvider("xiaomi-mimo-token-plan" as any)).toBe("mimo-v2-pro");
  });
});

describe("frontend Nous Portal provider catalog", () => {
  it("registers Nous Portal with fetchable defaults", () => {
    const nous = getProviderDefinition("nous-portal" as any);

    expect(nous).toBeDefined();
    expect(nous?.defaultBaseUrl).toBe("https://inference-api.nousresearch.com/v1");
    expect(nous?.defaultModel).toBe("nousresearch/hermes-4-70b");
    expect(nous?.supportsModelFetch).toBe(true);
    expect(nous?.models.map((model) => [model.id, model.contextWindow])).toEqual([
      ["nousresearch/hermes-4-70b", 131_072],
      ["nousresearch/hermes-4-405b", 131_072],
      ["nousresearch/hermes-3-llama-3.1-70b", 131_072],
      ["nousresearch/hermes-3-llama-3.1-405b", 131_072],
    ]);
  });

  it("recognizes Nous Portal as a valid provider id", () => {
    expect(normalizeAgentProviderId("nous-portal")).toBe("nous-portal");
    expect(getDefaultModelForProvider("nous-portal" as any)).toBe("nousresearch/hermes-4-70b");
  });
});
