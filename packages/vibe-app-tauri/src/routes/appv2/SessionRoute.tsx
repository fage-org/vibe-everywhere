import { SessionSurface } from "../../components/routes";
import type { ComposerSuggestion, Message } from "../../components/surfaces";
import type { DesktopSession } from "../../wave8-client";
import { describeSession } from "../../wave8-client";

type SessionRouteProps = {
  currentSession: DesktopSession | null;
  messages: Message[];
  composerValue: string;
  isSending: boolean;
  suggestions: ComposerSuggestion[];
  models: { id: string; name: string }[];
  selectedModel?: string;
  loading?: boolean;
  error?: string;
  emptyTitle: string;
  onComposerChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onSendMessage: () => Promise<void> | void;
};

export function SessionRoute({
  currentSession,
  messages,
  composerValue,
  isSending,
  suggestions,
  models,
  selectedModel,
  loading,
  error,
  emptyTitle,
  onComposerChange,
  onModelChange,
  onSendMessage,
}: SessionRouteProps) {
  if (!currentSession) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--text-tertiary)",
        }}
      >
        {emptyTitle}
      </div>
    );
  }

  return (
    <SessionSurface
      session={{
        id: currentSession.id,
        title: describeSession(currentSession).title,
        subtitle: currentSession.metadata?.path || undefined,
        lastActivityAt: new Date(currentSession.updatedAt || currentSession.createdAt),
      }}
      messages={messages}
      composerValue={composerValue}
      onComposerChange={onComposerChange}
      onSendMessage={onSendMessage}
      isSending={isSending}
      suggestions={suggestions}
      onSuggestionSelect={(suggestion) => onComposerChange(suggestion.insertText)}
      models={models}
      selectedModel={selectedModel}
      onModelChange={onModelChange}
      loading={loading}
      error={error}
    />
  );
}
