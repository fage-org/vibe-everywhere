import { describe, expect, it } from "vitest";
import {
  formatFeatureCount,
  formatModuleCount,
  wave8FeatureAreas,
  wave8Modules,
  wave8PriorityBuckets,
} from "./bootstrap";

describe("wave8 bootstrap inventory", () => {
  it("reports the tracked module count", () => {
    expect(formatModuleCount(wave8Modules)).toBe("8 Wave 8 modules tracked");
  });

  it("reports the scoped feature count", () => {
    expect(formatFeatureCount(wave8FeatureAreas)).toBe("48 scoped feature points");
  });

  it("keeps explicit P0, P1, and P2 buckets", () => {
    expect(wave8PriorityBuckets.map((bucket) => bucket.priority)).toEqual([
      "P0",
      "P1",
      "P2",
    ]);
  });
});
