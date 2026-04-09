import { describe, expect, it } from "vitest";
import {
  applyComposerSuggestion,
  buildComposerSuggestions,
  findActiveComposerToken,
} from "./session-composer-autocomplete";

describe("session composer autocomplete", () => {
  it("detects slash-command tokens at the cursor", () => {
    expect(findActiveComposerToken("Please /pla", 11)).toEqual({
      activeWord: "/pla",
      offset: 7,
      endOffset: 11,
    });
  });

  it("detects file mentions with nested paths", () => {
    expect(findActiveComposerToken("Read @src/components/App.tsx", 28)).toEqual({
      activeWord: "@src/components/App.tsx",
      offset: 5,
      endOffset: 28,
    });
  });

  it("builds matching slash-command suggestions", () => {
    expect(buildComposerSuggestions("/re", [])).toEqual([
      {
        key: "command-review",
        label: "/review",
        description: "Request a focused review of the current session work.",
        insertText: "/review",
        prefix: "/",
      },
    ]);
  });

  it("builds matching file mention suggestions", () => {
    expect(
      buildComposerSuggestions("@src/a", [
        "src/App.tsx",
        "src/agent/runtime.ts",
        "docs/plan.md",
      ]),
    ).toEqual([
      {
        key: "file-src/App.tsx",
        label: "@src/App.tsx",
        description: "Reference this workspace file in the next prompt.",
        insertText: "@src/App.tsx",
        prefix: "@",
      },
      {
        key: "file-src/agent/runtime.ts",
        label: "@src/agent/runtime.ts",
        description: "Reference this workspace file in the next prompt.",
        insertText: "@src/agent/runtime.ts",
        prefix: "@",
      },
    ]);
  });

  it("replaces the active token when applying a suggestion", () => {
    expect(
      applyComposerSuggestion(
        "Please inspect @src/Ap",
        22,
        22,
        {
          key: "file-src/App.tsx",
          label: "@src/App.tsx",
          description: "Reference this workspace file in the next prompt.",
          insertText: "@src/App.tsx",
          prefix: "@",
        },
      ),
    ).toEqual({
      text: "Please inspect @src/App.tsx ",
      cursorPosition: 28,
    });
  });
});
