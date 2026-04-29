import { beforeEach, expect, test, vi } from "vitest";
import {
  DEFAULT_OPERATOR_PROFILE_STATE,
  isOperatorProfileSessionCompleted,
  normalizeOperatorProfileInputKind,
} from "./operatorProfile";
import { useAgentStore } from "./store";

beforeEach(() => {
  useAgentStore.setState({
    operatorProfile: DEFAULT_OPERATOR_PROFILE_STATE,
  });
  vi.restoreAllMocks();
  Reflect.deleteProperty(globalThis, "window");
});

test("normalizeOperatorProfileInputKind treats daemon boolean input as boolean UI input", () => {
  expect(normalizeOperatorProfileInputKind("boolean")).toBe("bool");
  expect(normalizeOperatorProfileInputKind("bool")).toBe("bool");
});

test("normalizeOperatorProfileInputKind keeps unknown input kinds as text fallback", () => {
  expect(normalizeOperatorProfileInputKind("")).toBe("text");
  expect(normalizeOperatorProfileInputKind("unknown")).toBe("text");
});

test("operator profile completion guard does not match question payloads", () => {
  expect(isOperatorProfileSessionCompleted({
    session_id: "ops-1",
    question_id: "enabled",
    field_key: "enabled",
    prompt: "Enable operator modeling overall?",
    input_kind: "boolean",
    optional: false,
  })).toBe(false);
  expect(isOperatorProfileSessionCompleted({
    session_id: "ops-1",
    updated_fields: ["enabled"],
  })).toBe(true);
});

test("startOperatorProfileSession ignores duplicate calls while a start is pending", async () => {
  const agentStartOperatorProfileSession = vi.fn(async () => ({
    session_id: "ops-1",
    kind: "first_run_onboarding",
  }));
  const agentNextOperatorProfileQuestion = vi.fn(async () => ({
    session_id: "ops-1",
    question_id: "enabled",
    field_key: "enabled",
    prompt: "Enable operator modeling overall?",
    input_kind: "boolean",
    optional: false,
  }));

  Object.assign(globalThis, {
    window: {
      zorai: {
        agentStartOperatorProfileSession,
        agentNextOperatorProfileQuestion,
      },
    },
  });

  const first = useAgentStore.getState().startOperatorProfileSession("first_run_onboarding");
  const second = useAgentStore.getState().startOperatorProfileSession("first_run_onboarding");

  expect(agentStartOperatorProfileSession).toHaveBeenCalledTimes(1);
  await expect(first).resolves.toMatchObject({ question_id: "enabled" });
  await expect(second).resolves.toMatchObject({ question_id: "enabled" });
});

test("startOperatorProfileSession stays deduped if loading is cleared before bridge resolves", async () => {
  let resolveStarted: ((value: { session_id: string; kind: string }) => void) | null = null;
  const agentStartOperatorProfileSession = vi.fn(() => new Promise<{ session_id: string; kind: string }>((resolve) => {
    resolveStarted = resolve;
  }));
  const agentNextOperatorProfileQuestion = vi.fn(async () => ({
    session_id: "ops-1",
    question_id: "enabled",
    field_key: "enabled",
    prompt: "Enable operator modeling overall?",
    input_kind: "boolean",
    optional: false,
  }));

  Object.assign(globalThis, {
    window: {
      zorai: {
        agentStartOperatorProfileSession,
        agentNextOperatorProfileQuestion,
      },
    },
  });

  const first = useAgentStore.getState().startOperatorProfileSession("first_run_onboarding");
  useAgentStore.setState((state) => ({
    operatorProfile: {
      ...state.operatorProfile,
      loading: false,
    },
  }));
  const second = useAgentStore.getState().startOperatorProfileSession("first_run_onboarding");

  expect(agentStartOperatorProfileSession).toHaveBeenCalledTimes(1);
  resolveStarted?.({ session_id: "ops-1", kind: "first_run_onboarding" });
  await expect(first).resolves.toMatchObject({ question_id: "enabled" });
  await expect(second).resolves.toMatchObject({ question_id: "enabled" });
});
