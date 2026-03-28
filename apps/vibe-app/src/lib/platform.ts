import type { AppConfig, ControlClientKind, PlatformCapability } from "../types";

const MOBILE_USER_AGENT_PATTERN = /Android|iPhone|iPad|iPod/i;

export function detectControlClientKind(): ControlClientKind {
  if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
    return "tauri_desktop";
  }

  return "web";
}

export function isMobileUserAgent() {
  if (typeof window === "undefined") {
    return false;
  }

  return MOBILE_USER_AGENT_PATTERN.test(window.navigator.userAgent);
}

export function resolveCurrentPlatformCapability(
  config?: AppConfig | null
): PlatformCapability | null {
  const client = detectControlClientKind();
  return config?.platformMatrix.find((capability) => capability.client === client) ?? null;
}

export function prefersExplicitRemoteRelayUrl(config?: AppConfig | null) {
  const capability = resolveCurrentPlatformCapability(config);
  return capability?.prefersExplicitRemoteRelayUrl ?? isMobileUserAgent();
}

export function supportsSystemNotifications(config?: AppConfig | null) {
  const capability = resolveCurrentPlatformCapability(config);
  return capability?.supportsSystemNotifications ?? typeof Notification !== "undefined";
}

export function supportsPersistedRuntimeConfig(config?: AppConfig | null) {
  const capability = resolveCurrentPlatformCapability(config);
  return capability?.supportsPersistedRuntimeConfig ?? false;
}
