/**
 * AppV2.tsx - Production-ready Happy-aligned App component
 *
 * This component integrates the new Happy-aligned UI components
 * with the existing wave8-desktop state management and API clients.
 */

import { useState, useCallback, useEffect, useMemo } from "react";
import type { RuntimeTarget } from "../sources/shared/bootstrap-config";
import { useRuntimeBootstrapProfile } from "../sources/app/providers/RuntimeBootstrapProvider";
import { useAppShellState } from "./useAppShellState";
import { useDesktopRouter } from "./router";
import { useAppV2Shell } from "./useAppV2Shell";

import { ThemeProvider } from "./components/providers/ThemeProvider";
import { Shell, Sidebar, Header, MobileShell, MobileNavBar } from "./components/layout";
import { SessionList, type ComposerSuggestion } from "./components/surfaces";

import { useLanguage } from "./hooks/useLanguage";
import {
  loadAppearanceSettings,
  saveAppearanceSettings,
  resolveDesktopThemePreference,
  type DesktopAppearanceSettings,
} from "./desktop-preferences";
import { clearNewSessionDraft } from "./new-session-draft";
import { useAppV2SettingsSections } from "./useAppV2SettingsSections";
import { useAppV2Navigation } from "./useAppV2Navigation";
import { useSessionComposerDraft } from "./useSessionComposerDraft";
import { useNewSessionDraftForm } from "./useNewSessionDraftForm";
import { useCreateSessionAction } from "./useCreateSessionAction";
import { useAppV2HomeViewModel } from "./useAppV2HomeViewModel";
import { AppV2RouteOutlet } from "./AppV2RouteOutlet";
import { useAppV2ComposerPreferences } from "./useAppV2ComposerPreferences";

interface AppV2Props {
  runtimeTarget?: RuntimeTarget;
}

export function AppV2({ runtimeTarget }: AppV2Props) {
  const profile = useRuntimeBootstrapProfile();
  const target = runtimeTarget || profile?.runtimeTarget || "browser";
  const isMobile = target === "mobile" || window.innerWidth < 768;

  return (
    <ThemeProvider defaultScheme="system">
      <AppContentV2 isMobile={isMobile} />
    </ThemeProvider>
  );
}

// =============================================================================
// App Content Component
// =============================================================================

interface AppContentV2Props {
  isMobile: boolean;
}

function AppContentV2({ isMobile }: AppContentV2Props) {
  const { t, language, setLanguage } = useLanguage();
  const router = useDesktopRouter();
  const shell = useAppShellState(router.resolved.params.id ?? null);
  const appShell = useAppV2Shell(shell, router.resolved, router.navigate);

  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [inboxFilter, setInboxFilter] = useState<"all" | "unread">("all");
  const [appearance, setAppearance] = useState<DesktopAppearanceSettings>(() =>
    loadAppearanceSettings(),
  );
  const {
    composerValue,
    setComposerValue,
    clearComposerValue,
  } = useSessionComposerDraft(appShell.activeSessionId);
  const newSessionForm = useNewSessionDraftForm();
  const createSessionAction = useCreateSessionAction(shell, router.navigate, () => {
    clearNewSessionDraft();
    newSessionForm.reset();
  });
  const composerPreferences = useAppV2ComposerPreferences(
    appShell.activeSessionId,
    appShell.currentSession,
  );

  useEffect(() => {
    const theme = resolveDesktopThemePreference(appearance.themePreference);
    document.documentElement.setAttribute("data-theme", theme);
  }, [appearance.themePreference]);

  const settingsSections = useAppV2SettingsSections({
    appearance,
    sidebarCollapsed,
    language,
    setLanguage,
    setAppearance,
    persistAppearance: saveAppearanceSettings,
    setSidebarCollapsed,
    t,
  });

  const suggestions = useMemo((): ComposerSuggestion[] => {
    return [
      {
        id: "continue",
        label: t("routes:session.suggestions.continue.label"),
        insertText: t("routes:session.suggestions.continue.insertText"),
      },
      {
        id: "explain",
        label: t("routes:session.suggestions.explain.label"),
        insertText: t("routes:session.suggestions.explain.insertText"),
      },
      {
        id: "refactor",
        label: t("routes:session.suggestions.refactor.label"),
        insertText: t("routes:session.suggestions.refactor.insertText"),
      },
      {
        id: "test",
        label: t("routes:session.suggestions.test.label"),
        insertText: t("routes:session.suggestions.test.insertText"),
      },
    ];
  }, [t]);
  const homeViewModel = useAppV2HomeViewModel({
    t,
    sessionCount: shell.sessions.length,
    messageCount: appShell.messages.length,
    activeSessionCount: shell.sessions.filter((session) => session.active).length,
    onStartNewSession: appShell.startNewSession,
    onResumeLatestSession: appShell.resumeLatestSession,
    onOpenSettings: () => router.navigate("/(app)/settings/index"),
  });

  const handleSendMessage = useCallback(async () => {
    if (!composerValue.trim() || !appShell.activeSessionId) {
      return;
    }

    setIsSending(true);
    try {
      await shell.sendMessage(
        appShell.activeSessionId,
        composerValue,
        composerPreferences.sendMessageOptions,
      );
      clearComposerValue();
    } finally {
      setIsSending(false);
    }
  }, [
    appShell.activeSessionId,
    clearComposerValue,
    composerPreferences.sendMessageOptions,
    composerValue,
    shell,
  ]);

  const handleSessionSelect = useCallback(
    (session: import("./components/surfaces").Session) => {
      appShell.openSession(session.id);
    },
    [appShell],
  );

  const navigation = useAppV2Navigation({
    view: appShell.view,
    unreadCount: appShell.unreadCount,
    navigate: router.navigate,
    t,
  });
  const normalizedInboxFilter =
    inboxFilter === "unread" && typeof appShell.unreadCount !== "number" ? "all" : inboxFilter;
  const routeContent = (
    <AppV2RouteOutlet
      view={appShell.view}
      sessions={appShell.sessions}
      currentSession={appShell.currentSession}
      messages={appShell.messages}
      settingsSections={settingsSections}
      notifications={appShell.notifications}
      inboxFilter={normalizedInboxFilter}
      unreadCount={appShell.unreadCount}
      quickActions={homeViewModel.quickActions}
      stats={homeViewModel.stats}
      suggestions={suggestions}
      models={composerPreferences.models}
      selectedModel={composerPreferences.selectedModel}
      composerValue={composerValue}
      isSending={isSending}
      sessionLoading={
        appShell.currentSession
          ? shell.sessionState[appShell.currentSession.id]?.loading
          : undefined
      }
      sessionError={
        appShell.currentSession
          ? shell.sessionState[appShell.currentSession.id]?.error ?? appShell.errorMessage ?? undefined
          : appShell.errorMessage ?? undefined
      }
      newSessionWorkspace={newSessionForm.workspace}
      newSessionModel={newSessionForm.model}
      newSessionTitle={newSessionForm.title}
      newSessionPrompt={newSessionForm.prompt}
      newSessionErrors={newSessionForm.errors}
      newSessionFormError={createSessionAction.createSessionError}
      isCreatingSession={createSessionAction.isCreatingSession}
      emptySessionTitle={t("routes:session.emptyState.title")}
      unsupportedDescription={t("routes:unsupported.description", {
        routeTitle: router.resolved.definition.title,
      })}
      unsupportedTitle={t("routes:unsupported.title")}
      onSessionSelect={handleSessionSelect}
      onStartNewSession={appShell.startNewSession}
      onViewAllSessions={() => router.navigate("/(app)/session/recent")}
      onNewSessionWorkspaceChange={(value) => {
        createSessionAction.setCreateSessionError(null);
        newSessionForm.setErrors((current) => ({ ...current, workspace: undefined }));
        newSessionForm.setWorkspace(value);
      }}
      onNewSessionModelChange={(value) => {
        createSessionAction.setCreateSessionError(null);
        newSessionForm.setModel(value);
      }}
      onNewSessionTitleChange={newSessionForm.setTitle}
      onNewSessionPromptChange={(value) => {
        createSessionAction.setCreateSessionError(null);
        newSessionForm.setErrors((current) => ({ ...current, prompt: undefined }));
        newSessionForm.setPrompt(value);
      }}
      onCreateSession={async () => {
        const input = newSessionForm.validate();
        if (input) {
          await createSessionAction.createSession(input);
        }
      }}
      onBackToHome={() => router.navigate("/(app)/index")}
      onComposerChange={setComposerValue}
      onModelChange={composerPreferences.setSelectedModel}
      onSendMessage={handleSendMessage}
      onInboxFilterChange={setInboxFilter}
      t={t}
    />
  );

  // Mobile layout
  if (isMobile) {
    return (
      <MobileShell
        header={
          appShell.view === "session" && appShell.currentSession ? (
            <MobileNavBar
              title={appShell.currentSession.metadata?.name || t("routes:session.title")}
              leading={
                <button onClick={() => router.navigate("/(app)/index")} style={{ fontSize: "1.5rem" }}>
                  ←
                </button>
              }
            />
          ) : (
            <MobileNavBar title={t('common:app.name')} />
          )
        }
        tabs={[
          { id: "home", label: t('components:nav.home'), icon: "🏠" },
          { id: "sessions", label: t('components:nav.sessions'), icon: "💬" },
          { id: "inbox", label: t('components:nav.inbox'), icon: "🔔", badge: appShell.unreadCount },
          { id: "settings", label: t('components:nav.settings'), icon: "⚙️" },
        ]}
        activeTab={navigation.mobileActiveTab}
        onTabChange={(tab) => {
          if (tab === "home") router.navigate("/(app)/index");
          if (tab === "sessions") router.navigate("/(app)/session/recent");
          if (tab === "inbox") router.navigate("/(app)/inbox/index");
          if (tab === "settings") router.navigate("/(app)/settings/index");
        }}
      >
        {routeContent}
      </MobileShell>
    );
  }

  // Desktop layout
  return (
    <Shell
      sidebar={
        <Sidebar
          brand={
            <div
              style={{
                fontWeight: 700,
                fontSize: "1.5rem",
                display: "flex",
                alignItems: "center",
                gap: "12px",
              }}
            >
              <span>⚡</span>
              {!sidebarCollapsed && <span>{t('common:app.name')}</span>}
            </div>
          }
          primarySections={[{ items: navigation.primaryNavItems }]}
          secondarySections={[{ items: navigation.secondaryNavItems }]}
          connectionStatus={
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <span
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  backgroundColor: appShell.isConnected ? "var(--color-success)" : "var(--color-danger)",
                }}
              />
              {!sidebarCollapsed && (
                <span style={{ fontSize: "0.8125rem", color: "var(--text-tertiary)" }}>
                  {appShell.isConnected ? t('ui:connection.connected') : t('ui:connection.disconnected')}
                </span>
              )}
            </div>
          }
          collapsed={sidebarCollapsed}
        />
      }
      header={
        appShell.view !== "session" && (
          <Header
            eyebrow={t(navigation.headerEyebrowKey)}
            title={navigation.headerTitle}
            size="compact"
          />
        )
      }
      sidebarCollapsed={sidebarCollapsed}
    >
      {appShell.view === "home" ? (
        <div style={{ display: "flex", height: "100%" }}>
          {/* Session List Sidebar */}
          <div
            style={{
              width: "320px",
              borderRight: "1px solid var(--border-primary)",
              overflow: "auto",
            }}
          >
            <SessionList
              sessions={appShell.sessions}
              selectedId={appShell.activeSessionId || undefined}
              onSelect={handleSessionSelect}
              loading={appShell.isLoading}
            />
          </div>

          <div style={{ flex: 1, overflow: "auto" }}>{routeContent}</div>
        </div>
      ) : (
        routeContent
      )}
    </Shell>
  );
}

export default AppV2;
