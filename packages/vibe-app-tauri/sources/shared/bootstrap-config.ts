export const bootstrapSurfaces = {
  desktop: {
    key: "desktop",
    entryPoint: "sources/app/entry/desktop.tsx",
    htmlEntryPoint: "sources/app/entry/browser.tsx",
    outDir: "dist/desktop",
    tauriConfig: "src-tauri/tauri.conf.json",
  },
  mobileAndroid: {
    key: "mobile-android",
    entryPoint: "sources/app/entry/mobile.tsx",
    htmlEntryPoint: "sources/app/entry/browser.tsx",
    outDir: "dist/mobile",
    tauriConfig: "src-tauri/tauri.android.conf.json",
  },
  browser: {
    key: "browser",
    entryPoint: "sources/app/entry/browser.tsx",
    htmlEntryPoint: "sources/app/entry/browser.tsx",
    outDir: "dist/browser/production",
  },
} as const;

export const rootProviderOrder = [
  "StrictMode",
  "RuntimeBootstrapProvider",
  "ThemeBootstrap",
  "SplashScreenBridge",
] as const;

export const rootAssetPaths = [
  "sources/app/assets/fonts/IBMPlexSans-Regular.ttf",
  "sources/app/assets/fonts/IBMPlexSans-Italic.ttf",
  "sources/app/assets/fonts/IBMPlexSans-SemiBold.ttf",
  "sources/app/assets/fonts/IBMPlexMono-Regular.ttf",
  "sources/app/assets/fonts/IBMPlexMono-Italic.ttf",
  "sources/app/assets/fonts/IBMPlexMono-SemiBold.ttf",
  "sources/app/assets/fonts/BricolageGrotesque-Bold.ttf",
  "sources/app/assets/fonts/SpaceMono-Regular.ttf",
  "sources/app/assets/images/logo-black.png",
  "sources/app/assets/images/logo-white.png",
  "sources/app/assets/images/logotype-dark.png",
  "sources/app/assets/images/logotype-light.png",
  "public/splash.svg",
] as const;

export type BootstrapSurfaceKey = keyof typeof bootstrapSurfaces;
export type RuntimeTarget = "desktop" | "mobile" | "browser";
export type AppEnvironment = "development" | "preview" | "production";

export type BootstrapProfile = {
  appEnv: AppEnvironment;
  devHost: "127.0.0.1" | "0.0.0.0";
  devPort: 1420;
  mode: string;
  outDir: string;
  runtimeTarget: RuntimeTarget;
  surfaceKey: BootstrapSurfaceKey;
};

function normalizeMode(mode: string): string {
  return mode.trim().toLowerCase();
}

export function resolveAppEnvironment(mode: string): AppEnvironment {
  const normalizedMode = normalizeMode(mode);
  if (normalizedMode.includes("preview")) {
    return "preview";
  }
  if (normalizedMode.includes("production")) {
    return "production";
  }
  return "development";
}

export function resolveRuntimeTarget(mode: string): RuntimeTarget {
  const normalizedMode = normalizeMode(mode);
  if (normalizedMode.includes("browser")) {
    return "browser";
  }
  if (normalizedMode.includes("mobile") || normalizedMode.includes("android")) {
    return "mobile";
  }
  return "desktop";
}

export function resolveBootstrapSurfaceKey(mode: string): BootstrapSurfaceKey {
  const runtimeTarget = resolveRuntimeTarget(mode);
  if (runtimeTarget === "browser") {
    return "browser";
  }
  if (runtimeTarget === "mobile") {
    return "mobileAndroid";
  }
  return "desktop";
}

export function resolveBootstrapProfile(mode: string): BootstrapProfile {
  const surfaceKey = resolveBootstrapSurfaceKey(mode);
  const runtimeTarget = resolveRuntimeTarget(mode);
  const appEnv = resolveAppEnvironment(mode);
  const outDir =
    runtimeTarget === "browser"
      ? `dist/browser/${appEnv}`
      : bootstrapSurfaces[surfaceKey].outDir;

  return {
    appEnv,
    devHost: runtimeTarget === "mobile" ? "0.0.0.0" : "127.0.0.1",
    devPort: 1420,
    mode,
    outDir,
    runtimeTarget,
    surfaceKey,
  };
}
