import { useState } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1, Subheadline } from "../ui/Typography";

export interface DiffHunk {
  /** Old starting line number */
  oldStart: number;
  /** Old line count */
  oldLines: number;
  /** New starting line number */
  newStart: number;
  /** New line count */
  newLines: number;
  /** Lines in the hunk */
  lines: DiffLine[];
}

export interface DiffLine {
  /** Line type */
  type: "context" | "add" | "remove" | "info";
  /** Line content */
  content: string;
  /** Old line number */
  oldLineNumber?: number;
  /** New line number */
  newLineNumber?: number;
}

export interface DiffFile {
  /** File path */
  path: string;
  /** Old file path (for renames) */
  oldPath?: string;
  /** Change status */
  status: "added" | "modified" | "deleted" | "renamed" | "copied";
  /** File hunks */
  hunks: DiffHunk[];
  /** Additions count */
  additions?: number;
  /** Deletions count */
  deletions?: number;
}

export interface DiffRendererProps {
  /** Diff files to render */
  files: DiffFile[];
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Whether files are collapsible */
  collapsible?: boolean;
  /** Maximum height before scrolling */
  maxHeight?: number;
}

/**
 * DiffRenderer - Code diff rendering matching Happy's diff components
 *
 * Features:
 * - Unified diff view with syntax highlighting
 * - Line numbers for old and new versions
 * - Color-coded additions (green) and deletions (red)
 * - File headers with change statistics
 * - Collapsible file sections
 */
export function DiffRenderer({
  files,
  showLineNumbers = true,
  collapsible = true,
  maxHeight,
}: DiffRendererProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[4],
        fontFamily: tokens.typography.fontFamily.mono,
        fontSize: tokens.typography.fontSize.sm,
        lineHeight: tokens.typography.lineHeight.snug,
      }}
    >
      {files.map((file) => (
        <DiffFileView
          key={file.path}
          file={file}
          showLineNumbers={showLineNumbers}
          collapsible={collapsible}
          maxHeight={maxHeight}
        />
      ))}
    </div>
  );
}

interface DiffFileViewProps {
  file: DiffFile;
  showLineNumbers: boolean;
  collapsible: boolean;
  maxHeight?: number;
}

function DiffFileView({ file, showLineNumbers, collapsible, maxHeight }: DiffFileViewProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const statusConfig = {
    added: { label: "Added", color: "var(--color-success)" },
    modified: { label: "Modified", color: "var(--color-primary)" },
    deleted: { label: "Deleted", color: "var(--color-danger)" },
    renamed: { label: "Renamed", color: "var(--color-warning)" },
    copied: { label: "Copied", color: "var(--color-info)" },
  };

  const status = statusConfig[file.status];
  const totalChanges = (file.additions || 0) + (file.deletions || 0);

  return (
    <div
      style={{
        border: "1px solid var(--border-primary)",
        borderRadius: tokens.radii.lg,
        overflow: "hidden",
        backgroundColor: "var(--surface-primary)",
      }}
    >
      {/* File Header */}
      <div
        onClick={() => collapsible && setIsExpanded(!isExpanded)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: tokens.spacing[3],
          padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
          backgroundColor: "var(--surface-secondary)",
          borderBottom: isExpanded ? "1px solid var(--border-primary)" : "none",
          cursor: collapsible ? "pointer" : "default",
        }}
      >
        {collapsible && (
          <span
            style={{
              display: "inline-flex",
              transition: `transform ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
              transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)",
            }}
          >
            ▶
          </span>
        )}

        <div style={{ flex: 1, minWidth: 0 }}>
          <Subheadline truncate>
            {file.oldPath && file.oldPath !== file.path ? (
              <>
                <span style={{ color: "var(--text-tertiary)" }}>{file.oldPath}</span>
                {" → "}
                <span>{file.path}</span>
              </>
            ) : (
              file.path
            )}
          </Subheadline>
        </div>

        <span
          style={{
            padding: `${tokens.spacing[0.5]} ${tokens.spacing[2]}`,
            backgroundColor: status.color,
            color: "#000",
            borderRadius: tokens.radii.sm,
            fontSize: tokens.typography.fontSize.xs,
            fontWeight: tokens.typography.fontWeight.medium,
          }}
        >
          {status.label}
        </span>

        {totalChanges > 0 && (
          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            {(file.additions ?? 0) > 0 && (
              <span style={{ color: "var(--color-success)", fontWeight: tokens.typography.fontWeight.medium }}>
                +{file.additions}
              </span>
            )}
            {(file.deletions ?? 0) > 0 && (
              <span style={{ color: "var(--color-danger)", fontWeight: tokens.typography.fontWeight.medium }}>
                −{file.deletions}
              </span>
            )}
          </div>
        )}
      </div>

      {/* Diff Content */}
      {isExpanded && (
        <div
          style={{
            maxHeight: maxHeight ? `${maxHeight}px` : undefined,
            overflow: maxHeight ? "auto" : undefined,
          }}
        >
          {file.hunks.map((hunk, hunkIndex) => (
            <DiffHunkView
              key={hunkIndex}
              hunk={hunk}
              showLineNumbers={showLineNumbers}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface DiffHunkViewProps {
  hunk: DiffHunk;
  showLineNumbers: boolean;
}

function DiffHunkView({ hunk, showLineNumbers }: DiffHunkViewProps) {
  return (
    <div>
      {/* Hunk Header */}
      <div
        style={{
          padding: `${tokens.spacing[1]} ${tokens.spacing[4]}`,
          backgroundColor: "var(--surface-tertiary)",
          color: "var(--text-tertiary)",
          fontSize: tokens.typography.fontSize.xs,
        }}
      >
        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
      </div>

      {/* Hunk Lines */}
      <div>
        {hunk.lines.map((line, lineIndex) => (
          <DiffLineView
            key={lineIndex}
            line={line}
            showLineNumbers={showLineNumbers}
          />
        ))}
      </div>
    </div>
  );
}

interface DiffLineViewProps {
  line: DiffLine;
  showLineNumbers: boolean;
}

function DiffLineView({ line, showLineNumbers }: DiffLineViewProps) {
  const lineStyles: Record<DiffLine["type"], React.CSSProperties> = {
    context: {
      backgroundColor: "transparent",
    },
    add: {
      backgroundColor: "rgba(48, 209, 88, 0.15)",
    },
    remove: {
      backgroundColor: "rgba(255, 69, 58, 0.15)",
    },
    info: {
      backgroundColor: "var(--surface-tertiary)",
      color: "var(--text-tertiary)",
    },
  };

  const prefixStyles: Record<DiffLine["type"], string> = {
    context: " ",
    add: "+",
    remove: "−",
    info: " ",
  };

  const prefixColors: Record<DiffLine["type"], string> = {
    context: "var(--text-quaternary)",
    add: "var(--color-success)",
    remove: "var(--color-danger)",
    info: "var(--text-tertiary)",
  };

  return (
    <div
      style={{
        display: "flex",
        ...lineStyles[line.type],
      }}
    >
      {/* Line Numbers */}
      {showLineNumbers && (
        <>
          <div
            style={{
              width: "50px",
              padding: `0 ${tokens.spacing[2]}`,
              textAlign: "right",
              color: "var(--text-quaternary)",
              userSelect: "none",
              borderRight: "1px solid var(--border-secondary)",
            }}
          >
            <Caption1 color="quaternary">
              {line.oldLineNumber || " "}
            </Caption1>
          </div>
          <div
            style={{
              width: "50px",
              padding: `0 ${tokens.spacing[2]}`,
              textAlign: "right",
              color: "var(--text-quaternary)",
              userSelect: "none",
              borderRight: "1px solid var(--border-secondary)",
            }}
          >
            <Caption1 color="quaternary">
              {line.newLineNumber || " "}
            </Caption1>
          </div>
        </>
      )}

      {/* Line Content */}
      <div
        style={{
          flex: 1,
          display: "flex",
          padding: `0 ${tokens.spacing[2]}`,
          overflow: "hidden",
        }}
      >
        <span
          style={{
            color: prefixColors[line.type],
            userSelect: "none",
            marginRight: tokens.spacing[2],
            fontWeight: tokens.typography.fontWeight.bold,
          }}
        >
          {prefixStyles[line.type]}
        </span>
        <pre
          style={{
            margin: 0,
            padding: 0,
            background: "transparent",
            fontFamily: "inherit",
            fontSize: "inherit",
            lineHeight: "inherit",
            color: "var(--text-primary)",
            overflow: "auto",
          }}
        >
          {line.content}
        </pre>
      </div>
    </div>
  );
}

/**
 * Parse unified diff format into structured data
 */
export function parseUnifiedDiff(diffText: string): DiffFile[] {
  const files: DiffFile[] = [];
  const lines = diffText.split("\n");

  let currentFile: DiffFile | null = null;
  let currentHunk: DiffHunk | null = null;
  let oldLineNumber = 0;
  let newLineNumber = 0;

  for (const line of lines) {
    // New file
    if (line.startsWith("diff --git")) {
      if (currentFile) {
        files.push(currentFile);
      }
      currentFile = {
        path: "",
        status: "modified",
        hunks: [],
      };
      continue;
    }

    // File paths
    if (line.startsWith("--- ") && currentFile) {
      const path = line.slice(4);
      if (path !== "/dev/null") {
        currentFile.oldPath = path.replace(/^a\//, "");
      }
      continue;
    }

    if (line.startsWith("+++ ") && currentFile) {
      const path = line.slice(4);
      if (path !== "/dev/null") {
        currentFile.path = path.replace(/^b\//, "");
      }
      continue;
    }

    // File mode changes
    if (line.startsWith("new file mode") && currentFile) {
      currentFile.status = "added";
      continue;
    }

    if (line.startsWith("deleted file mode") && currentFile) {
      currentFile.status = "deleted";
      continue;
    }

    if (line.startsWith("rename from") && currentFile) {
      currentFile.status = "renamed";
      continue;
    }

    // Hunk header
    const hunkMatch = line.match(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/);
    if (hunkMatch && currentFile) {
      if (currentHunk) {
        currentFile.hunks.push(currentHunk);
      }
      oldLineNumber = parseInt(hunkMatch[1], 10);
      newLineNumber = parseInt(hunkMatch[3], 10);
      currentHunk = {
        oldStart: oldLineNumber,
        oldLines: parseInt(hunkMatch[2] || "1", 10),
        newStart: newLineNumber,
        newLines: parseInt(hunkMatch[4] || "1", 10),
        lines: [],
      };
      continue;
    }

    // Hunk lines
    if (currentHunk) {
      if (line.startsWith("+")) {
        currentHunk.lines.push({
          type: "add",
          content: line.slice(1),
          newLineNumber: newLineNumber++,
        });
      } else if (line.startsWith("-")) {
        currentHunk.lines.push({
          type: "remove",
          content: line.slice(1),
          oldLineNumber: oldLineNumber++,
        });
      } else if (line.startsWith(" ")) {
        currentHunk.lines.push({
          type: "context",
          content: line.slice(1),
          oldLineNumber: oldLineNumber++,
          newLineNumber: newLineNumber++,
        });
      } else if (line.startsWith("\\")) {
        currentHunk.lines.push({
          type: "info",
          content: line,
        });
      }
    }
  }

  // Push final hunk and file
  if (currentHunk && currentFile) {
    currentFile.hunks.push(currentHunk);
  }
  if (currentFile) {
    files.push(currentFile);
  }

  return files;
}
