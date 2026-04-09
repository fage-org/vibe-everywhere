import type { RuntimeTarget } from "../shared/bootstrap-config";

export type RuntimeShellCopy = {
  surfaceTitle: string;
  surfaceLabel: string;
  entryEyebrow: string;
  shellEyebrow: string;
  shellSummary: string;
  primaryNavLabel: string;
  settingsEyebrow: string;
  settingsHubTitle: string;
  createRestoreHeading: string;
  createRestoreCopy: string;
  continueHeading: string;
  continueCopy: string;
  essentialsTitle: string;
  backupCopy: string;
  restoreFlowCopy: string;
  settingsHubCopy: string;
  accountRouteCopy: string;
  languageCopy: string;
  storedPreferenceLabel: string;
  runtimeLanguageLabel: string;
  configTitle: string;
};

const runtimeShellCopy: Record<RuntimeTarget, RuntimeShellCopy> = {
  desktop: {
    surfaceTitle: "Desktop",
    surfaceLabel: "desktop",
    entryEyebrow: "Desktop entry",
    shellEyebrow: "Happy-aligned desktop shell",
    shellSummary:
      "Sessions, inbox, settings, and deep links now share one desktop shell backed by live Vibe data instead of a standalone preview layout.",
    primaryNavLabel: "Primary desktop routes",
    settingsEyebrow: "Desktop",
    settingsHubTitle: "Desktop Configuration",
    createRestoreHeading: "Create or restore a Vibe desktop account",
    createRestoreCopy:
      "Sign in with a fresh account, restore from a backup key, or link an existing mobile account to reach the live session flow.",
    continueHeading: "Continue with your desktop sessions",
    continueCopy:
      "Sessions stay front and center, with inbox and settings one step away in the same shell hierarchy Happy uses today.",
    essentialsTitle: "Desktop essentials",
    backupCopy:
      "This backup key restores the desktop app without relying on the current machine.",
    restoreFlowCopy:
      "The route immediately starts the mobile-link request, just like the current desktop restore entry. You can still create a fresh account or restore from the backup secret key instead.",
    settingsHubCopy:
      "The settings hub now tracks the current app more closely: connected accounts, feature routes, desktop configuration, and about links all stay grouped under one route.",
    accountRouteCopy:
      "Identity, linked services, restore material, and logout controls now stay reachable on a dedicated desktop settings route backed by the current desktop account state.",
    languageCopy:
      "Desktop language support now persists the preferred app language locally and applies it to the desktop document state, even though full translated copy switching remains a later step.",
    storedPreferenceLabel: "Current stored desktop preference",
    runtimeLanguageLabel: "Current runtime language",
    configTitle: "Desktop Configuration",
  },
  mobile: {
    surfaceTitle: "Android",
    surfaceLabel: "android",
    entryEyebrow: "Android entry",
    shellEyebrow: "Happy-aligned Android shell",
    shellSummary:
      "Home, inbox, settings, and restore flows now share one Android shell backed by the package-local Wave 9 runtime instead of a desktop-only preview.",
    primaryNavLabel: "Primary Android routes",
    settingsEyebrow: "Android",
    settingsHubTitle: "Android Configuration",
    createRestoreHeading: "Create or restore a Vibe account",
    createRestoreCopy:
      "Create an account, restore from a backup key, or link an existing device to reach the live session flow in the Android shell.",
    continueHeading: "Continue with your Android sessions",
    continueCopy:
      "Sessions stay front and center, with inbox and settings one step away in the same shell hierarchy the mobile app expects.",
    essentialsTitle: "Android essentials",
    backupCopy:
      "This backup key restores the account on another Android install or desktop shell without relying on the current device.",
    restoreFlowCopy:
      "The route keeps the same link and restore semantics on Android: start a device-link request, or fall back to the backup secret key when QR continuation is not available.",
    settingsHubCopy:
      "The settings hub keeps identity, feature routes, Android configuration, and about links grouped under one route before deeper session work lands.",
    accountRouteCopy:
      "Identity, linked services, restore material, and logout controls stay reachable on a dedicated Android account route backed by the shared Wave 9 state.",
    languageCopy:
      "Android language support now persists the preferred app language locally and applies it to the shared document state, even though full translated copy switching remains a later step.",
    storedPreferenceLabel: "Current stored Android preference",
    runtimeLanguageLabel: "Current runtime language",
    configTitle: "Android Configuration",
  },
  browser: {
    surfaceTitle: "Browser",
    surfaceLabel: "browser",
    entryEyebrow: "Browser entry",
    shellEyebrow: "Happy-aligned browser export shell",
    shellSummary:
      "Home, inbox, settings, and deep links remain available through the retained browser export path while the package owns the same Wave 9 runtime boundary.",
    primaryNavLabel: "Primary browser routes",
    settingsEyebrow: "Browser",
    settingsHubTitle: "Browser Configuration",
    createRestoreHeading: "Create or restore a Vibe account",
    createRestoreCopy:
      "Create an account, restore from a backup key, or link an existing device to validate the retained browser export path before session work broadens.",
    continueHeading: "Continue in the browser shell",
    continueCopy:
      "Sessions, inbox, and settings stay available through the retained browser shell so static export parity can be reviewed alongside desktop and Android.",
    essentialsTitle: "Browser essentials",
    backupCopy:
      "This backup key restores the account in the retained browser shell or another runtime without relying on the current environment.",
    restoreFlowCopy:
      "The retained browser route keeps the same create, link, and restore semantics while making browser-safe fallback links explicit.",
    settingsHubCopy:
      "The settings hub keeps identity, feature routes, browser configuration, and about links grouped under one route so retained export behavior stays reviewable.",
    accountRouteCopy:
      "Identity, linked services, restore material, and logout controls stay reachable on a dedicated browser account route backed by the shared Wave 9 state.",
    languageCopy:
      "Browser language support now persists the preferred app language locally and applies it to the retained export document state, even though full translated copy switching remains a later step.",
    storedPreferenceLabel: "Current stored browser preference",
    runtimeLanguageLabel: "Current runtime language",
    configTitle: "Browser Configuration",
  },
};

export function resolveRuntimeShellCopy(runtimeTarget: RuntimeTarget): RuntimeShellCopy {
  return runtimeShellCopy[runtimeTarget];
}

export function buildRuntimeDocumentTitle(
  runtimeTarget: RuntimeTarget,
  routeTitle?: string,
): string {
  const suffix =
    runtimeTarget === "browser"
      ? "Vibe Browser Export"
      : runtimeTarget === "mobile"
        ? "Vibe Android"
        : "Vibe Desktop";
  return routeTitle ? `${routeTitle} | ${suffix}` : suffix;
}

export function buildRuntimeMetaDescription(runtimeTarget: RuntimeTarget): string {
  return resolveRuntimeShellCopy(runtimeTarget).shellSummary;
}
