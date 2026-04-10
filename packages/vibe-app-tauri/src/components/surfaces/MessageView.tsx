import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Body, Caption1, Subheadline } from "../ui/Typography";

export type MessageRole = "user" | "assistant" | "system" | "tool";

export interface Message {
  /** Unique message ID */
  id: string;
  /** Message role */
  role: MessageRole;
  /** Message content */
  content: ReactNode;
  /** Timestamp */
  timestamp: Date;
  /** Sender name (for multi-user scenarios) */
  senderName?: string;
  /** Avatar element */
  avatar?: ReactNode;
  /** Whether message is streaming */
  isStreaming?: boolean;
  /** Tool calls associated with this message */
  toolCalls?: MessageToolCall[];
  /** Attachments */
  attachments?: Attachment[];
}

export interface MessageToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
  result?: unknown;
  status: "pending" | "running" | "completed" | "error";
}

export interface Attachment {
  id: string;
  type: "file" | "image" | "code";
  name: string;
  content?: string;
  url?: string;
}

export interface MessageViewProps {
  /** Message to display */
  message: Message;
  /** Whether to show avatar */
  showAvatar?: boolean;
  /** Whether this is part of a consecutive group */
  isGrouped?: boolean;
}

/**
 * MessageView - Individual message rendering matching Happy's MessageView
 *
 * Features:
 * - Role-based styling (user, assistant, system, tool)
 * - Avatar display
 * - Timestamp
 * - Streaming indicator
 * - Tool call visualization
 * - Attachment rendering
 */
export function MessageView({ message, showAvatar = true, isGrouped = false }: MessageViewProps) {
  const isUser = message.role === "user";
  const isAssistant = message.role === "assistant";
  const isSystem = message.role === "system";

  return (
    <div
      style={{
        display: "flex",
        gap: tokens.spacing[3],
        padding: isGrouped ? `${tokens.spacing[1]} 0` : `${tokens.spacing[3]} 0`,
        flexDirection: isUser ? "row-reverse" : "row",
      }}
    >
      {/* Avatar */}
      {showAvatar && !isGrouped && (
        <div style={{ flexShrink: 0, width: "32px" }}>
          <MessageAvatar role={message.role} avatar={message.avatar} />
        </div>
      )}

      {/* Content */}
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[2],
          maxWidth: isUser ? "80%" : "100%",
          alignItems: isUser ? "flex-end" : "flex-start",
        }}
      >
        {/* Header */}
        {!isGrouped && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: tokens.spacing[2],
            }}
          >
            <Subheadline bold color={isUser ? "primary" : "secondary"}>
              {message.senderName || getRoleLabel(message.role)}
            </Subheadline>
            <Caption1 color="quaternary">{formatTimestamp(message.timestamp)}</Caption1>
            {message.isStreaming && <StreamingIndicator />}
          </div>
        )}

        {/* Message Bubble */}
        <div
          style={{
            backgroundColor: isUser ? "var(--color-primary)" : "var(--surface-secondary)",
            color: isUser ? "#ffffff" : "var(--text-primary)",
            padding: isUser ? `${tokens.spacing[3]} ${tokens.spacing[4]}` : 0,
            borderRadius: isUser ? tokens.radii.xl : 0,
            maxWidth: "100%",
          }}
        >
          <Body style={{ color: "inherit" }}>{message.content}</Body>
        </div>

        {/* Tool Calls */}
        {message.toolCalls && message.toolCalls.length > 0 && (
          <div style={{ width: "100%", display: "flex", flexDirection: "column", gap: tokens.spacing[2] }}>
            {message.toolCalls.map((toolCall) => (
              <ToolCallView key={toolCall.id} toolCall={toolCall} />
            ))}
          </div>
        )}

        {/* Attachments */}
        {message.attachments && message.attachments.length > 0 && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: tokens.spacing[2] }}>
            {message.attachments.map((attachment) => (
              <AttachmentView key={attachment.id} attachment={attachment} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface MessageAvatarProps {
  role: MessageRole;
  avatar?: ReactNode;
}

function MessageAvatar({ role, avatar }: MessageAvatarProps) {
  if (avatar) return <>{avatar}</>;

  const backgroundColor =
    role === "user"
      ? "var(--color-primary)"
      : role === "assistant"
        ? "var(--surface-tertiary)"
        : "var(--surface-secondary)";

  const color = role === "user" ? "#ffffff" : "var(--text-secondary)";

  return (
    <div
      style={{
        width: "32px",
        height: "32px",
        borderRadius: tokens.radii.full,
        backgroundColor,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color,
        fontSize: "14px",
        fontWeight: tokens.typography.fontWeight.semibold,
      }}
    >
      {role === "user" ? "U" : role === "assistant" ? "A" : "S"}
    </div>
  );
}

function StreamingIndicator() {
  return (
    <span
      style={{
        display: "inline-flex",
        gap: "2px",
        alignItems: "center",
      }}
    >
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          style={{
            width: "4px",
            height: "4px",
            borderRadius: "50%",
            backgroundColor: "var(--color-primary)",
            animation: `bounce 1.4s ease-in-out ${i * 0.16}s infinite`,
          }}
        />
      ))}
    </span>
  );
}

interface ToolCallViewProps {
  toolCall: MessageToolCall;
}

function ToolCallView({ toolCall }: ToolCallViewProps) {
  const statusColor =
    toolCall.status === "completed"
      ? "var(--color-success)"
      : toolCall.status === "error"
        ? "var(--color-danger)"
        : toolCall.status === "running"
          ? "var(--color-primary)"
          : "var(--text-tertiary)";

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: tokens.spacing[2],
        padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
        backgroundColor: "var(--surface-tertiary)",
        borderRadius: tokens.radii.md,
        fontSize: tokens.typography.fontSize.sm,
        color: "var(--text-secondary)",
      }}
    >
      <span
        style={{
          width: "6px",
          height: "6px",
          borderRadius: "50%",
          backgroundColor: statusColor,
        }}
      />
      <span style={{ fontFamily: tokens.typography.fontFamily.mono }}>{toolCall.name}</span>
      <span style={{ color: "var(--text-quaternary)" }}>
        {toolCall.status === "pending" && "queued"}
        {toolCall.status === "running" && "running..."}
        {toolCall.status === "completed" && "done"}
        {toolCall.status === "error" && "failed"}
      </span>
    </div>
  );
}

interface AttachmentViewProps {
  attachment: Attachment;
}

function AttachmentView({ attachment }: AttachmentViewProps) {
  const icon =
    attachment.type === "image" ? "🖼️" : attachment.type === "code" ? "📄" : "📎";

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: tokens.spacing[2],
        padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
        backgroundColor: "var(--surface-secondary)",
        borderRadius: tokens.radii.md,
        border: "1px solid var(--border-primary)",
      }}
    >
      <span>{icon}</span>
      <Caption1 truncate style={{ maxWidth: "150px" }}>
        {attachment.name}
      </Caption1>
    </div>
  );
}

function getRoleLabel(role: MessageRole): string {
  switch (role) {
    case "user":
      return "You";
    case "assistant":
      return "Assistant";
    case "system":
      return "System";
    case "tool":
      return "Tool";
    default:
      return "Unknown";
  }
}

function formatTimestamp(date: Date): string {
  return date.toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
}
