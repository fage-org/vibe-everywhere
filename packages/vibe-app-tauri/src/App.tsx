import { Suspense, lazy, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { RuntimeTarget } from "../sources/shared/bootstrap-config";
import {
  buildRuntimeDocumentTitle,
  buildRuntimeMetaDescription,
  resolveRuntimeShellCopy,
  type RuntimeShellCopy,
} from "../sources/app/runtime-shell";
import { useRuntimeBootstrapProfile } from "../sources/app/providers/RuntimeBootstrapProvider";
import {
  firstUsableSlice,
  formatFeatureCount,
  formatModuleCount,
  lockedAuthCallbackRequirements,
  shellInvariants,
  wave8FeatureAreas,
  wave8Modules,
  wave8PriorityBuckets,
} from "./bootstrap";
import {
  desktopVoiceLanguages,
  plannedSurfaceExamples,
  routeReviewNotes,
} from "./mock-data";
import {
  defaultAppearanceSettings,
  defaultLanguageSettings,
  defaultVoiceSettings,
  loadAppearanceSettings,
  loadLanguageSettings,
  loadVoiceSettings,
  resolveDesktopThemePreference,
  saveAppearanceSettings,
  saveLanguageSettings,
  saveVoiceSettings,
  type DesktopAppearanceSettings,
  type DesktopLanguageSettings,
  type DesktopVoiceSettings,
} from "./desktop-preferences";
import {
  mapAccountSettingsToDesktopPreferences,
  mapDesktopPreferencePatchToAccountSettings,
  runOptimisticAccountSync,
} from "./desktop-account-settings";
import {
  desktopHotkeyRoutes,
  isEditableTarget,
  resolveDesktopShellKeyAction,
} from "./desktop-shell-hotkeys";
import {
  DEFAULT_LANGUAGE,
  SUPPORTED_LANGUAGES,
} from "../sources/shared/text/_all";
import {
  DEFAULT_PATH,
  desktopRoutes,
  hrefForPath,
  primaryNavigation,
  promotionNavigation,
  resolveRoute,
  routeInventoryByClass,
  type RouteDefinition,
  type ResolvedRoute,
  useDesktopRouter,
} from "./router";
import { useAppShellState, type AppShellState } from "./useAppShellState";
import changelogData from "../sources/shared/changelog/changelog.json";
import { buildResumeCommand } from "../sources/shared/utils/resumeCommand";
import {
  approveTerminalConnection,
  calculateUsageTotals,
  type UsageBucket,
  type UsagePeriod,
  copyTextToClipboard,
  type DesktopArtifact,
  type DesktopSession,
  type UserProfile,
  describeSession,
  openExternalUrl,
  openTextFileDialog,
  saveTextFileDialog,
  showDesktopNotification,
  type Settings,
  type UiMessage,
} from "./wave8-client";
import type {
  SessionWorkspaceFile,
  SessionWorkspaceFileContent,
} from "./session-files";
import {
  normalizeTerminalPublicKeyInput,
  readTerminalConnectKey,
} from "./terminal-connect";
import {
  clearSessionDraft,
  loadSessionDraft,
  saveSessionDraft,
} from "./session-drafts";
import {
  applyComposerSuggestion,
  buildComposerSuggestions,
  findActiveComposerToken,
  type ComposerSuggestion,
} from "./session-composer-autocomplete";
import {
  clearNewSessionDraft,
  loadNewSessionDraft,
  saveNewSessionDraft,
  type NewSessionDraft,
} from "./new-session-draft";
import {
  getSessionModelOptions,
  getSessionPermissionOptions,
  resolveSessionModeSelection,
} from "./session-mode-options";
import {
  loadSessionPreferences,
  saveSessionPreferences,
  type SessionComposerPreferences,
} from "./session-preferences";
import {
  resolveRuntimeNativeCapabilities,
  type RuntimeNativeCapabilities,
} from "./native-capabilities";
import logoBlack from "../sources/app/assets/images/logo-black.png";
import logoWhite from "../sources/app/assets/images/logo-white.png";
import logotypeDark from "../sources/app/assets/images/logotype-dark.png";
import logotypeLight from "../sources/app/assets/images/logotype-light.png";

const RichTimelineMessageBody = lazy(() =>
  import("./rich-message-renderers").then((module) => ({
    default: module.RichTimelineMessageBody,
  })),
);

type MainViewTab = "sessions" | "inbox" | "settings";

const DESKTOP_PREVIEW_VERSION = "0.1.0-preview";
const NEW_SESSION_DEFAULT_DRAFT: NewSessionDraft = {
  workspace: "/root/vibe-remote",
  model: "gpt-5.4",
  title: "Wave 8 Desktop Session",
  prompt: "Continue the Wave 8 desktop rewrite and report what changed.",
};

type SecondarySurfaceState = {
  artifacts: DesktopArtifact[];
  createArtifact: (input: {
    title: string | null;
    body: string | null;
    sessions?: string[];
    draft?: boolean;
  }) => Promise<DesktopArtifact>;
  updateArtifact: (
    artifactId: string,
    patch: {
      title: string | null;
      body: string | null;
      sessions?: string[];
      draft?: boolean;
    },
  ) => Promise<DesktopArtifact>;
  deleteArtifact: (artifactId: string) => Promise<void>;
  loadArtifact: (artifactId: string) => Promise<DesktopArtifact | null>;
  getSelectedSessionFilePath: (sessionId: string) => string | null;
  selectSessionFilePath: (sessionId: string, relativePath: string) => void;
};

function sessionFileRoutePath(sessionId: string, relativePath: string): string {
  return `/(app)/session/${sessionId}/file?path=${encodeURIComponent(relativePath)}`;
}

function sanitizeDownloadFileName(name: string, fallback: string): string {
  const sanitized = name
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1f]+/g, "-")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "");
  return sanitized || fallback;
}

type DesktopShellProps = {
  path: string;
  commandOpen: boolean;
  onNavigate: (path: string) => void;
  onCommandOpen: () => void;
  onCommandClose: () => void;
  runtimeTarget?: RuntimeTarget;
  hostMode?: "desktop" | "mobile";
};

type DesktopPreferencesState = {
  appearanceSettings: DesktopAppearanceSettings;
  updateAppearanceSettings: (
    patch:
      | Partial<DesktopAppearanceSettings>
      | ((current: DesktopAppearanceSettings) => DesktopAppearanceSettings),
  ) => void;
  voiceSettings: DesktopVoiceSettings;
  updateVoiceSettings: (
    patch:
      | Partial<DesktopVoiceSettings>
      | ((current: DesktopVoiceSettings) => DesktopVoiceSettings),
  ) => void;
  languageSettings: DesktopLanguageSettings;
  updateLanguageSettings: (
    patch:
      | Partial<DesktopLanguageSettings>
      | ((current: DesktopLanguageSettings) => DesktopLanguageSettings),
  ) => void;
  syncAccountSettings: (patch: Partial<Settings>) => Promise<void>;
  accountSettingsSyncing: boolean;
  accountSettingsError: string | null;
  commitAppearancePatch: (patch: Partial<DesktopAppearanceSettings>) => Promise<void>;
  commitVoicePatch: (patch: Partial<DesktopVoiceSettings>) => Promise<void>;
  commitLanguagePatch: (patch: Partial<DesktopLanguageSettings>) => Promise<void>;
};

type MainViewTabConfig = {
  key: MainViewTab;
  label: string;
  eyebrow: string;
};

function buildSettingsFeatureLinks(runtimeCopy: RuntimeShellCopy) {
  return [
    {
      title: "Account",
      subtitle: "Identity, subscription, restore history, and connected service status.",
      route: "/(app)/settings/account",
      badge: "Profile",
    },
    {
      title: "Appearance",
      subtitle: `Theme, layout density, and ${runtimeCopy.surfaceLabel} shell preferences.`,
      route: "/(app)/settings/appearance",
      badge: "Theme",
    },
    {
      title: "Voice Assistant",
      subtitle: `Voice controls, language, and ${runtimeCopy.surfaceLabel} microphone preferences.`,
      route: "/(app)/settings/voice",
      badge: "Audio",
    },
    {
      title: "Features",
      subtitle: `Feature flags and staged rollout controls for the ${runtimeCopy.surfaceLabel} shell.`,
      route: "/(app)/settings/features",
      badge: "Labs",
    },
    {
      title: "Usage",
      subtitle: "Plan, limits, and usage review surfaces used before promotion.",
      route: "/(app)/settings/usage",
      badge: "Quota",
    },
  ] as const;
}

function buildAboutLinks(runtimeCopy: RuntimeShellCopy) {
  return [
    {
      title: "What's New",
      subtitle: `Release notes and migration progress for the ${runtimeCopy.surfaceLabel} shell rebuild.`,
      action: "route" as const,
      value: "/(app)/changelog",
    },
    {
      title: "GitHub",
      subtitle: "fage-org/vibe-everywhere",
      action: "external" as const,
      value: "https://github.com/fage-org/vibe-everywhere",
    },
    {
      title: "Report Issue",
      subtitle: `Open a bug report for ${runtimeCopy.surfaceLabel} shell regressions.`,
      action: "external" as const,
      value: "https://github.com/fage-org/vibe-everywhere/issues",
    },
    {
      title: "Privacy Policy",
      subtitle: "Review the current Vibe privacy policy.",
      action: "external" as const,
      value: "https://app.vibe.engineering",
    },
    {
      title: "Terms of Service",
      subtitle: "Repository-hosted terms used by the current app.",
      action: "external" as const,
      value: "https://github.com/fage-org/vibe-everywhere/blob/main/TERMS.md",
    },
  ] as const;
}

function describeRuntimeLinkRoute(runtimeTarget: RuntimeTarget): string {
  if (runtimeTarget === "desktop") {
    return "Reuse the locked localhost loopback flow to link another desktop.";
  }
  if (runtimeTarget === "mobile") {
    return "Use the shared device-link flow to connect another signed-in device.";
  }
  return "Keep the retained browser export on the same shared create, link, and restore flow.";
}

function buildKeyboardShortcuts(runtimeCopy: RuntimeShellCopy) {
  return [
    { keys: "Ctrl/Cmd+K", description: `Open the ${runtimeCopy.surfaceLabel} route palette` },
    { keys: "?", description: "Open overlay help from the shell" },
    { keys: "Alt+1", description: `Jump to the ${runtimeCopy.surfaceLabel} entry route` },
    { keys: "Alt+2", description: "Jump to Inbox" },
    { keys: "Alt+3", description: "Jump to New Session" },
    { keys: "Alt+4", description: "Jump to Recent Sessions" },
    { keys: "Alt+5", description: "Jump to Settings" },
    { keys: "Alt+6", description: "Jump to Restore" },
    { keys: "Esc", description: "Dismiss the command palette" },
  ] as const;
}

export function App() {
  const { path, navigate } = useDesktopRouter();
  const runtimeProfile = useRuntimeBootstrapProfile();
  const runtimeTarget = runtimeProfile?.runtimeTarget ?? "desktop";
  const [commandOpen, setCommandOpen] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined" || runtimeTarget === "mobile") {
      return undefined;
    }

    const handleKeydown = (event: KeyboardEvent) => {
      const action = resolveDesktopShellKeyAction({
        key: event.key,
        metaKey: event.metaKey,
        ctrlKey: event.ctrlKey,
        altKey: event.altKey,
        targetIsEditable: isEditableTarget(event.target),
      });

      switch (action.type) {
        case "toggle-palette":
          event.preventDefault();
          setCommandOpen((previous) => !previous);
          return;
        case "open-palette":
          event.preventDefault();
          setCommandOpen(true);
          return;
        case "close-palette":
          setCommandOpen(false);
          return;
        case "navigate":
          event.preventDefault();
          navigate(action.path);
          return;
        default:
          return;
      }
    };

    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
    };
  }, [navigate, runtimeTarget]);

  useEffect(() => {
    setCommandOpen(false);
  }, [path]);

  return (
    <DesktopShell
      path={path}
      commandOpen={commandOpen}
      onNavigate={navigate}
      onCommandOpen={() => setCommandOpen(true)}
      onCommandClose={() => setCommandOpen(false)}
      runtimeTarget={runtimeTarget}
      hostMode={runtimeTarget === "mobile" ? "mobile" : "desktop"}
    />
  );
}

export function DesktopShell({
  path,
  commandOpen,
  onNavigate,
  onCommandOpen,
  onCommandClose,
  runtimeTarget = "desktop",
  hostMode = "desktop",
}: DesktopShellProps) {
  const resolved = useMemo(() => resolveRoute(path), [path]);
  const activeRoute = resolved.definition;
  const runtimeCopy = useMemo(
    () => resolveRuntimeShellCopy(runtimeTarget),
    [runtimeTarget],
  );
  const supportsKeyboardShortcuts = runtimeTarget !== "mobile";
  const liveSessionId =
    activeRoute.key === "session-detail" ? resolved.params.id ?? null : null;
  const appState = useAppShellState(liveSessionId);
  const desktop = appState;
  const ownerArea = wave8FeatureAreas.find(
    (area) => area.ownerModule === activeRoute.ownerModule,
  );
  const routePaletteRef = useRef<HTMLInputElement | null>(null);
  const [paletteQuery, setPaletteQuery] = useState("");
  const [appearanceSettings, setAppearanceSettings] = useState<DesktopAppearanceSettings>(
    () => loadAppearanceSettings(),
  );
  const [voiceSettings, setVoiceSettings] = useState<DesktopVoiceSettings>(
    () => loadVoiceSettings(),
  );
  const [languageSettings, setLanguageSettings] = useState<DesktopLanguageSettings>(
    () => loadLanguageSettings(),
  );
  const [accountSettingsSyncing, setAccountSettingsSyncing] = useState(false);
  const [accountSettingsError, setAccountSettingsError] = useState<string | null>(null);
  const accountSettingsSyncGenerationRef = useRef(0);
  const [selectedSessionFilePaths, setSelectedSessionFilePaths] = useState<Record<string, string>>(
    {},
  );
  const resolvedTheme = useMemo(() => {
    const mediaQuery =
      typeof window !== "undefined" && typeof window.matchMedia === "function"
        ? window.matchMedia("(prefers-color-scheme: dark)")
        : null;
    return resolveDesktopThemePreference(appearanceSettings.themePreference, mediaQuery?.matches);
  }, [appearanceSettings.themePreference]);
  const brandLogo = resolvedTheme === "light" ? logoBlack : logoWhite;
  const brandLogotype = resolvedTheme === "light" ? logotypeDark : logotypeLight;
  const mainViewTabs = useMemo<ReadonlyArray<MainViewTabConfig>>(
    () => [
      { key: "sessions" as const, label: "Sessions", eyebrow: "Live work" },
      { key: "inbox" as const, label: "Inbox", eyebrow: "Updates" },
      { key: "settings" as const, label: "Settings", eyebrow: runtimeCopy.settingsEyebrow },
    ],
    [runtimeCopy.settingsEyebrow],
  );
  const keyboardShortcuts = useMemo(
    () => buildKeyboardShortcuts(runtimeCopy),
    [runtimeCopy],
  );
  const nativeCapabilities = useMemo(
    () => resolveRuntimeNativeCapabilities(runtimeTarget),
    [runtimeTarget],
  );

  const syncOptimisticPreference = useCallback(
    async <T,>(
      previousState: T,
      nextState: T,
      setState: (value: T) => void,
      accountPatch: Partial<Settings>,
    ) => {
      setState(nextState);

      if (!desktop.credentials || Object.keys(accountPatch).length === 0) {
        return;
      }

      const generation = ++accountSettingsSyncGenerationRef.current;
      setAccountSettingsSyncing(true);
      setAccountSettingsError(null);

      const result = await runOptimisticAccountSync({
        previousState,
        nextState,
        accountPatch,
        syncAccountSettings: async (patch) => {
          await desktop.updateAccountSettings(patch);
        },
      });

      if (generation === accountSettingsSyncGenerationRef.current) {
        if (result.rolledBack) {
          setState(result.finalState);
          setAccountSettingsError(result.error);
        }
        setAccountSettingsSyncing(false);
      }
    },
    [desktop.credentials, desktop.updateAccountSettings],
  );

  const desktopPreferences: DesktopPreferencesState = {
    appearanceSettings,
    updateAppearanceSettings: (patch) => {
      setAppearanceSettings((current) =>
        typeof patch === "function" ? patch(current) : { ...current, ...patch },
      );
    },
    voiceSettings,
    updateVoiceSettings: (patch) => {
      setVoiceSettings((current) =>
        typeof patch === "function" ? patch(current) : { ...current, ...patch },
      );
    },
    languageSettings,
    updateLanguageSettings: (patch) => {
      setLanguageSettings((current) =>
        typeof patch === "function" ? patch(current) : { ...current, ...patch },
      );
    },
    syncAccountSettings: async (patch) => {
      if (!desktop.credentials) {
        return;
      }
      setAccountSettingsSyncing(true);
      setAccountSettingsError(null);
      try {
        await desktop.updateAccountSettings(patch);
      } catch (error) {
        setAccountSettingsError(
          error instanceof Error ? error.message : "Failed to sync desktop account settings",
        );
      } finally {
        setAccountSettingsSyncing(false);
      }
    },
    accountSettingsSyncing,
    accountSettingsError,
    commitAppearancePatch: async (patch) => {
      const previousState = appearanceSettings;
      const nextState = { ...appearanceSettings, ...patch };
      await syncOptimisticPreference(
        previousState,
        nextState,
        setAppearanceSettings,
        mapDesktopPreferencePatchToAccountSettings({ appearancePatch: patch }),
      );
    },
    commitVoicePatch: async (patch) => {
      const previousState = voiceSettings;
      const nextState = { ...voiceSettings, ...patch };
      await syncOptimisticPreference(
        previousState,
        nextState,
        setVoiceSettings,
        mapDesktopPreferencePatchToAccountSettings({ voicePatch: patch }),
      );
    },
    commitLanguagePatch: async (patch) => {
      const previousState = languageSettings;
      const nextState = { ...languageSettings, ...patch };
      await syncOptimisticPreference(
        previousState,
        nextState,
        setLanguageSettings,
        mapDesktopPreferencePatchToAccountSettings({ languagePatch: patch }),
      );
    },
  };

  const getSelectedSessionFilePath = (sessionId: string) =>
    selectedSessionFilePaths[sessionId] ?? null;

  const selectSessionFilePath = (sessionId: string, relativePath: string) => {
    setSelectedSessionFilePaths((current) => ({
      ...current,
      [sessionId]: relativePath,
    }));
  };

  const secondarySurfaces: SecondarySurfaceState = {
    artifacts: desktop.artifacts,
    createArtifact: desktop.createArtifact,
    updateArtifact: desktop.updateArtifact,
    deleteArtifact: desktop.deleteArtifact,
    loadArtifact: desktop.loadArtifact,
    getSelectedSessionFilePath,
    selectSessionFilePath,
  };
  const settingsFeatureLinks = useMemo(
    () => buildSettingsFeatureLinks(runtimeCopy),
    [runtimeCopy],
  );
  const aboutLinks = useMemo(() => buildAboutLinks(runtimeCopy), [runtimeCopy]);

  const sessionRoutes = useMemo(
    () =>
      desktop.sessionSummaries.map(({ session, title, subtitle }) => ({
        key: `session-${session.id}`,
        label: title,
        title: `Session ${title}`,
        pattern: "/(app)/session/[id]",
        examplePath: `/(app)/session/${session.id}`,
        summary: subtitle,
        promotionClass: "P0" as const,
        ownerModule: "session-ui-parity" as const,
        section: "Session" as const,
        status: "wired" as const,
      })),
    [desktop.sessionSummaries],
  );

  const paletteRoutes = useMemo(() => {
    const query = paletteQuery.trim().toLowerCase();
    const candidates = [...desktopRoutes, ...sessionRoutes];
    if (!query) {
      return candidates;
    }

    return candidates.filter((route) => {
      const searchText = [
        route.label,
        route.title,
        route.pattern,
        route.summary,
        route.ownerModule,
      ]
        .join(" ")
        .toLowerCase();
      return searchText.includes(query);
    });
  }, [paletteQuery, sessionRoutes]);

  useEffect(() => {
    if (!commandOpen) {
      setPaletteQuery("");
      return;
    }
    routePaletteRef.current?.focus();
  }, [commandOpen]);

  useEffect(() => {
    if (typeof document === "undefined") {
      return undefined;
    }
    document.body.classList.toggle("dialog-open", commandOpen);
    return () => {
      document.body.classList.remove("dialog-open");
    };
  }, [commandOpen]);

  useEffect(() => {
    saveAppearanceSettings(appearanceSettings);
  }, [appearanceSettings]);

  useEffect(() => {
    saveVoiceSettings(voiceSettings);
  }, [voiceSettings]);

  useEffect(() => {
    saveLanguageSettings(languageSettings);
  }, [languageSettings]);

  useEffect(() => {
    const mapped = mapAccountSettingsToDesktopPreferences({
      accountSettings: desktop.accountSettings,
      currentAppearance: appearanceSettings,
      currentVoice: voiceSettings,
      currentLanguage: languageSettings,
    });
    setAppearanceSettings((current) => ({ ...current, ...mapped.appearance }));
    setVoiceSettings((current) => ({ ...current, ...mapped.voice }));
    setLanguageSettings((current) => ({ ...current, ...mapped.language }));
  }, [desktop.accountSettingsVersion]);

  useEffect(() => {
    if (typeof document === "undefined") {
      return undefined;
    }

    const root = document.documentElement;

    root.dataset.theme = resolvedTheme;
    root.dataset.density = appearanceSettings.density;
    root.lang = languageSettings.appLanguage;

    return () => {
      delete root.dataset.theme;
      delete root.dataset.density;
      root.removeAttribute("lang");
    };
  }, [appearanceSettings.density, languageSettings.appLanguage, resolvedTheme]);

  useEffect(() => {
    if (typeof document === "undefined") {
      return undefined;
    }

    document.title = buildRuntimeDocumentTitle(runtimeTarget, activeRoute.title);

    let meta = document.querySelector('meta[name="description"]');
    let created = false;
    if (!meta) {
      meta = document.createElement("meta");
      meta.setAttribute("name", "description");
      document.head.appendChild(meta);
      created = true;
    }
    meta.setAttribute("content", buildRuntimeMetaDescription(runtimeTarget));

    return () => {
      if (created) {
        meta?.remove();
      }
    };
  }, [activeRoute.title, runtimeTarget]);

  if (hostMode === "mobile") {
    return (
      <MobileShellLayout
        resolved={resolved}
        appState={appState}
        preferences={desktopPreferences}
        secondarySurfaces={secondarySurfaces}
        onNavigate={onNavigate}
        runtimeTarget={runtimeTarget}
        runtimeCopy={runtimeCopy}
        nativeCapabilities={nativeCapabilities}
        settingsFeatureLinks={settingsFeatureLinks}
        aboutLinks={aboutLinks}
        brandLogoSrc={brandLogo}
      />
    );
  }

  return (
    <div
      className={`desktop-app-shell ${
        appearanceSettings.compactSessionView ? "desktop-shell-compact" : ""
      }`}
    >
      <a className="skip-link" href="#main-panel">
        Skip to main content
      </a>
      <div className="ambient ambient-left" />
      <div className="ambient ambient-right" />

      <aside className="sidebar">
        <div className="brand-block">
          <div className="brand-lockup">
            <img className="brand-mark" src={brandLogo} alt="" aria-hidden="true" />
            <div className="brand-copy-block">
              <img className="brand-logotype" src={brandLogotype} alt="Vibe" />
              <p className="eyebrow">{runtimeCopy.shellEyebrow}</p>
            </div>
          </div>
          <p className="brand-copy">
            {runtimeCopy.shellSummary}
          </p>
          <div className="pill-row shell-status-row">
            <span className="pill pill-accent">{statusLabel(desktop.status)}</span>
            <span className="pill pill-muted">{formatModuleCount(wave8Modules)}</span>
            <span className="pill pill-outline">{formatFeatureCount(wave8FeatureAreas)}</span>
          </div>
        </div>

        <nav className="nav-block" aria-label={runtimeCopy.primaryNavLabel}>
          <div className="section-heading">
            <span>Navigate</span>
            <small>Sessions first</small>
          </div>
          <div className="nav-list">
            {primaryNavigation.map((route) => (
              <SidebarLink
                key={route.key}
                route={route}
                active={route.key === activeRoute.key}
                onNavigate={onNavigate}
                shortcut={shortcutForRoute(route.examplePath)}
              />
            ))}
            {desktop.sessionSummaries.slice(0, 2).map(({ session, title }) => (
              <SidebarLink
                key={session.id}
                route={resolveRoute(`/(app)/session/${session.id}`).definition}
                labelOverride={title}
                active={path === `/(app)/session/${session.id}`}
                onNavigate={onNavigate}
                pathOverride={`/(app)/session/${session.id}`}
              />
            ))}
          </div>
        </nav>

        <section className="nav-block">
          <div className="section-heading">
            <span>More routes</span>
            <small>Settings and tools</small>
          </div>
          <div className="nav-list compact-list">
            {promotionNavigation.map((route) => (
              <SidebarLink
                key={route.key}
                route={route}
                active={route.key === activeRoute.key}
                onNavigate={onNavigate}
              />
            ))}
          </div>
        </section>

        <section className="nav-block meta-block">
          <div className="section-heading">
            <span>Connection</span>
            <small>{`Current ${runtimeCopy.surfaceTitle} state`}</small>
          </div>
          <div className="mini-card status-summary-card">
            <div className="mini-card-header">
              <strong>{statusLabel(desktop.status)}</strong>
              <span>{desktop.credentials ? "Authenticated" : "No credentials"}</span>
            </div>
            <p>
              {desktop.profile
                ? `Account ${desktop.profile.id} connected to ${desktop.profile.connectedServices.length} services.`
                : "Sign in to load profile, sessions, and live message history."}
            </p>
          </div>
          <ul className="bullet-list dense-list">
            {shellInvariants.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </section>
      </aside>

      <div className="workspace-shell">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeRoute.section}</p>
            <h2>{activeRoute.title}</h2>
            <p className="topbar-copy">{activeRoute.summary}</p>
          </div>
          <div className="topbar-actions">
            <div className="pill-row right-aligned">
              <span className="pill pill-muted">{statusLabel(desktop.status)}</span>
              <span className={`pill pill-priority pill-${activeRoute.promotionClass.toLowerCase()}`}>
                {activeRoute.promotionClass}
              </span>
              <span className="pill pill-outline">{activeRoute.ownerModule}</span>
            </div>
            <button className="command-trigger" type="button" onClick={onCommandOpen}>
              {supportsKeyboardShortcuts ? "Open Palette" : "Open Routes"}
              {supportsKeyboardShortcuts ? <span>Ctrl/Cmd+K</span> : null}
            </button>
          </div>
        </header>

        <div className="workspace-grid">
          <main className="main-panel" id="main-panel" tabIndex={-1}>
            {desktop.globalError ? (
              <ErrorBanner
                message={desktop.globalError}
                actionLabel={
                  desktop.storedSessionAvailable ? "Retry stored session" : undefined
                }
                onAction={
                  desktop.storedSessionAvailable
                    ? () => void desktop.retryStoredSession()
                    : undefined
                }
              />
            ) : null}
            <RouteSurface
              resolved={resolved}
              desktop={desktop}
              preferences={desktopPreferences}
              secondarySurfaces={secondarySurfaces}
              onNavigate={onNavigate}
              brandLogoSrc={brandLogo}
              brandLogotypeSrc={brandLogotype}
              runtimeTarget={runtimeTarget}
              runtimeCopy={runtimeCopy}
              nativeCapabilities={nativeCapabilities}
              mainViewTabs={mainViewTabs}
              settingsFeatureLinks={settingsFeatureLinks}
              aboutLinks={aboutLinks}
            />
          </main>

          <aside className="inspector-panel">
            <section className="panel-card inspector-route">
              <div className="card-header">
                <h3>Route details</h3>
                <span className={`status-dot status-${activeRoute.status}`}>
                  {activeRoute.status}
                </span>
              </div>
              <dl className="meta-grid">
                <div>
                  <dt>Canonical path</dt>
                  <dd>{resolved.canonicalPath}</dd>
                </div>
                <div>
                  <dt>Pattern</dt>
                  <dd>{activeRoute.pattern}</dd>
                </div>
                <div>
                  <dt>Owner</dt>
                  <dd>{activeRoute.ownerModule}</dd>
                </div>
                <div>
                  <dt>Promotion class</dt>
                  <dd>{activeRoute.promotionClass}</dd>
                </div>
              </dl>
              {ownerArea ? (
                <div className="inspector-stack">
                  <h4>{ownerArea.title}</h4>
                  <p>{ownerArea.description}</p>
                  <ul className="bullet-list dense-list">
                    {ownerArea.features.slice(0, 4).map((feature) => (
                      <li key={feature}>{feature}</li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </section>

            <section className="panel-card">
              <div className="card-header">
                <h3>Parity scope</h3>
                <span className="pill pill-muted">
                  {formatFeatureCount(wave8FeatureAreas)}
                </span>
              </div>
              <div className="priority-stack">
                {wave8PriorityBuckets.map((bucket) => (
                  <article key={bucket.priority} className="mini-card">
                    <div className="mini-card-header">
                      <strong>{bucket.priority}</strong>
                      <span>{bucket.title}</span>
                    </div>
                    <ul className="bullet-list dense-list">
                      {bucket.features.slice(0, 3).map((feature) => (
                        <li key={feature}>{feature}</li>
                      ))}
                    </ul>
                  </article>
                ))}
              </div>
            </section>

            <section className="panel-card">
              <div className="card-header">
                <h3>Navigation map</h3>
                <span className="pill pill-outline">All classes visible</span>
              </div>
              <div className="inventory-group">
                {Object.entries(routeInventoryByClass).map(([group, routes]) => (
                  <div key={group} className="inventory-bucket">
                    <div className="inventory-bucket-header">
                      <strong>{group}</strong>
                      <span>{routes.length} routes</span>
                    </div>
                    <ul className="inventory-list">
                      {routes.map((route) => (
                        <li key={route.key}>
                          <a
                            href={hrefForPath(route.examplePath)}
                            onClick={(event) => handleNavigation(event, route.examplePath, onNavigate)}
                            className={route.key === activeRoute.key ? "inventory-link active" : "inventory-link"}
                          >
                            <span>{route.label}</span>
                            <small>{route.status}</small>
                          </a>
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </section>

            {supportsKeyboardShortcuts ? (
              <section className="panel-card">
                <div className="card-header">
                  <h3>Keyboard shortcuts</h3>
                  <span className="pill pill-outline">Focus-safe</span>
                </div>
                <ul className="shortcut-list">
                  {keyboardShortcuts.map((shortcut) => (
                    <li key={shortcut.keys}>
                      <kbd>{shortcut.keys}</kbd>
                      <span>{shortcut.description}</span>
                    </li>
                  ))}
                </ul>
              </section>
            ) : null}
          </aside>
        </div>
      </div>

      {commandOpen ? (
        <div className="overlay-shell" role="presentation" onClick={onCommandClose}>
          <section
            className="command-palette"
            role="dialog"
            aria-modal="true"
            aria-labelledby="command-palette-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="card-header">
              <div>
                <p className="eyebrow">Overlay palette</p>
                <h3 id="command-palette-title">Go to route or session</h3>
              </div>
              <button className="ghost-button" type="button" onClick={onCommandClose}>
                Close
              </button>
            </div>
            <label className="palette-search">
              <span>Search routes and live sessions</span>
              <input
                ref={routePaletteRef}
                type="text"
                value={paletteQuery}
                onChange={(event) => setPaletteQuery(event.target.value)}
                placeholder="Search by route, title, module, or summary"
              />
            </label>
            <div className="palette-results">
              {paletteRoutes.length > 0 ? (
                paletteRoutes.map((route) => (
                  <button
                    key={`${route.key}-${route.examplePath}`}
                    type="button"
                    className="palette-result"
                    onClick={() => {
                      onNavigate(route.examplePath);
                      onCommandClose();
                    }}
                  >
                    <div>
                      <strong>{route.label}</strong>
                      <p>{route.summary}</p>
                    </div>
                    <span>{route.pattern}</span>
                  </button>
                ))
              ) : (
                <div className="empty-state compact-empty">
                  <h4>No routes match</h4>
                  <p>Try searching by promotion class, module, or canonical path.</p>
                </div>
              )}
            </div>
          </section>
        </div>
      ) : null}
    </div>
  );
}

function resolveMobilePrimaryTab(route: RouteDefinition): MainViewTab | null {
  if (route.key === "home") {
    return "sessions";
  }
  if (route.key === "inbox") {
    return "inbox";
  }
  if (route.key === "settings-index") {
    return "settings";
  }
  if (route.section === "Session") {
    return "sessions";
  }
  return null;
}

function pathForMobilePrimaryTab(tab: MainViewTab): string {
  switch (tab) {
    case "inbox":
      return "/(app)/inbox/index";
    case "settings":
      return "/(app)/settings/index";
    case "sessions":
    default:
      return "/(app)/index";
  }
}

function buildMobileBackPath(route: RouteDefinition): string {
  if (route.section === "Settings") {
    return "/(app)/settings/index";
  }
  if (route.section === "Auth") {
    return "/(app)/restore/index";
  }
  return "/(app)/index";
}

type MobileShellLayoutProps = {
  resolved: ResolvedRoute;
  appState: AppShellState;
  preferences: DesktopPreferencesState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
  runtimeTarget: RuntimeTarget;
  runtimeCopy: RuntimeShellCopy;
  nativeCapabilities: RuntimeNativeCapabilities;
  settingsFeatureLinks: ReturnType<typeof buildSettingsFeatureLinks>;
  aboutLinks: ReturnType<typeof buildAboutLinks>;
  brandLogoSrc: string;
};

function MobileShellLayout({
  resolved,
  appState,
  preferences,
  secondarySurfaces,
  onNavigate,
  runtimeTarget,
  runtimeCopy,
  nativeCapabilities,
  settingsFeatureLinks,
  aboutLinks,
  brandLogoSrc,
}: MobileShellLayoutProps) {
  const activePrimaryTab = resolveMobilePrimaryTab(resolved.definition);
  const signedIn = !!appState.credentials && !!appState.profile;
  let content: React.ReactNode;
  const isTopLevelSessionsRoute = resolved.definition.key === "home";
  const isTopLevelInboxRoute = resolved.definition.key === "inbox";
  const isTopLevelSettingsRoute = resolved.definition.key === "settings-index";
  const isTopLevelPrimaryRoute =
    isTopLevelSessionsRoute || isTopLevelInboxRoute || isTopLevelSettingsRoute;
  const mobileRouteAction =
    resolved.definition.key === "session-detail"
      ? { label: "Info", path: `/(app)/session/${resolved.params.id ?? ""}/info` }
      : resolved.definition.key === "session-message"
        ? { label: "Session", path: `/(app)/session/${resolved.params.id ?? ""}` }
        : resolved.definition.key === "session-info"
          ? { label: "Files", path: `/(app)/session/${resolved.params.id ?? ""}/files` }
          : resolved.definition.key === "session-files"
            ? { label: "Session", path: `/(app)/session/${resolved.params.id ?? ""}` }
            : resolved.definition.key === "session-file"
              ? { label: "Files", path: `/(app)/session/${resolved.params.id ?? ""}/files` }
              : null;
  const routeTitle =
    !isTopLevelPrimaryRoute
      ? resolved.definition.title
      : activePrimaryTab === "sessions"
        ? "Sessions"
        : activePrimaryTab === "inbox"
          ? "Inbox"
          : activePrimaryTab === "settings"
            ? "Settings"
            : resolved.definition.title;

  if (!signedIn && activePrimaryTab && (isTopLevelSessionsRoute || isTopLevelInboxRoute || isTopLevelSettingsRoute)) {
    content = (
      <MobileHomeSurface
        appState={appState}
        onNavigate={onNavigate}
        brandLogoSrc={brandLogoSrc}
        runtimeCopy={runtimeCopy}
      />
    );
  } else if (signedIn && activePrimaryTab === "sessions" && isTopLevelSessionsRoute) {
    content = <MobileSessionsSurface appState={appState} onNavigate={onNavigate} runtimeCopy={runtimeCopy} />;
  } else if (signedIn && activePrimaryTab === "inbox" && isTopLevelInboxRoute) {
    content = <MobileInboxSurface appState={appState} onNavigate={onNavigate} runtimeCopy={runtimeCopy} />;
  } else if (signedIn && activePrimaryTab === "settings" && isTopLevelSettingsRoute) {
    content = (
      <MobileSettingsSurface
        appState={appState}
        onNavigate={onNavigate}
        runtimeCopy={runtimeCopy}
        settingsFeatureLinks={settingsFeatureLinks}
        aboutLinks={aboutLinks}
      />
    );
  } else {
    content = (
      <RouteSurface
        resolved={resolved}
        desktop={appState}
        preferences={preferences}
        secondarySurfaces={secondarySurfaces}
        onNavigate={onNavigate}
        brandLogoSrc={brandLogoSrc}
        brandLogotypeSrc={brandLogoSrc}
        runtimeTarget={runtimeTarget}
        runtimeCopy={runtimeCopy}
        nativeCapabilities={nativeCapabilities}
        mainViewTabs={[
          { key: "sessions", label: "Sessions", eyebrow: "Live work" },
          { key: "inbox", label: "Inbox", eyebrow: "Updates" },
          { key: "settings", label: "Settings", eyebrow: runtimeCopy.settingsEyebrow },
        ]}
        settingsFeatureLinks={settingsFeatureLinks}
        aboutLinks={aboutLinks}
      />
    );
  }

  return (
    <div className="mobile-app-shell">
      <header className="mobile-topbar">
        <div className="mobile-topbar-copy">
          {activePrimaryTab && isTopLevelPrimaryRoute ? (
            <>
              <p className="eyebrow">{runtimeCopy.shellEyebrow}</p>
              <h2>{routeTitle}</h2>
            </>
          ) : (
            <>
              <button
                type="button"
                className="ghost-button mobile-back-button"
                onClick={() => onNavigate(buildMobileBackPath(resolved.definition))}
              >
                Back
              </button>
              <div>
                <p className="eyebrow">{runtimeCopy.surfaceTitle}</p>
                <h2>{routeTitle}</h2>
              </div>
            </>
          )}
        </div>
        <div className="mobile-topbar-actions">
          <span className="pill pill-accent">{statusLabel(appState.status)}</span>
          {!isTopLevelPrimaryRoute && mobileRouteAction ? (
            <button
              type="button"
              className="ghost-button mobile-topbar-button"
              onClick={() => onNavigate(mobileRouteAction.path)}
            >
              {mobileRouteAction.label}
            </button>
          ) : null}
          {signedIn && activePrimaryTab === "sessions" && isTopLevelPrimaryRoute ? (
            <button type="button" className="ghost-button mobile-topbar-button" onClick={() => onNavigate("/(app)/new/index")}>
              New
            </button>
          ) : null}
          {signedIn && activePrimaryTab === "inbox" && isTopLevelPrimaryRoute ? (
            <button type="button" className="ghost-button mobile-topbar-button" onClick={() => onNavigate("/(app)/friends/search")}>
              Add friend
            </button>
          ) : null}
        </div>
      </header>

      <main className="mobile-main-panel">{content}</main>

      {signedIn && activePrimaryTab && isTopLevelPrimaryRoute ? (
        <nav className="mobile-tabbar" aria-label={`${runtimeCopy.surfaceTitle} primary navigation`}>
          {(["inbox", "sessions", "settings"] as const).map((tab) => {
            const active = activePrimaryTab === tab;
            return (
              <a
                key={tab}
                href={hrefForPath(pathForMobilePrimaryTab(tab))}
                className={active ? "mobile-tab active" : "mobile-tab"}
                aria-current={active ? "page" : undefined}
                onClick={(event) => handleNavigation(event, pathForMobilePrimaryTab(tab), onNavigate)}
              >
                <span>
                  {tab === "sessions" ? "Sessions" : tab === "inbox" ? "Inbox" : "Settings"}
                </span>
              </a>
            );
          })}
        </nav>
      ) : null}
    </div>
  );
}

function MobileHomeSurface({
  appState,
  onNavigate,
  brandLogoSrc,
  runtimeCopy,
}: {
  appState: AppShellState;
  onNavigate: (path: string) => void;
  brandLogoSrc: string;
  runtimeCopy: RuntimeShellCopy;
}) {
  return (
    <div className="surface-stack">
      <section className="panel-card mobile-hero-card mobile-entry-card">
        <div className="mobile-brand-lockup">
          <img className="hero-brand-mark" src={brandLogoSrc} alt="" aria-hidden="true" />
          <div>
            <p className="eyebrow">{runtimeCopy.entryEyebrow}</p>
            <h3>{runtimeCopy.createRestoreHeading}</h3>
          </div>
        </div>
        <p className="panel-copy">{runtimeCopy.createRestoreCopy}</p>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="primary-button full-width"
            onClick={() => void appState.createFreshAccount()}
          >
            Create account
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/restore/index")}
          >
            Restore or link account
          </button>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>{runtimeCopy.essentialsTitle}</h3>
          <span className="pill pill-accent">Phone-first</span>
        </div>
        <ul className="bullet-list">
          <li>Primary navigation stays on inbox, sessions, and settings.</li>
          <li>Create and restore stay available from the Android entry flow.</li>
          <li>The mobile host no longer depends on the desktop sidebar shell.</li>
        </ul>
      </section>
    </div>
  );
}

function MobileSessionsSurface({
  appState,
  onNavigate,
  runtimeCopy,
}: {
  appState: AppShellState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
}) {
  if (!appState.credentials || !appState.profile) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const topSessions = appState.sessionSummaries.slice(0, 8);
  const isLoadingSessions = appState.status === "checking" || appState.status === "loading";
  return (
    <div className="surface-stack">
      <section className="panel-card mobile-hero-card">
        <p className="eyebrow">{runtimeCopy.entryEyebrow}</p>
        <h3>{runtimeCopy.continueHeading}</h3>
        <p className="panel-copy">{runtimeCopy.continueCopy}</p>
        <div className="button-row">
          <button type="button" className="primary-button full-width" onClick={() => onNavigate("/(app)/new/index")}>
            Create session
          </button>
        </div>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Quick actions</h3>
          <span className="pill pill-outline">Main view</span>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/session/recent")}
          >
            <div className="settings-row-copy">
              <strong>Recent sessions</strong>
              <p>Review the latest session inventory and resume flows.</p>
            </div>
            <span className="settings-row-badge">Open</span>
          </button>
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/restore/index")}
          >
            <div className="settings-row-copy">
              <strong>Link another device</strong>
              <p>Restore or link another shell without leaving the mobile main view.</p>
            </div>
            <span className="settings-row-badge">Auth</span>
          </button>
        </div>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Status</h3>
          <span className="pill pill-accent">Live</span>
        </div>
        <dl className="meta-grid compact-meta-grid">
          <div>
            <dt>Account</dt>
            <dd>{appState.profile.firstName || appState.profile.username || appState.profile.id}</dd>
          </div>
          <div>
            <dt>Connected services</dt>
            <dd>{appState.profile.connectedServices.length}</dd>
          </div>
          <div>
            <dt>Sessions</dt>
            <dd>{appState.sessions.length}</dd>
          </div>
          <div>
            <dt>Endpoint</dt>
            <dd>{appState.serverUrl}</dd>
          </div>
        </dl>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Recent sessions</h3>
          <span className="pill pill-accent">{appState.sessions.length} loaded</span>
        </div>
        {isLoadingSessions ? (
          <div className="empty-state compact-empty">
            <h4>Loading sessions</h4>
            <p className="panel-copy">Fetching the latest Android session inventory.</p>
          </div>
        ) : topSessions.length > 0 ? (
          <div className="settings-list">
            {topSessions.map(({ session, title, detail }) => (
              <button
                key={session.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/session/${session.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{title}</strong>
                  <p>{detail}</p>
                </div>
                <span className="settings-row-badge">{session.active ? "Active" : "Open"}</span>
              </button>
            ))}
          </div>
        ) : (
          <div className="empty-state compact-empty">
            <h4>No sessions yet</h4>
            <p className="panel-copy">
              {`Create the first ${runtimeCopy.surfaceLabel} session to verify the live backend path end to end.`}
            </p>
            <div className="button-row mobile-button-column">
              <button type="button" className="primary-button full-width" onClick={() => onNavigate("/(app)/new/index")}>
                Create session
              </button>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

function MobileInboxSurface({
  appState,
  onNavigate,
  runtimeCopy,
}: {
  appState: AppShellState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
}) {
  if (!appState.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const updateItems = appState.feedItems.map((item) => {
    const body = item.body;
    if (body.kind === "text") {
      return {
        key: item.id,
        title: body.text,
        subtitle: new Date(item.createdAt).toLocaleString(),
        actionPath: "/(app)/inbox/index",
        badge: "Update",
      };
    }

    const relatedUser = appState.friends.find((friend) => friend.id === body.uid);
    return {
      key: item.id,
      title:
        body.kind === "friend_request"
          ? `Friend request from ${relatedUser?.firstName || relatedUser?.username || body.uid}`
          : `Friend accepted: ${relatedUser?.firstName || relatedUser?.username || body.uid}`,
      subtitle: new Date(item.createdAt).toLocaleString(),
      actionPath: `/(app)/user/${body.uid}`,
      badge: body.kind === "friend_request" ? "Request" : "Friend",
    };
  });

  const pendingFriends = appState.friends.filter((friend) => friend.status === "pending");
  const requestedFriends = appState.friends.filter((friend) => friend.status === "requested");
  const acceptedFriends = appState.friends.filter((friend) => friend.status === "friend");

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-hero-card">
        <p className="eyebrow">Inbox</p>
        <h3>Updates and account activity</h3>
        <p className="panel-copy">
          Review updates, friend activity, and account-linked events in the {runtimeCopy.surfaceLabel} shell.
        </p>
        <div className="pill-row">
          <span className="pill pill-outline">{`${updateItems.length} updates`}</span>
          <span className="pill pill-outline">{`${pendingFriends.length + requestedFriends.length} requests`}</span>
          <span className="pill pill-accent">{`${acceptedFriends.length} friends`}</span>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Updates</h3>
          <span className="pill pill-outline">Feed</span>
        </div>
        {updateItems.length > 0 ? (
          <div className="settings-list">
            {updateItems.map((item) => (
              <button
                key={item.key}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(item.actionPath)}
              >
                <div className="settings-row-copy">
                  <strong>{item.title}</strong>
                  <p>{item.subtitle}</p>
                </div>
                <span className="settings-row-badge">{item.badge}</span>
              </button>
            ))}
          </div>
        ) : (
          <EmptyState
            title="No updates yet"
            body={`Create a session or update an artifact to populate the ${runtimeCopy.surfaceLabel} inbox feed.`}
            actionLabel="Create session"
            onAction={() => onNavigate("/(app)/new/index")}
          />
        )}
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Pending requests</h3>
          <span className="pill pill-outline">Social</span>
        </div>
        {pendingFriends.length > 0 || requestedFriends.length > 0 ? (
          <div className="settings-list">
            {pendingFriends.map((friend) => (
              <button
                key={`pending-${friend.id}`}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge">Pending</span>
              </button>
            ))}
            {requestedFriends.map((friend) => (
              <button
                key={`requested-${friend.id}`}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge">Requested</span>
              </button>
            ))}
          </div>
        ) : (
          <div className="settings-list">
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>No pending friend requests</strong>
                <p>Friend requests will appear here when social updates are available.</p>
              </div>
              <span className="settings-row-badge">Empty</span>
            </div>
          </div>
        )}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/friends/search")}
          >
            Find friends
          </button>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>My friends</h3>
          <span className="pill pill-accent">Account</span>
        </div>
        <div className="settings-list">
          {acceptedFriends.length > 0 ? (
            acceptedFriends.map((friend) => (
              <button
                key={friend.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge connected">Friend</span>
              </button>
            ))
          ) : (
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>No friends yet</strong>
                <p>Accepted friends will appear here once the social graph is populated.</p>
              </div>
              <span className="settings-row-badge">Empty</span>
            </div>
          )}
        </div>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/friends/index")}
          >
            Open friends
          </button>
        </div>
      </section>
    </div>
  );
}

function MobileSettingsSurface({
  appState,
  onNavigate,
  runtimeCopy,
  settingsFeatureLinks,
  aboutLinks,
}: {
  appState: AppShellState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
  settingsFeatureLinks: ReturnType<typeof buildSettingsFeatureLinks>;
  aboutLinks: ReturnType<typeof buildAboutLinks>;
}) {
  if (!appState.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-hero-card">
        <p className="eyebrow">{runtimeCopy.settingsEyebrow}</p>
        <h3>{runtimeCopy.settingsHubTitle}</h3>
        <p className="panel-copy">{runtimeCopy.settingsHubCopy}</p>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Account</h3>
          <span className="pill pill-accent">Profile</span>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/settings/account")}
          >
            <div className="settings-row-copy">
              <strong>Vibe account</strong>
              <p>{appState.profile?.id ?? "Signed out"}</p>
            </div>
            <span className="settings-row-badge">Open</span>
          </button>
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/restore/index")}
          >
            <div className="settings-row-copy">
              <strong>Restore or link device</strong>
              <p>{describeRuntimeLinkRoute("mobile")}</p>
            </div>
            <span className="settings-row-badge">Auth</span>
          </button>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Social</h3>
          <span className="pill pill-outline">Friends</span>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/friends/index")}
          >
            <div className="settings-row-copy">
              <strong>Friends</strong>
              <p>Manage pending requests, sent requests, and accepted friends.</p>
            </div>
            <span className="settings-row-badge">Open</span>
          </button>
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/friends/search")}
          >
            <div className="settings-row-copy">
              <strong>Find friends</strong>
              <p>Search by username and send or accept requests.</p>
            </div>
            <span className="settings-row-badge">Search</span>
          </button>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Features</h3>
          <span className="pill pill-outline">Routes</span>
        </div>
        <div className="settings-list">
          {settingsFeatureLinks.map((item) => (
            <button
              key={item.title}
              type="button"
              className="settings-row"
              onClick={() => onNavigate(item.route)}
            >
              <div className="settings-row-copy">
                <strong>{item.title}</strong>
                <p>{item.subtitle}</p>
              </div>
              <span className="settings-row-badge">{item.badge}</span>
            </button>
          ))}
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>Support</h3>
          <span className="pill pill-outline">Help</span>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/changelog")}
          >
            <div className="settings-row-copy">
              <strong>What&apos;s new</strong>
              <p>Review the latest mobile shell changes and migration progress.</p>
            </div>
            <span className="settings-row-badge">Route</span>
          </button>
          <button
            type="button"
            className="settings-row"
            onClick={() => void openExternalUrl("https://github.com/fage-org/vibe-everywhere")}
          >
            <div className="settings-row-copy">
              <strong>GitHub</strong>
              <p>fage-org/vibe-everywhere</p>
            </div>
            <span className="settings-row-badge">External</span>
          </button>
          <button
            type="button"
            className="settings-row"
            onClick={() => void openExternalUrl("https://github.com/fage-org/vibe-everywhere/issues")}
          >
            <div className="settings-row-copy">
              <strong>Report issue</strong>
              <p>Open a bug report for Android shell regressions.</p>
            </div>
            <span className="settings-row-badge">External</span>
          </button>
        </div>
      </section>

      <section className="panel-card">
        <div className="card-header">
          <h3>About</h3>
          <span className="pill pill-muted">{DESKTOP_PREVIEW_VERSION}</span>
        </div>
        <div className="settings-list">
          {aboutLinks.slice(0, 3).map((item) => (
            <button
              key={item.title}
              type="button"
              className="settings-row"
              onClick={() => {
                if (item.action === "route") {
                  onNavigate(item.value);
                }
              }}
            >
              <div className="settings-row-copy">
                <strong>{item.title}</strong>
                <p>{item.subtitle}</p>
              </div>
              <span className="settings-row-badge">
                {item.action === "route" ? "Route" : "External"}
              </span>
            </button>
          ))}
        </div>
      </section>
    </div>
  );
}

type DesktopState = AppShellState;

type RouteSurfaceProps = {
  resolved: ResolvedRoute;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
  brandLogoSrc: string;
  brandLogotypeSrc: string;
  runtimeTarget: RuntimeTarget;
  runtimeCopy: RuntimeShellCopy;
  nativeCapabilities: RuntimeNativeCapabilities;
  mainViewTabs: ReadonlyArray<MainViewTabConfig>;
  settingsFeatureLinks: ReturnType<typeof buildSettingsFeatureLinks>;
  aboutLinks: ReturnType<typeof buildAboutLinks>;
};

function RouteSurface({
  resolved,
  desktop,
  preferences,
  secondarySurfaces,
  onNavigate,
  brandLogoSrc,
  brandLogotypeSrc,
  runtimeTarget,
  runtimeCopy,
  nativeCapabilities,
  mainViewTabs,
  settingsFeatureLinks,
  aboutLinks,
}: RouteSurfaceProps) {
  const { definition } = resolved;

  switch (definition.key) {
    case "home":
      return (
        <HomeSurface
          desktop={desktop}
          onNavigate={onNavigate}
          brandLogoSrc={brandLogoSrc}
          brandLogotypeSrc={brandLogotypeSrc}
          runtimeCopy={runtimeCopy}
          mainViewTabs={mainViewTabs}
          settingsFeatureLinks={settingsFeatureLinks}
        />
      );
    case "restore-index":
      if (runtimeTarget === "mobile") {
        return (
          <MobileRestoreSurface
            desktop={desktop}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <RestoreSurface
          desktop={desktop}
          onNavigate={onNavigate}
          runtimeTarget={runtimeTarget}
          runtimeCopy={runtimeCopy}
        />
      );
    case "restore-manual":
      if (runtimeTarget === "mobile") {
        return (
          <MobileManualRestoreSurface
            desktop={desktop}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <ManualRestoreSurface
          desktop={desktop}
          onNavigate={onNavigate}
          runtimeCopy={runtimeCopy}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "inbox":
      return <InboxSurface desktop={desktop} onNavigate={onNavigate} />;
    case "new-session":
      return <NewSessionSurface desktop={desktop} onNavigate={onNavigate} />;
    case "session-detail":
      if (runtimeTarget === "mobile") {
        return (
          <MobileSessionSurface
            key={resolved.params.id ?? ""}
            desktop={desktop}
            preferences={preferences}
            sessionId={resolved.params.id ?? ""}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <SessionSurface
          key={resolved.params.id ?? ""}
          desktop={desktop}
          preferences={preferences}
          sessionId={resolved.params.id ?? ""}
          onNavigate={onNavigate}
        />
      );
    case "session-info":
      if (runtimeTarget === "mobile") {
        return (
          <MobileSessionInfoSurface
            sessionId={resolved.params.id ?? ""}
            desktop={desktop}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <SessionInfoSurface
          sessionId={resolved.params.id ?? ""}
          desktop={desktop}
          onNavigate={onNavigate}
        />
      );
    case "session-message":
      if (runtimeTarget === "mobile") {
        return (
          <MobileSessionMessageSurface
            sessionId={resolved.params.id ?? ""}
            messageId={resolved.params.messageId ?? ""}
            desktop={desktop}
            preferences={preferences}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <SessionMessageSurface
          sessionId={resolved.params.id ?? ""}
          messageId={resolved.params.messageId ?? ""}
          desktop={desktop}
          preferences={preferences}
          onNavigate={onNavigate}
        />
      );
    case "session-files":
      if (runtimeTarget === "mobile") {
        return (
          <MobileSessionFilesSurface
            sessionId={resolved.params.id ?? ""}
            desktop={desktop}
            secondarySurfaces={secondarySurfaces}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <SessionFilesSurface
          sessionId={resolved.params.id ?? ""}
          desktop={desktop}
          secondarySurfaces={secondarySurfaces}
          onNavigate={onNavigate}
        />
      );
    case "session-file":
      if (runtimeTarget === "mobile") {
        return (
          <MobileSessionFileSurface
            sessionId={resolved.params.id ?? ""}
            routeFilePath={resolved.searchParams.get("path")}
            desktop={desktop}
            preferences={preferences}
            secondarySurfaces={secondarySurfaces}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <SessionFileSurface
          sessionId={resolved.params.id ?? ""}
          routeFilePath={resolved.searchParams.get("path")}
          desktop={desktop}
          preferences={preferences}
          secondarySurfaces={secondarySurfaces}
          onNavigate={onNavigate}
        />
      );
    case "session-recent":
      return <RecentSurface desktop={desktop} onNavigate={onNavigate} />;
    case "settings-index":
      return (
        <SettingsSurface
          desktop={desktop}
          onNavigate={onNavigate}
          runtimeTarget={runtimeTarget}
          runtimeCopy={runtimeCopy}
          settingsFeatureLinks={settingsFeatureLinks}
          aboutLinks={aboutLinks}
        />
      );
    case "settings-account":
      if (runtimeTarget === "mobile") {
        return (
          <MobileAccountSettingsSurface
            desktop={desktop}
            onNavigate={onNavigate}
            runtimeCopy={runtimeCopy}
          />
        );
      }
      return (
        <AccountSettingsSurface
          desktop={desktop}
          onNavigate={onNavigate}
          runtimeCopy={runtimeCopy}
        />
      );
    case "settings-appearance":
      if (runtimeTarget === "mobile") {
        return <MobileAppearanceSettingsSurface preferences={preferences} />;
      }
      return <AppearanceSettingsSurface preferences={preferences} />;
    case "settings-features":
      if (runtimeTarget === "mobile") {
        return <MobileFeatureSettingsSurface preferences={preferences} />;
      }
      return <FeatureSettingsSurface preferences={preferences} />;
    case "settings-language":
      if (runtimeTarget === "mobile") {
        return (
          <MobileLanguageSettingsSurface
            preferences={preferences}
            runtimeCopy={runtimeCopy}
          />
        );
      }
      return (
        <LanguageSettingsSurface
          preferences={preferences}
          runtimeCopy={runtimeCopy}
        />
      );
    case "settings-usage":
      if (runtimeTarget === "mobile") {
        return <MobileUsageSettingsSurface desktop={desktop} onNavigate={onNavigate} />;
      }
      return <UsageSettingsSurface desktop={desktop} onNavigate={onNavigate} />;
    case "settings-voice":
      if (runtimeTarget === "mobile") {
        return (
          <MobileVoiceSettingsSurface
            preferences={preferences}
            onNavigate={onNavigate}
            nativeCapabilities={nativeCapabilities}
          />
        );
      }
      return (
        <VoiceSettingsSurface
          preferences={preferences}
          onNavigate={onNavigate}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "settings-voice-language":
      if (runtimeTarget === "mobile") {
        return <MobileVoiceLanguageSurface preferences={preferences} />;
      }
      return <VoiceLanguageSurface preferences={preferences} />;
    case "settings-connect-claude":
      if (runtimeTarget === "mobile") {
        return <MobileConnectClaudeSurface desktop={desktop} onNavigate={onNavigate} />;
      }
      return <ConnectClaudeSurface desktop={desktop} onNavigate={onNavigate} />;
    case "artifacts-index":
      return (
        <ArtifactsIndexSurface
          desktop={desktop}
          artifacts={secondarySurfaces.artifacts}
          onNavigate={onNavigate}
        />
      );
    case "artifacts-new":
      return (
        <ArtifactCreateSurface
          desktop={desktop}
          createArtifact={secondarySurfaces.createArtifact}
          onNavigate={onNavigate}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "artifacts-detail":
      return (
        <ArtifactDetailSurface
          artifactId={resolved.params.id ?? ""}
          desktop={desktop}
          artifacts={secondarySurfaces.artifacts}
          deleteArtifact={secondarySurfaces.deleteArtifact}
          loadArtifact={secondarySurfaces.loadArtifact}
          onNavigate={onNavigate}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "artifacts-edit":
      return (
        <ArtifactEditSurface
          artifactId={resolved.params.id ?? ""}
          desktop={desktop}
          artifacts={secondarySurfaces.artifacts}
          updateArtifact={secondarySurfaces.updateArtifact}
          loadArtifact={secondarySurfaces.loadArtifact}
          onNavigate={onNavigate}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "user-detail":
      if (runtimeTarget === "mobile") {
        return (
          <MobileUserDetailSurface
            userId={resolved.params.id ?? ""}
            desktop={desktop}
            onNavigate={onNavigate}
          />
        );
      }
      return (
        <UserDetailSurface
          userId={resolved.params.id ?? ""}
          desktop={desktop}
          onNavigate={onNavigate}
        />
      );
    case "friends-index":
      if (runtimeTarget === "mobile") {
        return <MobileFriendsIndexSurface desktop={desktop} onNavigate={onNavigate} />;
      }
      break;
    case "friends-search":
      if (runtimeTarget === "mobile") {
        return <MobileFriendsSearchSurface desktop={desktop} onNavigate={onNavigate} />;
      }
      break;
    case "changelog":
      return <ChangelogSurface />;
    case "terminal-index":
      return <TerminalUtilitiesSurface desktop={desktop} onNavigate={onNavigate} />;
    case "terminal-connect":
      return (
        <TerminalConnectSurface
          desktop={desktop}
          searchParams={resolved.searchParams}
          onNavigate={onNavigate}
          nativeCapabilities={nativeCapabilities}
        />
      );
    case "server":
      return <ServerConfigSurface desktop={desktop} onNavigate={onNavigate} />;
    case "machine-detail":
      return (
        <MachineDetailSurface
          machineId={resolved.params.id ?? ""}
          desktop={desktop}
          onNavigate={onNavigate}
        />
      );
    case "text-selection":
      return <TextSelectionSurface nativeCapabilities={nativeCapabilities} />;
    default:
      return (
        <PlannedSurface
          title={definition.title}
          canonicalPath={resolved.canonicalPath}
          summary={definition.summary}
          ownerArea={wave8FeatureAreas.find(
            (area) => area.ownerModule === definition.ownerModule,
          )}
          promotionClass={definition.promotionClass}
          onNavigate={onNavigate}
        />
      );
  }
}

function HomeSurface({
  desktop,
  onNavigate,
  brandLogoSrc,
  brandLogotypeSrc,
  runtimeCopy,
  mainViewTabs,
  settingsFeatureLinks,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  brandLogoSrc: string;
  brandLogotypeSrc: string;
  runtimeCopy: RuntimeShellCopy;
  mainViewTabs: ReadonlyArray<MainViewTabConfig>;
  settingsFeatureLinks: ReturnType<typeof buildSettingsFeatureLinks>;
}) {
  const [copyFeedback, setCopyFeedback] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<MainViewTab>("sessions");

  const copyBackupKey = async () => {
    if (!desktop.backupKey) {
      return;
    }

    try {
      await copyTextToClipboard(desktop.backupKey);
      setCopyFeedback("Backup key copied to the clipboard.");
    } catch (error) {
      setCopyFeedback(
        error instanceof Error ? error.message : "Failed to copy the backup key",
      );
    }
  };

  if (!desktop.credentials || !desktop.profile) {
    return (
      <div className="surface-stack">
        <section className="hero-panel home-entry-hero">
          <div className="desktop-home-brand">
            <img className="hero-brand-mark" src={brandLogoSrc} alt="" aria-hidden="true" />
            <img className="hero-brand-logotype" src={brandLogotypeSrc} alt="Vibe" />
          </div>
          <div>
            <p className="eyebrow">{runtimeCopy.entryEyebrow}</p>
            <h3>{runtimeCopy.createRestoreHeading}</h3>
            <p className="hero-copy">{runtimeCopy.createRestoreCopy}</p>
          </div>
          <div className="hero-actions">
            <button
              type="button"
              className="primary-button"
              onClick={() => void desktop.createFreshAccount()}
            >
              Create account
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate("/(app)/restore/index")}
            >
              Restore or link account
            </button>
          </div>
        </section>

        <section className="surface-grid two-up">
          <article className="panel-card">
            <div className="card-header">
              <h3>{runtimeCopy.essentialsTitle}</h3>
              <span className="pill pill-accent">Core flow</span>
            </div>
            <ul className="bullet-list">
              {shellInvariants.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </article>

          <article className="panel-card">
            <div className="card-header">
              <h3>{`Available in this ${runtimeCopy.surfaceLabel} slice`}</h3>
              <span className="pill pill-outline">Current coverage</span>
            </div>
            <ul className="bullet-list">
              {firstUsableSlice.slice(0, 5).map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </article>
        </section>
      </div>
    );
  }

  const profileName = formatProfileName(
    desktop.profile.firstName,
    desktop.profile.lastName,
    desktop.profile.username,
    desktop.profile.id,
  );
  const topSessions = desktop.sessionSummaries.slice(0, 4);

  return (
    <div className="surface-stack">
      <section className="hero-panel mainview-hero">
        <div className="mainview-hero-copy">
          <div className="desktop-home-brand compact">
            <img className="hero-brand-mark" src={brandLogoSrc} alt="" aria-hidden="true" />
            <img className="hero-brand-logotype" src={brandLogotypeSrc} alt="Vibe" />
          </div>
          <div>
            <p className="eyebrow">{runtimeCopy.entryEyebrow}</p>
            <h3>{runtimeCopy.continueHeading}</h3>
            <p className="hero-copy">{runtimeCopy.continueCopy}</p>
          </div>
        </div>
        <div className="hero-actions profile-summary-row">
          <ProfileAvatar label={profileName} />
          <div className="profile-summary-copy">
            <strong>{profileName}</strong>
            <span>
              {desktop.profile.connectedServices.length > 0
                ? `${desktop.profile.connectedServices.length} connected services`
                : "No connected services yet"}
            </span>
          </div>
          <button
            type="button"
            className="primary-button"
            onClick={() => onNavigate("/(app)/new/index")}
          >
            Create session
          </button>
        </div>
      </section>

      {desktop.backupKey ? (
        <section className="panel-card backup-card">
          <div className="card-header">
            <h3>Secret key backup</h3>
            <span className="pill pill-outline">Keep safe</span>
          </div>
          <p className="panel-copy">
            {runtimeCopy.backupCopy}
          </p>
          <code className="backup-code">{desktop.backupKey}</code>
          <div className="button-row">
            <button type="button" className="secondary-button" onClick={() => void copyBackupKey()}>
              Copy backup key
            </button>
          </div>
          {copyFeedback ? <p className="panel-copy small-copy">{copyFeedback}</p> : null}
        </section>
      ) : null}

      <section className="panel-card mainview-panel">
        <div
          className="mainview-tabbar"
          role="tablist"
          aria-label={`${runtimeCopy.surfaceTitle} main view tabs`}
        >
          {mainViewTabs.map((tab) => (
            <button
              key={tab.key}
              type="button"
              role="tab"
              aria-selected={activeTab === tab.key}
              className={activeTab === tab.key ? "mainview-tab active" : "mainview-tab"}
              onClick={() => setActiveTab(tab.key)}
            >
              <span>{tab.label}</span>
              <small>{tab.eyebrow}</small>
            </button>
          ))}
        </div>

        {activeTab === "sessions" ? (
          <div className="surface-stack">
            <div className="card-header">
              <h3>Sessions</h3>
              <span className="pill pill-accent">{desktop.sessions.length} loaded</span>
            </div>
            {topSessions.length > 0 ? (
              <div className="surface-grid two-up">
                {topSessions.map(({ session, title, subtitle, detail }) => (
                  <article key={session.id} className="panel-card session-card">
                    <div className="session-card-top">
                      <div>
                        <p className="eyebrow">{subtitle}</p>
                        <h3>{title}</h3>
                      </div>
                      <span
                        className={`status-chip status-${session.active ? "active" : "ready"}`}
                      >
                        {session.active ? "Active" : "Idle"}
                      </span>
                    </div>
                    <p className="panel-copy">{detail}</p>
                    <div className="button-row">
                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => onNavigate(`/(app)/session/${session.id}`)}
                      >
                        Open session
                      </button>
                    </div>
                  </article>
                ))}
              </div>
            ) : (
              <EmptyState
                title="No sessions yet"
                body={`Create the first ${runtimeCopy.surfaceLabel} session to verify the live backend path end to end.`}
                actionLabel="Create session"
                onAction={() => onNavigate("/(app)/new/index")}
              />
            )}
            <div className="button-row">
              <button
                type="button"
                className="secondary-button"
                onClick={() => onNavigate("/(app)/session/recent")}
              >
                Recent sessions
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => onNavigate("/(app)/inbox/index")}
              >
                Open route inventory session list
              </button>
            </div>
          </div>
        ) : null}

        {activeTab === "inbox" ? (
          <div className="surface-grid two-up">
            <article className="panel-card">
              <div className="card-header">
                <h3>Connected services</h3>
                <span className="pill pill-outline">Inbox context</span>
              </div>
              {desktop.profile.connectedServices.length > 0 ? (
                <div className="settings-list">
                  {desktop.profile.connectedServices.map((service: string) => (
                    <div key={service} className="settings-row static">
                      <div className="settings-row-copy">
                        <strong>{formatServiceLabel(service)}</strong>
                        <p>{`Connected on the current ${runtimeCopy.surfaceLabel} account.`}</p>
                      </div>
                      <span className="settings-row-badge connected">Connected</span>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="panel-copy">
                  No services are connected yet. Open settings to connect Claude Code
                  or review account details.
                </p>
              )}
            </article>

            <article className="panel-card">
              <div className="card-header">
                <h3>Recent activity</h3>
                <span className="pill pill-accent">Live data</span>
              </div>
              {desktop.sessionSummaries.slice(0, 3).length > 0 ? (
                <div className="settings-list">
                  {desktop.sessionSummaries.slice(0, 3).map(({ session, title, detail }) => (
                    <button
                      key={session.id}
                      type="button"
                      className="settings-row"
                      onClick={() => onNavigate(`/(app)/session/${session.id}`)}
                    >
                      <div className="settings-row-copy">
                        <strong>{title}</strong>
                        <p>{detail}</p>
                      </div>
                      <span className="settings-row-badge">
                        {new Date(session.updatedAt).toLocaleDateString()}
                      </span>
                    </button>
                  ))}
                </div>
              ) : (
                <p className="panel-copy">
                  {`Session activity will appear here as soon as the ${runtimeCopy.surfaceLabel} account has at least one live session.`}
                </p>
              )}
            </article>
          </div>
        ) : null}

        {activeTab === "settings" ? (
          <div className="surface-grid two-up">
            {settingsFeatureLinks.slice(0, 4).map((item) => (
              <button
                key={item.title}
                type="button"
                className="settings-feature-card"
                onClick={() => onNavigate(item.route)}
              >
                <div className="settings-row-copy">
                  <strong>{item.title}</strong>
                  <p>{item.subtitle}</p>
                </div>
                <span className="settings-row-badge">{item.badge}</span>
              </button>
            ))}
            <button
              type="button"
              className="settings-feature-card"
              onClick={() => onNavigate("/(app)/settings/index")}
            >
              <div className="settings-row-copy">
                <strong>Open full settings</strong>
                <p>{runtimeCopy.settingsHubCopy}</p>
              </div>
              <span className="settings-row-badge">Hub</span>
            </button>
          </div>
        ) : null}
      </section>

    </div>
  );
}

function RestoreSurface({
  desktop,
  onNavigate,
  runtimeTarget,
  runtimeCopy,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  runtimeTarget: RuntimeTarget;
  runtimeCopy: RuntimeShellCopy;
}) {
  const [copyFeedback, setCopyFeedback] = useState<string | null>(null);
  const autoStartedLinkRef = useRef(false);

  useEffect(() => {
    return () => {
      desktop.cancelMobileLink();
    };
  }, [desktop.cancelMobileLink]);

  useEffect(() => {
    if (
      desktop.credentials ||
      autoStartedLinkRef.current ||
      desktop.linkState.status !== "idle"
    ) {
      return;
    }

    autoStartedLinkRef.current = true;
    void desktop.startMobileLink();
  }, [
    desktop.credentials,
    desktop.linkState.status,
    desktop.startMobileLink,
  ]);

  const copyFallbackLink = async () => {
    if (!desktop.linkState.linkUrl) {
      return;
    }

    try {
      await copyTextToClipboard(desktop.linkState.linkUrl);
      setCopyFeedback("Fallback link copied to the clipboard.");
    } catch (error) {
      setCopyFeedback(
        error instanceof Error ? error.message : "Failed to copy the fallback link",
      );
    }
  };

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Auth and session state</p>
          <h3>Restore and account-link flow</h3>
          <p className="hero-copy">{runtimeCopy.restoreFlowCopy}</p>
        </div>
        <div className="hero-actions">
          <button type="button" className="primary-button" onClick={() => void desktop.createFreshAccount()}>
            Create account
          </button>
          <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/restore/manual")}>
            Manual restore
          </button>
        </div>
      </section>

      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>{runtimeTarget === "desktop" ? "Mobile link request" : "Device link request"}</h3>
            <span className="pill pill-p0">Live link</span>
          </div>
          <p className="panel-copy">
            {runtimeTarget === "desktop"
              ? "Start a request, then scan the QR code with the current Vibe mobile app and approve the device link."
              : "Start a request, then continue from another signed-in device or fallback link to approve the account link."}
          </p>
          <div className="button-row">
            <button type="button" className="primary-button" onClick={() => void desktop.startMobileLink()}>
              {desktop.linkState.status === "requesting" || desktop.linkState.status === "waiting"
                ? "Restart link request"
                : "Start link request"}
            </button>
            <button type="button" className="secondary-button" onClick={desktop.cancelMobileLink}>
              Cancel
            </button>
          </div>
          {desktop.linkState.linkUrl ? (
            <div className="link-request-card">
              {desktop.linkState.qrSvg ? (
                <div
                  className="qr-frame"
                  aria-label="Account link QR"
                  dangerouslySetInnerHTML={{ __html: desktop.linkState.qrSvg }}
                />
              ) : null}
              <label className="field-block">
                <span>Fallback link</span>
                <textarea readOnly rows={4} value={desktop.linkState.linkUrl} />
              </label>
              <div className="button-row">
                <button type="button" className="secondary-button" onClick={() => void copyFallbackLink()}>
                  Copy fallback link
                </button>
              </div>
              {desktop.linkState.browserUrl ? (
                <label className="field-block">
                  <span>Browser callback page</span>
                  <input readOnly value={desktop.linkState.browserUrl} />
                </label>
              ) : null}
              <p className="panel-copy small-copy">
                Status: {desktop.linkState.status}
                {desktop.linkState.error ? ` — ${desktop.linkState.error}` : ""}
              </p>
              {copyFeedback ? <p className="panel-copy small-copy">{copyFeedback}</p> : null}
            </div>
          ) : null}
        </article>

        <article className="panel-card">
          <div className="card-header">
            <h3>Locked callback requirements</h3>
            <span className="pill pill-outline">Security baseline</span>
          </div>
          <ul className="bullet-list dense-list">
            {lockedAuthCallbackRequirements.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>
      </section>
    </div>
  );
}

function ManualRestoreSurface({
  desktop,
  onNavigate,
  runtimeCopy,
  nativeCapabilities,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const [secret, setSecret] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadingFile, setLoadingFile] = useState(false);

  const handleLoadKeyFile = async () => {
    setLoadingFile(true);
    setError(null);
    try {
      const nextSecret = await openTextFileDialog(`Load ${runtimeCopy.surfaceLabel} backup key`);
      if (nextSecret) {
        setSecret(nextSecret.trim());
      }
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : "Failed to load backup key file");
    } finally {
      setLoadingFile(false);
    }
  };

  const handleSubmit = async () => {
    setSubmitting(true);
    setError(null);
    try {
      await desktop.restoreWithSecret(secret);
      onNavigate("/(app)/inbox/index");
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : "Failed to restore account");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>Manual restore</h3>
          <span className="pill pill-p0">Real auth</span>
        </div>
        <label className="field-block">
          <span>Secret key</span>
          <textarea
            rows={6}
            placeholder="Paste the backup key in base64url or grouped backup format"
            value={secret}
            onChange={(event) => setSecret(event.target.value)}
          />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleSubmit()} disabled={submitting}>
            {submitting ? "Restoring..." : "Restore account"}
          </button>
          {nativeCapabilities.fileImport.available ? (
            <button
              type="button"
              className="secondary-button"
              onClick={() => void handleLoadKeyFile()}
              disabled={loadingFile}
            >
              {loadingFile ? "Loading key..." : "Load key file"}
            </button>
          ) : null}
          <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/restore/index")}>
            Back to restore
          </button>
        </div>
        {!nativeCapabilities.fileImport.available ? (
          <p className="panel-copy small-copy">{nativeCapabilities.fileImport.summary}</p>
        ) : null}
      </article>

      <article className="panel-card">
        <div className="card-header">
          <h3>What this unlocks</h3>
          <span className="pill pill-outline">P0</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Challenge auth returns a real bearer token from the current Vibe backend.</li>
          <li>Encrypted session metadata and message content can be decrypted with the restored secret.</li>
          <li>The same secret key works across fresh installs and other Vibe runtime shells.</li>
        </ul>
      </article>
    </div>
  );
}

function MobileRestoreSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  useEffect(() => {
    if (desktop.credentials || desktop.linkState.status !== "idle") {
      return;
    }
    void desktop.startMobileLink();
  }, [desktop.credentials, desktop.linkState.status, desktop.startMobileLink]);

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Restore</p>
        <h3>Link this Android device</h3>
        <p className="panel-copy">
          Open Happy on another signed-in device, go to account settings, and scan the QR code to
          approve this Android shell.
        </p>
      </section>

      <section className="panel-card mobile-settings-section mobile-qr-card">
        {desktop.linkState.qrSvg ? (
          <div
            className="qr-frame"
            aria-label="Device link QR code"
            dangerouslySetInnerHTML={{ __html: desktop.linkState.qrSvg }}
          />
        ) : (
          <div className="empty-state compact-empty">
            <h4>Preparing QR code</h4>
            <p className="panel-copy">Starting the live device-link request.</p>
          </div>
        )}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => void desktop.startMobileLink()}
          >
            Refresh QR code
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/restore/manual")}
          >
            Restore with secret key instead
          </button>
        </div>
      </section>
    </div>
  );
}

function MobileManualRestoreSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [secret, setSecret] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async () => {
    setSubmitting(true);
    setError(null);
    try {
      await desktop.restoreWithSecret(secret);
      onNavigate("/(app)/index");
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : "Failed to restore account");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Manual restore</p>
        <h3>Restore with your secret key</h3>
        <p className="panel-copy">
          Enter your backup secret key to restore access to this account on Android.
        </p>
        <label className="field-block">
          <span>Secret key</span>
          <textarea
            rows={5}
            placeholder="XXXXX-XXXXX-XXXXX..."
            value={secret}
            onChange={(event) => setSecret(event.target.value)}
          />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="primary-button full-width"
            onClick={() => void handleSubmit()}
            disabled={submitting}
          >
            {submitting ? "Restoring..." : "Restore account"}
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/restore/index")}
          >
            Back to QR restore
          </button>
        </div>
      </section>
    </div>
  );
}

function InboxSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Inbox</h3>
          <span className="pill pill-p0">Live sessions</span>
        </div>
        <p className="panel-copy">
          Session inventory is loaded from `/v1/sessions`, decrypted locally, and ordered by recent activity.
        </p>
        <div className="button-row">
          <button type="button" className="secondary-button" onClick={() => void desktop.refreshSessions()}>
            Refresh sessions
          </button>
          <button type="button" className="primary-button" onClick={() => onNavigate("/(app)/new/index")}>
            New session
          </button>
        </div>
      </section>

      {desktop.sessionSummaries.length > 0 ? (
        <section className="surface-grid two-up">
          {desktop.sessionSummaries.map(({ session, title, subtitle, detail }) => (
            <article key={session.id} className="panel-card session-card">
              <div className="session-card-top">
                <div>
                  <p className="eyebrow">{subtitle}</p>
                  <h3>{title}</h3>
                </div>
                <span className={`status-chip status-${session.active ? "active" : "ready"}`}>
                  {session.active ? "Active" : "Idle"}
                </span>
              </div>
              <p className="panel-copy">{detail}</p>
              <dl className="meta-grid compact-meta-grid">
                <div>
                  <dt>Updated</dt>
                  <dd>{new Date(session.updatedAt).toLocaleString()}</dd>
                </div>
                <div>
                  <dt>Metadata</dt>
                  <dd>v{session.metadataVersion}</dd>
                </div>
              </dl>
              <button
                type="button"
                className="secondary-button full-width"
                onClick={() => onNavigate(`/(app)/session/${session.id}`)}
              >
                Open session
              </button>
            </article>
          ))}
        </section>
      ) : (
        <EmptyState
          title="No sessions yet"
          body="Create the first desktop session to verify the live backend path end to end."
          actionLabel="Create session"
          onAction={() => onNavigate("/(app)/new/index")}
        />
      )}
    </div>
  );
}

function NewSessionSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const initialDraftRef = useRef<NewSessionDraft | null>(null);
  if (!initialDraftRef.current) {
    initialDraftRef.current = loadNewSessionDraft(NEW_SESSION_DEFAULT_DRAFT);
  }

  const [workspace, setWorkspace] = useState(initialDraftRef.current.workspace);
  const [model, setModel] = useState(initialDraftRef.current.model);
  const [title, setTitle] = useState(initialDraftRef.current.title);
  const [prompt, setPrompt] = useState(initialDraftRef.current.prompt);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    saveNewSessionDraft({
      workspace,
      model,
      title,
      prompt,
    });
  }, [workspace, model, title, prompt]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const handleCreate = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const session = await desktop.createSession({ workspace, model, prompt, title });
      clearNewSessionDraft();
      onNavigate(`/(app)/session/${session.id}`);
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : "Failed to create session");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>New session launcher</h3>
          <span className="pill pill-p0">Live create</span>
        </div>
        <label className="field-block">
          <span>Workspace</span>
          <input value={workspace} onChange={(event) => setWorkspace(event.target.value)} />
        </label>
        <label className="field-block">
          <span>Model</span>
          <input value={model} onChange={(event) => setModel(event.target.value)} />
        </label>
        <label className="field-block">
          <span>Title</span>
          <input value={title} onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className="field-block">
          <span>Initial prompt</span>
          <textarea rows={7} value={prompt} onChange={(event) => setPrompt(event.target.value)} />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleCreate()} disabled={submitting}>
            {submitting ? "Creating..." : "Create live session"}
          </button>
          <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/inbox/index")}>
            Back to inbox
          </button>
        </div>
      </article>

      <article className="panel-card">
        <div className="card-header">
          <h3>What happens</h3>
          <span className="pill pill-outline">Backend path</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>The desktop app creates a real session record through `/v1/sessions`.</li>
          <li>Metadata is encrypted locally before it is sent to the server.</li>
          <li>The initial prompt is posted to `/v3/sessions/:id/messages` as a real user message.</li>
        </ul>
      </article>
    </div>
  );
}

function SessionSurface({
  desktop,
  preferences,
  sessionId,
  onNavigate,
}: {
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  sessionId: string;
  onNavigate: (path: string) => void;
}) {
  const [draft, setDraft] = useState(() => loadSessionDraft(sessionId));
  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const messageState = desktop.sessionState[sessionId];
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const [selectionRange, setSelectionRange] = useState<{ start: number; end: number }>({
    start: draft.length,
    end: draft.length,
  });
  const [sessionFileSuggestions, setSessionFileSuggestions] = useState<string[] | null>(null);
  const [suggestionsLoading, setSuggestionsLoading] = useState(false);
  const permissionOptions = useMemo(
    () => getSessionPermissionOptions(session?.metadata ?? null),
    [session?.metadata],
  );
  const modelOptions = useMemo(
    () => getSessionModelOptions(session?.metadata ?? null),
    [session?.metadata],
  );
  const defaultComposerPreferences = useMemo(
    () => ({
      permissionMode: resolveSessionModeSelection(permissionOptions, [
        session?.metadata?.currentOperatingModeCode,
        "default",
      ]),
      model: resolveSessionModeSelection(modelOptions, [
        session?.metadata?.currentModelCode,
        "default",
      ]),
    } satisfies SessionComposerPreferences),
    [modelOptions, permissionOptions, session?.metadata?.currentModelCode, session?.metadata?.currentOperatingModeCode],
  );
  const [composerPreferences, setComposerPreferences] = useState<SessionComposerPreferences>(() =>
    loadSessionPreferences(sessionId, defaultComposerPreferences),
  );
  const activeComposerToken = useMemo(
    () => findActiveComposerToken(draft, selectionRange.start, selectionRange.end),
    [draft, selectionRange.end, selectionRange.start],
  );
  const composerSuggestions = useMemo(
    () =>
      buildComposerSuggestions(
        activeComposerToken?.activeWord ?? null,
        sessionFileSuggestions ?? [],
      ),
    [activeComposerToken?.activeWord, sessionFileSuggestions],
  );

  useEffect(() => {
    if (session) {
      void desktop.loadMessages(session.id);
    }
  }, [desktop.loadMessages, session?.id]);

  useEffect(() => {
    setComposerPreferences(loadSessionPreferences(sessionId, defaultComposerPreferences));
  }, [defaultComposerPreferences, sessionId]);

  useEffect(() => {
    if (!sessionId) {
      return;
    }

    if (draft) {
      saveSessionDraft(sessionId, draft);
      return;
    }

    clearSessionDraft(sessionId);
  }, [draft, sessionId]);

  useEffect(() => {
    saveSessionPreferences(sessionId, composerPreferences);
  }, [composerPreferences, sessionId]);

  useEffect(() => {
    setSessionFileSuggestions(null);
  }, [sessionId]);

  useEffect(() => {
    if (activeComposerToken?.activeWord.startsWith("@") && sessionFileSuggestions === null && session) {
      setSuggestionsLoading(true);
      void desktop
        .loadSessionFiles(session.id)
        .then((inventory) => {
          setSessionFileSuggestions(inventory.files.map((file) => file.relativePath));
        })
        .catch(() => {
          setSessionFileSuggestions([]);
        })
        .finally(() => {
          setSuggestionsLoading(false);
        });
    }
  }, [activeComposerToken?.activeWord, desktop.loadSessionFiles, session, sessionFileSuggestions]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the session inventory or return to the inbox to pick a different session."
        actionLabel="Back to inbox"
        onAction={() => onNavigate("/(app)/inbox/index")}
      />
    );
  }

  const description = describeSession(session);
  const resumeCommand = buildResumeCommand(session.metadata ?? {});
  const isArchived = session.metadata?.lifecycleState === "archived";
  const showResumeHint = !session.active && !!resumeCommand;

  const handleSend = async () => {
    if (!draft.trim()) {
      return;
    }
    await desktop.sendMessage(session.id, draft, {
      permissionMode:
        composerPreferences.permissionMode === "default"
          ? undefined
          : composerPreferences.permissionMode,
      model: composerPreferences.model === "default" ? null : composerPreferences.model,
    });
    setDraft("");
  };

  const handleSuggestionSelect = (suggestion: ComposerSuggestion) => {
    const next = applyComposerSuggestion(
      draft,
      selectionRange.start,
      selectionRange.end,
      suggestion,
    );
    setDraft(next.text);
    setSelectionRange({ start: next.cursorPosition, end: next.cursorPosition });
    if (typeof window !== "undefined") {
      window.requestAnimationFrame(() => {
        textareaRef.current?.focus();
        textareaRef.current?.setSelectionRange(next.cursorPosition, next.cursorPosition);
      });
    }
  };

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero session-hero">
        <div>
          <p className="eyebrow">Live session</p>
          <h3>{description.title}</h3>
          <p className="hero-copy">{description.subtitle}</p>
        </div>
        <div className="hero-meta">
          <span className={`status-chip status-${session.active ? "active" : "ready"}`}>
            {session.active ? "Active" : "Idle"}
          </span>
          <span className="pill pill-outline">{description.detail}</span>
        </div>
      </section>

      <section className="surface-grid session-layout">
        <article className="panel-card timeline-card">
          <div className="card-header">
            <h3>Timeline</h3>
            <div className="button-row compact-actions">
              <button type="button" className="secondary-button" onClick={() => void desktop.loadMessages(session.id, true)}>
                Refresh
              </button>
            </div>
          </div>
          {messageState?.error ? <ErrorBanner message={messageState.error} /> : null}
          {messageState?.loading ? <p className="panel-copy">Loading encrypted message history...</p> : null}
          {messageState?.items?.length ? (
            <div className="timeline-list">
              {messageState.items.map((message) => (
                <article key={message.id} className={`timeline-entry accent-${timelineAccent(message.role)}`}>
                  <div className="timeline-entry-head">
                    <strong>{message.title}</strong>
                    <span>{new Date(message.createdAt).toLocaleTimeString()}</span>
                  </div>
                  <TimelineMessageBody
                    message={message}
                    appearanceSettings={preferences.appearanceSettings}
                  />
                  <div className="button-row compact-actions">
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={() => onNavigate(`/(app)/session/${session.id}/message/${message.id}`)}
                    >
                      Open message
                    </button>
                  </div>
                </article>
              ))}
            </div>
          ) : !messageState?.loading ? (
            <p className="panel-copy">No messages yet. Send the first prompt from the composer.</p>
          ) : null}
        </article>

        <article className="panel-card composer-card">
          <div className="card-header">
            <h3>Composer</h3>
            <span className="pill pill-p0">Live send</span>
          </div>
          <div className="composer-settings-row">
            <label className="field-block compact-field">
              <span>Permission</span>
              <select
                value={composerPreferences.permissionMode}
                onChange={(event) =>
                  setComposerPreferences((current) => ({
                    ...current,
                    permissionMode: event.target.value,
                  }))
                }
              >
                {permissionOptions.map((option) => (
                  <option key={option.key} value={option.key}>
                    {option.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="field-block compact-field">
              <span>Model</span>
              <select
                value={composerPreferences.model}
                onChange={(event) =>
                  setComposerPreferences((current) => ({
                    ...current,
                    model: event.target.value,
                  }))
                }
              >
                {modelOptions.map((option) => (
                  <option key={option.key} value={option.key}>
                    {option.name}
                  </option>
                ))}
              </select>
            </label>
          </div>
          <label className="field-block">
            <span>Prompt the agent</span>
            <textarea
              ref={textareaRef}
              rows={8}
              value={draft}
              onChange={(event) => {
                setDraft(event.target.value);
                setSelectionRange({
                  start: event.currentTarget.selectionStart ?? event.target.value.length,
                  end: event.currentTarget.selectionEnd ?? event.target.value.length,
                });
              }}
              onSelect={(event) =>
                setSelectionRange({
                  start: event.currentTarget.selectionStart ?? draft.length,
                  end: event.currentTarget.selectionEnd ?? draft.length,
                })
              }
              placeholder="Send a real message to the session"
            />
          </label>
          {activeComposerToken ? (
            <div className="composer-hint-row">
              <span className="pill pill-outline">
                {activeComposerToken.activeWord.startsWith("@")
                  ? "File mention autocomplete"
                  : "Slash command autocomplete"}
              </span>
              <span className="panel-copy compact-copy">
                {activeComposerToken.activeWord.startsWith("@")
                  ? "Reference workspace files directly in the next turn."
                  : "Queue a common session command without retyping it."}
              </span>
            </div>
          ) : null}
          {suggestionsLoading ? (
            <p className="panel-copy compact-copy">Loading live workspace file suggestions...</p>
          ) : composerSuggestions.length > 0 ? (
            <div className="composer-suggestion-list">
              {composerSuggestions.map((suggestion) => (
                <button
                  key={suggestion.key}
                  type="button"
                  className="composer-suggestion-button"
                  onClick={() => handleSuggestionSelect(suggestion)}
                >
                  <strong>{suggestion.label}</strong>
                  <span>{suggestion.description}</span>
                </button>
              ))}
            </div>
          ) : null}
          <div className="button-row">
            <button
              type="button"
              className="primary-button"
              onClick={() => void handleSend()}
              disabled={messageState?.sending || messageState?.aborting}
            >
              {messageState?.sending ? "Sending..." : "Send live message"}
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => void desktop.abortSession(session.id)}
              disabled={messageState?.aborting}
            >
              {messageState?.aborting ? "Aborting..." : "Abort turn"}
            </button>
            <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/session/recent")}>
              Recent sessions
            </button>
          </div>
          <div className="done-block">
            <strong>Transport notes</strong>
            <ul className="bullet-list dense-list">
              <li>Messages are encrypted locally before `/v3/sessions/:id/messages` receives them.</li>
              <li>Timeline entries are decrypted in the desktop app using the restored secret key.</li>
              <li>Selected model and permission mode persist locally and apply to the next message turn.</li>
              <li>Rich markdown, diff, tool, and file rendering now reuse the package-local parity renderers.</li>
            </ul>
          </div>
        </article>
      </section>
    </div>
  );
}

function MobileSessionSurface({
  desktop,
  preferences,
  sessionId,
  onNavigate,
}: {
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  sessionId: string;
  onNavigate: (path: string) => void;
}) {
  const [draft, setDraft] = useState(() => loadSessionDraft(sessionId));
  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const messageState = desktop.sessionState[sessionId];
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const [selectionRange, setSelectionRange] = useState<{ start: number; end: number }>({
    start: draft.length,
    end: draft.length,
  });
  const [sessionFileSuggestions, setSessionFileSuggestions] = useState<string[] | null>(null);
  const [suggestionsLoading, setSuggestionsLoading] = useState(false);
  const permissionOptions = useMemo(
    () => getSessionPermissionOptions(session?.metadata ?? null),
    [session?.metadata],
  );
  const modelOptions = useMemo(
    () => getSessionModelOptions(session?.metadata ?? null),
    [session?.metadata],
  );
  const defaultComposerPreferences = useMemo(
    () => ({
      permissionMode: resolveSessionModeSelection(permissionOptions, [
        session?.metadata?.currentOperatingModeCode,
        "default",
      ]),
      model: resolveSessionModeSelection(modelOptions, [
        session?.metadata?.currentModelCode,
        "default",
      ]),
    } satisfies SessionComposerPreferences),
    [modelOptions, permissionOptions, session?.metadata?.currentModelCode, session?.metadata?.currentOperatingModeCode],
  );
  const [composerPreferences, setComposerPreferences] = useState<SessionComposerPreferences>(() =>
    loadSessionPreferences(sessionId, defaultComposerPreferences),
  );
  const activeComposerToken = useMemo(
    () => findActiveComposerToken(draft, selectionRange.start, selectionRange.end),
    [draft, selectionRange.end, selectionRange.start],
  );
  const composerSuggestions = useMemo(
    () =>
      buildComposerSuggestions(
        activeComposerToken?.activeWord ?? null,
        sessionFileSuggestions ?? [],
      ),
    [activeComposerToken?.activeWord, sessionFileSuggestions],
  );

  useEffect(() => {
    if (session) {
      void desktop.loadMessages(session.id);
    }
  }, [desktop.loadMessages, session?.id]);

  useEffect(() => {
    setComposerPreferences(loadSessionPreferences(sessionId, defaultComposerPreferences));
  }, [defaultComposerPreferences, sessionId]);

  useEffect(() => {
    if (!sessionId) {
      return;
    }
    if (draft) {
      saveSessionDraft(sessionId, draft);
      return;
    }
    clearSessionDraft(sessionId);
  }, [draft, sessionId]);

  useEffect(() => {
    saveSessionPreferences(sessionId, composerPreferences);
  }, [composerPreferences, sessionId]);

  useEffect(() => {
    setSessionFileSuggestions(null);
  }, [sessionId]);

  useEffect(() => {
    if (activeComposerToken?.activeWord.startsWith("@") && sessionFileSuggestions === null && session) {
      setSuggestionsLoading(true);
      void desktop
        .loadSessionFiles(session.id)
        .then((inventory) => {
          setSessionFileSuggestions(inventory.files.map((file) => file.relativePath));
        })
        .catch(() => {
          setSessionFileSuggestions([]);
        })
        .finally(() => {
          setSuggestionsLoading(false);
        });
    }
  }, [activeComposerToken?.activeWord, desktop.loadSessionFiles, session, sessionFileSuggestions]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the session inventory or return to sessions to pick a different session."
        actionLabel="Back to sessions"
        onAction={() => onNavigate("/(app)/index")}
      />
    );
  }

  const description = describeSession(session);
  const resumeCommand = buildResumeCommand(session.metadata ?? {});
  const isArchived = session.metadata?.lifecycleState === "archived";
  const showResumeHint = !session.active && !!resumeCommand;

  const handleSend = async () => {
    if (!draft.trim()) {
      return;
    }
    await desktop.sendMessage(session.id, draft, {
      permissionMode:
        composerPreferences.permissionMode === "default"
          ? undefined
          : composerPreferences.permissionMode,
      model: composerPreferences.model === "default" ? null : composerPreferences.model,
    });
    setDraft("");
  };

  const handleSuggestionSelect = (suggestion: ComposerSuggestion) => {
    const next = applyComposerSuggestion(
      draft,
      selectionRange.start,
      selectionRange.end,
      suggestion,
    );
    setDraft(next.text);
    setSelectionRange({ start: next.cursorPosition, end: next.cursorPosition });
    if (typeof window !== "undefined") {
      window.requestAnimationFrame(() => {
        textareaRef.current?.focus();
        textareaRef.current?.setSelectionRange(next.cursorPosition, next.cursorPosition);
      });
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-session-header">
        <p className="eyebrow">Session</p>
        <h3>{description.title}</h3>
        <p className="panel-copy">{description.subtitle}</p>
        <div className="pill-row">
          <span className={`status-chip status-${session.active ? "active" : "ready"}`}>
            {session.active ? "Active" : "Idle"}
          </span>
          <span className="pill pill-outline">{description.detail}</span>
        </div>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(`/(app)/session/${sessionId}/info`)}
          >
            Session info
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
          >
            Session files
          </button>
        </div>
      </section>

      {showResumeHint ? (
        <section className="panel-card mobile-session-section">
          <div className="card-header">
            <h3>{isArchived ? "Archived session" : "Resume session"}</h3>
            <span className="pill pill-outline">Terminal</span>
          </div>
          <p className="panel-copy">
            {isArchived
              ? "This session is archived. Resume it from the terminal before sending a new turn."
              : "This session can be resumed directly from the terminal using the current metadata."}
          </p>
          <code className="backup-code">{resumeCommand}</code>
          <div className="button-row mobile-button-column">
            <button
              type="button"
              className="secondary-button full-width"
              onClick={() =>
                void copyTextToClipboard(resumeCommand).catch(() => undefined)
              }
            >
              Copy resume command
            </button>
          </div>
        </section>
      ) : null}

      <section className="panel-card mobile-session-section">
        <div className="card-header">
          <h3>Timeline</h3>
          <button
            type="button"
            className="secondary-button"
            onClick={() => void desktop.loadMessages(session.id, true)}
          >
            Refresh
          </button>
        </div>
        {messageState?.error ? <ErrorBanner message={messageState.error} /> : null}
        {messageState?.loading ? <p className="panel-copy">Loading encrypted message history...</p> : null}
        {messageState?.items?.length ? (
          <div className="timeline-list">
            {messageState.items.map((message) => (
              <article key={message.id} className={`timeline-entry accent-${timelineAccent(message.role)}`}>
                <div className="timeline-entry-head">
                  <strong>{message.title}</strong>
                  <span>{new Date(message.createdAt).toLocaleTimeString()}</span>
                </div>
                <TimelineMessageBody
                  message={message}
                  appearanceSettings={preferences.appearanceSettings}
                />
                <div className="button-row compact-actions">
                  <button
                    type="button"
                    className="secondary-button full-width"
                    onClick={() => onNavigate(`/(app)/session/${session.id}/message/${message.id}`)}
                  >
                    Open message
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : !messageState?.loading ? (
          <MobileEmptyMessages session={session} />
        ) : null}
      </section>

      <section className="panel-card mobile-session-section">
        <div className="card-header">
          <h3>Composer</h3>
          <span className="pill pill-p0">Live send</span>
        </div>
        <div className="composer-settings-row">
          <label className="field-block compact-field">
            <span>Permission</span>
            <select
              value={composerPreferences.permissionMode}
              onChange={(event) =>
                setComposerPreferences((current) => ({
                  ...current,
                  permissionMode: event.target.value,
                }))
              }
            >
              {permissionOptions.map((option) => (
                <option key={option.key} value={option.key}>
                  {option.name}
                </option>
              ))}
            </select>
          </label>
          <label className="field-block compact-field">
            <span>Model</span>
            <select
              value={composerPreferences.model}
              onChange={(event) =>
                setComposerPreferences((current) => ({
                  ...current,
                  model: event.target.value,
                }))
              }
            >
              {modelOptions.map((option) => (
                <option key={option.key} value={option.key}>
                  {option.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        <label className="field-block">
          <span>Prompt the agent</span>
          <textarea
            ref={textareaRef}
            rows={6}
            value={draft}
            onChange={(event) => {
              setDraft(event.target.value);
              setSelectionRange({
                start: event.currentTarget.selectionStart ?? event.target.value.length,
                end: event.currentTarget.selectionEnd ?? event.target.value.length,
              });
            }}
            onSelect={(event) =>
              setSelectionRange({
                start: event.currentTarget.selectionStart ?? draft.length,
                end: event.currentTarget.selectionEnd ?? draft.length,
              })
            }
            placeholder="Send a real message to the session"
          />
        </label>
        {activeComposerToken ? (
          <div className="composer-hint-row">
            <span className="pill pill-outline">
              {activeComposerToken.activeWord.startsWith("@")
                ? "File mention autocomplete"
                : "Slash command autocomplete"}
            </span>
          </div>
        ) : null}
        {suggestionsLoading ? (
          <p className="panel-copy compact-copy">Loading live workspace file suggestions...</p>
        ) : composerSuggestions.length > 0 ? (
          <div className="composer-suggestion-list">
            {composerSuggestions.map((suggestion) => (
              <button
                key={suggestion.key}
                type="button"
                className="composer-suggestion-button"
                onClick={() => handleSuggestionSelect(suggestion)}
              >
                <strong>{suggestion.label}</strong>
                <span>{suggestion.description}</span>
              </button>
            ))}
          </div>
        ) : null}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="primary-button full-width"
            onClick={() => void handleSend()}
            disabled={messageState?.sending || messageState?.aborting}
          >
            {messageState?.sending ? "Sending..." : "Send live message"}
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => void desktop.abortSession(session.id)}
            disabled={messageState?.aborting}
          >
            {messageState?.aborting ? "Aborting..." : "Abort turn"}
          </button>
        </div>
      </section>
    </div>
  );
}

function MobileEmptyMessages({
  session,
}: {
  session: DesktopSession;
}) {
  const host = typeof session.metadata?.host === "string" ? session.metadata.host : null;
  const workspace = typeof session.metadata?.path === "string" ? session.metadata.path : null;

  return (
    <div className="mobile-empty-messages">
      {host ? <strong>{host}</strong> : null}
      {workspace ? <span>{workspace}</span> : null}
      <h4>No messages yet</h4>
      <p className="panel-copy">{`Created ${formatRelativeTime(session.createdAt)}`}</p>
    </div>
  );
}

function MobileSessionContextCard({
  session,
  onNavigate,
  primaryAction,
}: {
  session: DesktopSession;
  onNavigate: (path: string) => void;
  primaryAction?: {
    label: string;
    path: string;
  };
}) {
  const description = describeSession(session);
  const workspace = typeof session.metadata?.path === "string" ? session.metadata.path : null;
  const host = typeof session.metadata?.host === "string" ? session.metadata.host : null;

  return (
    <section className="panel-card mobile-session-section">
      <p className="eyebrow">Session</p>
      <h3>{description.title}</h3>
      <p className="panel-copy">{description.subtitle}</p>
      <div className="pill-row">
        <span className={`status-chip status-${session.active ? "active" : "ready"}`}>
          {session.active ? "Active" : "Idle"}
        </span>
        <span className="pill pill-outline">{description.detail}</span>
      </div>
      {(workspace || host) ? (
        <dl className="meta-grid compact-meta-grid">
          {workspace ? (
            <div>
              <dt>Workspace</dt>
              <dd>{workspace}</dd>
            </div>
          ) : null}
          {host ? (
            <div>
              <dt>Host</dt>
              <dd>{host}</dd>
            </div>
          ) : null}
        </dl>
      ) : null}
      {primaryAction ? (
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(primaryAction.path)}
          >
            {primaryAction.label}
          </button>
        </div>
      ) : null}
    </section>
  );
}

function MobileSessionMessageSurface({
  sessionId,
  messageId,
  desktop,
  preferences,
  onNavigate,
}: {
  sessionId: string;
  messageId: string;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  onNavigate: (path: string) => void;
}) {
  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const messageState = desktop.sessionState[sessionId];

  useEffect(() => {
    if (session) {
      void desktop.loadMessages(session.id);
    }
  }, [desktop.loadMessages, session?.id]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const message = messageState?.items.find((item) => item.id === messageId) ?? null;
  if (!message) {
    return (
      <EmptyState
        title="Message not found"
        body="Refresh the session timeline and reopen the message once it is available."
        actionLabel="Back to session"
        onAction={() => onNavigate(`/(app)/session/${sessionId}`)}
      />
    );
  }

  const sessionData = session!;

  return (
    <div className="surface-stack">
      <MobileSessionContextCard
        session={sessionData}
        onNavigate={onNavigate}
        primaryAction={{
          label: "Back to session",
          path: `/(app)/session/${sessionId}`,
        }}
      />
      <section className="panel-card mobile-session-section">
        <p className="eyebrow">Message</p>
        <h3>{message.title}</h3>
        <p className="panel-copy">{new Date(message.createdAt).toLocaleString()}</p>
        <div className="pill-row">
          <span className="pill pill-outline">{message.role}</span>
          <span className="pill pill-outline">{message.rawType}</span>
        </div>
      </section>
      <section className="panel-card mobile-session-section">
        <div className="card-header">
          <h3>Message content</h3>
          <span className="pill pill-accent">Deep link</span>
        </div>
        <TimelineMessageBody
          message={message}
          appearanceSettings={preferences.appearanceSettings}
        />
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(`/(app)/session/${sessionId}`)}
          >
            Back to session
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
          >
            Open session files
          </button>
        </div>
      </section>
    </div>
  );
}

function MobileSessionInfoSurface({
  sessionId,
  desktop,
  onNavigate,
}: {
  sessionId: string;
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the session inventory and reopen the session info route."
        actionLabel="Back to session"
        onAction={() => onNavigate(`/(app)/session/${sessionId}`)}
      />
    );
  }

  const metadata = session.metadata;
  const description = describeSession(session);

  return (
    <div className="surface-stack">
      <MobileSessionContextCard
        session={session}
        onNavigate={onNavigate}
        primaryAction={{
          label: "Open session files",
          path: `/(app)/session/${sessionId}/files`,
        }}
      />
      <section className="panel-card mobile-session-section">
        <div className="card-header">
          <h3>Session info</h3>
          <span className="pill pill-accent">Live</span>
        </div>
        <dl className="meta-grid">
          <div>
            <dt>Session ID</dt>
            <dd>{session.id}</dd>
          </div>
          <div>
            <dt>Workspace</dt>
            <dd>{metadata?.path ?? "Unavailable"}</dd>
          </div>
          <div>
            <dt>Host</dt>
            <dd>{metadata?.host ?? "Unavailable"}</dd>
          </div>
          <div>
            <dt>Model</dt>
            <dd>{metadata?.currentModelCode ?? "Unavailable"}</dd>
          </div>
        </dl>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
          >
            Open session files
          </button>
        </div>
      </section>
    </div>
  );
}

function MobileSessionFilesSurface({
  sessionId,
  desktop,
  secondarySurfaces,
  onNavigate,
}: {
  sessionId: string;
  desktop: DesktopState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const [inventory, setInventory] = useState<{
    branch: string | null;
    files: SessionWorkspaceFile[];
    totalStaged: number;
    totalUnstaged: number;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!desktop.credentials) {
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .loadSessionFiles(sessionId)
      .then((nextInventory) => {
        if (canceled) {
          return;
        }
        setInventory(nextInventory);
        if (
          nextInventory.files.length > 0 &&
          !secondarySurfaces.getSelectedSessionFilePath(sessionId)
        ) {
          secondarySurfaces.selectSessionFilePath(sessionId, nextInventory.files[0].relativePath);
        }
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load session files");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop.credentials, desktop.loadSessionFiles, secondarySurfaces, sessionId]);

  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the session inventory before reviewing session files."
        actionLabel="Back to sessions"
        onAction={() => onNavigate("/(app)/index")}
      />
    );
  }

  return (
    <div className="surface-stack">
      <MobileSessionContextCard
        session={session}
        onNavigate={onNavigate}
        primaryAction={{
          label: "Back to session",
          path: `/(app)/session/${sessionId}`,
        }}
      />
      <section className="panel-card mobile-session-section">
        <div className="card-header">
          <h3>Session files</h3>
          <span className="pill pill-accent">
            {loading ? "Loading" : `${inventory?.files.length ?? 0} live`}
          </span>
        </div>
        {inventory ? (
          <dl className="meta-grid compact-meta-grid">
            <div>
              <dt>Branch</dt>
              <dd>{inventory.branch ?? "Detached"}</dd>
            </div>
            <div>
              <dt>Staged</dt>
              <dd>{inventory.totalStaged}</dd>
            </div>
            <div>
              <dt>Unstaged</dt>
              <dd>{inventory.totalUnstaged}</dd>
            </div>
          </dl>
        ) : null}
        <p className="panel-copy">
          Browse the current session workspace and open a file without leaving the mobile flow.
        </p>
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      {loading ? (
        <section className="panel-card empty-state-card">
          <h3>Loading files</h3>
          <p className="panel-copy">Inspecting the current session workspace and git status.</p>
        </section>
      ) : (inventory?.files.length ?? 0) > 0 ? (
        <section className="panel-card mobile-session-section">
          <div className="settings-list">
            {inventory!.files.map((file) => (
              <button
                key={`${file.relativePath}-${file.isStaged ? "staged" : "unstaged"}`}
                type="button"
                className="settings-row"
                onClick={() => {
                  secondarySurfaces.selectSessionFilePath(sessionId, file.relativePath);
                  onNavigate(sessionFileRoutePath(sessionId, file.relativePath));
                }}
              >
                <div className="settings-row-copy">
                  <strong>{file.fileName}</strong>
                  <p>{file.relativePath}</p>
                </div>
                <span className="settings-row-badge">{file.status}</span>
              </button>
            ))}
          </div>
        </section>
      ) : (
        <EmptyState
          title="No live files"
          body="The session workspace is clean, unavailable, or not under git control."
          actionLabel="Back to session"
          onAction={() => onNavigate(`/(app)/session/${sessionId}`)}
        />
      )}
    </div>
  );
}

function MobileSessionFileSurface({
  sessionId,
  routeFilePath,
  desktop,
  preferences,
  secondarySurfaces,
  onNavigate,
}: {
  sessionId: string;
  routeFilePath: string | null;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const selectedPath = routeFilePath ?? secondarySurfaces.getSelectedSessionFilePath(sessionId);
  const [file, setFile] = useState<SessionWorkspaceFileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (routeFilePath) {
      secondarySurfaces.selectSessionFilePath(sessionId, routeFilePath);
    }
  }, [routeFilePath, secondarySurfaces, sessionId]);

  useEffect(() => {
    if (!desktop.credentials || !selectedPath) {
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .readSessionFile(sessionId, selectedPath)
      .then((loadedFile) => {
        if (canceled) {
          return;
        }
        setFile(loadedFile);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load session file");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop.credentials, desktop.readSessionFile, selectedPath, sessionId]);

  if (!session || !selectedPath) {
    return (
      <EmptyState
        title="File not found"
        body="Open the session files route first to select a file."
        actionLabel="Back to files"
        onAction={() => onNavigate(`/(app)/session/${sessionId}/files`)}
      />
    );
  }

  return (
    <div className="surface-stack">
      <MobileSessionContextCard
        session={session}
        onNavigate={onNavigate}
        primaryAction={{
          label: "Back to files",
          path: `/(app)/session/${sessionId}/files`,
        }}
      />
      <section className="panel-card mobile-session-section">
        <p className="eyebrow">Session file</p>
        <h3>{selectedPath.split("/").at(-1) ?? selectedPath}</h3>
        <p className="panel-copy">{selectedPath}</p>
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      {loading ? (
        <section className="panel-card empty-state-card">
          <h3>Loading file</h3>
          <p className="panel-copy">Reading the selected file from the live session workspace.</p>
        </section>
      ) : file ? (
        <section className="panel-card mobile-session-section">
          <div className="card-header">
            <h3>File content</h3>
            <span className="pill pill-accent">{file.isBinary ? "Binary" : "Live view"}</span>
          </div>
          <TimelineMessageBody
            message={{
              id: `${sessionId}:${selectedPath}`,
              localId: null,
              createdAt: Date.now(),
              role: "tool",
              title: "File contents",
              text: file.content,
              rawType: file.isBinary ? "session:file:binary" : "session:file",
            }}
            appearanceSettings={preferences.appearanceSettings}
          />
          {file.diff ? (
            <>
              <div className="card-header compact-card-header">
                <h3>Diff preview</h3>
                <span className="pill pill-outline">Available</span>
              </div>
              <TimelineMessageBody
                message={{
                  id: `${sessionId}:${selectedPath}:diff`,
                  localId: null,
                  createdAt: Date.now(),
                  role: "assistant",
                  title: "Diff preview",
                  text: file.diff,
                  rawType: "session:file-diff",
                }}
                appearanceSettings={preferences.appearanceSettings}
              />
            </>
          ) : null}
          <div className="button-row mobile-button-column">
            <button
              type="button"
              className="secondary-button full-width"
              onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
            >
              Back to files
            </button>
            <button
              type="button"
              className="secondary-button full-width"
              onClick={() => onNavigate(`/(app)/session/${sessionId}`)}
            >
              Back to session
            </button>
          </div>
        </section>
      ) : null}
    </div>
  );
}

function SessionMessageSurface({
  sessionId,
  messageId,
  desktop,
  preferences,
  onNavigate,
}: {
  sessionId: string;
  messageId: string;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  onNavigate: (path: string) => void;
}) {
  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const messageState = desktop.sessionState[sessionId];

  useEffect(() => {
    if (session) {
      void desktop.loadMessages(session.id);
    }
  }, [desktop.loadMessages, session?.id]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Reload the session list before opening a deep-linked message route."
        actionLabel="Back to inbox"
        onAction={() => onNavigate("/(app)/inbox/index")}
      />
    );
  }

  const message = messageState?.items.find((item) => item.id === messageId) ?? null;
  if (messageState?.loading && !message) {
    return (
      <div className="panel-card empty-state-card">
        <h3>Loading message</h3>
        <p className="panel-copy">Decrypting the requested message from the live session history.</p>
      </div>
    );
  }

  if (!message) {
    return (
      <EmptyState
        title="Message not found"
        body="Refresh the session timeline and reopen the deep link once the message history is loaded."
        actionLabel="Back to session"
        onAction={() => onNavigate(`/(app)/session/${sessionId}`)}
      />
    );
  }

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Session message</p>
          <h3>{message.title}</h3>
          <p className="hero-copy">{new Date(message.createdAt).toLocaleString()}</p>
        </div>
        <div className="hero-meta">
          <span className="pill pill-outline">{message.role}</span>
          <span className="pill pill-outline">{message.rawType}</span>
        </div>
      </section>
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Rendered message</h3>
            <span className="pill pill-accent">Deep link</span>
          </div>
          <TimelineMessageBody
            message={message}
            appearanceSettings={preferences.appearanceSettings}
          />
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Message metadata</h3>
            <span className="pill pill-outline">P1 route</span>
          </div>
          <dl className="meta-grid compact-meta-grid">
            <div>
              <dt>Message ID</dt>
              <dd>{message.id}</dd>
            </div>
            <div>
              <dt>Role</dt>
              <dd>{message.role}</dd>
            </div>
            <div>
              <dt>Rendered as</dt>
              <dd>{message.rawType}</dd>
            </div>
            <div>
              <dt>Session</dt>
              <dd>{sessionId}</dd>
            </div>
          </dl>
          <div className="button-row">
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate(`/(app)/session/${sessionId}`)}
            >
              Back to session
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
            >
              Open session files
            </button>
          </div>
        </article>
      </section>
    </div>
  );
}

function SessionInfoSurface({
  sessionId,
  desktop,
  onNavigate,
}: {
  sessionId: string;
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the session inventory and reopen the live session info route."
        actionLabel="Back to session"
        onAction={() => onNavigate("/(app)/inbox/index")}
      />
    );
  }

  const description = describeSession(session);
  const metadata = session.metadata;

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Session info</p>
          <h3>{description.title}</h3>
          <p className="hero-copy">
            Live desktop session metadata route for parity review before the full mutation set is
            promoted.
          </p>
        </div>
        <div className="hero-meta">
          <span className={`status-chip status-${session.active ? "active" : "ready"}`}>
            {session.active ? "Active" : "Idle"}
          </span>
          <span className="pill pill-outline">{metadata?.flavor ?? "desktop"}</span>
        </div>
      </section>
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Metadata</h3>
            <span className="pill pill-accent">Live</span>
          </div>
          <dl className="meta-grid">
            <div>
              <dt>Session ID</dt>
              <dd>{session.id}</dd>
            </div>
            <div>
              <dt>Workspace</dt>
              <dd>{metadata?.path ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Host</dt>
              <dd>{metadata?.host ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Model</dt>
              <dd>{metadata?.currentModelCode ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>OS</dt>
              <dd>{metadata?.os ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Updated</dt>
              <dd>{new Date(session.updatedAt).toLocaleString()}</dd>
            </div>
          </dl>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Desktop review notes</h3>
            <span className="pill pill-outline">P1 route</span>
          </div>
          <ul className="bullet-list dense-list">
            <li>Session identity, workspace, host, and model now remain visible on a dedicated route.</li>
            <li>Full destructive actions and lifecycle mutations still remain a later promotion slice.</li>
            <li>Use the files route to review live file and diff surfaces tied to this session.</li>
          </ul>
          <div className="button-row">
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
            >
              Open session files
            </button>
          </div>
        </article>
      </section>
    </div>
  );
}

function SessionFilesSurface({
  sessionId,
  desktop,
  secondarySurfaces,
  onNavigate,
}: {
  sessionId: string;
  desktop: DesktopState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const [inventory, setInventory] = useState<{
    branch: string | null;
    files: SessionWorkspaceFile[];
    totalStaged: number;
    totalUnstaged: number;
  } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!desktop.credentials) {
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .loadSessionFiles(sessionId)
      .then((nextInventory) => {
        if (canceled) {
          return;
        }
        setInventory(nextInventory);
        if (
          nextInventory.files.length > 0 &&
          !secondarySurfaces.getSelectedSessionFilePath(sessionId)
        ) {
          secondarySurfaces.selectSessionFilePath(sessionId, nextInventory.files[0].relativePath);
        }
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load session files");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [
    desktop.credentials,
    desktop.loadSessionFiles,
    secondarySurfaces,
    sessionId,
  ]);

  if (!session) {
    return (
      <EmptyState
        title="Session not found"
        body="Refresh the desktop session list before reviewing session file surfaces."
        actionLabel="Back to inbox"
        onAction={() => onNavigate("/(app)/inbox/index")}
      />
    );
  }

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Session files</h3>
          <span className="pill pill-accent">
            {loading ? "Loading" : `${inventory?.files.length ?? 0} live`}
          </span>
        </div>
        <p className="panel-copy">
          This route now loads the current git file inventory through live session RPC instead of
          retained review fixtures.
        </p>
        {inventory ? (
          <dl className="meta-grid compact-meta-grid">
            <div>
              <dt>Branch</dt>
              <dd>{inventory.branch ?? "Detached"}</dd>
            </div>
            <div>
              <dt>Staged</dt>
              <dd>{inventory.totalStaged}</dd>
            </div>
            <div>
              <dt>Unstaged</dt>
              <dd>{inventory.totalUnstaged}</dd>
            </div>
          </dl>
        ) : null}
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      <section className="surface-grid two-up">
        {loading ? (
          <section className="panel-card empty-state-card">
            <h3>Loading files</h3>
            <p className="panel-copy">Inspecting the current session workspace and git status.</p>
          </section>
        ) : (inventory?.files.length ?? 0) > 0 ? (
          inventory!.files.map((file) => (
            <article key={`${file.relativePath}-${file.isStaged ? "staged" : "unstaged"}`} className="panel-card">
              <div className="card-header">
                <h3>{file.fileName}</h3>
                <span className="pill pill-outline">{file.status}</span>
              </div>
              <p className="panel-copy">
                {file.filePath || "Project root"} ·
                {` ${file.linesAdded > 0 ? `+${file.linesAdded}` : "0"} / ${
                  file.linesRemoved > 0 ? `-${file.linesRemoved}` : "0"
                }`}
              </p>
              <code className="backup-code">{file.relativePath}</code>
              <div className="button-row">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => {
                    secondarySurfaces.selectSessionFilePath(sessionId, file.relativePath);
                    onNavigate(sessionFileRoutePath(sessionId, file.relativePath));
                  }}
                >
                  Open file
                </button>
              </div>
            </article>
          ))
        ) : (
          <EmptyState
            title="No live files"
            body="The session workspace is clean, unavailable, or not under git control."
            actionLabel="Back to session"
            onAction={() => onNavigate(`/(app)/session/${sessionId}`)}
          />
        )}
      </section>
    </div>
  );
}

function SessionFileSurface({
  sessionId,
  routeFilePath,
  desktop,
  preferences,
  secondarySurfaces,
  onNavigate,
}: {
  sessionId: string;
  routeFilePath: string | null;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const session = desktop.sessions.find((item) => item.id === sessionId) ?? null;
  const selectedPath = routeFilePath ?? secondarySurfaces.getSelectedSessionFilePath(sessionId);
  const [file, setFile] = useState<SessionWorkspaceFileContent | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (routeFilePath) {
      secondarySurfaces.selectSessionFilePath(sessionId, routeFilePath);
    }
  }, [routeFilePath, secondarySurfaces, sessionId]);

  useEffect(() => {
    if (!desktop.credentials || !selectedPath) {
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .readSessionFile(sessionId, selectedPath)
      .then((loadedFile) => {
        if (canceled) {
          return;
        }
        setFile(loadedFile);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load session file");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop.credentials, desktop.readSessionFile, selectedPath, sessionId]);

  if (!session || !selectedPath) {
    return (
      <EmptyState
        title="File not found"
        body="Open the live files inventory first to select a file for inspection."
        actionLabel="Back to files"
        onAction={() => onNavigate(`/(app)/session/${sessionId}/files`)}
      />
    );
  }

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Session file</p>
          <h3>{selectedPath.split("/").at(-1) ?? selectedPath}</h3>
          <p className="hero-copy">
            Live file read and diff preview loaded through the current session RPC path.
          </p>
        </div>
        <div className="hero-meta">
          <span className="pill pill-outline">{file?.language ?? "text"}</span>
          <span className={`status-chip status-${loading ? "ready" : "active"}`}>
            {loading ? "Loading" : "Live"}
          </span>
        </div>
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>File contents</h3>
            <span className="pill pill-accent">{file?.isBinary ? "Binary" : "Live view"}</span>
          </div>
          <pre
            className={`diff-line diff-line-context ${
              preferences.appearanceSettings.wrapLinesInDiffs ? "diff-line-wrap" : ""
            }`}
          >
            {loading
              ? "Loading file contents..."
              : file?.isBinary
                ? "Binary file content is not rendered in the desktop shell."
                : file?.content || ""}
          </pre>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Diff preview</h3>
            <span className="pill pill-outline">{file?.diff ? "Available" : "None"}</span>
          </div>
          {file?.diff ? (
            <TimelineMessageBody
              message={{
                id: `${sessionId}:${selectedPath}:diff`,
                localId: null,
                createdAt: Date.now(),
                role: "assistant",
                title: "Diff preview",
                text: file.diff,
                rawType: "session:file-diff",
              }}
              appearanceSettings={preferences.appearanceSettings}
            />
          ) : (
            <p className="panel-copy">
              No live diff preview is currently available for this file.
            </p>
          )}
          <div className="button-row">
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate(`/(app)/session/${sessionId}/files`)}
            >
              Back to files
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => onNavigate(`/(app)/session/${sessionId}/info`)}
            >
              Session info
            </button>
          </div>
        </article>
      </section>
    </div>
  );
}

function RecentSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Recent sessions and resume affordances</h3>
          <span className="pill pill-p0">Resume flow</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Recent work is sorted by the real session update timestamp from the server.</li>
          <li>Each entry resolves to a real session route and decryptable message history.</li>
          <li>Refreshing the route re-fetches the current session inventory.</li>
        </ul>
      </section>

      <section className="surface-grid two-up">
        {desktop.sessionSummaries.map(({ session, title, subtitle }) => (
          <article key={session.id} className="panel-card mini-card">
            <div className="mini-card-header">
              <strong>{title}</strong>
              <span>{new Date(session.updatedAt).toLocaleString()}</span>
            </div>
            <p>{subtitle}</p>
            <button
              type="button"
              className="secondary-button full-width"
              onClick={() => onNavigate(`/(app)/session/${session.id}`)}
            >
              Resume session
            </button>
          </article>
        ))}
      </section>
    </div>
  );
}

function SettingsSurface({
  desktop,
  onNavigate,
  runtimeTarget,
  runtimeCopy,
  settingsFeatureLinks,
  aboutLinks,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  runtimeTarget: RuntimeTarget;
  runtimeCopy: RuntimeShellCopy;
  settingsFeatureLinks: ReturnType<typeof buildSettingsFeatureLinks>;
  aboutLinks: ReturnType<typeof buildAboutLinks>;
}) {
  const [serverUrlDraft, setServerUrlDraft] = useState(desktop.serverUrl);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    setServerUrlDraft(desktop.serverUrl);
  }, [desktop.serverUrl]);

  const handleSave = async () => {
    setUpdating(true);
    setError(null);
    try {
      await desktop.updateServerUrl(serverUrlDraft);
      setFeedback(`${runtimeCopy.surfaceTitle} server endpoint updated.`);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to update server URL");
    } finally {
      setUpdating(false);
    }
  };

  const profileName = formatProfileName(
    desktop.profile?.firstName,
    desktop.profile?.lastName,
    desktop.profile?.username,
    desktop.profile?.id ?? `${runtimeCopy.surfaceTitle} account`,
  );

  const handleCopyBackupKey = async () => {
    if (!desktop.backupKey) {
      return;
    }

    try {
      await copyTextToClipboard(desktop.backupKey);
      setFeedback("Backup key copied to the clipboard.");
    } catch (copyError) {
      setFeedback(
        copyError instanceof Error ? copyError.message : "Failed to copy the backup key",
      );
    }
  };

  const handleAboutLink = async (
    item: (typeof aboutLinks)[number],
    onNavigate: (path: string) => void,
  ) => {
    if (item.action === "route") {
      onNavigate(item.value);
      return;
    }

    try {
      await openExternalUrl(item.value);
    } catch (linkError) {
      setFeedback(
        linkError instanceof Error ? linkError.message : "Failed to open external link",
      );
    }
  };

  return (
    <div className="surface-stack">
      <section className="hero-panel settings-hero">
        <div className="settings-hero-copy">
          <ProfileAvatar label={profileName} />
          <div className="profile-summary-copy">
            <p className="eyebrow">Settings</p>
            <h3>{profileName}</h3>
            <p className="hero-copy">{runtimeCopy.settingsHubCopy}</p>
          </div>
        </div>
        <div className="hero-actions">
          <button
            type="button"
            className="secondary-button"
            onClick={() => void handleCopyBackupKey()}
          >
            Copy backup key
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => void desktop.logout()}
          >
            Logout
          </button>
        </div>
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}

      <section className="surface-grid two-up">
        <article className="panel-card settings-group-card">
          <div className="card-header">
            <h3>Connected Accounts</h3>
            <span className="pill pill-accent">Live</span>
          </div>
          <div className="settings-list">
            <button
              type="button"
              className="settings-row"
              onClick={() => onNavigate("/(app)/settings/account")}
            >
              <div className="settings-row-copy">
                <strong>Vibe Account</strong>
                <p>{desktop.profile?.id ?? "Signed out"}</p>
              </div>
              <span className="settings-row-badge">Profile</span>
            </button>
            <button
              type="button"
              className="settings-row"
              onClick={() => onNavigate("/(app)/settings/connect/claude")}
            >
              <div className="settings-row-copy">
                <strong>Claude Code</strong>
                <p>
                  {(desktop.profile?.connectedServices ?? []).includes("anthropic")
                    ? "Connected on this account."
                    : `Open the ${runtimeCopy.surfaceLabel} connect flow for command handoff and setup guidance.`}
                </p>
              </div>
              <span
                className={`settings-row-badge ${
                  (desktop.profile?.connectedServices ?? []).includes("anthropic")
                    ? "connected"
                    : ""
                }`}
              >
                {(desktop.profile?.connectedServices ?? []).includes("anthropic")
                  ? "Connected"
                  : "Connect"}
              </span>
            </button>
            <button
              type="button"
              className="settings-row"
              onClick={() => onNavigate("/(app)/restore/index")}
            >
              <div className="settings-row-copy">
                <strong>Restore or Link Device</strong>
                <p>{describeRuntimeLinkRoute(runtimeTarget)}</p>
              </div>
              <span className="settings-row-badge">Auth</span>
            </button>
          </div>
        </article>

        <article className="panel-card settings-group-card">
          <div className="card-header">
            <h3>{runtimeCopy.configTitle}</h3>
            <span className="pill pill-outline">Live endpoint</span>
          </div>
          <label className="field-block">
            <span>Server URL</span>
            <input value={serverUrlDraft} onChange={(event) => setServerUrlDraft(event.target.value)} />
          </label>
          <div className="button-row">
            <button type="button" className="primary-button" onClick={() => void handleSave()} disabled={updating}>
              {updating ? "Saving..." : "Save server URL"}
            </button>
          </div>
          <dl className="meta-grid">
            <div>
              <dt>Status</dt>
              <dd>{statusLabel(desktop.status)}</dd>
            </div>
            <div>
              <dt>Endpoint</dt>
              <dd>{desktop.serverUrl}</dd>
            </div>
            <div>
              <dt>Backup key</dt>
              <dd>{desktop.backupKey ? "Available" : "Not available"}</dd>
            </div>
            <div>
              <dt>Sessions</dt>
              <dd>{desktop.sessions.length}</dd>
            </div>
          </dl>
        </article>
      </section>

      <section className="surface-grid two-up">
        <article className="panel-card settings-group-card">
          <div className="card-header">
            <h3>Features</h3>
            <span className="pill pill-outline">Route parity</span>
          </div>
          <div className="settings-list">
            {settingsFeatureLinks.map((item) => (
              <button
                key={item.title}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(item.route)}
              >
                <div className="settings-row-copy">
                  <strong>{item.title}</strong>
                  <p>{item.subtitle}</p>
                </div>
                <span className="settings-row-badge">{item.badge}</span>
              </button>
            ))}
          </div>
        </article>

        <article className="panel-card settings-group-card">
          <div className="card-header">
            <h3>About</h3>
            <span className="pill pill-muted">{DESKTOP_PREVIEW_VERSION}</span>
          </div>
          <div className="settings-list">
            {aboutLinks.map((item) => (
              <button
                key={item.title}
                type="button"
                className="settings-row"
                onClick={() => void handleAboutLink(item, onNavigate)}
              >
                <div className="settings-row-copy">
                  <strong>{item.title}</strong>
                  <p>{item.subtitle}</p>
                </div>
                <span className="settings-row-badge">
                  {item.action === "route" ? "Route" : "External"}
                </span>
              </button>
            ))}
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>Version</strong>
                <p>{`${runtimeCopy.surfaceTitle} shell preview build`}</p>
              </div>
              <span className="settings-row-badge">{DESKTOP_PREVIEW_VERSION}</span>
            </div>
          </div>
        </article>
      </section>
    </div>
  );
}

function AccountSettingsSurface({
  desktop,
  onNavigate,
  runtimeCopy,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const profileName = formatProfileName(
    desktop.profile?.firstName,
    desktop.profile?.lastName,
    desktop.profile?.username,
    desktop.profile?.id ?? `${runtimeCopy.surfaceTitle} account`,
  );

  const handleCopyBackupKey = async () => {
    if (!desktop.backupKey) {
      return;
    }

    try {
      await copyTextToClipboard(desktop.backupKey);
      setFeedback("Backup key copied.");
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to copy backup key");
    }
  };

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Account</p>
          <h3>{profileName}</h3>
          <p className="hero-copy">{runtimeCopy.accountRouteCopy}</p>
        </div>
        <div className="hero-actions">
          <button type="button" className="secondary-button" onClick={() => void handleCopyBackupKey()}>
            Copy backup key
          </button>
          <button type="button" className="secondary-button" onClick={() => void desktop.logout()}>
            Logout
          </button>
        </div>
      </section>
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Identity</h3>
            <span className="pill pill-accent">Live profile</span>
          </div>
          <dl className="meta-grid">
            <div>
              <dt>Account ID</dt>
              <dd>{desktop.profile?.id ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Username</dt>
              <dd>{desktop.profile?.username ? `@${desktop.profile.username}` : "Unavailable"}</dd>
            </div>
            <div>
              <dt>Connected services</dt>
              <dd>{desktop.profile?.connectedServices.length ?? 0}</dd>
            </div>
            <div>
              <dt>{`${runtimeCopy.surfaceTitle} status`}</dt>
              <dd>{statusLabel(desktop.status)}</dd>
            </div>
          </dl>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Linked services</h3>
            <span className="pill pill-outline">Live account</span>
          </div>
          <div className="settings-list">
            {(desktop.profile?.connectedServices ?? []).length > 0 ? (
              desktop.profile!.connectedServices.map((service) => (
                <div key={service} className="settings-row static">
                  <div className="settings-row-copy">
                    <strong>{formatServiceLabel(service)}</strong>
                    <p>Connected on the current account.</p>
                  </div>
                  <span className="settings-row-badge connected">Connected</span>
                </div>
              ))
            ) : (
              <div className="settings-row static">
                <div className="settings-row-copy">
                  <strong>No connected services</strong>
                  <p>Open the vendor route to connect a supported integration.</p>
                </div>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => onNavigate("/(app)/settings/connect/claude")}
                >
                  Connect Claude
                </button>
              </div>
            )}
          </div>
        </article>
      </section>
    </div>
  );
}

function MobileAccountSettingsSurface({
  desktop,
  onNavigate,
  runtimeCopy,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
  runtimeCopy: RuntimeShellCopy;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const profileName = formatProfileName(
    desktop.profile?.firstName,
    desktop.profile?.lastName,
    desktop.profile?.username,
    desktop.profile?.id ?? `${runtimeCopy.surfaceTitle} account`,
  );

  const handleCopyBackupKey = async () => {
    if (!desktop.backupKey) {
      return;
    }

    try {
      await copyTextToClipboard(desktop.backupKey);
      setFeedback("Backup key copied.");
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to copy backup key");
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Account</p>
        <h3>{profileName}</h3>
        <p className="panel-copy">{runtimeCopy.accountRouteCopy}</p>
        <div className="button-row mobile-button-column">
          <button type="button" className="secondary-button full-width" onClick={() => void handleCopyBackupKey()}>
            Copy backup key
          </button>
          <button type="button" className="secondary-button full-width" onClick={() => void desktop.logout()}>
            Logout
          </button>
        </div>
      </section>
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
      <section className="panel-card mobile-settings-section">
        <div className="settings-list">
          <div className="settings-row static">
            <div className="settings-row-copy">
              <strong>Account ID</strong>
              <p>{desktop.profile?.id ?? "Unavailable"}</p>
            </div>
            <span className="settings-row-badge">Profile</span>
          </div>
          <div className="settings-row static">
            <div className="settings-row-copy">
              <strong>Username</strong>
              <p>{desktop.profile?.username ? `@${desktop.profile.username}` : "Unavailable"}</p>
            </div>
            <span className="settings-row-badge">Handle</span>
          </div>
          <button
            type="button"
            className="settings-row"
            onClick={() => onNavigate("/(app)/settings/connect/claude")}
          >
            <div className="settings-row-copy">
              <strong>Connected services</strong>
              <p>{desktop.profile?.connectedServices.length ?? 0} linked integrations</p>
            </div>
            <span className="settings-row-badge">Open</span>
          </button>
        </div>
      </section>
    </div>
  );
}

function AppearanceSettingsSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { appearanceSettings, updateAppearanceSettings, commitAppearancePatch } = preferences;

  return (
    <div className="surface-stack">
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Appearance</h3>
            <span className="pill pill-accent">Persisted</span>
          </div>
          <p className="panel-copy">
            These desktop settings now persist locally and immediately affect the shell review
            surface instead of behaving like temporary preview-only controls.
          </p>
          <div className="settings-list">
            {(["adaptive", "light", "dark"] as const).map((option) => (
              <button
                key={option}
                type="button"
                className="settings-row"
                onClick={() => updateAppearanceSettings({ themePreference: option })}
              >
                <div className="settings-row-copy">
                  <strong>{option[0].toUpperCase() + option.slice(1)}</strong>
                  <p>Desktop shell theme preference.</p>
                </div>
                <span className="settings-row-badge">
                  {appearanceSettings.themePreference === option ? "Selected" : "Available"}
                </span>
              </button>
            ))}
          </div>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Density</h3>
            <span className="pill pill-outline">{appearanceSettings.density}</span>
          </div>
          <div className="settings-list">
            {(["compact", "desktop", "comfortable"] as const).map((option) => (
              <button
                key={option}
                type="button"
                className="settings-row"
                onClick={() => updateAppearanceSettings({ density: option })}
              >
                <div className="settings-row-copy">
                  <strong>{option[0].toUpperCase() + option.slice(1)}</strong>
                  <p>Compare route hierarchy and panel density against the shipping desktop view.</p>
                </div>
                <span className="settings-row-badge">
                  {appearanceSettings.density === option ? "Active" : "Available"}
                </span>
              </button>
            ))}
          </div>
        </article>
      </section>

      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Display behavior</h3>
            <span className="pill pill-outline">Persisted flags</span>
          </div>
          <div className="settings-list">
            {[
              {
                key: "compactSessionView",
                title: "Compact session view",
                detail: "Use a tighter shell layout for session-heavy routes.",
              },
              {
                key: "showFlavorIcons",
                title: "Show provider icons",
                detail: "Keep avatar/provider display preferences reviewable on desktop.",
              },
              {
                key: "wrapLinesInDiffs",
                title: "Wrap lines in diffs",
                detail: "Persist diff readability preference for desktop review.",
              },
              {
                key: "showLineNumbersInToolViews",
                title: "Show line numbers in tool views",
                detail: "Persist tool-view readability preference.",
              },
            ].map((item) => {
              const enabled = appearanceSettings[item.key as keyof DesktopAppearanceSettings];
              return (
                <button
                  key={item.key}
                  type="button"
                  className="settings-row"
                  onClick={() => {
                    const patch = {
                      [item.key]: !enabled,
                    } as Partial<DesktopAppearanceSettings>;
                    void commitAppearancePatch(patch);
                  }}
                >
                  <div className="settings-row-copy">
                    <strong>{item.title}</strong>
                    <p>{item.detail}</p>
                  </div>
                  <span className="settings-row-badge">{enabled ? "On" : "Off"}</span>
                </button>
              );
            })}
          </div>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Reset</h3>
            <span className="pill pill-outline">Defaults</span>
          </div>
          <p className="panel-copy">
            Reset the desktop shell settings back to the current parity-first defaults.
          </p>
          <div className="button-row">
            <button
              type="button"
              className="secondary-button"
              onClick={() => {
                void commitAppearancePatch(defaultAppearanceSettings);
              }}
            >
              Reset appearance settings
            </button>
          </div>
        </article>
      </section>
      {preferences.accountSettingsError ? <ErrorBanner message={preferences.accountSettingsError} /> : null}
    </div>
  );
}

function MobileAppearanceSettingsSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { appearanceSettings, updateAppearanceSettings, commitAppearancePatch } = preferences;

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Appearance</h3>
          <span className="pill pill-accent">Persisted</span>
        </div>
        <div className="settings-list">
          {(["adaptive", "light", "dark"] as const).map((option) => (
            <button
              key={option}
              type="button"
              className="settings-row"
              onClick={() => updateAppearanceSettings({ themePreference: option })}
            >
              <div className="settings-row-copy">
                <strong>{option[0].toUpperCase() + option.slice(1)}</strong>
                <p>Theme preference for the Android shell.</p>
              </div>
              <span className="settings-row-badge">
                {appearanceSettings.themePreference === option ? "Selected" : "Available"}
              </span>
            </button>
          ))}
          <button
            type="button"
            className="settings-row"
            onClick={() => updateAppearanceSettings({ density: "compact" })}
          >
            <div className="settings-row-copy">
              <strong>Compact session view</strong>
              <p>Use a tighter mobile layout for session-heavy screens.</p>
            </div>
            <span className="settings-row-badge">
              {appearanceSettings.density === "compact" ? "Active" : "Available"}
            </span>
          </button>
        </div>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => {
              void commitAppearancePatch(defaultAppearanceSettings);
            }}
          >
            Reset appearance settings
          </button>
        </div>
      </section>
    </div>
  );
}

function FeatureSettingsSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { appearanceSettings, commitAppearancePatch } = preferences;
  const implementationFlags = [
    {
      name: "Loopback Auth Guardrails",
      status: "Enabled",
      detail: "Desktop auth is restricted to localhost callback ownership during coexistence.",
    },
    {
      name: "Shared Payload Validation",
      status: "Enabled",
      detail: "Realtime updates, artifacts, machines, and session payloads are validated through shared schemas.",
    },
    {
      name: "Live Session File RPC",
      status: "Enabled",
      detail: "Session file inventory and file reads now execute through live session RPC instead of retained fixtures.",
    },
    {
      name: "Promotion Route Inventory",
      status: `${routeInventoryByClass.P1.length} tracked`,
      detail: "P1 route progress is audited in the parity checklist instead of being inferred from static route labels.",
    },
    {
      name: "Persisted Appearance Settings",
      status: preferences.appearanceSettings.compactSessionView ? "Customized" : "Default",
      detail: "Desktop appearance flags now persist locally instead of resetting on every route review.",
    },
    {
      name: "Persisted Voice Settings",
      status: preferences.voiceSettings.customAgentId ? "Configured" : "Default",
      detail: "Voice route now stores preferred language and optional custom agent configuration.",
    },
  ] as const;

  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>Feature flags</h3>
          <span className="pill pill-accent">P1 route</span>
        </div>
        <div className="settings-list">
          {implementationFlags.map((flag) => (
            <div key={flag.name} className="settings-row static">
              <div className="settings-row-copy">
                <strong>{flag.name}</strong>
                <p>{flag.detail}</p>
              </div>
              <span className="settings-row-badge">{flag.status}</span>
            </div>
          ))}
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Desktop rendering flags</h3>
          <span className="pill pill-outline">Persisted mutation</span>
        </div>
        <div className="settings-list">
          {[
            {
              key: "showLineNumbersInDiffs",
              title: "Show line numbers in diffs",
              detail: "Controls diff line numbers for timeline and session-file previews.",
            },
            {
              key: "showLineNumbersInToolViews",
              title: "Show line numbers in tool views",
              detail: "Controls line numbers for code blocks and tool payloads.",
            },
            {
              key: "wrapLinesInDiffs",
              title: "Wrap long lines",
              detail: "Controls line wrapping for diff and file previews on desktop.",
            },
            {
              key: "compactSessionView",
              title: "Compact session shell",
              detail: "Applies the compact desktop shell density for session-heavy routes.",
            },
            {
              key: "showFlavorIcons",
              title: "Show provider icons",
              detail: "Keeps avatar/provider display preferences persisted on desktop.",
            },
          ].map((item) => {
            const enabled = appearanceSettings[item.key as keyof DesktopAppearanceSettings];
            return (
              <button
                key={item.key}
                type="button"
                className="settings-row"
                onClick={() => {
                  const patch = {
                    [item.key]: !enabled,
                  } as Partial<DesktopAppearanceSettings>;
                  void commitAppearancePatch(patch);
                }}
              >
                <div className="settings-row-copy">
                  <strong>{item.title}</strong>
                  <p>{item.detail}</p>
                </div>
                <span className="settings-row-badge">{enabled ? "On" : "Off"}</span>
              </button>
            );
          })}
        </div>
      </article>
    </div>
  );
}

function MobileFeatureSettingsSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { appearanceSettings, commitAppearancePatch, voiceSettings } = preferences;

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Features</h3>
          <span className="pill pill-accent">Mobile route set</span>
        </div>
        <div className="settings-list">
          <div className="settings-row static">
            <div className="settings-row-copy">
              <strong>Loopback auth guardrails</strong>
              <p>Account link and restore still respect the current auth boundary.</p>
            </div>
            <span className="settings-row-badge">Enabled</span>
          </div>
          <div className="settings-row static">
            <div className="settings-row-copy">
              <strong>Shared payload validation</strong>
              <p>Realtime and session payloads continue to validate through shared schemas.</p>
            </div>
            <span className="settings-row-badge">Enabled</span>
          </div>
          <button
            type="button"
            className="settings-row"
            onClick={() => {
              void commitAppearancePatch({ compactSessionView: !appearanceSettings.compactSessionView });
            }}
          >
            <div className="settings-row-copy">
              <strong>Compact session shell</strong>
              <p>Keep the Android session host denser on smaller screens.</p>
            </div>
            <span className="settings-row-badge">
              {appearanceSettings.compactSessionView ? "On" : "Off"}
            </span>
          </button>
          <div className="settings-row static">
            <div className="settings-row-copy">
              <strong>Voice custom agent</strong>
              <p>{voiceSettings.customAgentId ? "Configured" : "Default"}</p>
            </div>
            <span className="settings-row-badge">
              {voiceSettings.customAgentId ? "Configured" : "Default"}
            </span>
          </div>
        </div>
      </section>
    </div>
  );
}

function LanguageSettingsSurface({
  preferences,
  runtimeCopy,
}: {
  preferences: DesktopPreferencesState;
  runtimeCopy: RuntimeShellCopy;
}) {
  const { languageSettings, commitLanguagePatch } = preferences;
  const runtimeLanguage =
    typeof navigator !== "undefined" ? navigator.language || "en-US" : "en-US";
  const preferredLanguages =
    typeof navigator !== "undefined" && Array.isArray(navigator.languages)
      ? navigator.languages
      : [runtimeLanguage];
  const supportedLanguages = Object.values(SUPPORTED_LANGUAGES);

  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>Language</h3>
          <span className="pill pill-accent">{languageSettings.appLanguage}</span>
        </div>
        <p className="panel-copy">{runtimeCopy.languageCopy}</p>
        <div className="settings-list">
          {supportedLanguages.map((language) => (
            <button
              key={language.code}
              type="button"
              className="settings-row"
              onClick={() => {
                const patch = { appLanguage: language.code };
                void commitLanguagePatch(patch);
              }}
            >
              <div className="settings-row-copy">
                <strong>{language.nativeName}</strong>
                <p>
                  {language.englishName !== language.nativeName
                    ? language.englishName
                    : `Preferred ${runtimeCopy.surfaceTitle} language`}
                </p>
              </div>
              <span className="settings-row-badge">
                {languageSettings.appLanguage === language.code ? "Selected" : "Available"}
              </span>
            </button>
          ))}
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Runtime language signals</h3>
          <span className="pill pill-outline">{DEFAULT_LANGUAGE}</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>{`${runtimeCopy.runtimeLanguageLabel}: ${runtimeLanguage}`}</li>
          <li>Preferred languages: {preferredLanguages.join(", ")}</li>
          <li>{`${runtimeCopy.storedPreferenceLabel}: ${languageSettings.appLanguage}`}</li>
          <li>Full translated copy switching remains a later promotion step.</li>
        </ul>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() => {
              void commitLanguagePatch(defaultLanguageSettings);
            }}
          >
            Reset language preference
          </button>
        </div>
      </article>
      {preferences.accountSettingsError ? <ErrorBanner message={preferences.accountSettingsError} /> : null}
    </div>
  );
}

function MobileLanguageSettingsSurface({
  preferences,
  runtimeCopy,
}: {
  preferences: DesktopPreferencesState;
  runtimeCopy: RuntimeShellCopy;
}) {
  return (
    <div className="surface-stack">
      <LanguageSettingsSurface preferences={preferences} runtimeCopy={runtimeCopy} />
    </div>
  );
}

function UsageSettingsSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const [period, setPeriod] = useState<UsagePeriod>("7days");
  const [usageData, setUsageData] = useState<UsageBucket[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let canceled = false;
    setLoading(true);
    setError(null);

    void desktop
      .loadUsage(period)
      .then((response) => {
        if (canceled) {
          return;
        }
        setUsageData(response.usage);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load usage data");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop, period]);

  const totals = calculateUsageTotals(usageData);
  const topModels = Object.entries(totals.tokensByModel)
    .sort((left, right) => right[1] - left[1])
    .slice(0, 5);

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Usage</h3>
          <span className="pill pill-accent">Live query</span>
        </div>
        <div className="button-row">
          {(["today", "7days", "30days"] as const).map((candidate) => (
            <button
              key={candidate}
              type="button"
              className="secondary-button"
              onClick={() => setPeriod(candidate)}
            >
              {candidate}
            </button>
          ))}
        </div>
      </section>
      {error ? <ErrorBanner message={error} /> : null}
      <section className="surface-grid two-up">
        <article className="panel-card">
        <div className="card-header">
          <h3>Summary</h3>
          <span className="pill pill-accent">{period}</span>
        </div>
        {loading ? (
          <p className="panel-copy">Loading usage data from `/v1/usage/query`...</p>
        ) : (
          <dl className="meta-grid">
            <div>
              <dt>Total tokens</dt>
              <dd>{totals.totalTokens.toLocaleString()}</dd>
            </div>
            <div>
              <dt>Total cost</dt>
              <dd>${totals.totalCost.toFixed(4)}</dd>
            </div>
            <div>
              <dt>Usage buckets</dt>
              <dd>{usageData.length}</dd>
            </div>
            <div>
              <dt>Connected services</dt>
              <dd>{desktop.profile?.connectedServices.length ?? 0}</dd>
            </div>
          </dl>
        )}
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Top models</h3>
          <span className="pill pill-outline">Usage breakdown</span>
        </div>
        {loading ? (
          <p className="panel-copy">Computing desktop usage breakdown...</p>
        ) : topModels.length > 0 ? (
          <div className="settings-list">
            {topModels.map(([model, tokens]) => (
              <div key={model} className="settings-row static">
                <div className="settings-row-copy">
                  <strong>{model}</strong>
                  <p>{tokens.toLocaleString()} tokens</p>
                </div>
                <span className="settings-row-badge">
                  ${(totals.costByModel[model] ?? 0).toFixed(4)}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="panel-copy">
            No usage reports are available for the selected period yet.
          </p>
        )}
      </article>
      </section>
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Report buckets</h3>
            <span className="pill pill-outline">Recent points</span>
          </div>
          {loading ? (
            <p className="panel-copy">Loading recent usage buckets...</p>
          ) : usageData.length > 0 ? (
            <div className="settings-list">
              {usageData.slice(-5).reverse().map((bucket) => (
                <div key={bucket.timestamp} className="settings-row static">
                  <div className="settings-row-copy">
                    <strong>{new Date(bucket.timestamp * 1000).toLocaleString()}</strong>
                    <p>{bucket.reportCount} reports</p>
                  </div>
                  <span className="settings-row-badge">
                    {Object.values(bucket.tokens).reduce((sum, value) => sum + value, 0).toLocaleString()}
                  </span>
                </div>
              ))}
            </div>
          ) : (
            <p className="panel-copy">No recent usage buckets are available yet.</p>
          )}
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Review notes</h3>
            <span className="pill pill-outline">Backend path</span>
          </div>
          <ul className="bullet-list dense-list">
            <li>This route now queries real account usage through `/v1/usage/query`.</li>
            <li>Period changes update the server query instead of only reformatting local counts.</li>
            <li>Quota and billing mutations still remain part of the deeper account flow migration.</li>
          </ul>
        </article>
      </section>
    </div>
  );
}

function MobileUsageSettingsSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const [period, setPeriod] = useState<UsagePeriod>("7days");
  const [usageData, setUsageData] = useState<UsageBucket[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let canceled = false;
    setLoading(true);
    setError(null);

    void desktop
      .loadUsage(period)
      .then((response) => {
        if (canceled) {
          return;
        }
        setUsageData(response.usage);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load usage data");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop, period]);

  const totals = calculateUsageTotals(usageData);
  const topModels = Object.entries(totals.tokensByModel)
    .sort((left, right) => right[1] - left[1])
    .slice(0, 5);

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Usage</h3>
          <span className="pill pill-accent">Live query</span>
        </div>
        <div className="button-row">
          {(["today", "7days", "30days"] as const).map((candidate) => (
            <button
              key={candidate}
              type="button"
              className="secondary-button"
              onClick={() => setPeriod(candidate)}
            >
              {candidate}
            </button>
          ))}
        </div>
        {error ? <ErrorBanner message={error} /> : null}
        {loading ? (
          <p className="panel-copy">Loading usage data from `/v1/usage/query`...</p>
        ) : (
          <dl className="meta-grid">
            <div>
              <dt>Total tokens</dt>
              <dd>{totals.totalTokens.toLocaleString()}</dd>
            </div>
            <div>
              <dt>Total cost</dt>
              <dd>${totals.totalCost.toFixed(4)}</dd>
            </div>
          </dl>
        )}
      </section>
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Top models</h3>
          <span className="pill pill-outline">Usage breakdown</span>
        </div>
        {loading ? (
          <p className="panel-copy">Loading recent usage buckets...</p>
        ) : topModels.length > 0 ? (
          <div className="settings-list">
            {topModels.map(([model, tokens]) => (
              <div key={model} className="settings-row static">
                <div className="settings-row-copy">
                  <strong>{model}</strong>
                  <p>{tokens.toLocaleString()} tokens</p>
                </div>
                <span className="settings-row-badge">
                  ${(totals.costByModel[model] ?? 0).toFixed(4)}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="panel-copy">No usage reports are available for the selected period yet.</p>
        )}
      </section>
    </div>
  );
}

function VoiceSettingsSurface({
  preferences,
  onNavigate,
  nativeCapabilities,
}: {
  preferences: DesktopPreferencesState;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const { voiceSettings, commitVoicePatch } = preferences;
  const [customAgentDraft, setCustomAgentDraft] = useState(voiceSettings.customAgentId ?? "");
  const runtimeLabel =
    nativeCapabilities.runtimeTarget === "mobile"
      ? "Android shell"
      : nativeCapabilities.runtimeTarget === "browser"
        ? "browser shell"
        : "desktop shell";
  const capabilityBadge =
    nativeCapabilities.runtimeTarget === "mobile"
      ? "Android-backed"
      : nativeCapabilities.runtimeTarget === "browser"
        ? "Browser-backed"
        : "Desktop-backed";

  useEffect(() => {
    setCustomAgentDraft(voiceSettings.customAgentId ?? "");
  }, [voiceSettings.customAgentId]);

  const saveCustomAgentDraft = async () => {
    await commitVoicePatch({
      customAgentId: customAgentDraft.trim() || null,
    });
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>Voice</h3>
          <span className="pill pill-accent">Persisted</span>
        </div>
        <p className="panel-copy">
          Voice preferences now persist in the {runtimeLabel} so language and bring-your-own-agent
          review no longer depend on throwaway local preview state.
        </p>
        <label className="field-block">
          <span>Custom agent ID</span>
          <input
            value={customAgentDraft}
            onChange={(event) => setCustomAgentDraft(event.target.value)}
            placeholder="Optional desktop voice agent ID"
          />
        </label>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() => void saveCustomAgentDraft()}
            disabled={preferences.accountSettingsSyncing}
          >
            Save custom agent ID
          </button>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => {
              const patch = {
                bypassToken: !voiceSettings.bypassToken,
              };
              void commitVoicePatch(patch);
            }}
          >
            <div className="settings-row-copy">
              <strong>Bypass Vibe voice token</strong>
              <p>Keep the desktop BYO-agent switch state persisted across route reviews.</p>
            </div>
            <span className="settings-row-badge">
              {voiceSettings.bypassToken ? "Enabled" : "Disabled"}
            </span>
          </button>
        </div>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/settings/voice/language")}
          >
            Voice language
          </button>
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Status</h3>
          <span className="pill pill-outline">{capabilityBadge}</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Preferred language and custom agent settings are now persisted locally.</li>
          <li>{nativeCapabilities.voiceCapture.summary}</li>
          <li>Language review can proceed independently from live voice transport or capture APIs.</li>
        </ul>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() => {
              setCustomAgentDraft("");
              void commitVoicePatch(defaultVoiceSettings);
            }}
          >
            Reset voice settings
          </button>
        </div>
      </article>
      {preferences.accountSettingsError ? <ErrorBanner message={preferences.accountSettingsError} /> : null}
    </div>
  );
}

function MobileVoiceSettingsSurface({
  preferences,
  onNavigate,
  nativeCapabilities,
}: {
  preferences: DesktopPreferencesState;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const { voiceSettings, commitVoicePatch } = preferences;
  const [customAgentDraft, setCustomAgentDraft] = useState(voiceSettings.customAgentId ?? "");

  useEffect(() => {
    setCustomAgentDraft(voiceSettings.customAgentId ?? "");
  }, [voiceSettings.customAgentId]);

  const saveCustomAgentDraft = async () => {
    await commitVoicePatch({
      customAgentId: customAgentDraft.trim() || null,
    });
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Voice</h3>
          <span className="pill pill-accent">Persisted</span>
        </div>
        <label className="field-block">
          <span>Custom agent ID</span>
          <input
            value={customAgentDraft}
            onChange={(event) => setCustomAgentDraft(event.target.value)}
            placeholder="Optional Android voice agent ID"
          />
        </label>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => void saveCustomAgentDraft()}
            disabled={preferences.accountSettingsSyncing}
          >
            Save custom agent ID
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/settings/voice/language")}
          >
            Voice language
          </button>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => {
              void commitVoicePatch({ bypassToken: !voiceSettings.bypassToken });
            }}
          >
            <div className="settings-row-copy">
              <strong>Bypass Vibe voice token</strong>
              <p>Keep the mobile BYO-agent switch state persisted.</p>
            </div>
            <span className="settings-row-badge">
              {voiceSettings.bypassToken ? "Enabled" : "Disabled"}
            </span>
          </button>
        </div>
      </section>
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Status</h3>
          <span className="pill pill-outline">Android-backed</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Preferred language and custom agent settings are persisted locally.</li>
          <li>{nativeCapabilities.voiceCapture.summary}</li>
          <li>Language review can proceed independently from live voice transport or capture APIs.</li>
        </ul>
      </section>
    </div>
  );
}

function VoiceLanguageSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { voiceSettings, commitVoicePatch } = preferences;

  return (
    <div className="panel-card">
      <div className="card-header">
        <h3>Voice language</h3>
        <span className="pill pill-accent">Persisted route</span>
      </div>
      <div className="settings-list">
        <button
          type="button"
          className="settings-row"
          onClick={() => {
            const patch = { assistantLanguage: null };
            void commitVoicePatch(patch);
          }}
        >
          <div className="settings-row-copy">
            <strong>Auto-detect</strong>
            <p>Let the desktop voice route defer to runtime detection when supported.</p>
          </div>
          <span className="settings-row-badge">
            {voiceSettings.assistantLanguage === null ? "Selected" : "Available"}
          </span>
        </button>
        {desktopVoiceLanguages.map((language) => (
          <button
            key={language}
            type="button"
            className="settings-row"
            onClick={() => {
              const patch = { assistantLanguage: language };
              void commitVoicePatch(patch);
            }}
          >
            <div className="settings-row-copy">
              <strong>{language}</strong>
              <p>Persist the preferred desktop voice locale selection.</p>
            </div>
            <span className="settings-row-badge">
              {voiceSettings.assistantLanguage === language ? "Selected" : "Available"}
            </span>
          </button>
        ))}
      </div>
      {preferences.accountSettingsError ? <ErrorBanner message={preferences.accountSettingsError} /> : null}
    </div>
  );
}

function MobileVoiceLanguageSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
}) {
  const { voiceSettings, commitVoicePatch } = preferences;

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Voice language</h3>
          <span className="pill pill-accent">Persisted route</span>
        </div>
        <div className="settings-list">
          <button
            type="button"
            className="settings-row"
            onClick={() => {
              void commitVoicePatch({ assistantLanguage: null });
            }}
          >
            <div className="settings-row-copy">
              <strong>Auto-detect</strong>
              <p>Let Android defer to runtime detection when supported.</p>
            </div>
            <span className="settings-row-badge">
              {voiceSettings.assistantLanguage === null ? "Selected" : "Available"}
            </span>
          </button>
          {desktopVoiceLanguages.map((language) => (
            <button
              key={language}
              type="button"
              className="settings-row"
              onClick={() => {
                void commitVoicePatch({ assistantLanguage: language });
              }}
            >
              <div className="settings-row-copy">
                <strong>{language}</strong>
                <p>Persist the preferred Android voice locale selection.</p>
              </div>
              <span className="settings-row-badge">
                {voiceSettings.assistantLanguage === language ? "Selected" : "Available"}
              </span>
            </button>
          ))}
        </div>
      </section>
    </div>
  );
}

function ConnectClaudeSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);
  const isConnected = (desktop.profile?.connectedServices ?? []).includes("anthropic");

  const handleOpenDocs = async () => {
    try {
      await openExternalUrl("https://docs.anthropic.com/en/docs/claude-code/overview");
      setFeedback("Claude Code documentation opened in the external browser.");
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to open Claude Code docs");
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Claude Code</h3>
          <span className={`pill ${isConnected ? "pill-accent" : "pill-outline"}`}>
            {isConnected ? "Connected" : "Not connected"}
          </span>
        </div>
        <p className="panel-copy">
          This desktop vendor route mirrors the current app behavior: when direct OAuth is not the
          active desktop path, users still get an explicit command handoff instead of a dead-end.
        </p>
        <code className="backup-code">vibe connect claude</code>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() =>
              void copyTextToClipboard("vibe connect claude")
                .then(() => {
                  setFeedback("Copied: vibe connect claude");
                })
                .catch((error) => {
                  setFeedback(error instanceof Error ? error.message : "Failed to copy command");
                })
            }
          >
            Copy terminal command
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/terminal/index")}
          >
            Open terminal helpers
          </button>
          <button type="button" className="secondary-button" onClick={() => void handleOpenDocs()}>
            Open integration docs
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/settings/account")}
          >
            Back to account
          </button>
        </div>
      </section>
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Connected status</h3>
            <span className={`pill ${isConnected ? "pill-accent" : "pill-outline"}`}>
              {isConnected ? "Live account" : "Pending setup"}
            </span>
          </div>
          <ul className="bullet-list dense-list">
            <li>Current desktop profile state determines whether the service is already connected.</li>
            <li>The explicit terminal handoff matches the current app's unsupported-route behavior.</li>
            <li>Deeper OAuth/token mutation remains outside the current desktop package scope.</li>
          </ul>
        </article>
      </section>
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
    </div>
  );
}

function MobileConnectClaudeSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);
  const isConnected = (desktop.profile?.connectedServices ?? []).includes("anthropic");

  const handleOpenDocs = async () => {
    try {
      await openExternalUrl("https://docs.anthropic.com/en/docs/claude-code/overview");
      setFeedback("Claude Code documentation opened in the external browser.");
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to open Claude Code docs");
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Claude Code</h3>
          <span className={`pill ${isConnected ? "pill-accent" : "pill-outline"}`}>
            {isConnected ? "Connected" : "Not connected"}
          </span>
        </div>
        <p className="panel-copy">
          When direct in-app connect is not available, Android still exposes an explicit terminal handoff.
        </p>
        <code className="backup-code">vibe connect claude</code>
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() =>
              void copyTextToClipboard("vibe connect claude")
                .then(() => {
                  setFeedback("Copied: vibe connect claude");
                })
                .catch((error) => {
                  setFeedback(error instanceof Error ? error.message : "Failed to copy command");
                })
            }
          >
            Copy terminal command
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => void handleOpenDocs()}
          >
            Open integration docs
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/settings/account")}
          >
            Back to account
          </button>
        </div>
      </section>
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
    </div>
  );
}

function buildFriendAction(user: UserProfile, desktop: DesktopState) {
  if (user.status === "friend" || user.status === "requested") {
    return {
      label: user.status === "friend" ? "Remove friend" : "Cancel request",
      action: () => desktop.removeFriend(user.id),
    };
  }
  if (user.status === "pending") {
    return {
      label: "Accept request",
      action: () => desktop.addFriend(user.id),
    };
  }
  return {
    label: "Add friend",
    action: () => desktop.addFriend(user.id),
  };
}

function MobileFriendsIndexSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const pendingFriends = desktop.friends.filter((friend) => friend.status === "pending");
  const requestedFriends = desktop.friends.filter((friend) => friend.status === "requested");
  const acceptedFriends = desktop.friends.filter((friend) => friend.status === "friend");

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Friends</p>
        <h3>Manage your social graph</h3>
        <p className="panel-copy">
          Review pending requests, sent requests, and accepted friends from the Android shell.
        </p>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Pending requests</h3>
          <span className="pill pill-outline">{pendingFriends.length}</span>
        </div>
        <div className="settings-list">
          {pendingFriends.length > 0 ? (
            pendingFriends.map((friend) => (
              <button
                key={friend.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge">Pending</span>
              </button>
            ))
          ) : (
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>No pending requests</strong>
                <p>Incoming friend requests will appear here.</p>
              </div>
              <span className="settings-row-badge">Empty</span>
            </div>
          )}
        </div>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Sent requests</h3>
          <span className="pill pill-outline">{requestedFriends.length}</span>
        </div>
        <div className="settings-list">
          {requestedFriends.length > 0 ? (
            requestedFriends.map((friend) => (
              <button
                key={friend.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge">Requested</span>
              </button>
            ))
          ) : (
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>No sent requests</strong>
                <p>Outgoing requests will appear here.</p>
              </div>
              <span className="settings-row-badge">Empty</span>
            </div>
          )}
        </div>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>My friends</h3>
          <span className="pill pill-accent">{acceptedFriends.length}</span>
        </div>
        <div className="settings-list">
          {acceptedFriends.length > 0 ? (
            acceptedFriends.map((friend) => (
              <button
                key={friend.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${friend.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{friend.firstName || friend.username}</strong>
                  <p>@{friend.username}</p>
                </div>
                <span className="settings-row-badge connected">Friend</span>
              </button>
            ))
          ) : (
            <div className="settings-row static">
              <div className="settings-row-copy">
                <strong>No friends yet</strong>
                <p>Accepted friends will appear here.</p>
              </div>
              <span className="settings-row-badge">Empty</span>
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

function MobileFriendsSearchSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<UserProfile[]>([]);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const handleSearch = async () => {
    const trimmed = query.trim();
    if (!trimmed) {
      setResults([]);
      return;
    }

    setSearching(true);
    setError(null);
    try {
      const users = await desktop.searchUsers(trimmed);
      setResults(users);
    } catch (searchError) {
      setError(searchError instanceof Error ? searchError.message : "Failed to search users");
    } finally {
      setSearching(false);
    }
  };

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Friend search</p>
        <h3>Find people by username</h3>
        <label className="field-block">
          <span>Search</span>
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search by username"
          />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="primary-button full-width"
            onClick={() => void handleSearch()}
            disabled={searching}
          >
            {searching ? "Searching..." : "Search"}
          </button>
        </div>
      </section>

      <section className="panel-card mobile-settings-section">
        <div className="card-header">
          <h3>Results</h3>
          <span className="pill pill-outline">{results.length}</span>
        </div>
        {results.length > 0 ? (
          <div className="settings-list">
            {results.map((user) => (
              <button
                key={user.id}
                type="button"
                className="settings-row"
                onClick={() => onNavigate(`/(app)/user/${user.id}`)}
              >
                <div className="settings-row-copy">
                  <strong>{user.firstName || user.username}</strong>
                  <p>@{user.username}</p>
                </div>
                <span className="settings-row-badge">{user.status}</span>
              </button>
            ))}
          </div>
        ) : (
          <p className="panel-copy">
            Search by username to find and connect with other users.
          </p>
        )}
      </section>
    </div>
  );
}

function MobileUserDetailSurface({
  userId,
  desktop,
  onNavigate,
}: {
  userId: string;
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [user, setUser] = useState<UserProfile | null>(desktop.userProfiles[userId] ?? null);
  const [loading, setLoading] = useState(!desktop.userProfiles[userId]);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const cachedUser = desktop.userProfiles[userId] ?? null;
    if (cachedUser) {
      setUser(cachedUser);
      setLoading(false);
      return;
    }

    let canceled = false;
    setLoading(true);
    void desktop
      .loadUserProfile(userId)
      .then((profile) => {
        if (canceled) {
          return;
        }
        setUser(profile);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load profile");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [desktop, userId]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const applyFriendAction = async () => {
    if (!user) {
      return;
    }
    setWorking(true);
    setError(null);
    try {
      const action = buildFriendAction(user, desktop);
      const updated = await action.action();
      if (updated) {
        setUser(updated);
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : "Failed to update friend state");
    } finally {
      setWorking(false);
    }
  };

  if (loading) {
    return (
      <div className="panel-card empty-state-card">
        <h3>Loading profile</h3>
        <p className="panel-copy">Fetching the selected user profile.</p>
      </div>
    );
  }

  if (!user) {
    return (
      <EmptyState
        title="Profile not found"
        body="Return to friend search and try another user."
        actionLabel="Back to search"
        onAction={() => onNavigate("/(app)/friends/search")}
      />
    );
  }

  const action = buildFriendAction(user, desktop);

  return (
    <div className="surface-stack">
      <section className="panel-card mobile-settings-section">
        <p className="eyebrow">Profile</p>
        <h3>{user.firstName || user.username}</h3>
        <p className="panel-copy">@{user.username}</p>
        {user.bio ? <p className="panel-copy">{user.bio}</p> : null}
        <div className="pill-row">
          <span className="pill pill-outline">{user.status}</span>
        </div>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row mobile-button-column">
          <button
            type="button"
            className="primary-button full-width"
            onClick={() => void applyFriendAction()}
            disabled={working}
          >
            {working ? "Updating..." : action.label}
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => onNavigate("/(app)/friends/search")}
          >
            Back to search
          </button>
          <button
            type="button"
            className="secondary-button full-width"
            onClick={() => void openExternalUrl(`https://github.com/${user.username}`)}
          >
            Open GitHub profile
          </button>
        </div>
      </section>
    </div>
  );
}

function ChangelogSurface() {
  const entries = changelogData.entries.slice(0, 4);
  return (
    <div className="surface-stack">
      {entries.map((entry) => (
        <article key={`${entry.version}-${entry.date}`} className="panel-card">
          <div className="card-header">
            <div>
              <h3>{`Version ${entry.version}`}</h3>
              <p className="panel-copy small-copy">
                {entry.date}
              </p>
            </div>
            <span className="pill pill-accent">Desktop</span>
          </div>
          <p className="panel-copy">{entry.summary}</p>
          <ul className="bullet-list dense-list">
            {entry.changes.map((change) => (
              <li key={change}>{change}</li>
            ))}
          </ul>
        </article>
      ))}
    </div>
  );
}

function TerminalUtilitiesSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);

  const commandCards = useMemo(() => {
    const resumable = desktop.sessions
      .map((session) => {
        const command = buildResumeCommand(session.metadata ?? {});
        return command
          ? {
              title: describeSession(session).title,
              command,
              detail: "Resume this desktop session from the terminal using the current metadata.",
            }
          : null;
      })
      .filter(Boolean)
      .slice(0, 3) as Array<{
      title: string;
      command: string;
      detail: string;
    }>;

    const diagnostics = [
      {
        title: "Check daemon status",
        command: "vibe daemon status",
        detail: "Inspect whether the current desktop runtime is ready to resume or spawn sessions.",
      },
    ];

    return [...resumable, ...diagnostics];
  }, [desktop.sessions]);

  const handleCopy = async (command: string) => {
    try {
      await copyTextToClipboard(command);
      setFeedback(`Copied: ${command}`);
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to copy terminal command");
    }
  };

  return (
    <div className="surface-stack">
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
      <section className="surface-grid two-up">
        {commandCards.map((card) => (
          <article key={card.title} className="panel-card">
            <div className="card-header">
              <h3>{card.title}</h3>
              <span className="pill pill-outline">CLI</span>
            </div>
            <p className="panel-copy">{card.detail}</p>
            <code className="backup-code">{card.command}</code>
            <div className="button-row">
              <button type="button" className="secondary-button" onClick={() => void handleCopy(card.command)}>
                Copy command
              </button>
            </div>
          </article>
        ))}
      </section>
      <div className="button-row">
        <button
          type="button"
          className="secondary-button"
          onClick={() => onNavigate("/(app)/terminal/connect")}
        >
          Open connect flow
        </button>
      </div>
    </div>
  );
}

function TerminalConnectSurface({
  desktop,
  searchParams,
  onNavigate,
  nativeCapabilities,
}: {
  desktop: DesktopState;
  searchParams: URLSearchParams;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const seededKey = readTerminalConnectKey(searchParams) ?? "";
  const [keyInput, setKeyInput] = useState(seededKey);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setKeyInput(seededKey);
  }, [seededKey]);

  const handleConnect = async () => {
    if (!desktop.credentials) {
      onNavigate("/(app)/restore/index");
      return;
    }

    const publicKey = normalizeTerminalPublicKeyInput(keyInput);
    if (!publicKey) {
      setFeedback("Paste a terminal connection URL or public key first.");
      return;
    }

    setSubmitting(true);
    setFeedback(null);
    try {
      await approveTerminalConnection(desktop.serverUrl, desktop.credentials, publicKey);
      setFeedback("Terminal connection approved.");
      if (nativeCapabilities.notificationRouting.available) {
        void showDesktopNotification(
          "Terminal connected",
          "The desktop shell approved the terminal connection request.",
        ).catch(() => undefined);
      }
    } catch (error) {
      setFeedback(
        error instanceof Error ? error.message : "Failed to approve terminal connection",
      );
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>Terminal connect</h3>
          <span className="pill pill-accent">Live helper</span>
        </div>
        <label className="field-block">
          <span>Terminal auth URL or public key</span>
          <textarea
            rows={6}
            value={keyInput}
            onChange={(event) => setKeyInput(event.target.value)}
            placeholder="Paste vibe:///terminal?... or just the public key"
          />
        </label>
        <p className="panel-copy">
          This route now accepts a real terminal connection request and approves it against the live
          desktop account when credentials are available.
        </p>
        {feedback ? <div className="compact-feedback">{feedback}</div> : null}
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Next actions</h3>
          <span className="pill pill-outline">Desktop auth</span>
        </div>
        <div className="button-row">
          <button
            type="button"
            className="primary-button"
            onClick={() => void handleConnect()}
            disabled={submitting}
          >
            {submitting ? "Approving..." : "Approve terminal connection"}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/settings/connect/claude")}
          >
            Open vendor route
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/restore/index")}
          >
            {desktop.credentials ? "Switch account" : "Restore or link device"}
          </button>
        </div>
        <ul className="bullet-list dense-list">
          <li>The public key is processed locally in the desktop shell.</li>
          <li>Approval reuses the current desktop account credentials and encryption context.</li>
          <li>If you are signed out, restore or link an account before approving the request.</li>
        </ul>
      </article>
    </div>
  );
}

function ServerConfigSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [serverUrlDraft, setServerUrlDraft] = useState(desktop.serverUrl);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);

  useEffect(() => {
    setServerUrlDraft(desktop.serverUrl);
  }, [desktop.serverUrl]);

  const handleSave = async () => {
    setUpdating(true);
    setError(null);
    setFeedback(null);
    try {
      await desktop.updateServerUrl(serverUrlDraft);
      setFeedback("Desktop server URL updated.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to update server URL");
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>Server configuration</h3>
          <span className="pill pill-accent">Live route</span>
        </div>
        <label className="field-block">
          <span>Server URL</span>
          <input value={serverUrlDraft} onChange={(event) => setServerUrlDraft(event.target.value)} />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        {feedback ? <div className="compact-feedback">{feedback}</div> : null}
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleSave()} disabled={updating}>
            {updating ? "Saving..." : "Save server URL"}
          </button>
          <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/settings/index")}>
            Back to settings
          </button>
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Safety rules</h3>
          <span className="pill pill-outline">Validated input</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Desktop server URLs must use HTTPS unless they target localhost.</li>
          <li>Loopback HTTP remains reserved for local development and desktop auth callbacks.</li>
          <li>Changes are applied through the desktop adapter layer rather than hidden local mutation.</li>
        </ul>
      </article>
    </div>
  );
}

function TextSelectionSurface({
  nativeCapabilities,
}: {
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const [text, setText] = useState(
    "Paste or draft text here, then copy the normalized selection for desktop workflows.",
  );
  const [feedback, setFeedback] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  const handleCopy = async () => {
    try {
      await copyTextToClipboard(text.trim());
      setFeedback("Selection copied.");
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to copy selection");
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      const savedTo = await saveTextFileDialog({
        title: "Save selected text",
        suggestedName: "desktop-selection.txt",
        contents: text,
      });
      if (savedTo) {
        setFeedback(`Selection saved to ${savedTo}.`);
      }
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to save selection");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>Text selection utility</h3>
          <span className="pill pill-accent">Live utility</span>
        </div>
        <label className="field-block">
          <span>Selection text</span>
          <textarea rows={10} value={text} onChange={(event) => setText(event.target.value)} />
        </label>
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleCopy()}>
            Copy selection
          </button>
          {nativeCapabilities.fileExport.available ? (
            <button
              type="button"
              className="secondary-button"
              onClick={() => void handleSave()}
              disabled={saving}
            >
              {saving ? "Saving..." : "Save selection to file"}
            </button>
          ) : null}
        </div>
        {feedback ? <div className="compact-feedback">{feedback}</div> : null}
        {!nativeCapabilities.fileExport.available ? (
          <p className="panel-copy small-copy">{nativeCapabilities.fileExport.summary}</p>
        ) : null}
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Selection stats</h3>
          <span className="pill pill-outline">{text.length} chars</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>{text.trim().split(/\s+/).filter(Boolean).length} words</li>
          <li>{text.split(/\n/).length} lines</li>
          <li>Clipboard integration uses the same desktop-safe path as backup key and command copy.</li>
        </ul>
      </article>
    </div>
  );
}

function ArtifactsIndexSurface({
  desktop,
  artifacts,
  onNavigate,
}: {
  desktop: DesktopState;
  artifacts: DesktopArtifact[];
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  return (
    <div className="surface-stack">
      <section className="panel-card">
        <div className="card-header">
          <h3>Artifacts</h3>
          <span className="pill pill-accent">{artifacts.length} live</span>
        </div>
        <p className="panel-copy">
          This route now reads artifact inventory from the real backend. Create, edit, detail, and
          delete actions are desktop-backed instead of staying in retained local review state.
        </p>
        <div className="button-row">
          <button
            type="button"
            className="primary-button"
            onClick={() => onNavigate("/(app)/artifacts/new")}
          >
            Create artifact
          </button>
        </div>
      </section>
      <section className="surface-grid two-up">
        {artifacts.map((artifact) => (
          <article key={artifact.id} className="panel-card">
            <div className="card-header">
              <h3>{artifact.title || "Untitled artifact"}</h3>
              <span className="pill pill-outline">
                {artifact.draft ? "Draft" : artifact.isDecrypted ? "Published" : "Encrypted"}
              </span>
            </div>
            <p className="panel-copy">
              {(artifact.body ?? "").split(/\n+/).find(Boolean)?.slice(0, 120) ||
                "Artifact body is not loaded in the list route yet."}
            </p>
            <dl className="meta-grid compact-meta-grid">
              <div>
                <dt>Updated</dt>
                <dd>{new Date(artifact.updatedAt).toLocaleString()}</dd>
              </div>
              <div>
                <dt>Linked sessions</dt>
                <dd>{artifact.sessions?.length ?? 0}</dd>
              </div>
            </dl>
            <div className="button-row">
              <button
                type="button"
                className="secondary-button"
                onClick={() => onNavigate(`/(app)/artifacts/${artifact.id}`)}
              >
                Open artifact
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => onNavigate(`/(app)/artifacts/edit/${artifact.id}`)}
              >
                Edit
              </button>
            </div>
          </article>
        ))}
      </section>
    </div>
  );
}

function ArtifactCreateSurface({
  desktop,
  createArtifact,
  onNavigate,
  nativeCapabilities,
}: {
  desktop: DesktopState;
  createArtifact: (input: {
    title: string | null;
    body: string | null;
    sessions?: string[];
    draft?: boolean;
  }) => Promise<DesktopArtifact>;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const handleCreate = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const created = await createArtifact({
        title: title.trim() || null,
        body: body.trim() || null,
        sessions: desktop.sessionSummaries.slice(0, 1).map(({ session }) => session.id),
      });
      if (nativeCapabilities.notificationRouting.available) {
        void showDesktopNotification(
          "Artifact created",
          `${created.title || "Untitled artifact"} is now available in the desktop library.`,
        ).catch(() => undefined);
      }
      onNavigate(`/(app)/artifacts/${created.id}`);
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : "Failed to create artifact");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>Create artifact</h3>
          <span className="pill pill-accent">Live flow</span>
        </div>
        <label className="field-block">
          <span>Title</span>
          <input value={title} onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className="field-block">
          <span>Body</span>
          <textarea rows={12} value={body} onChange={(event) => setBody(event.target.value)} />
        </label>
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleCreate()} disabled={submitting}>
            {submitting ? "Creating..." : "Create live artifact"}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate("/(app)/artifacts/index")}
          >
            Back to artifacts
          </button>
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Scope</h3>
          <span className="pill pill-outline">P1 route</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>This route now creates artifacts through the real desktop backend chain.</li>
          <li>Artifact header and body are encrypted before the request is sent to the server.</li>
          <li>Session links are seeded from current desktop session context.</li>
        </ul>
      </article>
    </div>
  );
}

function ArtifactDetailSurface({
  artifactId,
  desktop,
  artifacts,
  deleteArtifact,
  loadArtifact,
  onNavigate,
  nativeCapabilities,
}: {
  artifactId: string;
  desktop: DesktopState;
  artifacts: DesktopArtifact[];
  deleteArtifact: (artifactId: string) => Promise<void>;
  loadArtifact: (artifactId: string) => Promise<DesktopArtifact | null>;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const artifact = artifacts.find((item) => item.id === artifactId) ?? null;
  const [deleting, setDeleting] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [savingFile, setSavingFile] = useState(false);

  useEffect(() => {
    if (!desktop.credentials || artifact?.body !== undefined) {
      return;
    }
    let canceled = false;

    void loadArtifact(artifactId).catch((error) => {
      if (!canceled) {
        setLoadError(
          error instanceof Error ? error.message : "Failed to load artifact body",
        );
      }
    });

    return () => {
      canceled = true;
    };
  }, [artifact?.body, artifactId, desktop.credentials, loadArtifact]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  if (!artifact) {
    return (
      <EmptyState
        title="Artifact not found"
        body="Return to the artifacts index and choose another artifact."
        actionLabel="Back to artifacts"
        onAction={() => onNavigate("/(app)/artifacts/index")}
      />
    );
  }

  const artifactBodyLoading = artifact.body === undefined && !loadError;
  const artifactBodyEncrypted = artifact.body === null && !artifact.isDecrypted;
  const artifactBodyUnavailable = artifact.body === undefined || artifactBodyEncrypted;

  const handleSaveBodyToFile = async () => {
    if (artifactBodyLoading) {
      setFeedback("Artifact body is still loading and cannot be exported yet.");
      return;
    }

    if (artifactBodyEncrypted) {
      setFeedback("Artifact body is encrypted on this desktop and cannot be exported.");
      return;
    }

    setSavingFile(true);
    setFeedback(null);
    try {
      const suggestedName = `${sanitizeDownloadFileName(
        artifact.title || artifact.id,
        "artifact",
      )}.md`;
      const savedTo = await saveTextFileDialog({
        title: "Save artifact body",
        suggestedName,
        contents: artifact.body ?? "",
      });
      if (savedTo) {
        setFeedback(`Artifact body saved to ${savedTo}.`);
      }
    } catch (error) {
      setFeedback(error instanceof Error ? error.message : "Failed to save artifact body");
    } finally {
      setSavingFile(false);
    }
  };

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Artifact detail</p>
          <h3>{artifact.title || "Untitled artifact"}</h3>
          <p className="hero-copy">
            {(artifact.body ?? "").split(/\n+/).find(Boolean)?.slice(0, 140) ||
              "Artifact body is loading or currently empty."}
          </p>
        </div>
        <div className="hero-actions">
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate(`/(app)/artifacts/edit/${artifact.id}`)}
          >
            Edit artifact
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => void handleSaveBodyToFile()}
            disabled={savingFile || artifactBodyUnavailable || !nativeCapabilities.fileExport.available}
          >
            {artifactBodyLoading
              ? "Loading body..."
              : artifactBodyEncrypted
                ? "Encrypted body"
                : !nativeCapabilities.fileExport.available
                  ? "Export unavailable"
                : savingFile
                  ? "Saving file..."
                  : "Save body to file"}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() =>
              void (async () => {
                setDeleting(true);
                setFeedback(null);
                try {
                  await deleteArtifact(artifact.id);
                  if (nativeCapabilities.notificationRouting.available) {
                    void showDesktopNotification(
                      "Artifact deleted",
                      `${artifact.title || "Untitled artifact"} was removed from the desktop library.`,
                    ).catch(() => undefined);
                  }
                  onNavigate("/(app)/artifacts/index");
                } catch (error) {
                  setFeedback(error instanceof Error ? error.message : "Failed to delete artifact");
                } finally {
                  setDeleting(false);
                }
              })()
            }
          >
            {deleting ? "Deleting..." : "Delete"}
          </button>
        </div>
      </section>
      {loadError ? <ErrorBanner message={loadError} /> : null}
      {feedback ? <div className="panel-card compact-feedback">{feedback}</div> : null}
      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>Body</h3>
            <span className="pill pill-outline">
              {artifact.draft ? "Draft" : artifact.isDecrypted ? "Published" : "Encrypted"}
            </span>
          </div>
          <pre className="diff-line diff-line-context">
            {artifact.body ?? (loadError ? "Artifact body failed to load." : "Loading artifact body...")}
          </pre>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Metadata</h3>
            <span className="pill pill-accent">{artifact.sessions?.length ?? 0} linked</span>
          </div>
          <dl className="meta-grid">
            <div>
              <dt>Created</dt>
              <dd>{new Date(artifact.createdAt).toLocaleString()}</dd>
            </div>
            <div>
              <dt>Updated</dt>
              <dd>{new Date(artifact.updatedAt).toLocaleString()}</dd>
            </div>
          </dl>
          <ul className="bullet-list dense-list">
            {(artifact.sessions ?? []).map((sessionId) => (
              <li key={sessionId}>{sessionId}</li>
            ))}
          </ul>
          {!nativeCapabilities.fileExport.available ? (
            <p className="panel-copy small-copy">{nativeCapabilities.fileExport.summary}</p>
          ) : null}
        </article>
      </section>
    </div>
  );
}

function ArtifactEditSurface({
  artifactId,
  desktop,
  artifacts,
  updateArtifact,
  loadArtifact,
  onNavigate,
  nativeCapabilities,
}: {
  artifactId: string;
  desktop: DesktopState;
  artifacts: DesktopArtifact[];
  updateArtifact: (
    artifactId: string,
    patch: {
      title: string | null;
      body: string | null;
      sessions?: string[];
      draft?: boolean;
    },
  ) => Promise<DesktopArtifact>;
  loadArtifact: (artifactId: string) => Promise<DesktopArtifact | null>;
  onNavigate: (path: string) => void;
  nativeCapabilities: RuntimeNativeCapabilities;
}) {
  const artifact = artifacts.find((item) => item.id === artifactId) ?? null;
  const [title, setTitle] = useState(artifact?.title ?? "");
  const [body, setBody] = useState(artifact?.body ?? "");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    if (!desktop.credentials || artifact?.body !== undefined) {
      return;
    }
    let canceled = false;

    void loadArtifact(artifactId).catch((loadArtifactError) => {
      if (!canceled) {
        setLoadError(
          loadArtifactError instanceof Error
            ? loadArtifactError.message
            : "Failed to load artifact body",
        );
      }
    });

    return () => {
      canceled = true;
    };
  }, [artifact?.body, artifactId, desktop.credentials, loadArtifact]);

  useEffect(() => {
    setTitle(artifact?.title ?? "");
    setBody(artifact?.body ?? "");
  }, [artifact?.body, artifact?.title]);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  if (!artifact) {
    return (
      <EmptyState
        title="Artifact not found"
        body="The requested artifact no longer exists."
        actionLabel="Back to artifacts"
        onAction={() => onNavigate("/(app)/artifacts/index")}
      />
    );
  }

  const handleSave = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const updated = await updateArtifact(artifactId, {
        title: title.trim() || null,
        body: body.trim() || null,
        sessions: artifact.sessions,
        draft: artifact.draft,
      });
      if (nativeCapabilities.notificationRouting.available) {
        void showDesktopNotification(
          "Artifact updated",
          `${updated.title || "Untitled artifact"} was saved from the desktop editor.`,
        ).catch(() => undefined);
      }
      onNavigate(`/(app)/artifacts/${artifactId}`);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to update artifact");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="surface-grid two-up">
      <article className="panel-card form-card">
        <div className="card-header">
          <h3>Edit artifact</h3>
          <span className="pill pill-accent">
            {artifact.draft ? "Draft" : artifact.isDecrypted ? "Published" : "Encrypted"}
          </span>
        </div>
        <label className="field-block">
          <span>Title</span>
          <input value={title} onChange={(event) => setTitle(event.target.value)} />
        </label>
        <label className="field-block">
          <span>Body</span>
          <textarea rows={12} value={body} onChange={(event) => setBody(event.target.value)} />
        </label>
        {loadError ? <ErrorBanner message={loadError} /> : null}
        {error ? <ErrorBanner message={error} /> : null}
        <div className="button-row">
          <button type="button" className="primary-button" onClick={() => void handleSave()} disabled={submitting}>
            {submitting ? "Saving..." : "Save live artifact"}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={() => onNavigate(`/(app)/artifacts/${artifactId}`)}
          >
            Cancel
          </button>
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Desktop review notes</h3>
          <span className="pill pill-outline">Local state</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Edits now persist through the real backend artifact mutation path.</li>
          <li>Encrypted header and body versions stay tracked for conflict detection.</li>
          <li>Route navigation and state continuity remain reviewable end to end.</li>
        </ul>
      </article>
    </div>
  );
}

function UserDetailSurface({
  userId,
  desktop,
  onNavigate,
}: {
  userId: string;
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const [user, setUser] = useState<typeof desktop.userProfiles[string] | null>(
    desktop.userProfiles[userId] ?? null,
  );
  const [loading, setLoading] = useState(!desktop.userProfiles[userId]);
  const [error, setError] = useState<string | null>(null);
  const cachedUser = desktop.userProfiles[userId] ?? null;

  useEffect(() => {
    if (cachedUser) {
      setUser(cachedUser);
      setLoading(false);
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .loadUserProfile(userId)
      .then((profile) => {
        if (canceled) {
          return;
        }
        setUser(profile);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load user profile");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [cachedUser, desktop.loadUserProfile, userId]);

  if (loading) {
    return (
      <section className="panel-card empty-state-card">
        <h3>Loading user</h3>
        <p className="panel-copy">Fetching the current desktop profile detail from the backend.</p>
      </section>
    );
  }

  if (error) {
    return <ErrorBanner message={error} />;
  }

  if (!user) {
    return (
      <EmptyState
        title="User not found"
        body="This desktop profile is not currently available."
        actionLabel="Back to settings"
        onAction={() => onNavigate("/(app)/settings/index")}
      />
    );
  }

  const displayName = [user.firstName, user.lastName].filter(Boolean).join(" ").trim() || user.username;

  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>{displayName}</h3>
          <span className="pill pill-accent">{user.status}</span>
        </div>
        <p className="panel-copy">@{user.username}</p>
        <p className="panel-copy">{user.bio ?? "No bio available."}</p>
        <div className="button-row">
          <button
            type="button"
            className="secondary-button"
            onClick={() => void openExternalUrl(`https://github.com/${user.username}`)}
          >
            Open GitHub
          </button>
        </div>
      </article>
      <article className="panel-card">
        <div className="card-header">
          <h3>Desktop profile route</h3>
          <span className="pill pill-outline">Backend</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Profile detail now loads from the real backend user endpoint.</li>
          <li>Current account routes still reuse live desktop profile state when the IDs match.</li>
          <li>Friendship mutations remain a later promotion-scope follow-up.</li>
        </ul>
      </article>
    </div>
  );
}

function MachineDetailSurface({
  machineId,
  desktop,
  onNavigate,
}: {
  machineId: string;
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const [machine, setMachine] = useState<DesktopState["machines"][number] | null>(
    desktop.machines.find((item) => item.id === machineId) ?? null,
  );
  const [loading, setLoading] = useState(!desktop.machines.find((item) => item.id === machineId));
  const [error, setError] = useState<string | null>(null);
  const cachedMachine = desktop.machines.find((item) => item.id === machineId) ?? null;

  useEffect(() => {
    if (cachedMachine) {
      setMachine(cachedMachine);
      setLoading(false);
      return;
    }

    let canceled = false;
    setLoading(true);
    setError(null);
    void desktop
      .loadMachine(machineId)
      .then((loadedMachine) => {
        if (canceled) {
          return;
        }
        setMachine(loadedMachine);
        setLoading(false);
      })
      .catch((loadError) => {
        if (canceled) {
          return;
        }
        setError(loadError instanceof Error ? loadError.message : "Failed to load machine");
        setLoading(false);
      });

    return () => {
      canceled = true;
    };
  }, [cachedMachine, desktop.loadMachine, machineId]);

  if (loading) {
    return (
      <section className="panel-card empty-state-card">
        <h3>Loading machine</h3>
        <p className="panel-copy">Fetching the current machine detail from the backend.</p>
      </section>
    );
  }

  if (error) {
    return <ErrorBanner message={error} />;
  }

  const recentSessions = desktop.sessionSummaries
    .filter(({ session }) => {
      const host = typeof session.metadata?.host === "string" ? session.metadata.host : "";
      const machineRef =
        typeof session.metadata?.machineId === "string" ? session.metadata.machineId : "";
      return host === machine?.metadata?.host || machineRef === machine?.id;
    })
    .slice(0, 3);

  if (!machine) {
    return (
      <EmptyState
        title="Machine not found"
        body="This machine route has no current backend snapshot."
        actionLabel="Back to home"
        onAction={() => onNavigate("/(app)/index")}
      />
    );
  }

  return (
    <div className="surface-stack">
      <section className="hero-panel compact-hero">
        <div>
          <p className="eyebrow">Machine detail</p>
          <h3>{machine.metadata?.displayName || machine.metadata?.host || machine.id}</h3>
          <p className="hero-copy">
            Current machine metadata and recent related sessions loaded from the desktop backend.
          </p>
        </div>
        <div className="hero-meta">
          <span className="pill pill-outline">{machine.metadata?.platform ?? "Unknown platform"}</span>
          <span className={`status-chip status-${machine.active ? "active" : "ready"}`}>
            {machine.active ? "Online" : "Offline"}
          </span>
        </div>
      </section>
      <section className="surface-grid two-up">
        <article className="panel-card">
        <div className="card-header">
          <h3>Machine metadata</h3>
          <span className="pill pill-accent">{machine.metadata?.host ?? machine.id}</span>
          </div>
          <dl className="meta-grid">
            <div>
              <dt>Home directory</dt>
              <dd>{machine.metadata?.homeDir ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Status</dt>
              <dd>{machine.active ? "Online" : "Offline"}</dd>
            </div>
            <div>
              <dt>CLI version</dt>
              <dd>{machine.metadata?.happyCliVersion ?? "Unavailable"}</dd>
            </div>
            <div>
              <dt>Last active</dt>
              <dd>{new Date(machine.activeAt).toLocaleString()}</dd>
            </div>
          </dl>
        </article>
        <article className="panel-card">
          <div className="card-header">
            <h3>Related sessions</h3>
            <span className="pill pill-outline">{recentSessions.length} mapped</span>
          </div>
          {recentSessions.length > 0 ? (
            <div className="settings-list">
              {recentSessions.map(({ session, title, detail }) => (
                <button
                  key={session.id}
                  type="button"
                  className="settings-row"
                  onClick={() => onNavigate(`/(app)/session/${session.id}`)}
                >
                  <div className="settings-row-copy">
                    <strong>{title}</strong>
                    <p>{detail}</p>
                  </div>
                  <span className="settings-row-badge">Open</span>
                </button>
              ))}
            </div>
          ) : (
            <p className="panel-copy">
              No current desktop session metadata resolves directly to this machine yet.
            </p>
          )}
        </article>
      </section>
    </div>
  );
}

function ProfileAvatar({ label }: { label: string }) {
  const initials = label
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((segment) => segment[0]?.toUpperCase() ?? "")
    .join("");

  return <div className="profile-avatar">{initials || "VD"}</div>;
}

function formatProfileName(
  firstName?: string | null,
  lastName?: string | null,
  username?: string | null,
  fallback?: string,
): string {
  const joined = [firstName, lastName].filter(Boolean).join(" ").trim();
  if (joined) {
    return joined;
  }
  if (username?.trim()) {
    return `@${username.trim()}`;
  }
  return fallback || "Vibe Desktop";
}

function formatServiceLabel(service: string): string {
  const normalized = service.trim().toLowerCase();
  if (normalized === "anthropic") {
    return "Claude Code";
  }
  if (normalized === "github") {
    return "GitHub";
  }
  return service
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((chunk) => chunk[0].toUpperCase() + chunk.slice(1))
    .join(" ");
}

function PlannedSurface({
  title,
  canonicalPath,
  summary,
  ownerArea,
  promotionClass,
  onNavigate,
}: {
  title: string;
  canonicalPath: string;
  summary: string;
  ownerArea?: (typeof wave8FeatureAreas)[number];
  promotionClass: string;
  onNavigate: (path: string) => void;
}) {
  return (
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>{title}</h3>
          <span className={`pill pill-${promotionClass.toLowerCase()}`}>{promotionClass}</span>
        </div>
        <p className="panel-copy">{summary}</p>
        <div className="done-block">
          <strong>Canonical path</strong>
          <p>{canonicalPath}</p>
        </div>
        <ul className="bullet-list dense-list">
          {routeReviewNotes.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      </article>

      <article className="panel-card">
        <div className="card-header">
          <h3>Owning slice</h3>
          <span className="pill pill-outline">Planned surface</span>
        </div>
        {ownerArea ? (
          <>
            <p className="panel-copy">{ownerArea.description}</p>
            <ul className="bullet-list dense-list">
              {ownerArea.features.map((feature) => (
                <li key={feature}>{feature}</li>
              ))}
            </ul>
          </>
        ) : null}
        <div className="done-block">
          <strong>Examples in this slice</strong>
          <ul className="bullet-list dense-list">
            {plannedSurfaceExamples.map((example) => (
              <li key={example}>{example}</li>
            ))}
          </ul>
        </div>
        <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/index")}>
          Return to desktop entry
        </button>
      </article>
    </div>
  );
}

function SignedOutState({ onNavigate }: { onNavigate: (path: string) => void }) {
  const runtimeProfile = useRuntimeBootstrapProfile();
  const runtimeCopy = resolveRuntimeShellCopy(runtimeProfile?.runtimeTarget ?? "desktop");

  return (
    <EmptyState
      title="Sign in required"
      body={`Create or restore an account first, then return to the ${runtimeCopy.surfaceLabel} shell to load real sessions and messages.`}
      actionLabel="Restore or link account"
      onAction={() => onNavigate("/(app)/restore/index")}
    />
  );
}

function EmptyState({
  title,
  body,
  actionLabel,
  onAction,
}: {
  title: string;
  body: string;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <section className="panel-card empty-state-card">
      <h3>{title}</h3>
      <p className="panel-copy">{body}</p>
      <div className="button-row">
        <button type="button" className="primary-button" onClick={onAction}>
          {actionLabel}
        </button>
      </div>
    </section>
  );
}

function ErrorBanner({
  message,
  actionLabel,
  onAction,
}: {
  message: string;
  actionLabel?: string;
  onAction?: () => void;
}) {
  return (
    <div className="error-banner">
      <span>{message}</span>
      {actionLabel && onAction ? (
        <button type="button" className="ghost-button" onClick={onAction}>
          {actionLabel}
        </button>
      ) : null}
    </div>
  );
}

function TimelineMessageBody({
  message,
  appearanceSettings,
}: {
  message: UiMessage;
  appearanceSettings: DesktopAppearanceSettings;
}) {
  return (
    <Suspense fallback={<PlainTimelineMessageBody message={message} />}>
      <RichTimelineMessageBody
        message={message}
        options={{
          showLineNumbersInDiffs: appearanceSettings.showLineNumbersInDiffs,
          showLineNumbersInToolViews: appearanceSettings.showLineNumbersInToolViews,
          wrapLinesInDiffs: appearanceSettings.wrapLinesInDiffs,
        }}
      />
    </Suspense>
  );
}

function PlainTimelineMessageBody({ message }: { message: UiMessage }) {
  return (
    <div className="markdown-surface">
      <p className="markdown-paragraph">{message.text}</p>
    </div>
  );
}

function SidebarLink({
  route,
  active,
  onNavigate,
  shortcut,
  labelOverride,
  pathOverride,
}: {
  route: { label: string; examplePath: string; promotionClass: string };
  active: boolean;
  onNavigate: (path: string) => void;
  shortcut?: string;
  labelOverride?: string;
  pathOverride?: string;
}) {
  const path = pathOverride ?? route.examplePath;
  return (
    <a
      href={hrefForPath(path)}
      className={active ? "sidebar-link active" : "sidebar-link"}
      aria-current={active ? "page" : undefined}
      onClick={(event) => handleNavigation(event, path, onNavigate)}
    >
      <span>{labelOverride ?? route.label}</span>
      <small>{shortcut ?? route.promotionClass}</small>
    </a>
  );
}

function handleNavigation(
  event: React.MouseEvent<HTMLAnchorElement>,
  path: string,
  onNavigate: (path: string) => void,
) {
  event.preventDefault();
  onNavigate(path);
}

function shortcutForRoute(path: string): string | undefined {
  const match = Object.entries(desktopHotkeyRoutes).find(([, value]) => value === path);
  return match ? `Alt+${match[0]}` : undefined;
}

function statusLabel(status: DesktopState["status"]): string {
  switch (status) {
    case "checking":
      return "Checking";
    case "loading":
      return "Loading";
    case "ready":
      return "Ready";
    case "signed-out":
      return "Signed out";
    default:
      return status;
  }
}

function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diffMs = Math.max(0, now - timestamp);
  const diffMinutes = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMinutes < 1) {
    return "just now";
  }
  if (diffMinutes < 60) {
    return `${diffMinutes} minute${diffMinutes === 1 ? "" : "s"} ago`;
  }
  if (diffHours < 24) {
    return `${diffHours} hour${diffHours === 1 ? "" : "s"} ago`;
  }
  return `${diffDays} day${diffDays === 1 ? "" : "s"} ago`;
}

function timelineAccent(role: "user" | "assistant" | "system" | "tool"): "amber" | "teal" | "slate" | "neutral" {
  if (role === "user") {
    return "amber";
  }
  if (role === "assistant") {
    return "teal";
  }
  if (role === "tool") {
    return "neutral";
  }
  return "slate";
}
