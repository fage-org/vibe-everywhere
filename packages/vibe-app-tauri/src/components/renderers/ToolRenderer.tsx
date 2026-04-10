import { useState, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Button } from "../ui/Button";
import { Caption1, Subheadline, Body } from "../ui/Typography";

export interface ToolCall {
  /** Unique tool call ID */
  id: string;
  /** Tool name */
  name: string;
  /** Tool arguments */
  arguments: Record<string, unknown>;
  /** Tool execution result */
  result?: unknown;
  /** Execution status */
  status: "pending" | "running" | "completed" | "error";
  /** Error message if failed */
  error?: string;
  /** Execution start time */
  startedAt?: Date;
  /** Execution end time */
  completedAt?: Date;
}

export interface Tool {
  /** Tool identifier */
  id: string;
  /** Display name */
  name: string;
  /** Description */
  description?: string;
  /** Icon component */
  icon?: ReactNode;
  /** Category for grouping */
  category?: string;
}

export interface ToolRendererProps {
  /** Tool calls to render */
  toolCalls: ToolCall[];
  /** Available tools metadata */
  tools?: Tool[];
  /** Whether to show expanded by default */
  defaultExpanded?: boolean;
  /** Callback when a tool call is clicked */
  onToolClick?: (toolCall: ToolCall) => void;
  /** Custom result renderer */
  renderResult?: (toolCall: ToolCall) => ReactNode;
}

/**
 * ToolRenderer - Tool call and result rendering
 *
 * Matches Happy's tool presentation:
 * - Collapsible tool call sections
 * - Arguments display with syntax highlighting
 * - Result visualization
 * - Status indicators
 * - Execution timing
 */
export function ToolRenderer({
  toolCalls,
  tools = [],
  defaultExpanded = false,
  onToolClick,
  renderResult,
}: ToolRendererProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[3],
      }}
    >
      {toolCalls.map((toolCall) => (
        <ToolCallView
          key={toolCall.id}
          toolCall={toolCall}
          tool={tools.find((t) => t.id === toolCall.name)}
          defaultExpanded={defaultExpanded}
          onClick={() => onToolClick?.(toolCall)}
          renderResult={renderResult}
        />
      ))}
    </div>
  );
}

interface ToolCallViewProps {
  toolCall: ToolCall;
  tool?: Tool;
  defaultExpanded: boolean;
  onClick?: () => void;
  renderResult?: (toolCall: ToolCall) => ReactNode;
}

function ToolCallView({ toolCall, tool, defaultExpanded, onClick, renderResult }: ToolCallViewProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  const statusConfig = {
    pending: {
      color: "var(--text-tertiary)",
      bgColor: "var(--surface-tertiary)",
      label: "Pending",
      icon: "⏳",
    },
    running: {
      color: "var(--color-primary)",
      bgColor: "rgba(10, 132, 255, 0.15)",
      label: "Running",
      icon: "▶️",
    },
    completed: {
      color: "var(--color-success)",
      bgColor: "rgba(48, 209, 88, 0.15)",
      label: "Completed",
      icon: "✓",
    },
    error: {
      color: "var(--color-danger)",
      bgColor: "rgba(255, 69, 58, 0.15)",
      label: "Error",
      icon: "✗",
    },
  };

  const status = statusConfig[toolCall.status];
  const duration =
    toolCall.startedAt && toolCall.completedAt
      ? toolCall.completedAt.getTime() - toolCall.startedAt.getTime()
      : null;

  return (
    <div
      style={{
        border: "1px solid var(--border-primary)",
        borderRadius: tokens.radii.lg,
        overflow: "hidden",
        backgroundColor: "var(--surface-primary)",
      }}
    >
      {/* Tool Header */}
      <div
        onClick={() => {
          setIsExpanded(!isExpanded);
          onClick?.();
        }}
        style={{
          display: "flex",
          alignItems: "center",
          gap: tokens.spacing[3],
          padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
          backgroundColor: status.bgColor,
          cursor: "pointer",
          userSelect: "none",
        }}
      >
        {/* Expand Icon */}
        <span
          style={{
            display: "inline-flex",
            transition: `transform ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
            transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)",
            color: status.color,
          }}
        >
          ▶
        </span>

        {/* Tool Icon */}
        <span style={{ fontSize: "1.2em" }}>{tool?.icon || status.icon}</span>

        {/* Tool Name */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <Subheadline truncate style={{ color: status.color }}>
            {tool?.name || toolCall.name}
          </Subheadline>
          {tool?.description && (
            <Caption1 color="tertiary" truncate>
              {tool.description}
            </Caption1>
          )}
        </div>

        {/* Status Badge */}
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

        {/* Duration */}
        {duration !== null && (
          <Caption1 color="tertiary">{formatDuration(duration)}</Caption1>
        )}
      </div>

      {/* Tool Details */}
      {isExpanded && (
        <div
          style={{
            padding: tokens.spacing[4],
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[4],
          }}
        >
          {/* Arguments */}
          <div>
            <Caption1
              color="tertiary"
              style={{
                textTransform: "uppercase",
                letterSpacing: "0.05em",
                marginBottom: tokens.spacing[2],
              }}
            >
              Arguments
            </Caption1>
            <CodeBlock content={JSON.stringify(toolCall.arguments, null, 2)} />
          </div>

          {/* Result */}
          {(toolCall.result !== undefined || toolCall.error) && (
            <div>
              <Caption1
                color="tertiary"
                style={{
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                  marginBottom: tokens.spacing[2],
                }}
              >
                Result
              </Caption1>
              {toolCall.error ? (
                <div
                  style={{
                    padding: tokens.spacing[3],
                    backgroundColor: "rgba(255, 69, 58, 0.1)",
                    borderRadius: tokens.radii.md,
                    color: "var(--color-danger)",
                  }}
                >
                  <Body>{toolCall.error}</Body>
                </div>
              ) : renderResult ? (
                renderResult(toolCall)
              ) : (
                <CodeBlock content={formatResult(toolCall.result)} />
              )}
            </div>
          )}

          {/* Timing Info */}
          {(toolCall.startedAt || toolCall.completedAt) && (
            <div
              style={{
                display: "flex",
                gap: tokens.spacing[4],
                paddingTop: tokens.spacing[3],
                borderTop: "1px solid var(--border-primary)",
              }}
            >
              {toolCall.startedAt && (
                <Caption1 color="quaternary">
                  Started: {toolCall.startedAt.toLocaleTimeString()}
                </Caption1>
              )}
              {toolCall.completedAt && (
                <Caption1 color="quaternary">
                  Completed: {toolCall.completedAt.toLocaleTimeString()}
                </Caption1>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface CodeBlockProps {
  content: string;
}

function CodeBlock({ content }: CodeBlockProps) {
  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(content);
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    } catch {
      // Ignore copy errors
    }
  };

  return (
    <div
      style={{
        position: "relative",
        backgroundColor: "var(--surface-secondary)",
        borderRadius: tokens.radii.md,
        overflow: "hidden",
      }}
    >
      <pre
        style={{
          margin: 0,
          padding: tokens.spacing[3],
          overflow: "auto",
          fontFamily: tokens.typography.fontFamily.mono,
          fontSize: tokens.typography.fontSize.sm,
          lineHeight: tokens.typography.lineHeight.normal,
          color: "var(--text-primary)",
          maxHeight: "300px",
        }}
      >
        <code>{content}</code>
      </pre>

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
  );
}

function formatResult(result: unknown): string {
  if (result === null) return "null";
  if (result === undefined) return "undefined";
  if (typeof result === "string") return result;
  if (typeof result === "number" || typeof result === "boolean") return String(result);

  try {
    return JSON.stringify(result, null, 2);
  } catch {
    return String(result);
  }
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}
