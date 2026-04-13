import { useState } from "react";
import { tokens } from "../../../design-system/tokens";
import { Caption1, Body } from "../../ui/Typography";
import { Button } from "../../ui/Button";
import { ToolResultSection } from "./ToolResultSection";

export interface BashToolResultProps {
  /** Command that was executed */
  command: string;
  /** Standard output */
  stdout?: string | null;
  /** Standard error */
  stderr?: string | null;
  /** Error message if command failed */
  error?: string | null;
  /** Working directory (optional) */
  cwd?: string;
  /** Exit code (optional) */
  exitCode?: number | null;
  /** Whether the output is truncated */
  truncated?: boolean;
}

/**
 * BashToolResult - Renders bash command output in terminal style
 *
 * Features:
 * - Terminal-style command display with prompt
 * - Syntax highlighted stdout/stderr
 * - Truncation for long outputs
 * - Copy functionality
 */
export function BashToolResult({
  command,
  stdout,
  stderr,
  error,
  cwd,
  exitCode,
  truncated = false,
}: BashToolResultProps) {
  const [isCopied, setIsCopied] = useState(false);
  const [isExpanded, setIsExpanded] = useState(!truncated);

  const hasOutput = stdout || stderr || error;
  const outputText = [
    stdout,
    stderr && `[stderr]\n${stderr}`,
    error && `[error]\n${error}`,
  ]
    .filter(Boolean)
    .join("\n");

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(outputText);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch {
      // Ignore copy errors
    }
  };

  const toggleExpand = () => setIsExpanded(!isExpanded);

  return (
    <ToolResultSection fullWidth>
      <div
        style={{
          backgroundColor: "var(--surface-secondary)",
          borderRadius: tokens.radii.md,
          overflow: "hidden",
          fontFamily: tokens.typography.fontFamily.mono,
          fontSize: tokens.typography.fontSize.sm,
        }}
      >
        {/* Command Line */}
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            padding: tokens.spacing[3],
            gap: tokens.spacing[2],
            backgroundColor: "var(--surface-tertiary)",
          }}
        >
          <span
            style={{
              color: "var(--color-success)",
              fontWeight: tokens.typography.fontWeight.bold,
              userSelect: "none",
            }}
          >
            $
          </span>
          <pre
            style={{
              margin: 0,
              flex: 1,
              color: "var(--text-primary)",
              whiteSpace: "pre-wrap",
              wordBreak: "break-word",
              lineHeight: tokens.typography.lineHeight.snug,
            }}
          >
            {command}
          </pre>
        </div>

        {/* Working Directory */}
        {cwd && (
          <div
            style={{
              padding: `${tokens.spacing[1]} ${tokens.spacing[3]}`,
              backgroundColor: "var(--surface-secondary)",
              borderBottom: "1px solid var(--border-primary)",
            }}
          >
            <Caption1 color="tertiary">cwd: {cwd}</Caption1>
          </div>
        )}

        {/* Output */}
        {hasOutput && (
          <div
            style={{
              position: "relative",
              maxHeight: isExpanded ? undefined : "200px",
              overflow: isExpanded ? "visible" : "auto",
            }}
          >
            {/* Stdout */}
            {stdout && (
              <pre
                style={{
                  margin: 0,
                  padding: tokens.spacing[3],
                  color: "var(--text-primary)",
                  lineHeight: tokens.typography.lineHeight.snug,
                  borderBottom:
                    stderr || error
                      ? "1px solid var(--border-primary)"
                      : undefined,
                }}
              >
                {stdout}
              </pre>
            )}

            {/* Stderr */}
            {stderr && (
              <pre
                style={{
                  margin: 0,
                  padding: tokens.spacing[3],
                  color: "var(--color-warning)",
                  lineHeight: tokens.typography.lineHeight.snug,
                  borderBottom: error
                    ? "1px solid var(--border-primary)"
                    : undefined,
                }}
              >
                {stderr}
              </pre>
            )}

            {/* Error */}
            {error && (
              <div
                style={{
                  padding: tokens.spacing[3],
                  backgroundColor: "rgba(255, 69, 58, 0.1)",
                }}
              >
                <pre
                  style={{
                    margin: 0,
                    color: "var(--color-danger)",
                    lineHeight: tokens.typography.lineHeight.snug,
                  }}
                >
                  {error}
                </pre>
              </div>
            )}

            {/* Expand/Collapse Button */}
            {truncated && (
              <Button
                variant="ghost"
                size="sm"
                onClick={toggleExpand}
                style={{
                  width: "100%",
                  borderTop: "1px solid var(--border-primary)",
                }}
              >
                {isExpanded ? "Collapse" : "Show more"}
              </Button>
            )}

            {/* Copy Button */}
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCopy}
              style={{
                position: "absolute",
                top: tokens.spacing[2],
                right: tokens.spacing[2],
              }}
            >
              {isCopied ? "Copied!" : "Copy"}
            </Button>
          </div>
        )}

        {/* Exit Code */}
        {exitCode !== undefined && exitCode !== null && exitCode !== 0 && (
          <div
            style={{
              padding: tokens.spacing[2],
              borderTop: "1px solid var(--border-primary)",
              backgroundColor: "rgba(255, 69, 58, 0.05)",
            }}
          >
            <span
              style={{
                fontSize: tokens.typography.fontSize.xs,
                color: "var(--color-danger)",
              }}
            >
              Exit code: {exitCode}
            </span>
          </div>
        )}

        {/* No Output */}
        {!hasOutput && (
          <div
            style={{
              padding: tokens.spacing[3],
              color: "var(--text-tertiary)",
              fontStyle: "italic",
            }}
          >
            <Body>[Command completed with no output]</Body>
          </div>
        )}
      </div>
    </ToolResultSection>
  );
}
