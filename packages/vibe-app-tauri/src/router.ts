import { useCallback, useEffect, useMemo, useState } from "react";
import type { PromotionClass, Wave8ModuleName } from "./bootstrap";

export type RouteStatus = "wired" | "retained" | "planned";
export type RouteSection =
  | "Landing"
  | "Auth"
  | "Session"
  | "Settings"
  | "Artifacts"
  | "Utilities"
  | "Social"
  | "Developer";

export type RouteDefinition = {
  key: string;
  label: string;
  title: string;
  pattern: string;
  examplePath: string;
  summary: string;
  promotionClass: PromotionClass;
  ownerModule: Wave8ModuleName;
  section: RouteSection;
  status: RouteStatus;
  notes?: string;
};

export type ResolvedRoute = {
  definition: RouteDefinition;
  params: Record<string, string>;
  canonicalPath: string;
  searchParams: URLSearchParams;
};

export const DEFAULT_PATH = "/(app)/index";

export const desktopRoutes: RouteDefinition[] = [
  {
    key: "home",
    label: "Home",
    title: "Desktop Entry",
    pattern: "/(app)/index",
    examplePath: "/(app)/index",
    summary: "Default desktop landing route with sessions, inbox, and settings overview.",
    promotionClass: "P0",
    ownerModule: "desktop-shell-and-routing",
    section: "Landing",
    status: "wired",
  },
  {
    key: "restore-index",
    label: "Restore",
    title: "Account Restore",
    pattern: "/(app)/restore/index",
    examplePath: "/(app)/restore/index",
    summary: "Desktop restore entry flow for auth, device linking, and account recovery.",
    promotionClass: "P0",
    ownerModule: "auth-and-session-state",
    section: "Auth",
    status: "wired",
  },
  {
    key: "restore-manual",
    label: "Manual Restore",
    title: "Manual Restore",
    pattern: "/(app)/restore/manual",
    examplePath: "/(app)/restore/manual",
    summary: "Manual secret-key and recovery-code flow for desktop restore.",
    promotionClass: "P0",
    ownerModule: "auth-and-session-state",
    section: "Auth",
    status: "wired",
  },
  {
    key: "inbox",
    label: "Inbox",
    title: "Session Inbox",
    pattern: "/(app)/inbox/index",
    examplePath: "/(app)/inbox/index",
    summary: "Primary session list route for the first usable desktop slice.",
    promotionClass: "P0",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "new-session",
    label: "New Session",
    title: "Create Session",
    pattern: "/(app)/new/index",
    examplePath: "/(app)/new/index",
    summary: "Desktop entry point for starting a new session against the Vibe backend.",
    promotionClass: "P0",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "session-recent",
    label: "Recent",
    title: "Recent Sessions",
    pattern: "/(app)/session/recent",
    examplePath: "/(app)/session/recent",
    summary: "Resume affordance for recently active sessions and desktop continuity.",
    promotionClass: "P0",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "session-detail",
    label: "Active Session",
    title: "Session Detail",
    pattern: "/(app)/session/[id]",
    examplePath: "/(app)/session/demo-ship-review",
    summary: "Timeline shell, composer, and primary active-session state loading route.",
    promotionClass: "P0",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "settings-index",
    label: "Settings",
    title: "Settings Hub",
    pattern: "/(app)/settings/index",
    examplePath: "/(app)/settings/index",
    summary: "Core desktop account, connections, feature controls, and desktop configuration route.",
    promotionClass: "P0",
    ownerModule: "desktop-shell-and-routing",
    section: "Settings",
    status: "wired",
  },
  {
    key: "session-info",
    label: "Session Info",
    title: "Session Info",
    pattern: "/(app)/session/[id]/info",
    examplePath: "/(app)/session/demo-ship-review/info",
    summary: "Promotion-scope session metadata and diagnostics surface.",
    promotionClass: "P1",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "session-files",
    label: "Session Files",
    title: "Session Files",
    pattern: "/(app)/session/[id]/files",
    examplePath: "/(app)/session/demo-ship-review/files",
    summary: "Promotion-scope file inspection list for a session.",
    promotionClass: "P1",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "session-file",
    label: "File Detail",
    title: "Session File Viewer",
    pattern: "/(app)/session/[id]/file",
    examplePath: "/(app)/session/demo-ship-review/file?path=src%2FApp.tsx",
    summary: "Single-file viewer tied to session rendering and file inspection.",
    promotionClass: "P1",
    ownerModule: "session-ui-parity",
    section: "Session",
    status: "wired",
  },
  {
    key: "artifacts-index",
    label: "Artifacts",
    title: "Artifacts Index",
    pattern: "/(app)/artifacts/index",
    examplePath: "/(app)/artifacts/index",
    summary: "Artifacts browse route required before promotion.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Artifacts",
    status: "wired",
  },
  {
    key: "artifacts-new",
    label: "New Artifact",
    title: "Create Artifact",
    pattern: "/(app)/artifacts/new",
    examplePath: "/(app)/artifacts/new",
    summary: "Create-artifact flow required before promotion.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Artifacts",
    status: "wired",
  },
  {
    key: "artifacts-detail",
    label: "Artifact Detail",
    title: "Artifact Detail",
    pattern: "/(app)/artifacts/[id]",
    examplePath: "/(app)/artifacts/demo-artifact",
    summary: "Artifact detail route for desktop browsing and review.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Artifacts",
    status: "wired",
  },
  {
    key: "artifacts-edit",
    label: "Edit Artifact",
    title: "Edit Artifact",
    pattern: "/(app)/artifacts/edit/[id]",
    examplePath: "/(app)/artifacts/edit/demo-artifact",
    summary: "Artifact edit surface required before desktop promotion.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Artifacts",
    status: "wired",
  },
  {
    key: "settings-account",
    label: "Account",
    title: "Account Settings",
    pattern: "/(app)/settings/account",
    examplePath: "/(app)/settings/account",
    summary: "Detailed account surface for identity, plans, and profile actions.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-appearance",
    label: "Appearance",
    title: "Appearance Settings",
    pattern: "/(app)/settings/appearance",
    examplePath: "/(app)/settings/appearance",
    summary: "Detailed appearance settings route with persisted desktop shell preferences.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-features",
    label: "Features",
    title: "Feature Flags",
    pattern: "/(app)/settings/features",
    examplePath: "/(app)/settings/features",
    summary: "Detailed feature preferences and experiments surface.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-language",
    label: "Language",
    title: "Language Settings",
    pattern: "/(app)/settings/language",
    examplePath: "/(app)/settings/language",
    summary: "Language and copy preferences backed by runtime inventory before promotion.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-usage",
    label: "Usage",
    title: "Usage Settings",
    pattern: "/(app)/settings/usage",
    examplePath: "/(app)/settings/usage",
    summary: "Usage detail route for limits, plans, and tracking visibility.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-voice",
    label: "Voice",
    title: "Voice Settings",
    pattern: "/(app)/settings/voice",
    examplePath: "/(app)/settings/voice",
    summary: "Voice preferences route with persisted desktop language and BYO-agent state.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-voice-language",
    label: "Voice Language",
    title: "Voice Language",
    pattern: "/(app)/settings/voice/language",
    examplePath: "/(app)/settings/voice/language",
    summary: "Voice language route with persisted desktop locale selection.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "settings-connect-claude",
    label: "Connect Claude",
    title: "Vendor Connect",
    pattern: "/(app)/settings/connect/claude",
    examplePath: "/(app)/settings/connect/claude",
    summary: "Representative connect or vendor route with explicit desktop command handoff.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Settings",
    status: "wired",
  },
  {
    key: "user-detail",
    label: "User Profile",
    title: "User Detail",
    pattern: "/(app)/user/[id]",
    examplePath: "/(app)/user/demo-user",
    summary: "Desktop user or profile detail route in promotion scope.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "changelog",
    label: "Changelog",
    title: "Changelog",
    pattern: "/(app)/changelog",
    examplePath: "/(app)/changelog",
    summary: "Whats-new surface required before promotion if still desktop-visible.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "terminal-index",
    label: "Terminal",
    title: "Terminal Utility",
    pattern: "/(app)/terminal/index",
    examplePath: "/(app)/terminal/index",
    summary: "Terminal utility route backed by live desktop session metadata and command helpers.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "terminal-connect",
    label: "Connect Terminal",
    title: "Connect Terminal",
    pattern: "/(app)/terminal/connect",
    examplePath: "/(app)/terminal/connect",
    summary: "Terminal connect flow that can approve a live desktop terminal request.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "server",
    label: "Server",
    title: "Server Config",
    pattern: "/(app)/server",
    examplePath: "/(app)/server",
    summary: "Self-hosted or server configuration route backed by desktop endpoint state.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "machine-detail",
    label: "Machine Detail",
    title: "Machine Detail",
    pattern: "/(app)/machine/[id]",
    examplePath: "/(app)/machine/demo-workstation",
    summary: "Machine detail and remote-control utility route in promotion scope.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "text-selection",
    label: "Text Selection",
    title: "Text Selection Utility",
    pattern: "/(app)/text-selection",
    examplePath: "/(app)/text-selection",
    summary: "Utility route for copy, export, and text-selection workflows.",
    promotionClass: "P1",
    ownerModule: "secondary-surfaces",
    section: "Utilities",
    status: "wired",
  },
  {
    key: "friends-index",
    label: "Friends",
    title: "Friends",
    pattern: "/(app)/friends/index",
    examplePath: "/(app)/friends/index",
    summary: "Social surface kept as optional or late desktop scope until value is confirmed.",
    promotionClass: "P2",
    ownerModule: "secondary-surfaces",
    section: "Social",
    status: "planned",
  },
  {
    key: "friends-search",
    label: "Find Friends",
    title: "Friend Search",
    pattern: "/(app)/friends/search",
    examplePath: "/(app)/friends/search",
    summary: "Optional search surface tied to the social route set.",
    promotionClass: "P2",
    ownerModule: "secondary-surfaces",
    section: "Social",
    status: "planned",
  },
  {
    key: "dev-index",
    label: "Developer Tools",
    title: "Developer Tools",
    pattern: "/(app)/dev/index",
    examplePath: "/(app)/dev/index",
    summary: "Developer-only route family reviewed route by route before promotion.",
    promotionClass: "P2",
    ownerModule: "secondary-surfaces",
    section: "Developer",
    status: "planned",
  },
];

const routeNotFound: RouteDefinition = {
  key: "route-review",
  label: "Route Review",
  title: "Route Review Required",
  pattern: "*",
  examplePath: DEFAULT_PATH,
  summary: "The requested path is outside the locked Wave 8 route inventory and needs an explicit planning decision.",
  promotionClass: "P2",
  ownerModule: "desktop-shell-and-routing",
  section: "Landing",
  status: "planned",
};

function splitPathAndQuery(value: string): {
  pathname: string;
  query: string;
  searchParams: URLSearchParams;
} {
  const queryIndex = value.indexOf("?");
  if (queryIndex === -1) {
    return {
      pathname: value,
      query: "",
      searchParams: new URLSearchParams(),
    };
  }

  const pathname = value.slice(0, queryIndex);
  const query = value.slice(queryIndex + 1);

  return {
    pathname,
    query,
    searchParams: new URLSearchParams(query),
  };
}

function trimSlashes(value: string): string {
  return value.replace(/^\/+/, "").replace(/\/+$/, "");
}

export function normalizeAppPath(value?: string): string {
  if (!value) {
    return DEFAULT_PATH;
  }

  let normalized = value.trim();

  if (normalized.startsWith("#")) {
    normalized = normalized.slice(1);
  }

  if (!normalized.startsWith("/")) {
    normalized = `/${normalized}`;
  }

  const { pathname: rawPathname, query } = splitPathAndQuery(normalized);
  normalized = rawPathname.replace(/\/+/g, "/");

  if (normalized === "/" || normalized === "/(app)" || normalized === "/(app)/") {
    return query ? `${DEFAULT_PATH}?${query}` : DEFAULT_PATH;
  }

  if (normalized.length > 1 && normalized.endsWith("/")) {
    normalized = normalized.slice(0, -1);
  }

  return query ? `${normalized}?${query}` : normalized;
}

export function matchRoutePattern(
  pattern: string,
  candidatePath: string,
): Record<string, string> | null {
  const patternSegments = trimSlashes(splitPathAndQuery(normalizeAppPath(pattern)).pathname).split("/");
  const pathSegments = trimSlashes(splitPathAndQuery(normalizeAppPath(candidatePath)).pathname).split("/");

  if (patternSegments.length !== pathSegments.length) {
    return null;
  }

  const params: Record<string, string> = {};

  for (let index = 0; index < patternSegments.length; index += 1) {
    const patternSegment = patternSegments[index];
    const pathSegment = pathSegments[index];

    if (patternSegment.startsWith("[") && patternSegment.endsWith("]")) {
      params[patternSegment.slice(1, -1)] = decodeURIComponent(pathSegment);
      continue;
    }

    if (patternSegment !== pathSegment) {
      return null;
    }
  }

  return params;
}

export function resolveRoute(candidatePath?: string): ResolvedRoute {
  const canonicalPath = normalizeAppPath(candidatePath);
  const { pathname, searchParams } = splitPathAndQuery(canonicalPath);

  for (const definition of desktopRoutes) {
    const params = matchRoutePattern(definition.pattern, pathname);

    if (params) {
      return {
        definition,
        params,
        canonicalPath,
        searchParams,
      };
    }
  }

  return {
    definition: routeNotFound,
    params: {},
    canonicalPath,
    searchParams,
  };
}

export const routeInventoryByClass = {
  P0: desktopRoutes.filter((route) => route.promotionClass === "P0"),
  P1: desktopRoutes.filter((route) => route.promotionClass === "P1"),
  P2: desktopRoutes.filter((route) => route.promotionClass === "P2"),
} as const;

export const primaryNavigation = desktopRoutes.filter((route) =>
  [
    "home",
    "inbox",
    "new-session",
    "session-recent",
    "settings-index",
    "restore-index",
  ].includes(route.key),
);

export const promotionNavigation = desktopRoutes.filter((route) =>
  [
    "artifacts-index",
    "settings-account",
    "terminal-index",
    "machine-detail",
    "text-selection",
    "changelog",
  ].includes(route.key),
);

function readHashPath(): string {
  if (typeof window === "undefined") {
    return DEFAULT_PATH;
  }

  return normalizeAppPath(window.location.hash.slice(1));
}

export function hrefForPath(path: string): string {
  return `#${normalizeAppPath(path)}`;
}

export function navigateToPath(path: string): void {
  if (typeof window === "undefined") {
    return;
  }

  const normalized = normalizeAppPath(path);
  if (window.location.hash.slice(1) === normalized) {
    window.dispatchEvent(new HashChangeEvent("hashchange"));
    return;
  }

  window.location.hash = normalized;
}

export function useDesktopRouter() {
  const [path, setPath] = useState<string>(() => readHashPath());

  useEffect(() => {
    if (typeof window === "undefined") {
      return undefined;
    }

    if (!window.location.hash) {
      window.location.hash = DEFAULT_PATH;
    }

    const syncPath = () => {
      setPath(readHashPath());
    };

    syncPath();
    window.addEventListener("hashchange", syncPath);
    return () => {
      window.removeEventListener("hashchange", syncPath);
    };
  }, []);

  const navigate = useCallback((nextPath: string) => {
    navigateToPath(nextPath);
  }, []);

  const resolved = useMemo(() => resolveRoute(path), [path]);

  return {
    path,
    resolved,
    navigate,
  };
}
