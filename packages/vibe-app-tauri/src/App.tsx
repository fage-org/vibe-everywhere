import { Suspense, lazy, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  firstUsableSlice,
  formatFeatureCount,
  formatModuleCount,
  lockedAuthCallbackRequirements,
  promotionSlice,
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
} from "../../vibe-app/sources/text/_all";
import {
  DEFAULT_PATH,
  desktopRoutes,
  hrefForPath,
  primaryNavigation,
  promotionNavigation,
  resolveRoute,
  routeInventoryByClass,
  type ResolvedRoute,
  useDesktopRouter,
} from "./router";
import { useWave8Desktop } from "./useWave8Desktop";
import changelogData from "../../vibe-app/sources/changelog/changelog.json";
import { buildResumeCommand } from "../../vibe-app/sources/utils/resumeCommand";
import {
  approveTerminalConnection,
  calculateUsageTotals,
  type UsageBucket,
  type UsagePeriod,
  copyTextToClipboard,
  type DesktopArtifact,
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

const RichTimelineMessageBody = lazy(() =>
  import("./rich-message-renderers").then((module) => ({
    default: module.RichTimelineMessageBody,
  })),
);

const keyboardShortcuts = [
  { keys: "Ctrl/Cmd+K", description: "Open the desktop route palette" },
  { keys: "?", description: "Open overlay help from the shell" },
  { keys: "Alt+1", description: "Jump to the desktop entry route" },
  { keys: "Alt+2", description: "Jump to Inbox" },
  { keys: "Alt+3", description: "Jump to New Session" },
  { keys: "Alt+4", description: "Jump to Recent Sessions" },
  { keys: "Alt+5", description: "Jump to Settings" },
  { keys: "Alt+6", description: "Jump to Restore" },
  { keys: "Esc", description: "Dismiss the command palette" },
] as const;

type MainViewTab = "sessions" | "inbox" | "settings";

const mainViewTabs: Array<{
  key: MainViewTab;
  label: string;
  eyebrow: string;
}> = [
  { key: "sessions", label: "Sessions", eyebrow: "Live work" },
  { key: "inbox", label: "Inbox", eyebrow: "Updates" },
  { key: "settings", label: "Settings", eyebrow: "Desktop" },
];

const settingsFeatureLinks = [
  {
    title: "Account",
    subtitle: "Identity, subscription, restore history, and connected service status.",
    route: "/(app)/settings/account",
    badge: "Profile",
  },
  {
    title: "Appearance",
    subtitle: "Theme, layout density, and desktop chrome preferences.",
    route: "/(app)/settings/appearance",
    badge: "Theme",
  },
  {
    title: "Voice Assistant",
    subtitle: "Voice controls, language, and desktop microphone preferences.",
    route: "/(app)/settings/voice",
    badge: "Audio",
  },
  {
    title: "Features",
    subtitle: "Feature flags and staged rollout controls for the desktop rewrite.",
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

const aboutLinks = [
  {
    title: "What's New",
    subtitle: "Release notes and migration progress for the desktop rewrite.",
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
    subtitle: "Open a bug report for desktop rewrite regressions.",
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

const DESKTOP_PREVIEW_VERSION = "0.1.0-preview";

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

export function App() {
  const { path, navigate } = useDesktopRouter();
  const [commandOpen, setCommandOpen] = useState(false);

  useEffect(() => {
    if (typeof window === "undefined") {
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
  }, [navigate]);

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
    />
  );
}

export function DesktopShell({
  path,
  commandOpen,
  onNavigate,
  onCommandOpen,
  onCommandClose,
}: DesktopShellProps) {
  const resolved = useMemo(() => resolveRoute(path), [path]);
  const activeRoute = resolved.definition;
  const liveSessionId =
    activeRoute.key === "session-detail" ? resolved.params.id ?? null : null;
  const desktop = useWave8Desktop(liveSessionId);
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
    const mediaQuery =
      typeof window !== "undefined" && typeof window.matchMedia === "function"
        ? window.matchMedia("(prefers-color-scheme: dark)")
        : null;
    const resolvedTheme = resolveDesktopThemePreference(
      appearanceSettings.themePreference,
      mediaQuery?.matches,
    );

    root.dataset.theme = resolvedTheme;
    root.dataset.density = appearanceSettings.density;
    root.lang = languageSettings.appLanguage;

    return () => {
      delete root.dataset.theme;
      delete root.dataset.density;
      root.removeAttribute("lang");
    };
  }, [appearanceSettings.density, appearanceSettings.themePreference, languageSettings.appLanguage]);

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
          <p className="eyebrow">Wave 8</p>
          <h1>Vibe Desktop Next</h1>
          <p className="brand-copy">
            Desktop rewrite workspace with real account bootstrapping, session loading,
            and message flow against the live Vibe backend.
          </p>
          <div className="pill-row">
            <span className="pill pill-accent">B17 active</span>
            <span className="pill pill-muted">{formatModuleCount(wave8Modules)}</span>
          </div>
        </div>

        <nav className="nav-block" aria-label="Primary desktop routes">
          <div className="section-heading">
            <span>Primary routes</span>
            <small>P0 shell</small>
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
            <span>Promotion routes</span>
            <small>P1 visibility</small>
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
            <span>Desktop status</span>
            <small>Current boot</small>
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
              <span className={`pill pill-priority pill-${activeRoute.promotionClass.toLowerCase()}`}>
                {activeRoute.promotionClass}
              </span>
              <span className="pill pill-outline">{activeRoute.ownerModule}</span>
              <span className="pill pill-muted">{statusLabel(desktop.status)}</span>
            </div>
            <button className="command-trigger" type="button" onClick={onCommandOpen}>
              Open Palette
              <span>Ctrl/Cmd+K</span>
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
            />
          </main>

          <aside className="inspector-panel">
            <section className="panel-card inspector-route">
              <div className="card-header">
                <h3>Route inspector</h3>
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
                <h3>Wave 8 scope</h3>
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
                <h3>Route inventory</h3>
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

            <section className="panel-card">
              <div className="card-header">
                <h3>Keyboard map</h3>
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
                <h3 id="command-palette-title">Desktop route launcher</h3>
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

type DesktopState = ReturnType<typeof useWave8Desktop>;

type RouteSurfaceProps = {
  resolved: ResolvedRoute;
  desktop: DesktopState;
  preferences: DesktopPreferencesState;
  secondarySurfaces: SecondarySurfaceState;
  onNavigate: (path: string) => void;
};

function RouteSurface({
  resolved,
  desktop,
  preferences,
  secondarySurfaces,
  onNavigate,
}: RouteSurfaceProps) {
  const { definition } = resolved;

  switch (definition.key) {
    case "home":
      return <HomeSurface desktop={desktop} onNavigate={onNavigate} />;
    case "restore-index":
      return <RestoreSurface desktop={desktop} onNavigate={onNavigate} />;
    case "restore-manual":
      return <ManualRestoreSurface desktop={desktop} onNavigate={onNavigate} />;
    case "inbox":
      return <InboxSurface desktop={desktop} onNavigate={onNavigate} />;
    case "new-session":
      return <NewSessionSurface desktop={desktop} onNavigate={onNavigate} />;
    case "session-detail":
      return (
        <SessionSurface
          desktop={desktop}
          preferences={preferences}
          sessionId={resolved.params.id ?? ""}
          onNavigate={onNavigate}
        />
      );
    case "session-info":
      return (
        <SessionInfoSurface
          sessionId={resolved.params.id ?? ""}
          desktop={desktop}
          onNavigate={onNavigate}
        />
      );
    case "session-files":
      return (
        <SessionFilesSurface
          sessionId={resolved.params.id ?? ""}
          desktop={desktop}
          secondarySurfaces={secondarySurfaces}
          onNavigate={onNavigate}
        />
      );
    case "session-file":
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
      return <SettingsSurface desktop={desktop} onNavigate={onNavigate} />;
    case "settings-account":
      return <AccountSettingsSurface desktop={desktop} onNavigate={onNavigate} />;
    case "settings-appearance":
      return <AppearanceSettingsSurface preferences={preferences} />;
    case "settings-features":
      return <FeatureSettingsSurface preferences={preferences} />;
    case "settings-language":
      return <LanguageSettingsSurface preferences={preferences} />;
    case "settings-usage":
      return <UsageSettingsSurface desktop={desktop} onNavigate={onNavigate} />;
    case "settings-voice":
      return <VoiceSettingsSurface preferences={preferences} onNavigate={onNavigate} />;
    case "settings-voice-language":
      return <VoiceLanguageSurface preferences={preferences} />;
    case "settings-connect-claude":
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
        />
      );
    case "user-detail":
      return (
        <UserDetailSurface
          userId={resolved.params.id ?? ""}
          desktop={desktop}
          onNavigate={onNavigate}
        />
      );
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
      return <TextSelectionSurface />;
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
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
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
        <section className="hero-panel">
          <div>
            <p className="eyebrow">Desktop entry</p>
            <h3>Create or restore a Vibe desktop account</h3>
            <p className="hero-copy">
              Sign in with a fresh account, restore from a backup key, or link an
              existing mobile account to reach the live session flow.
            </p>
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
              <h3>B17 First Usable Slice</h3>
              <span className="pill pill-accent">P0 critical path</span>
            </div>
            <ul className="bullet-list">
              {firstUsableSlice.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </article>

          <article className="panel-card">
            <div className="card-header">
              <h3>B18 Promotion Readiness</h3>
              <span className="pill pill-outline">P1 before switch</span>
            </div>
            <ul className="bullet-list">
              {promotionSlice.map((item) => (
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
        <div>
          <p className="eyebrow">Desktop entry</p>
          <h3>Continue with your desktop sessions</h3>
          <p className="hero-copy">
            The home route now mirrors the current app more closely: sessions stay
            front and center, inbox and settings remain one tab away, and account
            recovery is still available from the same shell.
          </p>
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
            This backup key restores the desktop app without relying on the current machine.
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
        <div className="mainview-tabbar" role="tablist" aria-label="Desktop main view tabs">
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
                body="Create the first desktop session to verify the live backend path end to end."
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
                        <p>Connected on the current desktop account.</p>
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
                  Session activity will appear here as soon as the desktop account has
                  at least one live session.
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
                <p>Review connected accounts, feature routes, desktop configuration, and about links.</p>
              </div>
              <span className="settings-row-badge">Hub</span>
            </button>
          </div>
        ) : null}
      </section>

      <section className="surface-grid two-up">
        <article className="panel-card">
          <div className="card-header">
            <h3>First usable slice</h3>
            <span className="pill pill-accent">P0 critical path</span>
          </div>
          <ul className="bullet-list">
            {firstUsableSlice.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>

        <article className="panel-card">
          <div className="card-header">
            <h3>B18 Promotion Readiness</h3>
            <span className="pill pill-outline">P1 before switch</span>
          </div>
          <ul className="bullet-list">
            {promotionSlice.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>
      </section>
    </div>
  );
}

function RestoreSurface({
  desktop,
  onNavigate,
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
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
          <p className="hero-copy">
            The route immediately starts the mobile-link request, just like the current
            desktop restore entry. You can still create a fresh account or restore from
            the backup secret key instead.
          </p>
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
            <h3>Mobile link request</h3>
            <span className="pill pill-p0">Live link</span>
          </div>
          <p className="panel-copy">
            Start a request, then scan the QR code with the current Vibe mobile app and approve the device link.
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
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [secret, setSecret] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loadingFile, setLoadingFile] = useState(false);

  const handleLoadKeyFile = async () => {
    setLoadingFile(true);
    setError(null);
    try {
      const nextSecret = await openTextFileDialog("Load desktop backup key");
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
          <button
            type="button"
            className="secondary-button"
            onClick={() => void handleLoadKeyFile()}
            disabled={loadingFile}
          >
            {loadingFile ? "Loading key..." : "Load key file"}
          </button>
          <button type="button" className="secondary-button" onClick={() => onNavigate("/(app)/restore/index")}>
            Back to restore
          </button>
        </div>
      </article>

      <article className="panel-card">
        <div className="card-header">
          <h3>What this unlocks</h3>
          <span className="pill pill-outline">P0</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Challenge auth returns a real bearer token from the current Vibe backend.</li>
          <li>Encrypted session metadata and message content can be decrypted with the restored secret.</li>
          <li>The same secret key works across fresh installs and desktop rebuilds.</li>
        </ul>
      </article>
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
  const [workspace, setWorkspace] = useState("/root/vibe-remote");
  const [model, setModel] = useState("gpt-5.4");
  const [title, setTitle] = useState("Wave 8 Desktop Session");
  const [prompt, setPrompt] = useState("Continue the Wave 8 desktop rewrite and report what changed.");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const handleCreate = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const session = await desktop.createSession({ workspace, model, prompt, title });
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
  const [draft, setDraft] = useState("");
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
        body="Refresh the session inventory or return to the inbox to pick a different session."
        actionLabel="Back to inbox"
        onAction={() => onNavigate("/(app)/inbox/index")}
      />
    );
  }

  const description = describeSession(session);

  const handleSend = async () => {
    if (!draft.trim()) {
      return;
    }
    await desktop.sendMessage(session.id, draft);
    setDraft("");
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
          <label className="field-block">
            <span>Prompt the agent</span>
            <textarea
              rows={8}
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              placeholder="Send a real message to the session"
            />
          </label>
          <div className="button-row">
            <button
              type="button"
              className="primary-button"
              onClick={() => void handleSend()}
              disabled={messageState?.sending}
            >
              {messageState?.sending ? "Sending..." : "Send live message"}
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
              <li>Rich markdown and tool rendering still remain part of the session-parity follow-up.</li>
            </ul>
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
    try {
      await desktop.updateServerUrl(serverUrlDraft);
      setFeedback("Desktop server endpoint updated.");
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
    desktop.profile?.id ?? "Desktop account",
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
            <p className="hero-copy">
              The settings hub now tracks the current app more closely: connected
              accounts, feature routes, desktop configuration, and about links all stay
              grouped under one route.
            </p>
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
                    : "Open the desktop connect flow for command handoff and setup guidance."}
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
                <p>Reuse the locked localhost loopback flow to link another desktop.</p>
              </div>
              <span className="settings-row-badge">Auth</span>
            </button>
          </div>
        </article>

        <article className="panel-card settings-group-card">
          <div className="card-header">
            <h3>Desktop Configuration</h3>
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
                <p>Desktop rewrite preview build</p>
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
}: {
  desktop: DesktopState;
  onNavigate: (path: string) => void;
}) {
  const [feedback, setFeedback] = useState<string | null>(null);

  if (!desktop.credentials) {
    return <SignedOutState onNavigate={onNavigate} />;
  }

  const profileName = formatProfileName(
    desktop.profile?.firstName,
    desktop.profile?.lastName,
    desktop.profile?.username,
    desktop.profile?.id ?? "Desktop account",
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
          <p className="hero-copy">
            Identity, linked services, restore material, and logout controls now stay reachable on
            a dedicated desktop settings route backed by the current desktop account state.
          </p>
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
              <dt>Desktop status</dt>
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
                  <p>Open the vendor route to connect a supported desktop integration.</p>
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

function LanguageSettingsSurface({
  preferences,
}: {
  preferences: DesktopPreferencesState;
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
        <p className="panel-copy">
          Desktop language support now persists the preferred app language locally and applies it to
          the desktop document state, even though full translated copy switching remains a later step.
        </p>
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
                    : "Preferred desktop language"}
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
          <li>Current browser language: {runtimeLanguage}</li>
          <li>Preferred languages: {preferredLanguages.join(", ")}</li>
          <li>Current stored desktop preference: {languageSettings.appLanguage}</li>
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

function VoiceSettingsSurface({
  preferences,
  onNavigate,
}: {
  preferences: DesktopPreferencesState;
  onNavigate: (path: string) => void;
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
    <div className="surface-grid two-up">
      <article className="panel-card">
        <div className="card-header">
          <h3>Voice</h3>
          <span className="pill pill-accent">Persisted</span>
        </div>
        <p className="panel-copy">
          Voice preferences now persist in the desktop shell so language and bring-your-own-agent
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
          <span className="pill pill-outline">Desktop-backed</span>
        </div>
        <ul className="bullet-list dense-list">
          <li>Preferred language and custom agent settings are now persisted locally.</li>
          <li>Microphone capture and realtime device behavior remain a later promotion decision.</li>
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
}: {
  desktop: DesktopState;
  searchParams: URLSearchParams;
  onNavigate: (path: string) => void;
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
      void showDesktopNotification(
        "Terminal connected",
        "The desktop shell approved the terminal connection request.",
      ).catch(() => undefined);
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

function TextSelectionSurface() {
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
          <button
            type="button"
            className="secondary-button"
            onClick={() => void handleSave()}
            disabled={saving}
          >
            {saving ? "Saving..." : "Save selection to file"}
          </button>
        </div>
        {feedback ? <div className="compact-feedback">{feedback}</div> : null}
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
}: {
  desktop: DesktopState;
  createArtifact: (input: {
    title: string | null;
    body: string | null;
    sessions?: string[];
    draft?: boolean;
  }) => Promise<DesktopArtifact>;
  onNavigate: (path: string) => void;
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
      void showDesktopNotification(
        "Artifact created",
        `${created.title || "Untitled artifact"} is now available in the desktop library.`,
      ).catch(() => undefined);
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
}: {
  artifactId: string;
  desktop: DesktopState;
  artifacts: DesktopArtifact[];
  deleteArtifact: (artifactId: string) => Promise<void>;
  loadArtifact: (artifactId: string) => Promise<DesktopArtifact | null>;
  onNavigate: (path: string) => void;
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
            disabled={savingFile || artifactBodyUnavailable}
          >
            {artifactBodyLoading
              ? "Loading body..."
              : artifactBodyEncrypted
                ? "Encrypted body"
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
                  void showDesktopNotification(
                    "Artifact deleted",
                    `${artifact.title || "Untitled artifact"} was removed from the desktop library.`,
                  ).catch(() => undefined);
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
      void showDesktopNotification(
        "Artifact updated",
        `${updated.title || "Untitled artifact"} was saved from the desktop editor.`,
      ).catch(() => undefined);
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
  return (
    <EmptyState
      title="Sign in required"
      body="Create or restore an account first, then return to the desktop shell to load real sessions and messages."
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
