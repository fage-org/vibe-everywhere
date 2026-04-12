import type { Notification, QuickAction, SettingSection, StatItem } from "./components/routes";
import type { ComposerSuggestion, Message, Session } from "./components/surfaces";
import type { DesktopSession } from "./wave8-client";
import type { AppV2View } from "./useAppV2RouteModel";
import {
  HomeRoute,
  InboxRoute,
  NewSessionRoute,
  RecentSessionsRoute,
  SessionRoute,
  SettingsRoute,
  UnsupportedRoute,
} from "./routes/appv2";

type NewSessionErrors = {
  workspace?: string;
  prompt?: string;
};

type AppV2RouteOutletProps = {
  view: AppV2View;
  sessions: Session[];
  currentSession: DesktopSession | null;
  messages: Message[];
  settingsSections: SettingSection[];
  notifications: Notification[];
  inboxFilter: "all" | "unread";
  unreadCount?: number;
  quickActions: QuickAction[];
  stats: StatItem[];
  suggestions: ComposerSuggestion[];
  models: { id: string; name: string }[];
  selectedModel?: string;
  composerValue: string;
  isSending: boolean;
  sessionLoading?: boolean;
  sessionError?: string;
  newSessionWorkspace: string;
  newSessionModel: string;
  newSessionTitle: string;
  newSessionPrompt: string;
  newSessionErrors: NewSessionErrors;
  newSessionFormError?: string | null;
  isCreatingSession: boolean;
  emptySessionTitle: string;
  unsupportedDescription: string;
  unsupportedTitle: string;
  onSessionSelect: (session: Session) => void;
  onStartNewSession: () => void;
  onViewAllSessions: () => void;
  onNewSessionWorkspaceChange: (value: string) => void;
  onNewSessionModelChange: (value: string) => void;
  onNewSessionTitleChange: (value: string) => void;
  onNewSessionPromptChange: (value: string) => void;
  onCreateSession: () => Promise<void> | void;
  onBackToHome: () => void;
  onComposerChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onSendMessage: () => Promise<void> | void;
  onInboxFilterChange: (filter: "all" | "unread") => void;
  t: (key: string) => string;
};

export function AppV2RouteOutlet({
  view,
  sessions,
  currentSession,
  messages,
  settingsSections,
  notifications,
  inboxFilter,
  unreadCount,
  quickActions,
  stats,
  suggestions,
  models,
  selectedModel,
  composerValue,
  isSending,
  sessionLoading,
  sessionError,
  newSessionWorkspace,
  newSessionModel,
  newSessionTitle,
  newSessionPrompt,
  newSessionErrors,
  newSessionFormError,
  isCreatingSession,
  emptySessionTitle,
  unsupportedDescription,
  unsupportedTitle,
  onSessionSelect,
  onStartNewSession,
  onViewAllSessions,
  onNewSessionWorkspaceChange,
  onNewSessionModelChange,
  onNewSessionTitleChange,
  onNewSessionPromptChange,
  onCreateSession,
  onBackToHome,
  onComposerChange,
  onModelChange,
  onSendMessage,
  onInboxFilterChange,
  t,
}: AppV2RouteOutletProps) {
  switch (view) {
    case "home":
      return (
        <HomeRoute
          sessions={sessions}
          onSessionSelect={onSessionSelect}
          onNewSession={onStartNewSession}
          onViewAllSessions={onViewAllSessions}
          quickActions={quickActions}
          stats={stats}
        />
      );
    case "new-session":
      return (
        <NewSessionRoute
          workspace={newSessionWorkspace}
          model={newSessionModel}
          title={newSessionTitle}
          prompt={newSessionPrompt}
          validationErrors={newSessionErrors}
          formError={newSessionFormError}
          isCreatingSession={isCreatingSession}
          onWorkspaceChange={onNewSessionWorkspaceChange}
          onModelChange={onNewSessionModelChange}
          onTitleChange={onNewSessionTitleChange}
          onPromptChange={onNewSessionPromptChange}
          onCreateSession={onCreateSession}
          onBack={onBackToHome}
          titleText={t("routes:home.actions.newSession")}
        />
      );
    case "session-recent":
      return (
        <RecentSessionsRoute
          eyebrow={t("routes:home.sections.recentSessions")}
          title={t("routes:home.actions.resume")}
          sessions={sessions}
          onSessionSelect={onSessionSelect}
          isLoading={false}
        />
      );
    case "session":
      return (
        <SessionRoute
          currentSession={currentSession}
          messages={messages}
          composerValue={composerValue}
          onComposerChange={onComposerChange}
          onSendMessage={onSendMessage}
          isSending={isSending}
          suggestions={suggestions}
          models={models}
          selectedModel={selectedModel}
          loading={sessionLoading}
          error={sessionError}
          emptyTitle={emptySessionTitle}
          onModelChange={onModelChange}
        />
      );
    case "settings":
      return <SettingsRoute sections={settingsSections} />;
    case "inbox":
      return (
        <InboxRoute
          notifications={notifications}
          unreadCount={unreadCount}
          supportsUnreadFilter={typeof unreadCount === "number"}
          filter={inboxFilter}
          onFilterChange={onInboxFilterChange}
        />
      );
    case "unsupported":
      return (
        <UnsupportedRoute
          title={unsupportedTitle}
          description={unsupportedDescription}
        />
      );
    default:
      return null;
  }
}
