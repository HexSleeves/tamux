import { describe, expect, it } from "vitest";
import { getDefaultZoraiSettingsTab, zoraiSettingsTabs } from "./settingsTabs";

describe("Zorai settings tabs", () => {
  it("opens runtime settings by default", () => {
    expect(getDefaultZoraiSettingsTab()).toBe("runtime");
  });

  it("lists all operator settings sections in left-rail order", () => {
    expect(zoraiSettingsTabs.map((tab) => tab.id)).toEqual([
      "runtime",
      "model",
      "auth",
      "interface",
      "tools",
      "concierge",
      "subagents",
      "gateway",
      "keyboard",
      "plugins",
      "about",
    ]);
  });
});
