// @ts-nocheck
/**
 * AppV2.tsx - Production-ready Happy-aligned App component
 *
 * This component integrates the new Happy-aligned UI components
 * with the existing wave8-desktop state management and API clients.
 */

import { useState, useCallback, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { RuntimeTarget } from "../sources/shared/bootstrap-config";
import { useRuntimeBootstrapProfile } from "../sources/app/providers/RuntimeBootstrapProvider";
import { useAppShellState } from "./useAppShellState";
import { useDesktopRouter, type ResolvedRoute } from "./router";
import {
  type DesktopSession,
  type DesktopArtifact,
  type UserProfile,
  type UiMessage,
  describeSession,
  calculateUsageTotals,
} from "./wave8-client";

// New Happy-aligned components
import { ThemeProvider } from "./components/providers/ThemeProvider";
import { Shell, Sidebar, Header, MobileShell, MobileNavBar } from "./components/layout";
import { Button } from "./components/ui";
import { SessionList, Timeline, Composer, type Message, type ComposerSuggestion } from "./components/surfaces";
import { HomeSurface, SessionSurface, SettingsSurface, InboxSurface } from "./components/routes";
import type { Notification, SettingSection, QuickAction, StatItem } from "./components/routes";

import { useLanguage } from "./hooks/useLanguage";
import { SUPPORTED_LANGUAGES } from "./i18n/types";

// Import existing hooks and utilities
import {
  loadAppearanceSettings,
  saveAppearanceSettings,
  resolveDesktopThemePreference,
  type DesktopAppearanceSettings,
} from "./desktop-preferences";

// =============================================================================
// Types
// =============================================================================

interface AppV2Props {
  runtimeTarget?: RuntimeTarget;
}

// =============================================================================
// Main App Component
// =============================================================================

export function AppV2({ runtimeTarget }: AppV2Props) {
  // Get runtime profile
  const profile = useRuntimeBootstrapProfile();
  const target = runtimeTarget || profile.runtimeTarget;

  // Determine if mobile
  const isMobile = target === "mobile" || window.innerWidth < 768;

  return (
    <ThemeProvider defaultScheme="system">
      <AppContentV2 target={target} isMobile={isMobile} />
    </ThemeProvider>
  );
}

// =============================================================================
// App Content Component
// =============================================================================

interface AppContentV2Props {
  target: RuntimeTarget;
  isMobile: boolean;
}

function AppContentV2({ target, isMobile }: AppContentV2Props) {
  // i18n
  const { t, language, setLanguage } = useLanguage();

  // Use existing state management
  const shell = useAppShellState();
  const router = useDesktopRouter();

  // Local UI state
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [composerValue, setComposerValue] = useState("");
  const [isSending, setIsSending] = useState(false);
  const [inboxFilter, setInboxFilter] = useState<"all" | "unread">("all");

  // Load appearance settings
  const [appearance, setAppearance] = useState<DesktopAppearanceSettings>(() =>
    loadAppearanceSettings(),
  );

  // Handle theme changes
  useEffect(() => {
    const theme = resolveDesktopThemePreference(appearance);
    document.documentElement.setAttribute("data-theme", theme);
  }, [appearance]);

  // Convert DesktopSession to Session format
  const sessions = useMemo(() => {
    return (shell.sessions || []).map((session): import("./components/surfaces").Session => ({
      id: session.id,
      title: describeSession(session).title,
      subtitle: session.metadata?.workspace || undefined,
      lastMessage: session.messages[session.messages.length - 1]?.body || undefined,
      lastActivityAt: new Date(session.updatedAt || session.createdAt),
      status: session.status === "running" ? "active" : session.status === "error" ? "error" : "idle",
      unread: session.unreadCount > 0,
      unreadCount: session.unreadCount,
    }));
  }, [shell.sessions]);

  // Get current session messages
  const currentSession = useMemo(() => {
    return (shell.sessions || []).find((s) => s.id === shell.currentSessionId);
  }, [shell.sessions, shell.currentSessionId]);
  const messages = useMemo((): Message[] => {
    if (!currentSession) return [];
    return currentSession.messages.map((msg): Message => ({
      id: msg.id,
      role: msg.role === "user" ? "user" : msg.role === "assistant" ? "assistant" : "system",
      content: msg.body,
      timestamp: new Date(msg.createdAt),
      isStreaming: msg.isStreaming,
    }));
  }, [currentSession]);

  // Convert notifications
  const notifications = useMemo((): Notification[] => {
    return (shell.notifications || []).map((notif): Notification => ({
      id: notif.id,
      title: notif.title,
      message: notif.body,
      type: notif.type === "error" ? "error" : notif.type === "success" ? "success" : "info",
      read: notif.read,
      createdAt: new Date(notif.createdAt),
      action: notif.actionUrl
        ? {
            label: "View",
            onClick: () => router.navigate(notif.actionUrl!),
          }
        : undefined,
    }));
  }, [shell.notifications, router]);

  // Settings sections
  const settingsSections = useMemo((): SettingSection[] => {
    return [
      {
        id: "language",
        title: t('routes:settings.language.title'),
        description: t('routes:settings.language.description'),
        settings: [
          {
            id: "language-select",
            label: t('routes:settings.language.title'),
            type: "select",
            value: language,
            options: SUPPORTED_LANGUAGES.map(lang => ({
              label: `${lang.flag} ${lang.nativeName}`,
              value: lang.code,
            })),
            onChange: (value) => setLanguage(value as typeof language),
          },
        ],
      },
      {
        id: "appearance",
        title: t('routes:settings.sections.appearance'),
        description: "Customize the look and feel of the app",
        settings: [
          {
            id: "theme",
            label: t('settings:theme.title'),
            description: "Choose your preferred color scheme",
            type: "select",
            value: appearance.theme,
            options: [
              { label: t('settings:theme.system'), value: "system" },
              { label: t('settings:theme.light'), value: "light" },
              { label: t('settings:theme.dark'), value: "dark" },
            ],
            onChange: (value) => {
              const newAppearance = { ...appearance, theme: value as DesktopAppearanceSettings["theme"] };
              setAppearance(newAppearance);
              saveAppearanceSettings(newAppearance);
            },
          },
          {
            id: "sidebar",
            label: "Collapsed Sidebar",
            description: "Show only icons in the sidebar",
            type: "toggle",
            value: sidebarCollapsed,
            onChange: (value) => setSidebarCollapsed(value as boolean),
          },
        ],
      },
      {
        id: "notifications",
        title: t('routes:settings.sections.notifications'),
        settings: [
          {
            id: "desktop",
            label: t('settings:notifications.desktop'),
            description: "Show notifications when the app is in the background",
            type: "toggle",
            value: appearance.notificationsEnabled ?? true,
            onChange: (value) => {
              const newAppearance = { ...appearance, notificationsEnabled: value as boolean };
              setAppearance(newAppearance);
              saveAppearanceSettings(newAppearance);
            },
          },
        ],
      },
    ];
  }, [appearance, sidebarCollapsed, language, setLanguage, t]);

  // Quick actions for home
  const quickActions = useMemo((): QuickAction[] => {
    return [
      {
        id: "new-session",
        label: t('routes:home.actions.newSession'),
        description: "Start a new coding session",
        icon: "💬",
        onClick: () => shell.createSession(),
      },
      {
        id: "resume",
        label: t('routes:home.actions.resume'),
        description: "Continue where you left off",
        icon: "▶️",
        onClick: () => {
          const lastSession = (shell.sessions || [])[0];
          if (lastSession) {
            shell.selectSession(lastSession.id);
          }
        },
      },
      {
        id: "settings",
        label: t('routes:home.actions.settings'),
        description: "Customize your experience",
        icon: "⚙️",
        onClick: () => router.navigate("/(app)/settings"),
      },
    ];
  }, [shell, router, t]);

  // Stats for home
  const stats = useMemo((): StatItem[] => {
    const usage = calculateUsageTotals(shell.usage || []);
    return [
      { label: t('routes:home.sections.recentSessions'), value: (shell.sessions || []).length },
      { label: t('components:composer.send'), value: usage.messages },
      { label: "Active", value: (shell.sessions || []).filter((s) => s.status === "running").length },
    ];
  }, [shell.sessions, shell.usage]);

  // Composer suggestions
  const suggestions = useMemo((): ComposerSuggestion[] => {
    return [
      { id: "continue", label: "Continue", insertText: "Continue" },
      { id: "explain", label: "Explain", insertText: "Can you explain this code?" },
      { id: "refactor", label: "Refactor", insertText: "Refactor this to be more efficient" },
      { id: "test", label: "Add Tests", insertText: "Write tests for this code" },
    ];
  }, []);

  // Handlers
  const handleSendMessage = useCallback(async () => {
    if (!composerValue.trim() || !shell.currentSessionId) return;

    setIsSending(true);
    try {
      await shell.sendMessage(composerValue);
      setComposerValue("");
    } finally {
      setIsSending(false);
    }
  }, [composerValue, shell]);

  const handleSessionSelect = useCallback(
    (session: import("./components/surfaces").Session) => {
      shell.selectSession(session.id);
    },
    [shell],
  );

  const handleMarkNotificationRead = useCallback(
    (id: string) => {
      shell.markNotificationRead(id);
    },
    [shell],
  );

  const handleDismissNotification = useCallback(
    (id: string) => {
      shell.dismissNotification(id);
    },
    [shell],
  );

  // Navigation items
  const unreadCount = notifications.filter((n) => !n.read).length;
  const unreadSessions = sessions.filter((s) => s.unread).length;

  const primaryNavItems = [
    {
      id: "home",
      label: t('components:nav.home'),
      icon: "🏠",
      state: router.path === "/(app)" ? ("active" as const) : ("default" as const),
      onClick: () => router.navigate("/(app)"),
    },
    {
      id: "sessions",
      label: t('components:nav.sessions'),
      icon: "💬",
      badge: unreadSessions,
      state: router.path.startsWith("/(app)/session") ? ("active" as const) : ("default" as const),
      onClick: () => router.navigate("/(app)"),
    },
    {
      id: "inbox",
      label: t('components:nav.inbox'),
      icon: "🔔",
      badge: unreadCount,
      state: router.path === "/(app)/inbox" ? ("active" as const) : ("default" as const),
      onClick: () => router.navigate("/(app)/inbox"),
    },
  ];

  const secondaryNavItems = [
    {
      id: "settings",
      label: t('components:nav.settings'),
      icon: "⚙️",
      state: router.path === "/(app)/settings" ? ("active" as const) : ("default" as const),
      onClick: () => router.navigate("/(app)/settings"),
    },
  ];

  // Determine current view
  const currentView = useMemo(() => {
    if (router.path === "/(app)") return "home";
    if (router.path.startsWith("/(app)/session")) return "session";
    if (router.path === "/(app)/settings") return "settings";
    if (router.path === "/(app)/inbox") return "inbox";
    return "home";
  }, [router.path]);

  // Render content based on route
  const renderContent = () => {
    switch (currentView) {
      case "home":
        return (
          <HomeSurface
            recentSessions={sessions}
            onSessionSelect={handleSessionSelect}
            onNewSession={() => shell.createSession()}
            quickActions={quickActions}
            stats={stats}
          />
        );

      case "session":
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
              {t('routes:session.emptyState.title')}
            </div>
          );
        }
        return (
          <SessionSurface
            session={{
              id: currentSession.id,
              title: describeSession(currentSession).title,
              subtitle: currentSession.metadata?.workspace || undefined,
              lastActivityAt: new Date(currentSession.updatedAt || currentSession.createdAt),
            }}
            messages={messages}
            composerValue={composerValue}
            onComposerChange={setComposerValue}
            onSendMessage={handleSendMessage}
            isSending={isSending}
            suggestions={suggestions}
            onSuggestionSelect={(s) => setComposerValue(s.insertText)}
            actions={[
              { label: t('routes:session.actions.share'), onClick: () => { /* TODO: implement share */ } },
              { label: t('routes:session.actions.export'), onClick: () => { /* TODO: implement export */ } },
            ]}
            models={[
              { id: "gpt-4", name: "GPT-4" },
              { id: "gpt-3.5", name: "GPT-3.5" },
            ]}
            selectedModel={currentSession.model || "gpt-4"}
            onModelChange={(id) => shell.updateSession({ model: id })}
            error={shell.error || undefined}
          />
        );

      case "settings":
        return (
          <SettingsSurface
            sections={settingsSections}
            onSave={() => saveAppearanceSettings(appearance)}
            hasChanges={true}
          />
        );

      case "inbox":
        return (
          <InboxSurface
            notifications={notifications}
            onMarkAsRead={handleMarkNotificationRead}
            onDismiss={handleDismissNotification}
            onMarkAllAsRead={() => shell.markAllNotificationsRead()}
            filter={inboxFilter}
            onFilterChange={setInboxFilter}
          />
        );

      default:
        return null;
    }
  };

  // Mobile layout
  if (isMobile) {
    return (
      <MobileShell
        header={
          currentView === "session" && currentSession ? (
            <MobileNavBar
              title={describeSession(currentSession).title}
              leading={
                <button onClick={() => router.navigate("/(app)")} style={{ fontSize: "1.5rem" }}>
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
          { id: "inbox", label: t('components:nav.inbox'), icon: "🔔", badge: unreadCount },
          { id: "settings", label: t('components:nav.settings'), icon: "⚙️" },
        ]}
        activeTab={currentView}
        onTabChange={(tab) => {
          if (tab === "home") router.navigate("/(app)");
          if (tab === "sessions") router.navigate("/(app)");
          if (tab === "inbox") router.navigate("/(app)/inbox");
          if (tab === "settings") router.navigate("/(app)/settings");
        }}
      >
        {renderContent()}
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
          primarySections={[{ items: primaryNavItems }]}
          secondarySections={[{ items: secondaryNavItems }]}
          connectionStatus={
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <span
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  backgroundColor: shell.connected ? "var(--color-success)" : "var(--color-danger)",
                }}
              />
              {!sidebarCollapsed && (
                <span style={{ fontSize: "0.8125rem", color: "var(--text-tertiary)" }}>
                  {shell.connected ? t('ui:connection.connected') : t('ui:connection.disconnected')}
                </span>
              )}
            </div>
          }
          collapsed={sidebarCollapsed}
        />
      }
      header={
        currentView !== "session" && (
          <Header
            eyebrow={t(`components:nav.${currentView}`)}
            title={
              currentView === "home"
                ? t('common:app.name')
                : currentView === "inbox"
                  ? t('routes:inbox.title')
                  : currentView === "settings"
                    ? t('routes:settings.title')
                    : t('common:app.name')
            }
            size="compact"
          />
        )
      }
      sidebarCollapsed={sidebarCollapsed}
    >
      {currentView === "home" ? (
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
              sessions={sessions}
              selectedId={shell.currentSessionId || undefined}
              onSelect={handleSessionSelect}
              loading={shell.loading}
            />
          </div>

          {/* Main Content */}
          <div style={{ flex: 1, overflow: "auto" }}>{renderContent()}</div>
        </div>
      ) : (
        renderContent()
      )}
    </Shell>
  );
}

export default AppV2;
