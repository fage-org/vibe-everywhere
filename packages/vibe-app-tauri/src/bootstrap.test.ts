import { describe, expect, it } from "vitest";
import {
  formatFeatureCount,
  formatModuleCount,
  desktopFeatureAreas,
  desktopModules,
  desktopPriorityBuckets,
} from "./bootstrap";

describe("desktop bootstrap inventory", () => {
  it("reports the tracked module count", () => {
    expect(formatModuleCount(desktopModules)).toBe("8 Desktop modules tracked");
  });

  it("reports the scoped feature count", () => {
    expect(formatFeatureCount(desktopFeatureAreas)).toBe("48 scoped feature points");
  });

  it("keeps explicit P0, P1, and P2 buckets", () => {
    expect(desktopPriorityBuckets.map((bucket) => bucket.priority)).toEqual([
      "P0",
      "P1",
      "P2",
    ]);
  });
});
