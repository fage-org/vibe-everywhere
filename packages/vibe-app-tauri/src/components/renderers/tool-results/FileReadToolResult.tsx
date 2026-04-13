import { Suspense, lazy, useMemo } from "react";
import { tokens } from "../../../design-system/tokens";
import { Caption1 } from "../../ui/Typography";
import { ToolResultSection } from "./ToolResultSection";

const LazySyntaxCodeBlock = lazy(() =>
  import("../../../syntax-code-block").then((module) => ({
    default: module.SyntaxCodeBlock,
  })),
);

export interface FileReadToolResultProps {
  /** File path that was read */
  filePath: string;
  /** File content */
  content: string;
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Whether to wrap long lines */
  wrapLines?: boolean;
  /** Number of lines to show (for preview) */
  maxLines?: number;
  /** Total lines in file (if truncated) */
  totalLines?: number;
}

/**
 * FileReadToolResult - Renders file content with syntax highlighting
 *
 * Features:
 * - Syntax highlighting based on file extension
 * - Line number display (configurable)
 * - Truncation for large files
 * - File path header
 */
export function FileReadToolResult({
  filePath,
  content,
  showLineNumbers = false,
  wrapLines = false,
  maxLines,
  totalLines,
}: FileReadToolResultProps) {
  // Detect language from file extension
  const language = useMemo(() => detectLanguage(filePath), [filePath]);

  // Handle content truncation
  const { displayContent, isTruncated, lineCount } = useMemo(() => {
    const lines = content.split("\n");
    const count = lines.length;

    if (maxLines && count > maxLines) {
      return {
        displayContent: lines.slice(0, maxLines).join("\n"),
        isTruncated: true,
        lineCount: count,
      };
    }

    return {
      displayContent: content,
      isTruncated: false,
      lineCount: count,
    };
  }, [content, maxLines]);

  return (
    <ToolResultSection fullWidth>
      {/* File Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
          backgroundColor: "var(--surface-secondary)",
          borderBottom: "1px solid var(--border-primary)",
          borderRadius: `${tokens.radii.md} ${tokens.radii.md} 0 0`,
        }}
      >
        <Caption1
          color="secondary"
          style={{
            fontFamily: tokens.typography.fontFamily.mono,
          }}
        >
          {filePath}
        </Caption1>
        <Caption1 color="tertiary">
          {totalLines ?? lineCount} lines
          {isTruncated && ` (showing first ${maxLines})`}
        </Caption1>
      </div>

      {/* File Content */}
      <div
        style={{
          borderRadius: `0 0 ${tokens.radii.md} ${tokens.radii.md}`,
          overflow: "hidden",
        }}
      >
        <Suspense fallback={<CodeBlockFallback content={displayContent} />}>
          <LazySyntaxCodeBlock
            code={displayContent}
            language={language}
            showLineNumbers={showLineNumbers}
            wrapLines={wrapLines}
          />
        </Suspense>

        {/* Truncation Notice */}
        {isTruncated && (
          <div
            style={{
              padding: tokens.spacing[2],
              backgroundColor: "var(--surface-tertiary)",
              textAlign: "center",
            }}
          >
            <Caption1 color="tertiary">
              File truncated. Showing {maxLines} of {lineCount} lines.
            </Caption1>
          </div>
        )}
      </div>
    </ToolResultSection>
  );
}

function CodeBlockFallback({ content }: { content: string }) {
  return (
    <pre
      style={{
        margin: 0,
        padding: tokens.spacing[3],
        backgroundColor: "var(--surface-secondary)",
        color: "var(--text-primary)",
        fontFamily: tokens.typography.fontFamily.mono,
        fontSize: tokens.typography.fontSize.sm,
        lineHeight: tokens.typography.lineHeight.snug,
        overflow: "auto",
        maxHeight: "400px",
      }}
    >
      {content}
    </pre>
  );
}

/**
 * Detect programming language from file extension
 */
function detectLanguage(filePath: string): string {
  const ext = filePath.split(".").pop()?.toLowerCase() || "";

  const extensionMap: Record<string, string> = {
    // JavaScript/TypeScript
    js: "javascript",
    jsx: "jsx",
    ts: "typescript",
    tsx: "tsx",
    mjs: "javascript",
    cjs: "javascript",

    // Web
    html: "html",
    htm: "html",
    css: "css",
    scss: "scss",
    sass: "sass",
    less: "less",
    vue: "vue",
    svelte: "svelte",

    // Data formats
    json: "json",
    json5: "json5",
    yaml: "yaml",
    yml: "yaml",
    toml: "toml",
    xml: "xml",

    // Programming languages
    py: "python",
    rb: "ruby",
    go: "go",
    rs: "rust",
    java: "java",
    kt: "kotlin",
    kts: "kotlin",
    swift: "swift",
    c: "c",
    cpp: "cpp",
    cc: "cpp",
    cxx: "cpp",
    h: "c",
    hpp: "cpp",
    cs: "csharp",
    php: "php",

    // Shell
    sh: "bash",
    bash: "bash",
    zsh: "bash",
    fish: "bash",

    // Config
    ini: "ini",
    cfg: "ini",
    conf: "ini",
    env: "bash",
    dockerfile: "docker",
    makefile: "makefile",

    // Documentation
    md: "markdown",
    mdx: "mdx",
    rst: "rest",
    txt: "text",

    // Other
    sql: "sql",
    graphql: "graphql",
    gql: "graphql",
    prisma: "prisma",
    docker: "docker",
  };

  // Check for special files
  const fileName = filePath.split("/").pop()?.toLowerCase() || "";
  if (fileName === "dockerfile") return "dockerfile";
  if (fileName === "makefile") return "makefile";
  if (fileName === "package.json") return "json";
  if (fileName === "cargo.toml") return "toml";
  if (fileName === "pyproject.toml") return "toml";
  if (fileName.startsWith(".env")) return "bash";

  return extensionMap[ext] || "text";
}
