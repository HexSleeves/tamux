import { describe, expect, it } from "vitest";
import { SUB_AGENT_ROLE_PRESETS, SUB_AGENT_ROLE_PRESET_IDS, findSubAgentRolePreset } from "./subAgentRolePresets";

describe("subAgentRolePresets", () => {
    it("includes execution and broad task-scope presets", () => {
        expect(SUB_AGENT_ROLE_PRESET_IDS).toEqual(expect.arrayContaining([
            "executor",
            "technical",
            "non_technical",
        ]));
        expect(findSubAgentRolePreset("executor")?.label).toBe("Executor / Performer");
        expect(findSubAgentRolePreset("performer")?.id).toBe("executor");
    });

    it("keeps labels unique for role dropdowns", () => {
        const labels = SUB_AGENT_ROLE_PRESETS.map((preset) => preset.label);

        expect(new Set(labels).size).toBe(labels.length);
    });
});
