import { Suspense, lazy } from "react";
import { parsePatch } from "diff";
import type { UiMessage } from "./wave8-client";

const LazySyntaxCodeBlock = lazy(() =>
  import("./syntax-code-block").then((module) => ({
    default: module.SyntaxCodeBlock,
  })),
);

type MarkdownBlock =
  | { type: "paragraph"; content: string }
  | { type: "heading"; content: string }
  | { type: "list"; items: string[] }
  | { type: "code"; content: string; language: string };

export type RichRenderOptions = {
  showLineNumbersInDiffs: boolean;
  showLineNumbersInToolViews: boolean;
  wrapLinesInDiffs: boolean;
};

const defaultOptions: RichRenderOptions = {
  showLineNumbersInDiffs: false,
  showLineNumbersInToolViews: false,
  wrapLinesInDiffs: false,
};

export function RichTimelineMessageBody({
  message,
  options = defaultOptions,
}: {
  message: UiMessage;
  options?: RichRenderOptions;
}) {
  if (message.rawType === "session:file") {
    return (
      <div className="message-surface file-surface">
        <strong>File artifact</strong>
        <p>{message.text}</p>
      </div>
    );
  }

  if (looksLikeDiff(message.text)) {
    return <DiffSurface diffText={message.text} options={options} />;
  }

  if (message.role === "tool") {
    if (looksLikeJson(message.text)) {
      return (
        <div className="message-surface tool-surface">
          <CodeBlock
            language="json"
            code={formatJson(message.text)}
            options={options}
          />
        </div>
      );
    }

    return (
      <div className="message-surface tool-surface">
        <MarkdownSurface text={message.text} options={options} />
      </div>
    );
  }

  if (message.rawType.includes("terminal-output")) {
    return (
      <div className="message-surface terminal-surface">
        <CodeBlock language="bash" code={message.text} options={options} />
      </div>
    );
  }

  return <MarkdownSurface text={message.text} options={options} />;
}

function MarkdownSurface({
  text,
  options,
}: {
  text: string;
  options: RichRenderOptions;
}) {
  const blocks = splitMarkdownBlocks(text);

  return (
    <div className="markdown-surface">
      {blocks.map((block, index) => {
        if (block.type === "code") {
          return (
            <CodeBlock
              key={`${block.type}-${index}`}
              language={block.language || "text"}
              code={block.content}
              options={options}
            />
          );
        }

        if (block.type === "list") {
          return (
            <ul key={`${block.type}-${index}`} className="markdown-list">
              {block.items.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          );
        }

        if (block.type === "heading") {
          return (
            <h4 key={`${block.type}-${index}`} className="markdown-heading">
              {block.content}
            </h4>
          );
        }

        return (
          <p key={`${block.type}-${index}`} className="markdown-paragraph">
            {block.content}
          </p>
        );
      })}
    </div>
  );
}

function DiffSurface({
  diffText,
  options,
}: {
  diffText: string;
  options: RichRenderOptions;
}) {
  const patches = parsePatch(diffText);
  if (patches.length === 0) {
    return <MarkdownSurface text={diffText} options={options} />;
  }

  return (
    <div className="diff-surface">
      {patches.map((patch) => (
        <section
          key={`${patch.oldFileName ?? "old"}-${patch.newFileName ?? "new"}-${patch.hunks.length}`}
          className="diff-file"
        >
          <div className="diff-file-header">
            <strong>{patch.newFileName ?? patch.oldFileName ?? "patch"}</strong>
            <span>{patch.hunks.length} hunks</span>
          </div>
          {patch.hunks.map((hunk, index) => {
            let oldLine = hunk.oldStart;
            let newLine = hunk.newStart;

            return (
              <div key={`${patch.oldFileName ?? "file"}-${index}`} className="diff-hunk">
                <div className="diff-hunk-header">
                  {`@@ -${hunk.oldStart},${hunk.oldLines} +${hunk.newStart},${hunk.newLines} @@`}
                </div>
                {hunk.lines.map((line, lineIndex) => {
                  const isAdd = line.startsWith("+") && !line.startsWith("+++");
                  const isRemove = line.startsWith("-") && !line.startsWith("---");
                  const oldNumber = isAdd ? null : oldLine++;
                  const newNumber = isRemove ? null : newLine++;

                  return (
                    <pre
                      key={`${patch.oldFileName ?? "file"}-${hunk.oldStart}-${lineIndex}`}
                      className={`diff-line ${diffLineClass(line)} ${
                        options.wrapLinesInDiffs ? "diff-line-wrap" : ""
                      }`}
                    >
                      {options.showLineNumbersInDiffs ? (
                        <span className="diff-line-numbers">
                          <span className="diff-line-number">
                            {oldNumber === null ? "" : oldNumber}
                          </span>
                          <span className="diff-line-number">
                            {newNumber === null ? "" : newNumber}
                          </span>
                        </span>
                      ) : null}
                      <span>{line}</span>
                    </pre>
                  );
                })}
              </div>
            );
          })}
        </section>
      ))}
    </div>
  );
}

function CodeBlock({
  code,
  language,
  options,
}: {
  code: string;
  language: string;
  options: RichRenderOptions;
}) {
  return (
    <Suspense fallback={<CodeBlockFallback code={code} />}>
      <LazySyntaxCodeBlock
        code={code}
        language={language}
        showLineNumbers={options.showLineNumbersInToolViews}
        wrapLines={options.wrapLinesInDiffs}
      />
    </Suspense>
  );
}

function CodeBlockFallback({ code }: { code: string }) {
  return <pre className="diff-line diff-line-context">{code}</pre>;
}

function splitMarkdownBlocks(text: string): MarkdownBlock[] {
  const normalized = text.replace(/\r\n/g, "\n").trim();
  if (!normalized) {
    return [{ type: "paragraph", content: "" }];
  }

  const blocks: MarkdownBlock[] = [];
  const segments = normalized.split(/```/);

  segments.forEach((segment, index) => {
    if (index % 2 === 1) {
      const firstBreak = segment.indexOf("\n");
      const language = firstBreak >= 0 ? segment.slice(0, firstBreak).trim() : "";
      const content = firstBreak >= 0 ? segment.slice(firstBreak + 1) : segment;
      blocks.push({
        type: "code",
        content: content.trimEnd(),
        language,
      });
      return;
    }

    const paragraphs = segment
      .split(/\n{2,}/)
      .map((part) => part.trim())
      .filter(Boolean);
    paragraphs.forEach((paragraph) => {
      if (/^#{1,6}\s/.test(paragraph)) {
        blocks.push({
          type: "heading",
          content: paragraph.replace(/^#{1,6}\s*/, ""),
        });
        return;
      }

      const lines = paragraph.split("\n");
      if (lines.every((line) => /^[-*]\s+/.test(line))) {
        blocks.push({
          type: "list",
          items: lines.map((line) => line.replace(/^[-*]\s+/, "")),
        });
        return;
      }

      blocks.push({
        type: "paragraph",
        content: paragraph,
      });
    });
  });

  return blocks.length > 0 ? blocks : [{ type: "paragraph", content: normalized }];
}

function looksLikeDiff(text: string): boolean {
  return /(^diff --git|^--- .*\n\+\+\+ |^@@ )/m.test(text);
}

function looksLikeJson(text: string): boolean {
  const trimmed = text.trim();
  return (
    (trimmed.startsWith("{") && trimmed.endsWith("}")) ||
    (trimmed.startsWith("[") && trimmed.endsWith("]"))
  );
}

function formatJson(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return text;
  }
}

function diffLineClass(line: string): string {
  if (line.startsWith("+") && !line.startsWith("+++")) {
    return "diff-line-add";
  }
  if (line.startsWith("-") && !line.startsWith("---")) {
    return "diff-line-remove";
  }
  return "diff-line-context";
}
