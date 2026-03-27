import type { AppConfig } from "../types";

const RELAY_STORAGE_KEY = "vibe.everywhere.relay.baseUrl";
const RELAY_ACCESS_TOKEN_STORAGE_KEY = "vibe.everywhere.relay.accessToken";
const LEGACY_RELAY_STORAGE_KEY = "vibe.remote.relay.baseUrl";
const LEGACY_RELAY_ACCESS_TOKEN_STORAGE_KEY = "vibe.remote.relay.accessToken";
const MOBILE_USER_AGENT_PATTERN = /Android|iPhone|iPad|iPod/i;
const LOOPBACK_HOSTS = new Set(["127.0.0.1", "localhost", "::1"]);

export async function resolveInitialRelayBaseUrl(): Promise<string> {
  const saved = loadStoredValue(RELAY_STORAGE_KEY, LEGACY_RELAY_STORAGE_KEY);
  if (saved) {
    return saved;
  }

  const tauriConfig = await loadTauriConfig();
  if (tauriConfig?.defaultRelayBaseUrl) {
    return tauriConfig.defaultRelayBaseUrl;
  }

  return import.meta.env.VITE_RELAY_BASE_URL ?? "";
}

export function isMobileControlClient() {
  return MOBILE_USER_AGENT_PATTERN.test(window.navigator.userAgent);
}

export function isLoopbackRelayBaseUrl(value: string) {
  const normalized = value.trim();
  if (!normalized) {
    return false;
  }

  try {
    const url = new URL(normalized);
    return LOOPBACK_HOSTS.has(url.hostname.toLowerCase());
  } catch {
    return /^https?:\/\/(localhost|127\.0\.0\.1|\[::1\])(?::\d+)?$/i.test(normalized);
  }
}

export function getRelayBaseUrlPlaceholder() {
  return isMobileControlClient() ? "http://192.168.1.10:8787" : "http://127.0.0.1:8787";
}

export function resolveInitialRelayAccessToken(): string {
  const saved = loadStoredValue(
    RELAY_ACCESS_TOKEN_STORAGE_KEY,
    LEGACY_RELAY_ACCESS_TOKEN_STORAGE_KEY
  );
  if (saved) {
    return saved;
  }

  return import.meta.env.VITE_RELAY_ACCESS_TOKEN ?? "";
}

export async function loadTauriConfig(): Promise<AppConfig | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<AppConfig>("app_config");
  } catch {
    return null;
  }
}

export function persistRelayBaseUrl(baseUrl: string) {
  persistStoredValue(RELAY_STORAGE_KEY, LEGACY_RELAY_STORAGE_KEY, normalizeRelayBaseUrl(baseUrl));
}

export function persistRelayAccessToken(accessToken: string) {
  persistStoredValue(
    RELAY_ACCESS_TOKEN_STORAGE_KEY,
    LEGACY_RELAY_ACCESS_TOKEN_STORAGE_KEY,
    accessToken.trim()
  );
}

export function buildApiUrl(baseUrl: string, path: string): string {
  const normalizedPath = normalizePath(path);
  const normalizedBaseUrl = normalizeRelayBaseUrl(baseUrl);

  if (!normalizedBaseUrl) {
    return normalizedPath;
  }

  return new URL(normalizedPath, `${normalizedBaseUrl}/`).toString();
}

export function buildEventStreamUrl(baseUrl: string, accessToken: string): string {
  const url = buildApiUrl(baseUrl, "/api/events/stream");
  if (!accessToken.trim()) {
    return url;
  }

  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}access_token=${encodeURIComponent(accessToken.trim())}`;
}

export function buildWebSocketUrl(
  baseUrl: string,
  path: string,
  accessToken: string
): string {
  const rawUrl = buildApiUrl(baseUrl, path);
  const url = new URL(rawUrl, window.location.origin);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  if (accessToken.trim()) {
    url.searchParams.set("access_token", accessToken.trim());
  }
  return url.toString();
}

export function normalizeRelayBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, "");
}

function normalizePath(path: string) {
  return path.startsWith("/") ? path : `/${path}`;
}

function loadStoredValue(primaryKey: string, legacyKey: string) {
  const primaryValue = window.localStorage.getItem(primaryKey);
  if (primaryValue) {
    return primaryValue;
  }

  const legacyValue = window.localStorage.getItem(legacyKey);
  if (legacyValue) {
    window.localStorage.setItem(primaryKey, legacyValue);
  }
  return legacyValue ?? "";
}

function persistStoredValue(primaryKey: string, legacyKey: string, value: string) {
  if (value) {
    window.localStorage.setItem(primaryKey, value);
  } else {
    window.localStorage.removeItem(primaryKey);
  }
  window.localStorage.removeItem(legacyKey);
}
