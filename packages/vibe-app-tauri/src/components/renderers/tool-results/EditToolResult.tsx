import { tokens } from "../../../design-system/tokens";
import { Caption1 } from "../../ui/Typography";
import { ToolResultSection } from "./ToolResultSection";
import {
  DiffRendererWithSettings,
  parseUnifiedDiff,
  type DiffFile,
} from "../DiffRenderer";

export interface EditToolResultProps {
  /** File path that was edited */
  filePath: string;
  /** Old content (before edit) */
  oldString: string;
  /** New content (after edit) */
  newString: string;
  /** Optional unified diff text */
  diffText?: string;
}

/**
 * EditToolResult - Renders file edit as a diff
 *
 * Features:
 * - Unified diff view with syntax highlighting
 * - File path header
 * - Automatic diff generation from old/new strings
 * - Settings-aware (line numbers, wrap lines)
 */
export function EditToolResult({
  filePath,
  oldString,
  newString,
  diffText,
}: EditToolResultProps) {
  // Parse diff or generate from old/new strings
  const diffFiles = diffText
    ? parseUnifiedDiff(diffText)
    : generateDiffFromStrings(filePath, oldString, newString);

  // Calculate stats
  const stats = useMemo(() => {
    let additions = 0;
    let deletions = 0;

    diffFiles.forEach((file) => {
      additions += file.additions ?? 0;
      deletions += file.deletions ?? 0;
    });

    return { additions, deletions };
  }, [diffFiles]);

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
        <div style={{ display: "flex", gap: tokens.spacing[3] }}>
          {stats.additions > 0 && (
            <span
              style={{
                fontSize: tokens.typography.fontSize.xs,
                color: "var(--color-success)",
              }}
            >
              +{stats.additions}
            </span>
          )}
          {stats.deletions > 0 && (
            <span
              style={{
                fontSize: tokens.typography.fontSize.xs,
                color: "var(--color-danger)",
              }}
            >
              -{stats.deletions}
            </span>
          )}
        </div>
      </div>

      {/* Diff Content */}
      <div
        style={{
          borderRadius: `0 0 ${tokens.radii.md} ${tokens.radii.md}`,
          overflow: "hidden",
        }}
      >
        <DiffRendererWithSettings files={diffFiles} collapsible={false} />
      </div>
    </ToolResultSection>
  );
}

import { useMemo } from "react";

/**
 * Generate a DiffFile from old and new strings
 */
function generateDiffFromStrings(
  filePath: string,
  oldString: string,
  newString: string,
): DiffFile[] {
  const oldLines = oldString.split("\n");
  const newLines = newString.split("\n");

  // Simple diff: show all old lines as removed, all new lines as added
  // For more sophisticated diff, use a diff algorithm library
  const lines: {
    type: "context" | "add" | "remove" | "info";
    content: string;
    oldLineNumber?: number;
    newLineNumber?: number;
  }[] = [];

  let oldLineNum = 1;
  let newLineNum = 1;

  // Check if strings are identical
  if (oldString === newString) {
    // No changes - show as context
    oldLines.forEach((line, index) => {
      lines.push({
        type: "context",
        content: line,
        oldLineNumber: index + 1,
        newLineNumber: index + 1,
      });
    });
  } else {
    // Show old content as removed
    oldLines.forEach((line, index) => {
      lines.push({
        type: "remove",
        content: line,
        oldLineNumber: index + 1,
      });
    });

    // Show new content as added
    newLines.forEach((line, index) => {
      lines.push({
        type: "add",
        content: line,
        newLineNumber: index + 1,
      });
    });
  }

  const additions = lines.filter((l) => l.type === "add").length;
  const deletions = lines.filter((l) => l.type === "remove").length;

  return [
    {
      path: filePath,
      status: "modified",
      hunks: [
        {
          oldStart: 1,
          oldLines: oldLines.length,
          newStart: 1,
          newLines: newLines.length,
          lines,
        },
      ],
      additions,
      deletions,
    },
  ];
}
