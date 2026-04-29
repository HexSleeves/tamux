import { expect, test } from "vitest";
import { normalizeOperatorProfileInputKind } from "./operatorProfile";

test("normalizeOperatorProfileInputKind treats daemon boolean input as boolean UI input", () => {
  expect(normalizeOperatorProfileInputKind("boolean")).toBe("bool");
  expect(normalizeOperatorProfileInputKind("bool")).toBe("bool");
});

test("normalizeOperatorProfileInputKind keeps unknown input kinds as text fallback", () => {
  expect(normalizeOperatorProfileInputKind("")).toBe("text");
  expect(normalizeOperatorProfileInputKind("unknown")).toBe("text");
});
