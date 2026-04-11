export type Wave8ModuleName = (typeof wave8Modules)[number];
export type PromotionClass = "P0" | "P1" | "P2";

export type Wave8FeatureArea = {
  key: string;
  title: string;
  batch: "B17" | "B18";
  ownerModule: Wave8ModuleName;
  description: string;
  features: string[];
  definitionOfDone: string[];
};

export type PriorityBucket = {
  priority: PromotionClass;
  title: string;
  features: string[];
};

export const wave8Modules = [
  "bootstrap-and-package",
  "desktop-shell-and-routing",
  "core-logic-extraction",
  "desktop-platform-adapters",
  "auth-and-session-state",
  "session-ui-parity",
  "secondary-surfaces",
  "release-and-promotion",
] as const;

export const firstUsableSlice = [
  "Create an isolated Tauri 2 desktop package.",
  "Stand up desktop routing, shell chrome, and focus-safe overlays.",
  "Wire desktop auth-safe storage and localhost callback handling.",
  "Reach a real session flow against the Vibe backend.",
];

export const promotionSlice = [
  "Port P1 secondary routes and utility surfaces.",
  "Validate Linux, macOS, and Windows startup behavior.",
  "Record performance and memory review on realistic session loads.",
  "Keep `packages/vibe-app` as the default path until explicit promotion.",
];

export const shellInvariants = [
  "Recognizable header, sidebar, and main-panel structure.",
  "Desktop-accessible entry points for account, settings, and session flows.",
  "Modal and overlay semantics that preserve focus and action ownership.",
  "Keyboard navigation and focus order acceptable for desktop use.",
  "Session resume and active-state affordances in familiar locations.",
];

export const lockedAuthCallbackRequirements = [
  "Bind only to 127.0.0.1.",
  "Use an ephemeral per-attempt port.",
  "Validate a per-attempt state value.",
  "Keep listeners one-shot and short-lived.",
  "Allow only one active auth attempt per process instance.",
  "Reject stale, replayed, or wrong-instance callbacks.",
];

export const wave8FeatureAreas: Wave8FeatureArea[] = [
  {
    key: "package-and-bootstrap",
    title: "Package And Bootstrap",
    batch: "B17",
    ownerModule: "bootstrap-and-package",
    description:
      "Create the new desktop-only workspace and keep it isolated from the shipping app lane.",
    features: [
      "Create `packages/vibe-app-tauri` with package metadata and workspace integration.",
      "Add a desktop web frontend bootstrap independent of `packages/vibe-app`.",
      "Add a Tauri 2 shell with isolated bundle identifiers and package-local state directories.",
      "Define package-local `dev`, `build`, `typecheck`, `test`, and `release` scripts.",
      "Keep build outputs, artifacts, updater channels, and local state isolated from `packages/vibe-app`.",
      "Add package-local CI and release hooks without mutating the shipping app lane.",
    ],
    definitionOfDone: [
      "Package exists in-repo.",
      "Local desktop dev boot works.",
      "Tauri bundle smoke build works.",
      "Scripts do not depend on mutating `packages/vibe-app`.",
    ],
  },
  {
    key: "desktop-shell-and-routing",
    title: "Desktop Shell And Routing",
    batch: "B17",
    ownerModule: "desktop-shell-and-routing",
    description:
      "Recreate the desktop route tree, shell chrome, and navigation semantics so parity review can start.",
    features: [
      "Recreate the desktop shell chrome: header, sidebar, main panel, and modal or overlay behavior.",
      "Rebuild the route tree for desktop-only navigation.",
      "Preserve desktop entry routes and expected route semantics.",
      "Preserve keyboard and focus behavior acceptable for desktop use.",
      "Keep layout hierarchy and information density close to current desktop behavior.",
    ],
    definitionOfDone: [
      "Shell routes are navigable.",
      "No placeholder dead-end exists in the P0 shell flow.",
      "Side-by-side route and shell review is possible.",
    ],
  },
  {
    key: "core-logic-extraction",
    title: "Core Logic Extraction",
    batch: "B17",
    ownerModule: "core-logic-extraction",
    description:
      "Extract the minimum auth, session, account, and sync logic the new desktop app must reuse.",
    features: [
      "Extract minimum auth, session, account, and sync logic needed for the new desktop app.",
      "Copy or adapt pure TypeScript logic into `packages/vibe-app-tauri`.",
      "Isolate Expo and React Native assumptions behind adapter seams.",
      "Keep extraction package-local during early phases.",
      "Copy text, utility, encryption, constants, assets, and type inputs only where needed.",
    ],
    definitionOfDone: [
      "Extracted logic compiles without React Native UI imports.",
      "Auth and session bootstrap code in `vibe-app-tauri` uses package-local extracted seams.",
    ],
  },
  {
    key: "desktop-platform-adapters",
    title: "Desktop Platform Adapters",
    batch: "B17",
    ownerModule: "desktop-platform-adapters",
    description:
      "Replace mobile-only platform dependencies with desktop-safe integrations required for parity.",
    features: [
      "Secure credential storage.",
      "Session-safe clipboard integration.",
      "External browser launch for auth and connect flows.",
      "Localhost loopback callback listener for auth completion.",
      "File open and save dialogs where desktop parity requires them.",
      "Notifications where desktop parity requires them.",
    ],
    definitionOfDone: [
      "Auth-critical flows no longer depend on Expo or mobile-only platform APIs.",
      "Clipboard and later desktop adapters behave correctly for supported flows.",
    ],
  },
  {
    key: "auth-and-session-state",
    title: "Auth And Session State",
    batch: "B17",
    ownerModule: "auth-and-session-state",
    description:
      "Make the desktop app authenticate, restore accounts, and bootstrap state against the real backend.",
    features: [
      "Desktop login flow.",
      "Account restore flow.",
      "Logout and re-auth flow.",
      "Credential persistence via desktop-safe storage.",
      "Backend client wiring for auth, session, and account bootstrap.",
      "Callback-driven auth completion through the locked localhost loopback strategy.",
    ],
    definitionOfDone: [
      "Desktop app can authenticate.",
      "Account restore works.",
      "Session and account bootstrap works against the real Vibe backend.",
    ],
  },
  {
    key: "session-ui-parity",
    title: "Session UI Parity",
    batch: "B17",
    ownerModule: "session-ui-parity",
    description:
      "Port the primary session list, timeline, composer, and resume affordances into the new desktop shell.",
    features: [
      "Session list shell.",
      "Session detail shell.",
      "Message timeline rendering.",
      "Composer and input interaction model.",
      "Active-session and resume affordances.",
      "Markdown, diff, tool, and file rendering for promotion scope.",
    ],
    definitionOfDone: [
      "Users can open a session, read messages, and send input end-to-end.",
      "Session UI is good enough for internal dogfooding before broader surface migration.",
    ],
  },
  {
    key: "secondary-surfaces",
    title: "Secondary Product Surfaces",
    batch: "B18",
    ownerModule: "secondary-surfaces",
    description:
      "Close the promotion-scope parity gaps for artifacts, settings detail, utility routes, and desktop-only helpers.",
    features: [
      "Artifacts routes and detail, edit, and create flows.",
      "Account, profile, and settings detail screens.",
      "Connect and vendor flows retained for desktop users.",
      "Changelog and diagnostics routes.",
      "Terminal utility routes.",
      "Self-hosted server config, machine detail, and text-selection routes.",
      "Optional friends, social, and developer-only routes after explicit review.",
    ],
    definitionOfDone: [
      "All required P1 routes are present.",
      "Any omitted P2 route is explicitly marked deferred.",
    ],
  },
  {
    key: "release-and-promotion",
    title: "Release, Validation, And Promotion",
    batch: "B18",
    ownerModule: "release-and-promotion",
    description:
      "Validate cross-platform packaging, performance, and the explicit promotion gate without flipping the default path early.",
    features: [
      "Package-local release scripts.",
      "Desktop CI and release automation.",
      "Artifact naming and channel isolation from the shipping app.",
      "Parity checklist maintenance.",
      "Linux, macOS, and Windows startup validation before promotion.",
      "Realistic session-load performance and memory review.",
      "Explicit fallback and deprecation strategy for the current desktop path.",
    ],
    definitionOfDone: [
      "Release artifacts are produced reliably.",
      "Coexistence rules remain documented.",
      "Promotion gate is explicit and archived in the planning record.",
    ],
  },
];

export const wave8PriorityBuckets: PriorityBucket[] = [
  {
    priority: "P0",
    title: "First Usable Desktop Slice",
    features: [
      "Separate `packages/vibe-app-tauri` package and Tauri shell.",
      "Desktop shell, route tree, and core navigation.",
      "Desktop login, account restore, logout, and re-auth.",
      "Secure storage plus callback-driven auth completion.",
      "Core session list and session detail flow.",
      "Message rendering, composer interaction model, and active session resume affordances.",
      "Clipboard behavior needed by the primary session and text-selection flows.",
      "One real app-tauri to backend session chain.",
    ],
  },
  {
    priority: "P1",
    title: "Required Before Promotion",
    features: [
      "Markdown, diff, tool, and file rendering parity.",
      "Artifacts flows.",
      "Account, profile, and settings detail flows.",
      "Connect and vendor flows used by desktop users.",
      "Changelog and diagnostics routes.",
      "File picker or save dialogs and notifications where parity requires them.",
      "Linux, macOS, and Windows package and startup validation.",
      "Performance and memory review plus promotion and deprecation plan.",
    ],
  },
  {
    priority: "P2",
    title: "Late Or Optional Scope",
    features: [
      "Friends and social surfaces after desktop-value review.",
      "Developer-only routes after route-by-route review.",
      "Telemetry or tracking after release and privacy review.",
      "Mobile-only camera, sensor, and location capabilities only if a later plan reactivates them.",
    ],
  },
];

export function formatModuleCount(modules: readonly string[]): string {
  return `${modules.length} Wave 8 modules tracked`;
}

export function formatFeatureCount(areas: readonly Wave8FeatureArea[]): string {
  const total = areas.reduce((sum, area) => sum + area.features.length, 0);
  return `${total} scoped feature points`; 
}
