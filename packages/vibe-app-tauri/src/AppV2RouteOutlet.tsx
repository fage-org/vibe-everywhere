import type { Notification, QuickAction, SettingSection, StatItem } from "./components/routes";
import type { ComposerSuggestion, Message } from "./components/surfaces";
import type { DesktopSession } from "./desktop-client";
import type { AppV2View } from "./useAppV2RouteModel";
import type { SessionWorkspaceFile, SessionWorkspaceFileContent } from "./session-files";
import {
  HomeRoute,
  InboxRoute,
  NewSessionRoute,
  RecentSessionsRoute,
  SessionRoute,
  SettingsRoute,
  SettingsAIProvidersRoute,
  SettingsAppearanceRoute,
  SettingsFeaturesRoute,
  SettingsLanguageRoute,
  SettingsUsageRoute,
  SettingsVoiceRoute,
  UnsupportedRoute,
  RestoreRoute,
  ManualRestoreRoute,
  SessionInfoRoute,
  SessionMessageRoute,
  SessionFilesRoute,
  SessionFileRoute,
  MachineDetailRoute,
  ArtifactsRoute,
  ArtifactDetailRoute,
  ArtifactEditRoute,
  ArtifactNewRoute,
  FriendsRoute,
  FriendsSearchRoute,
  UserDetailRoute,
  TerminalRoute,
  TerminalConnectRoute,
} from "./routes/appv2";

type NewSessionErrors = {
  workspace?: string;
  prompt?: string;
};

type RestoreState = {
  qrSvg: string | null;
  error: string | null;
  isLoading: boolean;
};

type SessionFilesState = {
  files: SessionWorkspaceFile[];
  branch: string | null;
  totalStaged: number;
  totalUnstaged: number;
  loading: boolean;
  error: string | null;
};

type SessionFileState = {
  file: SessionWorkspaceFileContent | null;
  loading: boolean;
  error: string | null;
};

type SessionItem = {
  id: string;
  title: string;
  subtitle?: string;
  lastActivityAt: Date;
};

type AppV2RouteOutletProps = {
  view: AppV2View;
  sessions: SessionItem[];
  currentSession: DesktopSession | null;
  messages: Message[];
  currentMessage: Message | null;
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
  // Restore props
  restoreState: RestoreState;
  // Session files props
  sessionFilesState: SessionFilesState;
  sessionFileState: SessionFileState;
  // Machine props
  activeMachineId: string | null;
  // Artifact props
  activeArtifactId: string | null;
  // Social props
  activeUserId: string | null;
  // Navigation callbacks
  onSessionSelect: (session: { id: string }) => void;
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
  // Restore callbacks
  onRefreshQr: () => void;
  onManualRestore: () => void;
  onSubmitManualRestore: (secret: string) => Promise<void>;
  // Session navigation callbacks
  onNavigateToSession: (sessionId: string) => void;
  onNavigateToSessionFiles: (sessionId: string) => void;
  onNavigateToSessionFile: (sessionId: string, path: string) => void;
  // Session files callbacks
  onSelectFile: (path: string) => void;
  t: (key: string) => string;
};

export function AppV2RouteOutlet({
  view,
  sessions,
  currentSession,
  messages,
  currentMessage,
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
  restoreState,
  sessionFilesState,
  sessionFileState,
  activeMachineId,
  activeArtifactId,
  activeUserId,
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
  onRefreshQr,
  onManualRestore,
  onSubmitManualRestore,
  onNavigateToSession,
  onNavigateToSessionFiles,
  onNavigateToSessionFile,
  onSelectFile,
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
    case "settings-appearance":
      return <SettingsAppearanceRoute />;
    case "settings-ai-providers":
      return <SettingsAIProvidersRoute />;
    case "settings-features":
      return <SettingsFeaturesRoute />;
    case "settings-language":
      return <SettingsLanguageRoute />;
    case "settings-usage":
      return <SettingsUsageRoute />;
    case "settings-voice":
      return <SettingsVoiceRoute />;
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
    case "restore":
      return (
        <RestoreRoute
          qrSvg={restoreState.qrSvg}
          error={restoreState.error}
          isLoading={restoreState.isLoading}
          onRefreshQr={onRefreshQr}
          onManualRestore={onManualRestore}
        />
      );
    case "restore-manual":
      return (
        <ManualRestoreRoute
          onSubmit={onSubmitManualRestore}
          isSubmitting={restoreState.isLoading}
          error={restoreState.error}
        />
      );
    case "session-info":
      return (
        <SessionInfoRoute
          session={currentSession}
          loading={sessionLoading}
          error={sessionError}
          onNavigateToSession={() => onNavigateToSession(currentSession?.id ?? "")}
          onNavigateToFiles={() => onNavigateToSessionFiles(currentSession?.id ?? "")}
        />
      );
    case "session-message":
      return (
        <SessionMessageRoute
          sessionId={currentSession?.id ?? ""}
          messageId={""}
          message={currentMessage}
          loading={sessionLoading}
          error={sessionError}
          onNavigateToSession={() => onNavigateToSession(currentSession?.id ?? "")}
          onNavigateToFiles={() => onNavigateToSessionFiles(currentSession?.id ?? "")}
        />
      );
    case "session-files":
      return (
        <SessionFilesRoute
          session={currentSession}
          files={sessionFilesState.files}
          branch={sessionFilesState.branch}
          totalStaged={sessionFilesState.totalStaged}
          totalUnstaged={sessionFilesState.totalUnstaged}
          loading={sessionFilesState.loading}
          error={sessionFilesState.error}
          onSelectFile={onSelectFile}
          onNavigateToSession={() => onNavigateToSession(currentSession?.id ?? "")}
        />
      );
    case "session-file":
      return (
        <SessionFileRoute
          sessionId={currentSession?.id ?? ""}
          filePath={null}
          file={sessionFileState.file}
          loading={sessionFileState.loading}
          error={sessionFileState.error}
          onNavigateToFiles={() => onNavigateToSessionFiles(currentSession?.id ?? "")}
          onNavigateToSession={() => onNavigateToSession(currentSession?.id ?? "")}
        />
      );
    case "machine-detail":
      return (
        <MachineDetailRoute
          machineId={activeMachineId ?? ""}
          onNavigateToSession={onNavigateToSession}
        />
      );
    case "artifacts":
      return <ArtifactsRoute />;
    case "artifact-detail":
      return (
        <ArtifactDetailRoute
          artifactId={activeArtifactId ?? ""}
        />
      );
    case "artifact-edit":
      return (
        <ArtifactEditRoute
          artifactId={activeArtifactId ?? ""}
        />
      );
    case "artifact-new":
      return <ArtifactNewRoute />;
    case "friends":
      return <FriendsRoute />;
    case "friends-search":
      return <FriendsSearchRoute />;
    case "user-detail":
      return <UserDetailRoute userId={activeUserId ?? ""} />;
    case "terminal":
      return <TerminalRoute />;
    case "terminal-connect":
      // Get publicKey from URL search params
      return <TerminalConnectRoute publicKey={new URLSearchParams(window.location.hash.split("?")[1] || "").get("publicKey") || ""} />;
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
