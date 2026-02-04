import { describe, expect, it } from "vitest";
import { buildLineDiff } from "./DocumentPreview";

describe("buildLineDiff", () => {
  it("marks added and removed lines", () => {
    const diff = buildLineDiff("line a\nline b", "line a\nline c");
    expect(diff).toContain("- line b");
    expect(diff).toContain("+ line c");
  });

  it("keeps unchanged lines with neutral prefix", () => {
    const diff = buildLineDiff("same", "same");
    expect(diff).toBe("  same");
  });
});
