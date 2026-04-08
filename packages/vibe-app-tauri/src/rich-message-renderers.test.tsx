import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RichTimelineMessageBody } from "./rich-message-renderers";
import type { UiMessage } from "./wave8-client";

function renderMessage(
  message: UiMessage,
  options?: {
    showLineNumbersInDiffs: boolean;
    showLineNumbersInToolViews: boolean;
    wrapLinesInDiffs: boolean;
  },
): string {
  return renderToStaticMarkup(
    <RichTimelineMessageBody message={message} options={options} />,
  );
}

describe("rich message renderers", () => {
  it("renders file artifacts distinctly", () => {
    const html = renderMessage({
      id: "file-1",
      localId: null,
      createdAt: 1,
      role: "system",
      title: "File",
      text: "demo.txt (42 bytes)",
      rawType: "session:file",
    });

    expect(html).toContain("File artifact");
    expect(html).toContain("demo.txt");
  });

  it("renders unified diffs with hunk metadata", () => {
    const html = renderMessage({
      id: "diff-1",
      localId: null,
      createdAt: 1,
      role: "assistant",
      title: "Assistant",
      text: "diff --git a/a.ts b/a.ts\n--- a/a.ts\n+++ b/a.ts\n@@ -1,1 +1,1 @@\n-old\n+new\n",
      rawType: "agent:assistant",
    });

    expect(html).toContain("a.ts");
    expect(html).toContain("@@ -1,1 +1,1 @@");
    expect(html).toContain("+new");
  });

  it("renders tool JSON payloads through the tool surface", () => {
    const html = renderMessage({
      id: "tool-1",
      localId: null,
      createdAt: 1,
      role: "tool",
      title: "Tool result",
      text: "{\"path\":\"/tmp/demo.txt\",\"ok\":true}",
      rawType: "agent:assistant:tool-result",
    });

    expect(html).toContain("/tmp/demo.txt");
    expect(html).toContain("&quot;ok&quot;");
  });

  it("renders diff line numbers when the desktop preference enables them", () => {
    const html = renderMessage(
      {
        id: "diff-2",
        localId: null,
        createdAt: 1,
        role: "assistant",
        title: "Assistant",
        text: "diff --git a/a.ts b/a.ts\n--- a/a.ts\n+++ b/a.ts\n@@ -2,2 +2,2 @@\n-old\n same\n+new\n",
        rawType: "agent:assistant",
      },
      {
        showLineNumbersInDiffs: true,
        showLineNumbersInToolViews: false,
        wrapLinesInDiffs: false,
      },
    );

    expect(html).toContain("diff-line-numbers");
    expect(html).toContain(">2<");
    expect(html).toContain(">3<");
  });

  it("adds wrap styling when the desktop preference enables wrapped diff lines", () => {
    const html = renderMessage(
      {
        id: "diff-3",
        localId: null,
        createdAt: 1,
        role: "assistant",
        title: "Assistant",
        text: "diff --git a/a.ts b/a.ts\n--- a/a.ts\n+++ b/a.ts\n@@ -1,1 +1,1 @@\n-old line that is intentionally long for wrapping\n+new line that is intentionally long for wrapping\n",
        rawType: "agent:assistant",
      },
      {
        showLineNumbersInDiffs: false,
        showLineNumbersInToolViews: false,
        wrapLinesInDiffs: true,
      },
    );

    expect(html).toContain("diff-line-wrap");
  });
});
