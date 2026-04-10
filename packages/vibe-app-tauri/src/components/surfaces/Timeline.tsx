import { useRef, useEffect, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1 } from "../ui/Typography";
import { MessageView, type Message } from "./MessageView";

export interface TimelineProps {
  /** Array of messages to display */
  messages: Message[];
  /** Whether messages are being loaded */
  loading?: boolean;
  /** Whether more messages are being fetched (infinite scroll) */
  loadingMore?: boolean;
  /** Callback when scrolled to top (for infinite scroll) */
  onLoadMore?: () => void;
  /** Empty state content */
  emptyState?: ReactNode;
  /** Custom message renderer */
  renderMessage?: (message: Message, index: number) => ReactNode;
  /** Reference to scroll container */
  scrollRef?: React.RefObject<HTMLDivElement | null>;
  /** Whether to auto-scroll to bottom on new messages */
  autoScroll?: boolean;
}

/**
 * Timeline - Message timeline container
 *
 * Matches Happy's timeline:
 * - Scrollable message list
 * - Auto-scroll to bottom on new messages
 * - Infinite scroll support
 * - Message grouping (consecutive messages from same sender)
 * - Date separators
 */
export function Timeline({
  messages,
  loading,
  loadingMore,
  onLoadMore,
  emptyState,
  renderMessage,
  scrollRef: externalScrollRef,
  autoScroll = true,
}: TimelineProps) {
  const internalScrollRef = useRef<HTMLDivElement>(null);
  const scrollRef = externalScrollRef || internalScrollRef;
  const bottomRef = useRef<HTMLDivElement>(null);
  const shouldScrollRef = useRef(true);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    if (autoScroll && shouldScrollRef.current && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, autoScroll]);

  // Handle scroll to detect if user has scrolled up
  const handleScroll = () => {
    const container = scrollRef.current;
    if (!container) return;

    const { scrollTop, scrollHeight, clientHeight } = container;
    const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;
    shouldScrollRef.current = isNearBottom;

    // Load more when scrolled to top
    if (scrollTop < 50 && onLoadMore && !loadingMore) {
      onLoadMore();
    }
  };

  // Group consecutive messages from the same sender
  const groupedMessages = groupMessages(messages);

  if (loading) {
    return <TimelineSkeleton />;
  }

  if (messages.length === 0 && emptyState) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          padding: tokens.spacing[6],
        }}
      >
        {emptyState}
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      onScroll={handleScroll}
      style={{
        flex: 1,
        overflow: "auto",
        padding: `${tokens.spacing[4]} ${tokens.spacing[6]}`,
      }}
    >
      {/* Loading More Indicator */}
      {loadingMore && (
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            padding: tokens.spacing[4],
          }}
        >
          <LoadingSpinner />
        </div>
      )}

      {/* Messages */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[2],
          maxWidth: "800px",
          margin: "0 auto",
        }}
      >
        {groupedMessages.map((group, groupIndex) => (
          <MessageGroup key={group.key} group={group} renderMessage={renderMessage} />
        ))}
      </div>

      {/* Bottom Anchor */}
      <div ref={bottomRef} />
    </div>
  );
}

interface MessageGroupData {
  key: string;
  date: Date;
  messages: Message[];
  showDateSeparator: boolean;
}

function groupMessages(messages: Message[]): MessageGroupData[] {
  const groups: MessageGroupData[] = [];
  let currentGroup: MessageGroupData | null = null;

  messages.forEach((message, index) => {
    const messageDate = new Date(message.timestamp);
    const prevMessage = index > 0 ? messages[index - 1] : null;

    // Check if we need a new group
    const needsNewGroup =
      !currentGroup ||
      !prevMessage ||
      // Different sender
      prevMessage.role !== message.role ||
      // More than 5 minutes between messages
      messageDate.getTime() - new Date(prevMessage.timestamp).getTime() > 5 * 60 * 1000;

    // Check if we need a date separator
    const showDateSeparator =
      !currentGroup || !isSameDay(messageDate, new Date(currentGroup.date));

    if (needsNewGroup) {
      currentGroup = {
        key: `group-${message.id}`,
        date: messageDate,
        messages: [message],
        showDateSeparator,
      };
      groups.push(currentGroup);
    } else {
      currentGroup!.messages.push(message);
    }
  });

  return groups;
}

interface MessageGroupProps {
  group: MessageGroupData;
  renderMessage?: (message: Message, index: number) => ReactNode;
}

function MessageGroup({ group, renderMessage }: MessageGroupProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      {/* Date Separator */}
      {group.showDateSeparator && (
        <DateSeparator date={group.date} />
      )}

      {/* Messages */}
      <div style={{ display: "flex", flexDirection: "column" }}>
        {group.messages.map((message, index) => {
          const isFirst = index === 0;
          const isLast = index === group.messages.length - 1;

          if (renderMessage) {
            return (
              <div key={message.id}>
                {renderMessage(message, index)}
              </div>
            );
          }

          return (
            <MessageView
              key={message.id}
              message={message}
              showAvatar={isFirst}
              isGrouped={!isFirst}
            />
          );
        })}
      </div>
    </div>
  );
}

interface DateSeparatorProps {
  date: Date;
}

function DateSeparator({ date }: DateSeparatorProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: `${tokens.spacing[4]} 0`,
      }}
    >
      <Caption1
        color="tertiary"
        style={{
          padding: `${tokens.spacing[1]} ${tokens.spacing[3]}`,
          backgroundColor: "var(--surface-secondary)",
          borderRadius: tokens.radii.full,
        }}
      >
        {formatDate(date)}
      </Caption1>
    </div>
  );
}

function TimelineSkeleton() {
  return (
    <div style={{ padding: tokens.spacing[6] }}>
      {[1, 2, 3].map((i) => (
        <div
          key={i}
          style={{
            display: "flex",
            gap: tokens.spacing[3],
            marginBottom: tokens.spacing[4],
          }}
        >
          <div
            style={{
              width: "32px",
              height: "32px",
              borderRadius: tokens.radii.full,
              backgroundColor: "var(--surface-secondary)",
              animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
            }}
          />
          <div style={{ flex: 1 }}>
            <div
              style={{
                height: "12px",
                width: "100px",
                borderRadius: tokens.radii.sm,
                backgroundColor: "var(--surface-secondary)",
                marginBottom: tokens.spacing[2],
                animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
              }}
            />
            <div
              style={{
                height: "60px",
                width: "80%",
                borderRadius: tokens.radii.md,
                backgroundColor: "var(--surface-secondary)",
                animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

function LoadingSpinner() {
  return (
    <div
      style={{
        width: "20px",
        height: "20px",
        border: "2px solid var(--border-primary)",
        borderTopColor: "var(--color-primary)",
        borderRadius: "50%",
        animation: "spin 1s linear infinite",
      }}
    />
  );
}

function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

function formatDate(date: Date): string {
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);

  if (isSameDay(date, now)) {
    return "Today";
  }
  if (isSameDay(date, yesterday)) {
    return "Yesterday";
  }

  return date.toLocaleDateString(undefined, {
    weekday: "long",
    month: "short",
    day: "numeric",
  });
}
