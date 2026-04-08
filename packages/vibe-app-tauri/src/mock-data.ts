export type SessionSummary = {
  id: string;
  title: string;
  agent: string;
  workspace: string;
  branch: string;
  status: "Active" | "Ready" | "Review";
  unread: number;
  updatedAt: string;
  lastMessage: string;
  tags: string[];
};

export type SessionMessage = {
  id: string;
  role: "system" | "user" | "assistant" | "tool";
  title: string;
  body: string;
  timestamp: string;
  accent: "neutral" | "teal" | "amber" | "slate";
};

export type RetainedArtifact = {
  id: string;
  title: string;
  summary: string;
  body: string;
  status: "Draft" | "Review" | "Published";
  linkedSessionIds: string[];
  createdAt: string;
  updatedAt: string;
};

export type RetainedUserProfile = {
  id: string;
  displayName: string;
  username: string;
  bio: string;
  status: "Friend" | "Requested" | "Review";
  githubUrl: string;
};

export type RetainedMachine = {
  id: string;
  label: string;
  host: string;
  platform: string;
  status: "Online" | "Offline" | "Review";
  homeDir: string;
  note: string;
};

export type RetainedSessionFile = {
  id: string;
  sessionId: string;
  path: string;
  status: "modified" | "added" | "review";
  language: string;
  summary: string;
  content: string;
  diff?: string;
};

export const sessionSummaries: SessionSummary[] = [
  {
    id: "demo-ship-review",
    title: "Ship Review Parity",
    agent: "gpt-5.4",
    workspace: "/root/vibe-remote",
    branch: "wave8-shell",
    status: "Active",
    unread: 2,
    updatedAt: "2 min ago",
    lastMessage: "Header chrome now matches the desktop route hierarchy; next up is focus trapping.",
    tags: ["wave8", "shell", "desktop"],
  },
  {
    id: "demo-auth-hardening",
    title: "Auth Loopback Hardening",
    agent: "gpt-5.3-codex",
    workspace: "/root/vibe-remote",
    branch: "wave8-auth",
    status: "Ready",
    unread: 0,
    updatedAt: "19 min ago",
    lastMessage: "State tokens are scoped per attempt and stale callbacks are rejected after timeout.",
    tags: ["auth", "loopback", "security"],
  },
  {
    id: "demo-session-rendering",
    title: "Timeline Rendering Audit",
    agent: "gpt-5.2",
    workspace: "/root/vibe-remote",
    branch: "wave8-session-ui",
    status: "Review",
    unread: 1,
    updatedAt: "47 min ago",
    lastMessage: "Markdown, diff, and tool cards still need side-by-side review before promotion.",
    tags: ["timeline", "rendering", "parity"],
  },
  {
    id: "demo-release-gate",
    title: "Release Gate Checklist",
    agent: "gpt-5.4",
    workspace: "/root/vibe-remote",
    branch: "wave8-release",
    status: "Ready",
    unread: 0,
    updatedAt: "1 hr ago",
    lastMessage: "Linux smoke bundle passes; macOS and Windows still need startup validation artifacts.",
    tags: ["release", "validation", "promotion"],
  },
];

export const sessionTimelineById: Record<string, SessionMessage[]> = {
  "demo-ship-review": [
    {
      id: "m1",
      role: "system",
      title: "Execution brief",
      body: "Wave 8 stays parity-first. Rebuild shell chrome and the locked P0 routes before deeper feature migration.",
      timestamp: "09:14",
      accent: "slate",
    },
    {
      id: "m2",
      role: "user",
      title: "User",
      body: "Please list every Wave 8 function point, wire the desktop shell, and leave a route tree we can review end-to-end.",
      timestamp: "09:16",
      accent: "amber",
    },
    {
      id: "m3",
      role: "assistant",
      title: "Desktop shell",
      body: "Primary navigation, command palette, route inspector, and session review surfaces are now available for the P0 paths.",
      timestamp: "09:21",
      accent: "teal",
    },
    {
      id: "m4",
      role: "tool",
      title: "Validation",
      body: "Typecheck, tests, and production build all pass in the new `vibe-app-tauri` workspace.",
      timestamp: "09:26",
      accent: "neutral",
    },
  ],
  "demo-auth-hardening": [
    {
      id: "a1",
      role: "system",
      title: "Locked auth callback",
      body: "Bind only to 127.0.0.1, use an ephemeral port, validate state, and allow only one live auth attempt.",
      timestamp: "08:52",
      accent: "slate",
    },
    {
      id: "a2",
      role: "assistant",
      title: "Adapter seam",
      body: "Desktop-safe browser launch, secure storage, and callback listeners stay behind package-local seams until parity is proven.",
      timestamp: "09:03",
      accent: "teal",
    },
  ],
  "demo-session-rendering": [
    {
      id: "s1",
      role: "assistant",
      title: "Rendering matrix",
      body: "Markdown, diff, tool, and file rendering remain promotion blockers; they do not block the shell slice itself.",
      timestamp: "08:37",
      accent: "teal",
    },
  ],
  "demo-release-gate": [
    {
      id: "r1",
      role: "assistant",
      title: "Promotion gate",
      body: "`packages/vibe-app` stays the default desktop path until all required P0 and P1 items are done or explicitly deferred.",
      timestamp: "08:11",
      accent: "teal",
    },
  ],
};

export const restoreChecklist = [
  "Launch desktop auth in the external browser instead of claiming the default `vibe:///` route.",
  "Listen on a one-shot localhost callback bound to `127.0.0.1`.",
  "Persist credentials in desktop-safe storage only after state validation succeeds.",
  "Resume the shell on the same active route after restore completes.",
];

export const restoreArtifacts = [
  {
    label: "Callback host",
    value: "127.0.0.1 only",
  },
  {
    label: "Port policy",
    value: "Ephemeral per attempt",
  },
  {
    label: "Replay handling",
    value: "Reject stale or wrong-instance callbacks",
  },
];

export const settingsCards = [
  {
    title: "Account",
    description: "Identity, subscription, restore history, and logout controls.",
    route: "/(app)/settings/account",
  },
  {
    title: "Appearance",
    description: "Theme, layout density, and desktop chrome tolerances.",
    route: "/(app)/settings/appearance",
  },
  {
    title: "Features",
    description: "Feature gates and staged rollout preferences for the desktop rewrite.",
    route: "/(app)/settings/features",
  },
  {
    title: "Usage",
    description: "Rate, plan, and quota views used during dogfooding and promotion review.",
    route: "/(app)/settings/usage",
  },
];

export const newSessionPresets = [
  {
    name: "Parity audit",
    model: "gpt-5.4",
    prompt: "Compare the current desktop route tree against the Wave 8 shell and call out layout drift.",
  },
  {
    name: "Auth callback drill",
    model: "gpt-5.3-codex",
    prompt: "Validate localhost loopback state, port lifecycle, and stale callback rejection rules.",
  },
  {
    name: "Release review",
    model: "gpt-5.2",
    prompt: "Check Linux, macOS, and Windows bundle readiness before the promotion gate opens.",
  },
];

export const routeReviewNotes = [
  "P0 routes must be usable enough for side-by-side shell review.",
  "P1 routes stay explicitly visible in the inventory so nothing disappears silently.",
  "P2 routes remain classified and reviewable instead of becoming hidden backlog items.",
];

export const plannedSurfaceExamples = [
  "Artifacts management",
  "Profile and detailed settings routes",
  "Terminal and machine utility flows",
  "Text-selection helpers and diagnostics",
];

export const retainedArtifacts: RetainedArtifact[] = [
  {
    id: "artifact-wave8-plan",
    title: "Wave 8 Retained Surface Plan",
    summary: "Execution notes for the first retained secondary-surface routes.",
    body: [
      "# Wave 8 retained routes",
      "",
      "- keep artifacts visible before promotion",
      "- wire settings detail routes first",
      "- review machine and profile detail routes side by side",
    ].join("\n"),
    status: "Review",
    linkedSessionIds: ["demo-ship-review", "demo-release-gate"],
    createdAt: "2026-04-05",
    updatedAt: "2026-04-07",
  },
  {
    id: "artifact-auth-audit",
    title: "Auth Hardening Audit",
    summary: "Records loopback auth guardrails and unresolved promotion blockers.",
    body: [
      "# Auth hardening",
      "",
      "Loopback auth now validates per-attempt state, timeout, and attempt replacement.",
      "",
      "Promotion blockers still include broader cross-platform startup validation.",
    ].join("\n"),
    status: "Published",
    linkedSessionIds: ["demo-auth-hardening"],
    createdAt: "2026-04-04",
    updatedAt: "2026-04-07",
  },
  {
    id: "artifact-rendering-audit",
    title: "Rendering Audit Notes",
    summary: "Tracks markdown, diff, tool, and file rendering review.",
    body: [
      "# Rendering",
      "",
      "Rich rendering now loads through smaller lazy chunks.",
      "",
      "Side-by-side parity review is still required before promotion sign-off.",
    ].join("\n"),
    status: "Draft",
    linkedSessionIds: ["demo-session-rendering"],
    createdAt: "2026-04-06",
    updatedAt: "2026-04-07",
  },
] as const;

export const retainedUserProfiles: RetainedUserProfile[] = [
  {
    id: "demo-user",
    displayName: "Avery Stone",
    username: "avery",
    bio: "Desktop parity reviewer focusing on route drift, release gates, and auth hardening.",
    status: "Review",
    githubUrl: "https://github.com/fage-org",
  },
  {
    id: "release-owner",
    displayName: "Jordan Fields",
    username: "jordan",
    bio: "Owns promotion readiness and cross-platform bundle verification.",
    status: "Friend",
    githubUrl: "https://github.com/fage-org",
  },
] as const;

export const retainedMachines: RetainedMachine[] = [
  {
    id: "demo-workstation",
    label: "Desktop Review Workstation",
    host: "dev-server-01",
    platform: "Linux",
    status: "Online",
    homeDir: "/root",
    note: "Primary Wave 8 review machine with active session inventory.",
  },
  {
    id: "release-runner",
    label: "Release Validation Runner",
    host: "release-runner-02",
    platform: "macOS",
    status: "Review",
    homeDir: "/Users/release",
    note: "Reserved for bundle startup checks and final promotion validation.",
  },
] as const;

export const retainedSessionFiles: RetainedSessionFile[] = [
  {
    id: "ship-review-bootstrap",
    sessionId: "demo-ship-review",
    path: "/root/vibe-remote/packages/vibe-app-tauri/src/bootstrap.ts",
    status: "modified",
    language: "typescript",
    summary: "Tracks Wave 8 delivery modules, feature counts, and quality gates.",
    content: [
      "export const wave8Modules = [",
      '  "bootstrap-and-package",',
      '  "desktop-shell-and-routing",',
      '  "core-logic-extraction",',
      '  "desktop-platform-adapters",',
      "];",
    ].join("\n"),
    diff: [
      "diff --git a/src/bootstrap.ts b/src/bootstrap.ts",
      "--- a/src/bootstrap.ts",
      "+++ b/src/bootstrap.ts",
      "@@ -1,3 +1,5 @@",
      " export const wave8Modules = [",
      '+  "desktop-platform-adapters",',
      '+  "auth-and-session-state",',
      " ];",
    ].join("\n"),
  },
  {
    id: "ship-review-router",
    sessionId: "demo-ship-review",
    path: "/root/vibe-remote/packages/vibe-app-tauri/src/router.ts",
    status: "modified",
    language: "typescript",
    summary: "Locks the retained P0/P1 route inventory for desktop review.",
    content: [
      "export const desktopRoutes = [",
      "  { key: 'home', pattern: '/(app)/index' },",
      "  { key: 'session-detail', pattern: '/(app)/session/[id]' },",
      "];",
    ].join("\n"),
  },
  {
    id: "auth-hardening-lib",
    sessionId: "demo-auth-hardening",
    path: "/root/vibe-remote/packages/vibe-app-tauri/src-tauri/src/lib.rs",
    status: "review",
    language: "rust",
    summary: "Implements loopback auth start, cancellation, timeout, and state validation.",
    content: [
      "fn start_account_link_callback(...) -> Result<BeginAccountLinkCallbackResponse, String> {",
      "    // binds 127.0.0.1:0 and stores one active attempt",
      "}",
    ].join("\n"),
  },
  {
    id: "rendering-rich",
    sessionId: "demo-session-rendering",
    path: "/root/vibe-remote/packages/vibe-app-tauri/src/rich-message-renderers.tsx",
    status: "modified",
    language: "tsx",
    summary: "Retained rich rendering route with lazy code blocks and diff fallback.",
    content: [
      "export function RichTimelineMessageBody({ message }: { message: UiMessage }) {",
      "  if (looksLikeDiff(message.text)) {",
      "    return <DiffSurface diffText={message.text} />;",
      "  }",
      "}",
    ].join("\n"),
  },
] as const;

export const desktopChangelogEntries = [
  {
    version: "0.1.0-preview",
    date: "2026-04-07",
    title: "Desktop protocol and auth hardening",
    bullets: [
      "Desktop message and update parsing now validates payloads through shared compatibility schemas.",
      "Loopback auth callback coverage now checks wrong-state rejection, timeout, and attempt replacement.",
      "Tauri security defaults now prefer explicit CSP plus loopback-only HTTP exceptions.",
    ],
  },
  {
    version: "0.1.0-alpha.3",
    date: "2026-04-06",
    title: "Desktop shell and settings parity push",
    bullets: [
      "Home and settings hub now stay closer to the current desktop-visible app structure.",
      "Route palette, keyboard navigation, and inspector surfaces remain available for side-by-side review.",
      "CI now validates the Tauri package through typecheck, tests, build, and smoke bundle steps.",
    ],
  },
] as const;

export const desktopFeatureFlags = [
  {
    name: "Loopback Auth Guardrails",
    status: "Enabled",
    detail: "Desktop auth is restricted to localhost callback ownership during coexistence.",
  },
  {
    name: "Shared Payload Validation",
    status: "Enabled",
    detail: "Realtime updates and message pages are validated against the compatibility schemas.",
  },
  {
    name: "Promotion Routes",
    status: "In Review",
    detail: "P1 secondary routes are being migrated slice by slice instead of landing as placeholders.",
  },
] as const;

export const desktopVoiceLanguages = [
  "English",
  "Japanese",
  "Simplified Chinese",
  "Spanish",
] as const;

export const terminalCommandCards = [
  {
    title: "Resume Recent Session",
    command: "vibe session resume --recent",
    detail: "Reopen the last active desktop session from the local shell.",
  },
  {
    title: "Connect Helper",
    command: "vibe terminal connect --device current",
    detail: "Launch the retained desktop helper connection flow.",
  },
] as const;
