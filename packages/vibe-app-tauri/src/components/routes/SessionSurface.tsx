import { useState, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { Header } from "../layout";
import { Timeline, Composer, type Message, type ComposerSuggestion, type Session } from "../surfaces";
import { Button, Badge } from "../ui";
import { Body } from "../ui/Typography";

export interface SessionSurfaceProps {
  /** Current session */
  session: Session;
  /** Session messages */
  messages: Message[];
  /** Composer value */
  composerValue: string;
  /** Callback when composer value changes */
  onComposerChange: (value: string) => void;
  /** Callback when message is sent */
  onSendMessage: () => void;
  /** Whether message is being sent */
  isSending?: boolean;
  /** Composer suggestions */
  suggestions?: ComposerSuggestion[];
  /** Callback when suggestion is selected */
  onSuggestionSelect?: (suggestion: ComposerSuggestion) => void;
  /** Session actions */
  actions?: {
    label: string;
    onClick: () => void;
    variant?: "primary" | "secondary" | "ghost" | "danger";
  }[];
  /** Available models for footer */
  models?: { id: string; name: string }[];
  /** Selected model */
  selectedModel?: string;
  /** Callback when model changes */
  onModelChange?: (modelId: string) => void;
  /** Loading state */
  loading?: boolean;
  /** Error state */
  error?: string;
}

/**
 * SessionSurface - Session detail route surface
 *
 * Matches Happy's SessionView:
 * - Header with session info and actions
 * - Scrollable message timeline
 * - Composer at bottom
 * - Model selector in footer
 */
export function SessionSurface({
  session,
  messages,
  composerValue,
  onComposerChange,
  onSendMessage,
  isSending = false,
  suggestions = [],
  onSuggestionSelect,
  actions = [],
  models = [],
  selectedModel,
  onModelChange,
  loading,
  error,
}: SessionSurfaceProps) {
  const { t } = useTranslation(['routes', 'components']);
  const timelineRef = useRef<HTMLDivElement>(null);
  const [showModelSelector, setShowModelSelector] = useState(false);

  const handleSend = useCallback(() => {
    onSendMessage();
    // Scroll to bottom after sending
    setTimeout(() => {
      timelineRef.current?.scrollTo({
        top: timelineRef.current.scrollHeight,
        behavior: "smooth",
      });
    }, 100);
  }, [onSendMessage]);

  const headerActions = (
    <>
      {actions.map((action, index) => (
        <Button
          key={index}
          variant={action.variant || "ghost"}
          size="sm"
          onClick={action.onClick}
        >
          {action.label}
        </Button>
      ))}
    </>
  );

  const selectedModelName = models.find((m) => m.id === selectedModel)?.name || selectedModel;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
      }}
    >
      {/* Header */}
      <Header
        eyebrow={t('routes:session.header.eyebrow')}
        title={session.title}
        subtitle={session.subtitle}
        size="compact"
        actions={headerActions}
      />

      {/* Error Banner */}
      {error && (
        <div
          style={{
            padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
            backgroundColor: "rgba(255, 69, 58, 0.1)",
            borderBottom: "1px solid var(--color-danger)",
            color: "var(--color-danger)",
          }}
        >
          <Body>{error}</Body>
        </div>
      )}

      {/* Timeline */}
      <div style={{ flex: 1, overflow: "hidden" }}>
        <Timeline
          messages={messages}
          loading={loading}
          scrollRef={timelineRef}
          autoScroll={true}
          emptyState={
            <div
              style={{
                textAlign: "center",
                color: "var(--text-tertiary)",
              }}
            >
              <div style={{ fontSize: "3rem", marginBottom: tokens.spacing[4] }}>💬</div>
              <Body>{t('routes:session.timeline.empty.title')}</Body>
              <Body color="tertiary" style={{ marginTop: tokens.spacing[2] }}>
                {t('routes:session.timeline.empty.subtitle')}
              </Body>
            </div>
          }
        />
      </div>

      {/* Composer */}
      <Composer
        value={composerValue}
        onChange={onComposerChange}
        onSubmit={handleSend}
        placeholder={t('routes:session.composer.placeholder', { sessionTitle: session.title })}
        disabled={!!error}
        isSending={isSending}
        suggestions={suggestions}
        onSuggestionSelect={onSuggestionSelect}
        toolbar={
          <div style={{ display: "flex", gap: tokens.spacing[2], alignItems: "center" }}>
            <Body color="quaternary" style={{ fontSize: tokens.typography.fontSize.xs }}>
              {t("routes:session.composer.capabilityNotice")}
            </Body>
          </div>
        }
        footer={
          <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
            {models.length > 0 && (
              <div style={{ position: "relative" }}>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setShowModelSelector(!showModelSelector)}
                >
                  <Badge variant="outline" size="sm">
                    {selectedModelName || t('routes:session.models.selectPlaceholder')}
                  </Badge>
                </Button>

                {showModelSelector && (
                  <div
                    style={{
                      position: "absolute",
                      bottom: "100%",
                      left: 0,
                      marginBottom: tokens.spacing[2],
                      backgroundColor: "var(--surface-elevated)",
                      borderRadius: tokens.radii.lg,
                      boxShadow: tokens.shadows.ios.large,
                      border: "1px solid var(--border-primary)",
                      minWidth: "200px",
                      zIndex: tokens.zIndex.dropdown,
                    }}
                  >
                    {models.map((model) => (
                      <button
                        key={model.id}
                        onClick={() => {
                          onModelChange?.(model.id);
                          setShowModelSelector(false);
                        }}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          width: "100%",
                          padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
                          backgroundColor:
                            model.id === selectedModel
                              ? "var(--surface-hover)"
                              : "transparent",
                          border: "none",
                          borderBottom: "1px solid var(--border-primary)",
                          cursor: "pointer",
                          textAlign: "left",
                          color: "var(--text-primary)",
                        }}
                      >
                        <Body style={{ fontSize: tokens.typography.fontSize.sm }}>{model.name}</Body>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}
            <Body color="quaternary" style={{ fontSize: tokens.typography.fontSize.xs }}>
              {t('routes:session.composer.hint')}
            </Body>
          </div>
        }
      />
    </div>
  );
}
