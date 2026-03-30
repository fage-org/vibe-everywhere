import { describe, expect, it } from "vitest";
import {
  clearRediscoveredHiddenProjectKeys,
  filterVisibleProjectKeys,
  removeSuppressedProjectKey,
  suppressProjectKey
} from "@/lib/projectInventory";

describe("project inventory suppression", () => {
  it("suppresses a project key only once", () => {
    expect(suppressProjectKey([], "device::/repo/worktree")).toEqual(["device::/repo/worktree"]);
    expect(suppressProjectKey(["device::/repo/worktree"], "device::/repo/worktree")).toEqual([
      "device::/repo/worktree"
    ]);
  });

  it("removes rediscovered keys from suppression", () => {
    expect(
      clearRediscoveredHiddenProjectKeys(
        ["device::/repo/worktree", "device::/repo/old"],
        ["device::/repo/worktree"]
      )
    ).toEqual(["device::/repo/old"]);
  });

  it("filters hidden projects from visible summaries", () => {
    expect(
      filterVisibleProjectKeys(
        [
          { key: "device::/repo/current", label: "current" },
          { key: "device::/repo/worktree", label: "worktree" }
        ],
        ["device::/repo/worktree"]
      )
    ).toEqual([{ key: "device::/repo/current", label: "current" }]);
  });

  it("can remove a suppressed key explicitly", () => {
    expect(removeSuppressedProjectKey(["a", "b"], "a")).toEqual(["b"]);
  });
});
