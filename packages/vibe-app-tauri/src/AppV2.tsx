/**
 * AppV2.tsx - Production-ready Happy-aligned App component
 *
 * This component is the sole UI shell for the Vibe application.
 * All routes are handled through AppV2RouteOutlet.
 *
 * Historical note: This replaces the legacy App.tsx which was a 283KB
 * monolithic component. The new architecture uses modular components
 * from the design-system.
 */

import { useState, useCallback, useEffect, useMemo } from "react";
import type { RuntimeTarget } from "../sources/shared/bootstrap-config";
import { useRuntimeBootstrapProfile } from "../sources/app/providers/RuntimeBootstrapProvider";
import { useAppShellState } from "./useAppShellState";
import { useDesktopRouter } from "./router";
import { useAppV2Shell } from "./useAppV2Shell";

import { ThemeProvider } from "./components/providers/ThemeProvider";
import { Shell, Sidebar, Header, MobileShell, MobileNavBar } from "./components/layout";
import { SessionList, type ComposerSuggestion, type Message } from "./components/surfaces";
import { CommandPalette } from "./components/ui";
import { LocalSettingsProvider } from "./LocalSettingsContext";

import { useLanguage } from "./hooks/useLanguage";
import { useCommandPalette, createDefaultCommands } from "./hooks/useCommandPalette";
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
import type { SessionWorkspaceFile, SessionWorkspaceFileContent } from "./session-files";

interface AppV2Props {
  runtimeTarget?: RuntimeTarget;
}

export function AppV2({ runtimeTarget }: AppV2Props) {
  const profile = useRuntimeBootstrapProfile();
  const target = runtimeTarget || profile?.runtimeTarget || "browser";
  const isMobile = target === "mobile" || window.innerWidth < 768;

  return (
    <LocalSettingsProvider>
      <ThemeProvider defaultScheme="system">
        <AppContentV2 isMobile={isMobile} />
      </ThemeProvider>
    </LocalSettingsProvider>
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

  // Command palette
  const commandPaletteCommands = useMemo(
    () => createDefaultCommands(router.navigate),
    [router.navigate],
  );
  const commandPalette = useCommandPalette(commandPaletteCommands);

  // Restore state
  const [restoreQrSvg, setRestoreQrSvg] = useState<string | null>(null);
  const [restoreError, setRestoreError] = useState<string | null>(null);
  const [restoreLoading, setRestoreLoading] = useState(false);

  // Session files state
  const [sessionFiles, setSessionFiles] = useState<SessionWorkspaceFile[]>([]);
  const [sessionFilesBranch, setSessionFilesBranch] = useState<string | null>(null);
  const [sessionFilesStaged, setSessionFilesStaged] = useState(0);
  const [sessionFilesUnstaged, setSessionFilesUnstaged] = useState(0);
  const [sessionFilesLoading, setSessionFilesLoading] = useState(false);
  const [sessionFilesError, setSessionFilesError] = useState<string | null>(null);

  // Session file content state
  const [sessionFileContent, setSessionFileContent] = useState<SessionWorkspaceFileContent | null>(null);
  const [sessionFileLoading, setSessionFileLoading] = useState(false);
  const [sessionFileError, setSessionFileError] = useState<string | null>(null);

  // Current message for session-message view
  const currentMessage = useMemo((): Message | null => {
    if (appShell.view !== "session-message" || !appShell.messages.length) {
      return null;
    }
    // For now, return the first message - this should be enhanced to find by ID
    return appShell.messages[0] || null;
  }, [appShell.view, appShell.messages]);

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
    (session: { id: string }) => {
      appShell.openSession(session.id);
    },
    [appShell],
  );

  // Restore callbacks
  const handleRefreshQr = useCallback(async () => {
    setRestoreLoading(true);
    setRestoreError(null);
    try {
      // Trigger QR code generation via shell
      await shell.startMobileLink?.();
    } catch (err) {
      setRestoreError(err instanceof Error ? err.message : "Failed to generate QR code");
    } finally {
      setRestoreLoading(false);
    }
  }, [shell]);

  const handleManualRestore = useCallback(() => {
    router.navigate("/(app)/restore/manual");
  }, [router]);

  const handleSubmitManualRestore = useCallback(async (secret: string) => {
    setRestoreLoading(true);
    setRestoreError(null);
    try {
      await shell.restoreWithSecret?.(secret);
      router.navigate("/(app)/index");
    } catch (err) {
      setRestoreError(err instanceof Error ? err.message : "Failed to restore account");
    } finally {
      setRestoreLoading(false);
    }
  }, [shell, router]);

  // Session navigation callbacks
  const handleNavigateToSession = useCallback((sessionId: string) => {
    router.navigate(`/(app)/session/${sessionId}`);
  }, [router]);

  const handleNavigateToSessionFiles = useCallback((sessionId: string) => {
    router.navigate(`/(app)/session/${sessionId}/files`);
  }, [router]);

  const handleNavigateToSessionFile = useCallback((sessionId: string, path: string) => {
    router.navigate(`/(app)/session/${sessionId}/file?path=${encodeURIComponent(path)}`);
  }, [router]);

  const handleSelectFile = useCallback((path: string) => {
    if (appShell.activeSessionId) {
      handleNavigateToSessionFile(appShell.activeSessionId, path);
    }
  }, [appShell.activeSessionId, handleNavigateToSessionFile]);

  // Load session files when view changes
  useEffect(() => {
    if (appShell.view === "session-files" && appShell.activeSessionId && shell.credentials) {
      setSessionFilesLoading(true);
      setSessionFilesError(null);
      shell.loadSessionFiles?.(appShell.activeSessionId)
        .then((inventory) => {
          setSessionFiles(inventory.files);
          setSessionFilesBranch(inventory.branch);
          setSessionFilesStaged(inventory.totalStaged);
          setSessionFilesUnstaged(inventory.totalUnstaged);
        })
        .catch((err) => {
          setSessionFilesError(err instanceof Error ? err.message : "Failed to load files");
        })
        .finally(() => {
          setSessionFilesLoading(false);
        });
    }
  }, [appShell.view, appShell.activeSessionId, shell]);

  const navigation = useAppV2Navigation({
    view: appShell.view,
    navigate: router.navigate,
    t,
  });
  const routeContent = (
    <AppV2RouteOutlet
      view={appShell.view}
      sessions={appShell.sessions}
      currentSession={appShell.currentSession}
      messages={appShell.messages}
      currentMessage={currentMessage}
      settingsSections={settingsSections}
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
      // Machine props
      activeMachineId={appShell.activeMachineId}
      // Artifact props
      activeArtifactId={appShell.activeArtifactId}
      // Restore props
      restoreState={{
        qrSvg: restoreQrSvg,
        error: restoreError,
        isLoading: restoreLoading,
      }}
      // Session files props
      sessionFilesState={{
        files: sessionFiles,
        branch: sessionFilesBranch,
        totalStaged: sessionFilesStaged,
        totalUnstaged: sessionFilesUnstaged,
        loading: sessionFilesLoading,
        error: sessionFilesError,
      }}
      // Session file props
      sessionFileState={{
        file: sessionFileContent,
        loading: sessionFileLoading,
        error: sessionFileError,
      }}
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
      // Restore callbacks
      onRefreshQr={handleRefreshQr}
      onManualRestore={handleManualRestore}
      onSubmitManualRestore={handleSubmitManualRestore}
      // Session navigation callbacks
      onNavigateToSession={handleNavigateToSession}
      onNavigateToSessionFiles={handleNavigateToSessionFiles}
      onNavigateToSessionFile={handleNavigateToSessionFile}
      // Session files callback
      onSelectFile={handleSelectFile}
      t={t}
    />
  );

  // Mobile layout
  if (isMobile) {
    return (
      <>
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
            { id: "artifacts", label: t('components:nav.artifacts'), icon: "📁" },
            { id: "terminal", label: t('components:nav.terminal'), icon: "💻" },
            { id: "settings", label: t('components:nav.settings'), icon: "⚙️" },
          ]}
          activeTab={navigation.mobileActiveTab}
          onTabChange={(tab) => {
            if (tab === "home") router.navigate("/(app)/index");
            if (tab === "sessions") router.navigate("/(app)/session/recent");
            if (tab === "artifacts") router.navigate("/(app)/artifacts/index");
            if (tab === "terminal") router.navigate("/(app)/terminal/index");
            if (tab === "settings") router.navigate("/(app)/settings/index");
          }}
        >
          {routeContent}
        </MobileShell>
        <CommandPalette
          isOpen={commandPalette.isOpen}
          close={commandPalette.close}
          query={commandPalette.query}
          setQuery={commandPalette.setQuery}
          commands={commandPalette.filteredCommands}
          selectedIndex={commandPalette.selectedIndex}
          selectPrevious={commandPalette.selectPrevious}
          selectNext={commandPalette.selectNext}
          executeSelected={commandPalette.executeSelected}
        />
      </>
    );
  }

  // Desktop layout
  return (
    <>
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
      <CommandPalette
        isOpen={commandPalette.isOpen}
        close={commandPalette.close}
        query={commandPalette.query}
        setQuery={commandPalette.setQuery}
        commands={commandPalette.filteredCommands}
        selectedIndex={commandPalette.selectedIndex}
        selectPrevious={commandPalette.selectPrevious}
        selectNext={commandPalette.selectNext}
        executeSelected={commandPalette.executeSelected}
      />
    </>
  );
}

export default AppV2;
