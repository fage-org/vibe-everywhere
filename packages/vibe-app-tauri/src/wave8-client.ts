import { invoke, isTauri } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  applySettings,
  settingsDefaults,
  settingsParse,
  type Settings,
} from "../sources/shared/sync/settings";
import {
  decodeBase64 as decodeBase64Shared,
  encodeBase64 as encodeBase64Shared,
} from "../sources/shared/encryption/base64";
import {
  formatSecretKeyForBackup as formatSecretKeyForBackupShared,
  normalizeSecretKey as normalizeSecretKeyShared,
} from "../sources/shared/auth/secret-key-backup";
import {
  buildOutgoingUserMessageRecord,
  type OutgoingSessionMessageOptions,
} from "./session-message-meta";
import {
  AccountLinkRequestResponseSchema,
  AccountProfileSchema,
  AccountTokenResponseSchema,
  ArtifactBodySchema,
  ArtifactHeaderSchema,
  ArtifactUpdateResponseSchema,
  ArtifactsResponseSchema,
  LoopbackAttemptSchema,
  LoopbackAttemptStatusSchema,
  MachineDaemonStateSchema,
  MachineDetailResponseSchema,
  MachineMetadataSchema,
  MachinesResponseSchema,
  RemoteArtifactRecordSchema,
  RemoteMachineRecordSchema,
  RemoteSessionRecordSchema,
  SessionBashResponseSchema,
  SessionReadFileResponseSchema,
  SessionRpcAckSchema,
  SessionResponseSchema,
  SessionAgentStateSchema,
  SessionMessagesResponseSchema,
  SessionMetadataSchema,
  SessionsResponseSchema,
  StoredCredentialsSchema,
  UsageQueryResponseSchema,
  UserResponseSchema,
  parseWithSchema,
  type AccountProfile,
  type ArtifactBody,
  type ArtifactHeader,
  type ArtifactUpdateResponse,
  type LoopbackAttempt,
  type LoopbackAttemptStatus,
  type MachineMetadata,
  type RemoteMachineRecord,
  type RemoteArtifactRecord,
  type RemoteMessageRecord,
  type RemoteSessionRecord,
  type SessionBashRequest,
  type SessionBashResponse,
  type SessionReadFileResponse,
  type SessionRpcAck,
  type SessionAgentState,
  type SessionAgentStateUpdate,
  type SessionMetadata,
  type SessionMetadataUpdate,
  type StoredCredentials,
  type UsageBucket,
  type UserProfile,
  type FeedPostResponse,
  FriendsListResponseSchema,
  FeedListResponseSchema,
  UserProfileSchema,
} from "./wave8-wire";

export type {
  AccountProfile,
  ArtifactUpdateResponse,
  LoopbackAttempt,
  LoopbackAttemptStatus,
  RemoteMachineRecord,
  RemoteArtifactRecord,
  RemoteMessageRecord,
  SessionBashRequest,
  SessionBashResponse,
  SessionReadFileResponse,
  SessionRpcAck,
  SessionAgentStateUpdate,
  SessionMetadata,
  SessionMetadataUpdate,
  StoredCredentials,
  UsageBucket,
  UserProfile,
} from "./wave8-wire";
export type { Settings } from "../sources/shared/sync/settings";
export type { FeedPostResponse } from "./wave8-wire";

export type SendMessageOptions = Omit<OutgoingSessionMessageOptions, "sentFrom">;

const DEFAULT_SERVER_URL = "https://api.cluster-fluster.com";
const SERVER_URL_KEY = "vibe-app-tauri.server-url";
const CREDENTIALS_KEY = "vibe-app-tauri.credentials";

export type DesktopSession = {
  id: string;
  seq: number;
  createdAt: number;
  updatedAt: number;
  active: boolean;
  activeAt: number;
  metadata: SessionMetadata | null;
  metadataVersion: number;
  agentState: Record<string, unknown> | null;
  agentStateVersion: number;
  dataEncryptionKey: string | null;
};

export type DesktopArtifact = {
  id: string;
  title: string | null;
  sessions?: string[];
  draft?: boolean;
  body?: string | null;
  headerVersion: number;
  bodyVersion?: number;
  seq: number;
  createdAt: number;
  updatedAt: number;
  isDecrypted: boolean;
};

export type DesktopMachine = {
  id: string;
  seq: number;
  createdAt: number;
  updatedAt: number;
  active: boolean;
  activeAt: number;
  metadata: MachineMetadata | null;
  metadataVersion: number;
  daemonState: unknown | null;
  daemonStateVersion: number;
};

export type UiMessage = {
  id: string;
  localId: string | null;
  createdAt: number;
  role: "user" | "assistant" | "system" | "tool";
  title: string;
  text: string;
  rawType: string;
};

export type SessionMessagesState = {
  items: UiMessage[];
  loadedAt: number;
  lastSeq: number | null;
};

export type CreateSessionInput = {
  workspace: string;
  model: string;
  prompt: string;
  title?: string;
};

export type UsagePeriod = "today" | "7days" | "30days";

export type AccountLinkRequest = {
  publicKey: Uint8Array;
  secretKey: Uint8Array;
  linkUrl: string;
};

const AccountSettingsResponseSchema = z.object({
  settings: z.string().nullable(),
  settingsVersion: z.number(),
});

const UpdateAccountSettingsResponseSchema = z.discriminatedUnion("success", [
  z.object({
    success: z.literal(true),
    version: z.number(),
  }),
  z.object({
    success: z.literal(false),
    error: z.literal("version-mismatch"),
    currentVersion: z.number(),
    currentSettings: z.string().nullable(),
  }),
]);

type AuthRequestStatus = {
  status: "not_found" | "pending" | "authorized";
  supportsV2: boolean;
};

type SessionCipher = {
  encryptRecord(record: unknown): Promise<string>;
  decryptRecord(ciphertext: string): Promise<unknown | null>;
  decryptMetadata(ciphertext: string): Promise<SessionMetadata | null>;
  decryptAgentState(ciphertext: string | null): Promise<SessionAgentState | null>;
};

type EncryptionContext = {
  masterSecret: Uint8Array;
  contentPrivateKey: Uint8Array;
  contentPublicKey: Uint8Array;
};

const TerminalAuthRequestStatusSchema = z.object({
  status: z.enum(["not_found", "pending", "authorized"]),
  supportsV2: z.boolean(),
});

type SodiumModule = (typeof import("libsodium-wrappers"))["default"];
let sodiumPromise: Promise<SodiumModule> | null = null;

function getStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function isDesktopRuntime(): boolean {
  return typeof window !== "undefined" && isTauri();
}

function isLoopbackHostname(hostname: string): boolean {
  const normalized = hostname.trim().toLowerCase();
  return normalized === "127.0.0.1" || normalized === "localhost" || normalized === "::1";
}

export function normalizeServerUrl(url: string): string {
  const candidate = url.trim() || DEFAULT_SERVER_URL;
  let parsed: URL;
  try {
    parsed = new URL(candidate);
  } catch {
    throw new Error("Server URL must be an absolute URL");
  }

  if (parsed.protocol === "https:") {
    return `${parsed.origin}${parsed.pathname === "/" ? "" : parsed.pathname}`.replace(/\/+$/, "");
  }

  if (parsed.protocol === "http:" && isLoopbackHostname(parsed.hostname)) {
    return `${parsed.origin}${parsed.pathname === "/" ? "" : parsed.pathname}`.replace(/\/+$/, "");
  }

  throw new Error("Server URL must use https unless it targets localhost");
}

export function loadServerUrl(): string {
  const storage = getStorage();
  const stored = storage?.getItem(SERVER_URL_KEY)?.trim();
  if (!stored) {
    return DEFAULT_SERVER_URL;
  }

  try {
    return normalizeServerUrl(stored);
  } catch {
    return DEFAULT_SERVER_URL;
  }
}

export function saveServerUrl(url: string): string {
  const normalized = normalizeServerUrl(url);
  getStorage()?.setItem(SERVER_URL_KEY, normalized);
  return normalized;
}

export async function loadStoredCredentials(): Promise<StoredCredentials | null> {
  const raw = isDesktopRuntime()
    ? await invoke<string | null>("secure_store_get_credentials")
    : getStorage()?.getItem(CREDENTIALS_KEY) ?? null;
  if (!raw) {
    return null;
  }

  try {
    return parseWithSchema(
      StoredCredentialsSchema,
      JSON.parse(raw),
      "Stored desktop credentials",
    );
  } catch {
    return null;
  }
}

export async function saveStoredCredentials(credentials: StoredCredentials): Promise<void> {
  const value = JSON.stringify(credentials);
  if (isDesktopRuntime()) {
    await invoke("secure_store_set_credentials", { value });
    return;
  }
  getStorage()?.setItem(CREDENTIALS_KEY, value);
}

export async function clearStoredCredentials(): Promise<void> {
  if (isDesktopRuntime()) {
    await invoke("secure_store_clear_credentials");
    return;
  }
  getStorage()?.removeItem(CREDENTIALS_KEY);
}

export async function copyTextToClipboard(text: string): Promise<void> {
  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  if (typeof document === "undefined") {
    throw new Error("Clipboard is unavailable in this runtime");
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  textarea.style.pointerEvents = "none";
  document.body.appendChild(textarea);
  textarea.select();

  const copied = document.execCommand("copy");
  document.body.removeChild(textarea);

  if (!copied) {
    throw new Error("Clipboard copy was rejected");
  }
}

export async function openTextFileDialog(title: string): Promise<string | null> {
  if (isDesktopRuntime()) {
    return invoke<string | null>("open_text_file_dialog", { title });
  }

  if (typeof document === "undefined" || typeof window === "undefined") {
    throw new Error("File dialogs are unavailable in this runtime");
  }

  return new Promise<string | null>((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".txt,.md,.key,.json";
    input.style.position = "fixed";
    input.style.opacity = "0";
    input.style.pointerEvents = "none";
    document.body.appendChild(input);

    let settled = false;
    let cancelTimer: number | null = null;

    const finish = (callback: () => void) => {
      if (settled) {
        return;
      }
      settled = true;
      if (cancelTimer !== null) {
        window.clearTimeout(cancelTimer);
      }
      window.removeEventListener("focus", handleWindowFocus);
      cleanup();
      callback();
    };

    const cleanup = () => {
      input.remove();
    };

    const handleWindowFocus = () => {
      cancelTimer = window.setTimeout(() => {
        if (!settled && !(input.files?.length)) {
          finish(() => resolve(null));
        }
      }, 250);
    };

    input.addEventListener(
      "change",
      () => {
        const file = input.files?.[0];
        if (!file) {
          finish(() => resolve(null));
          return;
        }

        const reader = new FileReader();
        reader.onload = () => {
          finish(() => resolve(typeof reader.result === "string" ? reader.result : null));
        };
        reader.onerror = () => {
          finish(() => reject(new Error(`Failed to read ${file.name}`)));
        };
        reader.readAsText(file);
      },
      { once: true },
    );

    window.addEventListener("focus", handleWindowFocus);
    input.click();
  });
}

export async function saveTextFileDialog(input: {
  title: string;
  suggestedName: string;
  contents: string;
}): Promise<string | null> {
  if (isDesktopRuntime()) {
    return invoke<string | null>("save_text_file_dialog", input);
  }

  if (typeof document === "undefined" || typeof URL === "undefined") {
    throw new Error("File save is unavailable in this runtime");
  }

  const blob = new Blob([input.contents], { type: "text/plain;charset=utf-8" });
  const objectUrl = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = objectUrl;
  anchor.download = input.suggestedName;
  anchor.style.display = "none";
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(objectUrl);
  return input.suggestedName;
}

export async function showDesktopNotification(title: string, body: string): Promise<void> {
  if (isDesktopRuntime()) {
    await invoke("show_desktop_notification", { title, body });
    return;
  }

  if (typeof Notification === "undefined") {
    return;
  }

  if (Notification.permission === "granted") {
    new Notification(title, { body });
    return;
  }

  if (Notification.permission === "default") {
    const permission = await Notification.requestPermission();
    if (permission === "granted") {
      new Notification(title, { body });
    }
  }
}

export const encodeBase64 = encodeBase64Shared;
export const decodeBase64 = decodeBase64Shared;

function randomBytes(size: number): Uint8Array {
  const bytes = new Uint8Array(size);
  crypto.getRandomValues(bytes);
  return bytes;
}

async function ensureSodium(): Promise<SodiumModule> {
  sodiumPromise ??= import("libsodium-wrappers").then(async (module) => {
    await module.default.ready;
    return module.default;
  });
  return sodiumPromise;
}

async function hmacSha512(key: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
  const importedKey = await crypto.subtle.importKey(
    "raw",
    Uint8Array.from(key),
    { name: "HMAC", hash: "SHA-512" },
    false,
    ["sign"],
  );
  const signature = await crypto.subtle.sign("HMAC", importedKey, Uint8Array.from(data));
  return new Uint8Array(signature);
}

async function deriveKey(
  master: Uint8Array,
  usage: string,
  path: string[],
): Promise<Uint8Array> {
  let state = await hmacSha512(master, new TextEncoder().encode(`${usage} Master Seed`));
  let key = state.slice(0, 32);
  let chainCode = state.slice(32);

  for (const index of path) {
    const indexBytes = new TextEncoder().encode(index);
    const payload = new Uint8Array(indexBytes.length + 1);
    payload[0] = 0;
    payload.set(indexBytes, 1);
    state = await hmacSha512(chainCode, payload);
    key = state.slice(0, 32);
    chainCode = state.slice(32);
  }

  return key;
}

export const formatSecretKeyForBackup = formatSecretKeyForBackupShared;
export const normalizeSecretKey = normalizeSecretKeyShared;

async function encryptBox(
  plaintext: Uint8Array,
  recipientPublicKey: Uint8Array,
): Promise<Uint8Array> {
  const libsodium = await ensureSodium();
  const ephemeralKeyPair = libsodium.crypto_box_keypair();
  const nonce = randomBytes(libsodium.crypto_box_NONCEBYTES);
  const encrypted = libsodium.crypto_box_easy(
    plaintext,
    nonce,
    recipientPublicKey,
    ephemeralKeyPair.privateKey,
  );

  const output = new Uint8Array(
    ephemeralKeyPair.publicKey.length + nonce.length + encrypted.length,
  );
  output.set(ephemeralKeyPair.publicKey, 0);
  output.set(nonce, ephemeralKeyPair.publicKey.length);
  output.set(encrypted, ephemeralKeyPair.publicKey.length + nonce.length);
  return output;
}

async function decryptBox(
  encryptedBundle: Uint8Array,
  recipientSecretKey: Uint8Array,
): Promise<Uint8Array | null> {
  const libsodium = await ensureSodium();
  const publicKeyLength = libsodium.crypto_box_PUBLICKEYBYTES;
  const nonceLength = libsodium.crypto_box_NONCEBYTES;
  const ephemeralPublicKey = encryptedBundle.slice(0, publicKeyLength);
  const nonce = encryptedBundle.slice(publicKeyLength, publicKeyLength + nonceLength);
  const encrypted = encryptedBundle.slice(publicKeyLength + nonceLength);

  try {
    return libsodium.crypto_box_open_easy(
      encrypted,
      nonce,
      ephemeralPublicKey,
      recipientSecretKey,
    );
  } catch {
    return null;
  }
}

async function encryptSecretBox(payload: unknown, secret: Uint8Array): Promise<Uint8Array> {
  const libsodium = await ensureSodium();
  const nonce = randomBytes(libsodium.crypto_secretbox_NONCEBYTES);
  const plaintext = new TextEncoder().encode(JSON.stringify(payload));
  const encrypted = libsodium.crypto_secretbox_easy(plaintext, nonce, secret);
  const output = new Uint8Array(nonce.length + encrypted.length);
  output.set(nonce, 0);
  output.set(encrypted, nonce.length);
  return output;
}

async function decryptSecretBox(
  encryptedBundle: Uint8Array,
  secret: Uint8Array,
): Promise<unknown | null> {
  const libsodium = await ensureSodium();
  const nonce = encryptedBundle.slice(0, libsodium.crypto_secretbox_NONCEBYTES);
  const encrypted = encryptedBundle.slice(libsodium.crypto_secretbox_NONCEBYTES);

  try {
    const decrypted = libsodium.crypto_secretbox_open_easy(encrypted, nonce, secret);
    return JSON.parse(new TextDecoder().decode(decrypted));
  } catch {
    return null;
  }
}

async function importAesKey(keyBase64: string): Promise<CryptoKey> {
  const raw = decodeBase64(keyBase64);
  return crypto.subtle.importKey("raw", Uint8Array.from(raw), { name: "AES-GCM" }, false, [
    "encrypt",
    "decrypt",
  ]);
}

async function encryptAesPayload(payload: unknown, keyBase64: string): Promise<Uint8Array> {
  const key = await importAesKey(keyBase64);
  const iv = randomBytes(12);
  const plaintext = new TextEncoder().encode(JSON.stringify(payload));
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: Uint8Array.from(iv) },
    key,
    Uint8Array.from(plaintext),
  );
  const encryptedBytes = new Uint8Array(encrypted);
  const output = new Uint8Array(1 + iv.length + encryptedBytes.length);
  output[0] = 0;
  output.set(iv, 1);
  output.set(encryptedBytes, 13);
  return output;
}

async function decryptAesPayload(
  encryptedBundle: Uint8Array,
  keyBase64: string,
): Promise<unknown | null> {
  if (encryptedBundle[0] !== 0) {
    return null;
  }

  const key = await importAesKey(keyBase64);
  const iv = encryptedBundle.slice(1, 13);
  const ciphertext = encryptedBundle.slice(13);

  try {
    const plaintext = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: Uint8Array.from(iv) },
      key,
      Uint8Array.from(ciphertext),
    );
    return JSON.parse(new TextDecoder().decode(plaintext));
  } catch {
    return null;
  }
}

async function createEncryptionContext(secret: string): Promise<EncryptionContext> {
  const masterSecret = decodeBase64(secret, "base64url");
  const libsodium = await ensureSodium();
  const contentSeed = await deriveKey(masterSecret, "Happy EnCoder", ["content"]);
  const contentKeyPair = libsodium.crypto_box_seed_keypair(contentSeed);
  return {
    masterSecret,
    contentPrivateKey: contentKeyPair.privateKey,
    contentPublicKey: contentKeyPair.publicKey,
  };
}

async function encryptLegacyRaw(value: unknown, masterSecret: Uint8Array): Promise<string> {
  const encrypted = await encryptSecretBox(value, masterSecret);
  return encodeBase64(encrypted, "base64");
}

async function decryptLegacyRaw(
  encrypted: string | null,
  masterSecret: Uint8Array,
): Promise<unknown | null> {
  if (!encrypted) {
    return null;
  }

  try {
    return await decryptSecretBox(decodeBase64(encrypted, "base64"), masterSecret);
  } catch {
    return null;
  }
}

function sessionCipherFromKeys(
  masterSecret: Uint8Array,
  dataKey: Uint8Array | null,
): SessionCipher {
  const aesKeyBase64 = dataKey ? encodeBase64(dataKey) : null;

  return {
    async encryptRecord(record: unknown): Promise<string> {
      const encrypted = aesKeyBase64
        ? await encryptAesPayload(record, aesKeyBase64)
        : await encryptSecretBox(record, masterSecret);
      return encodeBase64(encrypted);
    },

    async decryptRecord(ciphertext: string): Promise<unknown | null> {
      const encrypted = decodeBase64(ciphertext);
      return aesKeyBase64
        ? decryptAesPayload(encrypted, aesKeyBase64)
        : decryptSecretBox(encrypted, masterSecret);
    },

    async decryptMetadata(ciphertext: string): Promise<SessionMetadata | null> {
      const raw = await this.decryptRecord(ciphertext);
      if (raw === null) {
        return null;
      }
      return parseWithSchema(
        SessionMetadataSchema,
        raw,
        "Decrypted desktop session metadata",
      );
    },

    async decryptAgentState(
      ciphertext: string | null,
    ): Promise<SessionAgentState | null> {
      if (!ciphertext) {
        return null;
      }
      const raw = await this.decryptRecord(ciphertext);
      if (raw === null) {
        return null;
      }
      return parseWithSchema(
        SessionAgentStateSchema,
        raw,
        "Decrypted desktop session agent state",
      );
    },
  };
}

function artifactCipherFromKey(dataKey: Uint8Array) {
  const aesKeyBase64 = encodeBase64(dataKey);

  return {
    async encryptHeader(header: ArtifactHeader): Promise<string> {
      const encrypted = await encryptAesPayload(header, aesKeyBase64);
      return encodeBase64(encrypted, "base64");
    },

    async decryptHeader(ciphertext: string): Promise<ArtifactHeader | null> {
      const encrypted = decodeBase64(ciphertext, "base64");
      const decrypted = await decryptAesPayload(encrypted, aesKeyBase64);
      if (decrypted === null) {
        return null;
      }
      return parseWithSchema(
        ArtifactHeaderSchema,
        decrypted,
        "Decrypted desktop artifact header",
      );
    },

    async encryptBody(body: ArtifactBody): Promise<string> {
      const encrypted = await encryptAesPayload(body, aesKeyBase64);
      return encodeBase64(encrypted, "base64");
    },

    async decryptBody(ciphertext: string): Promise<ArtifactBody | null> {
      const encrypted = decodeBase64(ciphertext, "base64");
      const decrypted = await decryptAesPayload(encrypted, aesKeyBase64);
      if (decrypted === null) {
        return null;
      }
      return parseWithSchema(
        ArtifactBodySchema,
        decrypted,
        "Decrypted desktop artifact body",
      );
    },
  };
}

function resolveSentFromPlatform(): string {
  if (typeof navigator === "undefined") {
    return "desktop-tauri";
  }

  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes("android")) {
    return "android";
  }
  if (userAgent.includes("iphone") || userAgent.includes("ipad")) {
    return "ios";
  }
  if (isTauri()) {
    return "desktop-tauri";
  }
  return "web";
}

function buildUserMessageRecord(
  text: string,
  options: SendMessageOptions | undefined,
): Record<string, unknown> {
  return buildOutgoingUserMessageRecord(text, {
    sentFrom: resolveSentFromPlatform(),
    permissionMode: options?.permissionMode ?? undefined,
    model: options?.model ?? null,
    displayText: options?.displayText ?? null,
  });
}

function summarizeUnknown(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function asObject(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" ? (value as Record<string, unknown>) : null;
}

function contentText(value: unknown): string {
  return typeof value === "string" ? value : summarizeUnknown(value);
}

function extractTextFromToolResult(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  if (Array.isArray(value)) {
    return value
      .map((entry) => {
        const item = asObject(entry);
        if (item?.type === "text") {
          return contentText(item.text);
        }
        return summarizeUnknown(entry);
      })
      .join("\n\n");
  }

  return summarizeUnknown(value);
}

function mapContentItemsToUiMessages(
  baseId: string,
  localId: string | null,
  createdAt: number,
  items: unknown[],
  fallbackRole: "assistant" | "user",
  prefix: string,
): UiMessage[] {
  const output: string[] = [];
  const messages: UiMessage[] = [];

  items.forEach((item, index) => {
    const content = asObject(item);
    if (!content) {
      messages.push({
        id: `${baseId}:${index}`,
        localId,
        createdAt,
        role: fallbackRole,
        title: fallbackRole === "user" ? "User" : "Assistant",
        text: summarizeUnknown(item),
        rawType: `${prefix}:unknown`,
      });
      return;
    }

    const type = typeof content.type === "string" ? content.type : "unknown";
    if (type === "text") {
      messages.push({
        id: `${baseId}:${index}`,
        localId,
        createdAt,
        role: fallbackRole,
        title: fallbackRole === "user" ? "User" : "Assistant",
        text: contentText(content.text),
        rawType: `${prefix}:text`,
      });
      return;
    }
    if (type === "thinking") {
      messages.push({
        id: `${baseId}:${index}`,
        localId,
        createdAt,
        role: "system",
        title: "Reasoning",
        text: contentText(content.thinking),
        rawType: `${prefix}:thinking`,
      });
      return;
    }
    if (type === "tool_use" || type === "tool-call") {
      messages.push({
        id: `${baseId}:${index}`,
        localId,
        createdAt,
        role: "tool",
        title: `Tool call: ${String(content.name ?? "unknown")}`,
        text: summarizeUnknown(content.input ?? {}),
        rawType: `${prefix}:tool-call`,
      });
      return;
    }
    if (type === "tool_result" || type === "tool-call-result") {
      messages.push({
        id: `${baseId}:${index}`,
        localId,
        createdAt,
        role: "tool",
        title: "Tool result",
        text: extractTextFromToolResult(content.content ?? content.output ?? {}),
        rawType: `${prefix}:tool-result`,
      });
      return;
    }
    messages.push({
      id: `${baseId}:${index}`,
      localId,
      createdAt,
      role: fallbackRole,
      title: fallbackRole === "user" ? "User" : "Assistant",
      text: summarizeUnknown(content),
      rawType: `${prefix}:${type}`,
    });
  });

  return messages.length > 0
    ? messages
    : [
        {
          id: baseId,
          localId,
          createdAt,
          role: fallbackRole,
          title: fallbackRole === "user" ? "User" : "Assistant",
          text: "",
          rawType: `${prefix}:empty`,
        },
      ];
}

export function parseRawRecordToUiMessages(
  id: string,
  localId: string | null,
  createdAt: number,
  rawRecord: unknown,
): UiMessage[] {
  const record = asObject(rawRecord);
  if (!record) {
    return [
      {
        id,
        localId,
        createdAt,
        role: "system",
        title: "Unsupported message",
        text: summarizeUnknown(rawRecord),
        rawType: "unknown",
      },
    ];
  }

  const role = typeof record.role === "string" ? record.role : "unknown";
  const meta = asObject(record.meta);

  if (role === "user") {
    const content = asObject(record.content);
    const displayText = typeof meta?.displayText === "string" ? meta.displayText : undefined;
    return [
      {
        id,
        localId,
        createdAt,
        role: "user",
        title: "User",
        text:
          displayText ??
          (typeof content?.text === "string"
            ? content.text
            : summarizeUnknown(content ?? record.content)),
        rawType: String(content?.type ?? "text"),
      },
    ];
  }

  if (role === "session") {
    const content = asObject(record.content);
    const data = asObject(content?.data);
    const event = asObject(data?.ev);
    const eventType = typeof event?.t === "string" ? event.t : "event";
    const envelopeRole = data?.role === "user" ? "user" : "assistant";

    if (eventType === "text") {
      return [
        {
          id,
          localId,
          createdAt,
          role: envelopeRole,
          title: envelopeRole === "user" ? "User" : "Agent",
          text: typeof event?.text === "string" ? event.text : summarizeUnknown(event),
          rawType: `session:${eventType}`,
        },
      ];
    }

    if (eventType === "service") {
      return [
        {
          id,
          localId,
          createdAt,
          role: "assistant",
          title: "Service",
          text: typeof event?.text === "string" ? event.text : summarizeUnknown(event),
          rawType: `session:${eventType}`,
        },
      ];
    }

    if (eventType === "tool-call-start") {
      return [
        {
          id,
          localId,
          createdAt,
          role: "tool",
          title: String(event?.title ?? event?.name ?? "Tool call"),
          text: summarizeUnknown(event?.args ?? {}),
          rawType: `session:${eventType}`,
        },
      ];
    }

    if (eventType === "file") {
      return [
        {
          id,
          localId,
          createdAt,
          role: "system",
          title: "File",
          text: `${String(event?.name ?? "file")} (${String(event?.size ?? 0)} bytes)`,
          rawType: `session:${eventType}`,
        },
      ];
    }

    return [
      {
        id,
        localId,
        createdAt,
        role: "system",
        title: "Session event",
        text: summarizeUnknown(event ?? data),
        rawType: `session:${eventType}`,
      },
    ];
  }

  if (role === "agent") {
    const content = asObject(record.content);
    const type = typeof content?.type === "string" ? content.type : "unknown";

    if (type === "output") {
      const data = asObject(content?.data);
      const dataType = typeof data?.type === "string" ? data.type : "unknown";

      if (dataType === "assistant") {
        const message = asObject(data?.message);
        return Array.isArray(message?.content)
          ? mapContentItemsToUiMessages(
              id,
              localId,
              createdAt,
              message.content,
              "assistant",
              "agent:assistant",
            )
          : [
              {
                id,
                localId,
                createdAt,
                role: "assistant",
                title: "Assistant",
                text: summarizeUnknown(message?.content ?? message),
                rawType: `agent:${dataType}`,
              },
            ];
      }

      if (dataType === "user") {
        const message = asObject(data?.message);
        return Array.isArray(message?.content)
          ? mapContentItemsToUiMessages(
              id,
              localId,
              createdAt,
              message.content,
              "user",
              "agent:user",
            )
          : [
              {
                id,
                localId,
                createdAt,
                role: "user",
                title: "User",
                text: summarizeUnknown(message?.content ?? data?.message),
                rawType: `agent:${dataType}`,
              },
            ];
      }

      if (dataType === "summary") {
        return [
          {
            id,
            localId,
            createdAt,
            role: "system",
            title: "Summary",
            text: typeof data?.summary === "string" ? data.summary : summarizeUnknown(data),
            rawType: `agent:${dataType}`,
          },
        ];
      }

      if (dataType === "message") {
        return [
          {
            id,
            localId,
            createdAt,
            role: "assistant",
            title: "Assistant",
            text: typeof data?.message === "string" ? data.message : summarizeUnknown(data),
            rawType: `agent:${dataType}`,
          },
        ];
      }
    }

    if (type === "event") {
      const data = asObject(content?.data);
      return [
        {
          id,
          localId,
          createdAt,
          role: "system",
          title: "Agent event",
          text: typeof data?.message === "string" ? data.message : summarizeUnknown(data),
          rawType: `agent:${type}`,
        },
      ];
    }

    if (type === "acp" || type === "codex") {
      const data = asObject(content?.data);
      const dataType = typeof data?.type === "string" ? data.type : "unknown";

      if (dataType === "message" || dataType === "reasoning") {
        return [
          {
            id,
            localId,
            createdAt,
            role: dataType === "reasoning" ? "system" : "assistant",
            title: dataType === "reasoning" ? "Reasoning" : "Assistant",
            text: typeof data?.message === "string" ? data.message : summarizeUnknown(data),
            rawType: `${type}:${dataType}`,
          },
        ];
      }

      if (dataType === "tool-call") {
        return [
          {
            id,
            localId,
            createdAt,
            role: "tool",
            title: `Tool call: ${String(data?.name ?? "unknown")}`,
            text: summarizeUnknown(data?.input ?? {}),
            rawType: `${type}:${dataType}`,
          },
        ];
      }

      if (dataType === "tool-result" || dataType === "tool-call-result") {
        return [
          {
            id,
            localId,
            createdAt,
            role: "tool",
            title: "Tool result",
            text: summarizeUnknown(data?.output ?? {}),
            rawType: `${type}:${dataType}`,
          },
        ];
      }

      if (dataType === "terminal-output") {
        return [
          {
            id,
            localId,
            createdAt,
            role: "system",
            title: "Terminal output",
            text: typeof data?.data === "string" ? data.data : summarizeUnknown(data),
            rawType: `${type}:${dataType}`,
          },
        ];
      }
    }
  }

  return [
    {
      id,
      localId,
      createdAt,
      role: "system",
      title: "Unsupported message",
      text: summarizeUnknown(rawRecord),
      rawType: role,
    },
  ];
}

export function describeSession(session: DesktopSession): {
  title: string;
  subtitle: string;
  detail: string;
} {
  const metadata = session.metadata;
  const path = typeof metadata?.path === "string" ? metadata.path : null;
  const summary = typeof metadata?.summary?.text === "string" ? metadata.summary.text : null;
  const pathTitle = path ? path.split(/[\\/]/).filter(Boolean).at(-1) : null;

  return {
    title: summary || pathTitle || metadata?.name?.toString() || session.id,
    subtitle: path || metadata?.host?.toString() || session.id,
    detail:
      metadata?.currentModelCode?.toString() ||
      metadata?.os?.toString() ||
      new Date(session.updatedAt).toLocaleString(),
  };
}

export function buildUsageQueryParams(
  period: UsagePeriod,
  sessionId?: string,
): {
  sessionId?: string;
  startTime: number;
  endTime: number;
  groupBy: "hour" | "day";
} {
  const now = Math.floor(Date.now() / 1000);
  const oneDaySeconds = 24 * 60 * 60;

  switch (period) {
    case "today": {
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      return {
        sessionId,
        startTime: Math.floor(today.getTime() / 1000),
        endTime: now,
        groupBy: "hour",
      };
    }
    case "30days":
      return {
        sessionId,
        startTime: now - 30 * oneDaySeconds,
        endTime: now,
        groupBy: "day",
      };
    case "7days":
    default:
      return {
        sessionId,
        startTime: now - 7 * oneDaySeconds,
        endTime: now,
        groupBy: "day",
      };
  }
}

export function calculateUsageTotals(usage: UsageBucket[]): {
  totalTokens: number;
  totalCost: number;
  tokensByModel: Record<string, number>;
  costByModel: Record<string, number>;
} {
  const result = {
    totalTokens: 0,
    totalCost: 0,
    tokensByModel: {} as Record<string, number>,
    costByModel: {} as Record<string, number>,
  };

  for (const bucket of usage) {
    for (const [model, tokens] of Object.entries(bucket.tokens)) {
      result.totalTokens += tokens;
      result.tokensByModel[model] = (result.tokensByModel[model] || 0) + tokens;
    }

    for (const [model, cost] of Object.entries(bucket.cost)) {
      result.totalCost += cost;
      result.costByModel[model] = (result.costByModel[model] || 0) + cost;
    }
  }

  return result;
}

async function fetchJson<T>(
  input: RequestInfo | URL,
  init?: RequestInit,
  parser?: (value: unknown) => T,
): Promise<T> {
  const response = await fetch(input, init);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `${response.status} ${response.statusText}`);
  }
  const rawText = await response.text();
  if (!rawText) {
    return parser ? parser(undefined) : (undefined as T);
  }

  const payload = JSON.parse(rawText) as unknown;
  return parser ? parser(payload) : (payload as T);
}

export async function createAccount(
  serverUrl: string,
): Promise<{ credentials: StoredCredentials; backupKey: string }> {
  const libsodium = await ensureSodium();
  const secret = randomBytes(32);
  const keypair = libsodium.crypto_sign_seed_keypair(secret);
  const challenge = randomBytes(32);
  const signature = libsodium.crypto_sign_detached(challenge, keypair.privateKey);

  const response = await fetchJson<{ success: boolean; token: string }>(
    `${serverUrl}/v1/auth`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        challenge: encodeBase64(challenge),
        signature: encodeBase64(signature),
        publicKey: encodeBase64(keypair.publicKey),
      }),
    },
    (value) => parseWithSchema(AccountTokenResponseSchema, value, "Create account response"),
  );

  const credentials = {
    token: response.token,
    secret: encodeBase64(secret, "base64url"),
  } satisfies StoredCredentials;

  return {
    credentials,
    backupKey: formatSecretKeyForBackup(credentials.secret),
  };
}

export async function restoreAccount(
  serverUrl: string,
  secretInput: string,
): Promise<StoredCredentials> {
  const libsodium = await ensureSodium();
  const secret = decodeBase64(normalizeSecretKey(secretInput), "base64url");
  const keypair = libsodium.crypto_sign_seed_keypair(secret);
  const challenge = randomBytes(32);
  const signature = libsodium.crypto_sign_detached(challenge, keypair.privateKey);

  const response = await fetchJson<{ success: boolean; token: string }>(
    `${serverUrl}/v1/auth`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        challenge: encodeBase64(challenge),
        signature: encodeBase64(signature),
        publicKey: encodeBase64(keypair.publicKey),
      }),
    },
    (value) => parseWithSchema(AccountTokenResponseSchema, value, "Restore account response"),
  );

  return {
    token: response.token,
    secret: encodeBase64(secret, "base64url"),
  };
}

export async function createAccountLinkRequest(): Promise<AccountLinkRequest> {
  const libsodium = await ensureSodium();
  const seed = randomBytes(32);
  const keypair = libsodium.crypto_box_seed_keypair(seed);
  return {
    publicKey: keypair.publicKey,
    secretKey: keypair.privateKey,
    linkUrl: `vibe:///account?${encodeBase64(keypair.publicKey, "base64url")}`,
  };
}

export async function beginLoopbackAccountLink(
  serverUrl: string,
  request: AccountLinkRequest,
): Promise<LoopbackAttempt> {
  if (!isDesktopRuntime()) {
    throw new Error("Loopback auth callback requires the Tauri desktop runtime");
  }

  const response = await invoke<unknown>("begin_account_link_callback", {
    serverUrl,
    publicKey: encodeBase64(request.publicKey),
    deepLink: request.linkUrl,
  });
  return parseWithSchema(
    LoopbackAttemptSchema,
    response,
    "Desktop loopback auth start response",
  );
}

export async function getLoopbackAccountLinkStatus(
  attemptId: string,
): Promise<LoopbackAttemptStatus> {
  if (!isDesktopRuntime()) {
    return { status: "not_found" };
  }

  const response = await invoke<unknown>("get_account_link_callback_status", {
    attemptId,
  });
  return parseWithSchema(
    LoopbackAttemptStatusSchema,
    response,
    "Desktop loopback auth status",
  );
}

export async function cancelLoopbackAccountLink(attemptId: string): Promise<void> {
  if (!isDesktopRuntime()) {
    return;
  }

  await invoke("cancel_account_link_callback", { attemptId });
}

export async function openExternalUrl(url: string): Promise<void> {
  if (!isDesktopRuntime()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }

  await invoke("open_external_url", { url });
}

export async function completeAccountLink(
  serverUrl: string,
  request: AccountLinkRequest,
): Promise<StoredCredentials> {
  const response = await fetchJson<
    | { state: "requested" }
    | { state: "authorized"; token: string; response: string }
  >(
    `${serverUrl}/v1/auth/account/request`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        publicKey: encodeBase64(request.publicKey),
      }),
    },
    (value) =>
      parseWithSchema(
        AccountLinkRequestResponseSchema,
        value,
        "Complete account link response",
      ),
  );

  if (response.state !== "authorized") {
    throw new Error("Authorization has not completed yet");
  }

  const decrypted = await decryptBox(decodeBase64(response.response), request.secretKey);
  if (!decrypted) {
    throw new Error("Failed to decrypt linked account secret");
  }

  return {
    token: response.token,
    secret: encodeBase64(decrypted, "base64url"),
  };
}

export async function approveTerminalConnection(
  serverUrl: string,
  credentials: StoredCredentials,
  publicKeyBase64Url: string,
): Promise<void> {
  const publicKey = decodeBase64(publicKeyBase64Url, "base64url");
  const encryption = await createEncryptionContext(credentials.secret);
  const responseV1 = await encryptBox(
    decodeBase64(credentials.secret, "base64url"),
    publicKey,
  );
  const responseV2Bundle = new Uint8Array(encryption.contentPublicKey.length + 1);
  responseV2Bundle[0] = 0;
  responseV2Bundle.set(encryption.contentPublicKey, 1);
  const responseV2 = await encryptBox(responseV2Bundle, publicKey);

  const status = await fetchJson<AuthRequestStatus>(
    `${serverUrl}/v1/auth/request/status?publicKey=${encodeURIComponent(
      encodeBase64(publicKey),
    )}`,
    undefined,
    (value) =>
      parseWithSchema(
        TerminalAuthRequestStatusSchema,
        value,
        "Terminal auth request status",
      ),
  );

  if (status.status === "not_found" || status.status === "authorized") {
    return;
  }

  await fetchJson<void>(
    `${serverUrl}/v1/auth/response`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${credentials.token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        publicKey: encodeBase64(publicKey),
        response: encodeBase64(status.supportsV2 ? responseV2 : responseV1),
      }),
    },
    () => undefined,
  );
}

export class Wave8Client {
  private readonly encryption: EncryptionContext;
  private readonly sessionCipherCache = new Map<string, SessionCipher>();
  private readonly artifactDataKeyCache = new Map<string, Uint8Array>();

  private constructor(
    readonly serverUrl: string,
    readonly credentials: StoredCredentials,
    encryption: EncryptionContext,
  ) {
    this.encryption = encryption;
  }

  static async connect(
    serverUrl: string,
    credentials: StoredCredentials,
  ): Promise<Wave8Client> {
    const encryption = await createEncryptionContext(credentials.secret);
    return new Wave8Client(serverUrl, credentials, encryption);
  }

  async fetchProfile(): Promise<AccountProfile> {
    return fetchJson<AccountProfile>(
      `${this.serverUrl}/v1/account/profile`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value) => {
        const profile = parseWithSchema(
          AccountProfileSchema,
          value,
          "Account profile response",
        );
        return {
          ...profile,
          connectedServices: profile.connectedServices ?? [],
        };
      },
    );
  }

  async fetchAccountSettings(): Promise<{ settings: Settings; version: number }> {
    const response = await fetchJson<{ settings: string | null; settingsVersion: number }>(
      `${this.serverUrl}/v1/account/settings`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
      },
      (value) =>
        parseWithSchema(
          AccountSettingsResponseSchema,
          value,
          "Account settings response",
        ),
    );

    const decrypted = response.settings
      ? await decryptLegacyRaw(response.settings, this.encryption.masterSecret)
      : null;

    return {
      settings: decrypted ? settingsParse(decrypted) : { ...settingsDefaults },
      version: response.settingsVersion,
    };
  }

  async decodeAccountSettingsPayload(encrypted: string | null): Promise<Settings> {
    const decrypted = encrypted
      ? await decryptLegacyRaw(encrypted, this.encryption.masterSecret)
      : null;
    return decrypted ? settingsParse(decrypted) : { ...settingsDefaults };
  }

  async updateAccountSettings(
    currentSettings: Settings,
    currentVersion: number | null,
    patch: Partial<Settings>,
  ): Promise<{ settings: Settings; version: number }> {
    let nextSettings = applySettings(currentSettings, patch);
    let expectedVersion = currentVersion ?? 0;

    for (let attempt = 0; attempt < 2; attempt += 1) {
      const response = await fetchJson<
        | { success: true; version: number }
        | {
            success: false;
            error: "version-mismatch";
            currentVersion: number;
            currentSettings: string | null;
          }
      >(
        `${this.serverUrl}/v1/account/settings`,
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${this.credentials.token}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            settings: await encryptLegacyRaw(nextSettings, this.encryption.masterSecret),
            expectedVersion,
          }),
        },
        (value) =>
          parseWithSchema(
            UpdateAccountSettingsResponseSchema,
            value,
            "Update account settings response",
          ),
      );

      if (response.success) {
        return {
          settings: nextSettings,
          version: response.version,
        };
      }

      const serverSettings = response.currentSettings
        ? settingsParse(
            await decryptLegacyRaw(response.currentSettings, this.encryption.masterSecret),
          )
        : { ...settingsDefaults };
      expectedVersion = response.currentVersion;
      nextSettings = applySettings(serverSettings, patch);
    }

    throw new Error("Failed to update account settings after resolving version conflicts");
  }

  async queryUsage(
    period: UsagePeriod,
    sessionId?: string,
  ): Promise<{
    usage: UsageBucket[];
    groupBy: "hour" | "day";
    totalReports: number;
  }> {
    const params = buildUsageQueryParams(period, sessionId);
    return fetchJson<{
      usage: UsageBucket[];
      groupBy: "hour" | "day";
      totalReports: number;
    }>(
      `${this.serverUrl}/v1/usage/query`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify(params),
      },
      (value) => {
        const parsed = parseWithSchema(
          UsageQueryResponseSchema,
          value,
          "Usage query response",
        );
        return {
          usage: parsed.usage.map((bucket) => ({
            timestamp: bucket.timestamp,
            tokens: bucket.tokens ?? {},
            cost: bucket.cost ?? {},
            reportCount: bucket.reportCount,
          })),
          groupBy: parsed.groupBy,
          totalReports: parsed.totalReports,
        };
      },
    );
  }

  async listSessions(): Promise<DesktopSession[]> {
    const response = await fetchJson<{ sessions: RemoteSessionRecord[] }>(
      `${this.serverUrl}/v1/sessions`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value) => parseWithSchema(SessionsResponseSchema, value, "List sessions response"),
    );

    const sessions = await Promise.all(
      response.sessions.map(async (session) => {
        const cipher = await this.getSessionCipher(session.id, session.dataEncryptionKey);
        const metadata = await cipher.decryptMetadata(session.metadata);
        const agentState = await cipher.decryptAgentState(session.agentState);
        return {
          ...session,
          metadata,
          agentState,
        } satisfies DesktopSession;
      }),
    );

    return sessions.sort((left, right) => right.updatedAt - left.updatedAt);
  }

  async listArtifacts(): Promise<DesktopArtifact[]> {
    const response = await fetchJson<RemoteArtifactRecord[]>(
      `${this.serverUrl}/v1/artifacts`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
      },
      (value) => parseWithSchema(ArtifactsResponseSchema, value, "List artifacts response"),
    );

    const artifacts = await Promise.all(
      response.map(async (artifact) => this.decodeArtifactRecord(artifact)),
    );

    return artifacts.sort((left, right) => right.updatedAt - left.updatedAt);
  }

  async fetchArtifact(artifactId: string): Promise<DesktopArtifact | null> {
    const artifact = await this.fetchRemoteArtifactRecord(artifactId);
    return this.decodeArtifactRecord(artifact);
  }

  async fetchUserProfile(userId: string): Promise<UserProfile | null> {
    return fetchJson<UserProfile | null>(
      `${this.serverUrl}/v1/user/${userId}`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value) => {
        if (value === null) {
          return null;
        }
        const payload = parseWithSchema(
          UserResponseSchema,
          value,
          `User detail response for ${userId}`,
        );
        return payload.user;
      },
    );
  }

  async listFriends(): Promise<UserProfile[]> {
    return fetchJson<UserProfile[]>(
      `${this.serverUrl}/v1/friends`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value) => parseWithSchema(FriendsListResponseSchema, value, "Friends list response").friends,
    );
  }

  async listFeed(): Promise<FeedPostResponse[]> {
    return fetchJson<FeedPostResponse[]>(
      `${this.serverUrl}/v1/feed`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value) => parseWithSchema(FeedListResponseSchema, value, "Feed list response").items,
    );
  }

  async searchUsers(query: string): Promise<UserProfile[]> {
    return fetchJson<UserProfile[]>(
      `${this.serverUrl}/v1/user/search?${new URLSearchParams({ query })}`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      (value): UserProfile[] =>
        parseWithSchema(
          z.object({ users: z.array(UserProfileSchema) }),
          value,
          "User search response",
        ).users,
    );
  }

  async addFriend(userId: string): Promise<UserProfile | null> {
    return fetchJson<UserProfile | null>(
      `${this.serverUrl}/v1/friends/add`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ uid: userId }),
      },
      (value): UserProfile | null => {
        const payload = parseWithSchema(
          UserResponseSchema.extend({ user: UserProfileSchema.nullable() }),
          value,
          "Add friend response",
        );
        return payload.user;
      },
    );
  }

  async removeFriend(userId: string): Promise<UserProfile | null> {
    return fetchJson<UserProfile | null>(
      `${this.serverUrl}/v1/friends/remove`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ uid: userId }),
      },
      (value): UserProfile | null => {
        const payload = parseWithSchema(
          UserResponseSchema.extend({ user: UserProfileSchema.nullable() }),
          value,
          "Remove friend response",
        );
        return payload.user;
      },
    );
  }

  async listMachines(): Promise<DesktopMachine[]> {
    const response = await fetchJson<RemoteMachineRecord[]>(
      `${this.serverUrl}/v1/machines`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
      },
      (value) => parseWithSchema(MachinesResponseSchema, value, "List machines response"),
    );

    const machines = await Promise.all(
      response.map(async (machine) => this.decodeMachineRecord(machine)),
    );
    return machines.sort((left, right) => right.activeAt - left.activeAt);
  }

  async fetchMachine(machineId: string): Promise<DesktopMachine | null> {
    const response = await fetchJson<RemoteMachineRecord | null>(
      `${this.serverUrl}/v1/machines/${machineId}`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
      },
      (value) => {
        if (value === null) {
          return null;
        }
        const payload = parseWithSchema(
          MachineDetailResponseSchema,
          value,
          `Machine detail response for ${machineId}`,
        );
        return payload.machine;
      },
    );

    if (!response) {
      return null;
    }
    return this.decodeMachineRecord(response);
  }

  async encryptSessionRpcPayload(
    sessionId: string,
    dataEncryptionKey: string | null,
    payload: unknown,
  ): Promise<string> {
    const cipher = await this.getSessionCipher(sessionId, dataEncryptionKey);
    return cipher.encryptRecord(payload);
  }

  async decryptSessionRpcPayload(
    sessionId: string,
    dataEncryptionKey: string | null,
    payload: string,
  ): Promise<unknown | null> {
    const cipher = await this.getSessionCipher(sessionId, dataEncryptionKey);
    return cipher.decryptRecord(payload);
  }

  async createArtifact(input: {
    title: string | null;
    body: string | null;
    sessions?: string[];
    draft?: boolean;
  }): Promise<DesktopArtifact> {
    const artifactId = crypto.randomUUID();
    const dataKey = randomBytes(32);
    this.artifactDataKeyCache.set(artifactId, dataKey);
    const cipher = artifactCipherFromKey(dataKey);

    const response = await fetchJson<RemoteArtifactRecord>(
      `${this.serverUrl}/v1/artifacts`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          id: artifactId,
          header: await cipher.encryptHeader({
            title: input.title,
            sessions: input.sessions,
            draft: input.draft,
          }),
          body: await cipher.encryptBody({
            body: input.body,
          }),
          dataEncryptionKey: await this.encryptDataEncryptionKey(dataKey),
        }),
      },
      (value) => parseWithSchema(RemoteArtifactRecordSchema, value, "Create artifact response"),
    );

    return {
      id: response.id,
      title: input.title,
      sessions: input.sessions,
      draft: input.draft,
      body: input.body,
      headerVersion: response.headerVersion,
      bodyVersion: response.bodyVersion,
      seq: response.seq,
      createdAt: response.createdAt,
      updatedAt: response.updatedAt,
      isDecrypted: true,
    };
  }

  async updateArtifact(
    currentArtifact: DesktopArtifact,
    input: {
      title: string | null;
      body: string | null;
      sessions?: string[];
      draft?: boolean;
    },
  ): Promise<DesktopArtifact> {
    let remoteArtifact: RemoteArtifactRecord | null = null;
    let dataKey = this.artifactDataKeyCache.get(currentArtifact.id) ?? null;

    if (
      dataKey === null ||
      currentArtifact.headerVersion === undefined ||
      currentArtifact.bodyVersion === undefined
    ) {
      remoteArtifact = await this.fetchRemoteArtifactRecord(currentArtifact.id);
      dataKey = await this.decryptDataEncryptionKey(remoteArtifact.dataEncryptionKey);
      if (!dataKey) {
        throw new Error("Failed to decrypt artifact encryption key");
      }
      this.artifactDataKeyCache.set(currentArtifact.id, dataKey);
    }

    const cipher = artifactCipherFromKey(dataKey);
    const updateRequest: Record<string, unknown> = {};

    if (
      input.title !== currentArtifact.title ||
      JSON.stringify(input.sessions ?? []) !== JSON.stringify(currentArtifact.sessions ?? []) ||
      input.draft !== currentArtifact.draft
    ) {
      updateRequest.header = await cipher.encryptHeader({
        title: input.title,
        sessions: input.sessions,
        draft: input.draft,
      });
      updateRequest.expectedHeaderVersion =
        remoteArtifact?.headerVersion ?? currentArtifact.headerVersion;
    }

    if (input.body !== (currentArtifact.body ?? null)) {
      updateRequest.body = await cipher.encryptBody({
        body: input.body,
      });
      updateRequest.expectedBodyVersion =
        remoteArtifact?.bodyVersion ?? currentArtifact.bodyVersion;
    }

    if (Object.keys(updateRequest).length === 0) {
      return currentArtifact;
    }

    const response = await fetchJson<ArtifactUpdateResponse>(
      `${this.serverUrl}/v1/artifacts/${currentArtifact.id}`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify(updateRequest),
      },
      (value) =>
        parseWithSchema(
          ArtifactUpdateResponseSchema,
          value,
          `Update artifact response for ${currentArtifact.id}`,
        ),
    );

    if (!response.success) {
      throw new Error("Artifact was modified by another client. Please refresh and try again.");
    }

    const nextArtifact = await this.fetchRemoteArtifactRecord(currentArtifact.id);
    return this.decodeArtifactRecord(nextArtifact);
  }

  async deleteArtifact(artifactId: string): Promise<void> {
    await fetchJson<void>(
      `${this.serverUrl}/v1/artifacts/${artifactId}`,
      {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      () => undefined,
    );
    this.artifactDataKeyCache.delete(artifactId);
  }

  async createSession(input: CreateSessionInput): Promise<DesktopSession> {
    const metadata: SessionMetadata = {
      path: input.workspace.trim() || "/root/vibe-remote",
      host: typeof window !== "undefined" ? window.location.hostname || "desktop" : "desktop",
      name: input.title?.trim() || "Wave 8 Desktop Session",
      os: typeof navigator !== "undefined" ? navigator.platform : "desktop",
      currentModelCode: input.model.trim() || "gpt-5.4",
      models: [
        {
          code: input.model.trim() || "gpt-5.4",
          value: input.model.trim() || "gpt-5.4",
          description: "Desktop session model",
        },
      ],
      summary: input.prompt.trim()
        ? {
            text: input.prompt.trim(),
            updatedAt: Date.now(),
          }
        : undefined,
      flavor: "wave8-tauri",
    };
    const cipher = sessionCipherFromKeys(this.encryption.masterSecret, null);
    const encryptedMetadata = await cipher.encryptRecord(metadata);
    const tagSeed =
      input.title?.trim() ||
      input.workspace.trim().split(/[\\/]/).filter(Boolean).at(-1) ||
      "wave8-session";

    const response = await fetchJson<{ session: RemoteSessionRecord }>(
      `${this.serverUrl}/v1/sessions`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          tag: `${slugify(tagSeed)}-${Date.now()}`,
          metadata: encryptedMetadata,
          agentState: null,
          dataEncryptionKey: null,
        }),
      },
      (value) => parseWithSchema(SessionResponseSchema, value, "Create session response"),
    );

    const created = await this.decodeSessionRecord(response.session);
    if (input.prompt.trim()) {
      await this.sendMessage(created.id, input.prompt.trim(), response.session.dataEncryptionKey);
    }
    return created;
  }

  async deleteSession(sessionId: string): Promise<void> {
    await fetchJson<void>(
      `${this.serverUrl}/v1/sessions/${sessionId}`,
      {
        method: "DELETE",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
        },
      },
      () => undefined,
    );
  }

  async listMessages(session: DesktopSession): Promise<SessionMessagesState> {
    const messages: RemoteMessageRecord[] = [];
    let afterSeq = 0;
    let hasMore = true;

    while (hasMore) {
      const response = await fetchJson<{ messages: RemoteMessageRecord[]; hasMore: boolean }>(
        `${this.serverUrl}/v3/sessions/${session.id}/messages?after_seq=${afterSeq}&limit=100`,
        {
          headers: {
            Authorization: `Bearer ${this.credentials.token}`,
          },
        },
        (value) =>
          parseWithSchema(
            SessionMessagesResponseSchema,
            value,
            `List messages response for session ${session.id}`,
          ),
      );

      messages.push(...response.messages);
      hasMore = response.hasMore;
      if (response.messages.length > 0) {
        afterSeq = Math.max(...response.messages.map((message) => message.seq));
      } else {
        hasMore = false;
      }
    }

    const cipher = await this.getSessionCipher(session.id, session.dataEncryptionKey);
    const uiMessages: UiMessage[] = [];

    for (const message of messages) {
      const decrypted = await cipher.decryptRecord(message.content.c);
      uiMessages.push(
        ...parseRawRecordToUiMessages(
          message.id,
          message.localId ?? null,
          message.createdAt,
          decrypted,
        ),
      );
    }

    return {
      items: uiMessages.sort((left, right) => left.createdAt - right.createdAt),
      loadedAt: Date.now(),
      lastSeq:
        messages.length > 0
          ? messages.reduce(
              (maxSeq, message) => Math.max(maxSeq, message.seq),
              messages[0].seq,
            )
          : null,
    };
  }

  async decodeMessage(
    sessionId: string,
    dataEncryptionKey: string | null,
    message: RemoteMessageRecord,
  ): Promise<UiMessage[]> {
    const cipher = await this.getSessionCipher(sessionId, dataEncryptionKey);
    const decrypted = await cipher.decryptRecord(message.content.c);
    return parseRawRecordToUiMessages(
      message.id,
      message.localId ?? null,
      message.createdAt,
      decrypted,
    );
  }

  async applySessionUpdate(
    session: DesktopSession,
    update: {
      metadata?: SessionMetadataUpdate | null;
      agentState?: SessionAgentStateUpdate | null;
      seq?: number;
      updatedAt?: number;
    },
  ): Promise<DesktopSession> {
    const cipher = await this.getSessionCipher(session.id, session.dataEncryptionKey);
    const nextMetadata = update.metadata
      ? await cipher.decryptMetadata(update.metadata.value)
      : session.metadata;
    const nextAgentState = update.agentState
      ? await cipher.decryptAgentState(update.agentState.value)
      : session.agentState;

    return {
      ...session,
      metadata: nextMetadata,
      metadataVersion: update.metadata?.version ?? session.metadataVersion,
      agentState: nextAgentState,
      agentStateVersion: update.agentState?.version ?? session.agentStateVersion,
      seq: update.seq ?? session.seq,
      updatedAt: update.updatedAt ?? session.updatedAt,
    };
  }

  async sendMessage(
    sessionId: string,
    text: string,
    dataEncryptionKey?: string | null,
    options?: SendMessageOptions,
  ): Promise<void> {
    const cipher = await this.getSessionCipher(sessionId, dataEncryptionKey ?? null);
    const content = buildUserMessageRecord(text, options);
    const encryptedContent = await cipher.encryptRecord(content);

    await fetchJson<void>(
      `${this.serverUrl}/v3/sessions/${sessionId}/messages`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          messages: [
            {
              localId: crypto.randomUUID(),
              content: encryptedContent,
            },
          ],
        }),
      },
      () => undefined,
    );
  }

  private async decodeSessionRecord(session: RemoteSessionRecord): Promise<DesktopSession> {
    const cipher = await this.getSessionCipher(session.id, session.dataEncryptionKey);
    const metadata = await cipher.decryptMetadata(session.metadata);
    const agentState = await cipher.decryptAgentState(session.agentState);
    return {
      ...session,
      metadata,
      agentState,
    } satisfies DesktopSession;
  }

  private async fetchRemoteArtifactRecord(artifactId: string): Promise<RemoteArtifactRecord> {
    return fetchJson<RemoteArtifactRecord>(
      `${this.serverUrl}/v1/artifacts/${artifactId}`,
      {
        headers: {
          Authorization: `Bearer ${this.credentials.token}`,
          "Content-Type": "application/json",
        },
      },
      (value) =>
        parseWithSchema(
          RemoteArtifactRecordSchema,
          value,
          `Artifact detail response for ${artifactId}`,
        ),
    );
  }

  private async decodeArtifactRecord(artifact: RemoteArtifactRecord): Promise<DesktopArtifact> {
    const dataKey = await this.decryptDataEncryptionKey(artifact.dataEncryptionKey);
    if (!dataKey) {
      return {
        id: artifact.id,
        title: null,
        body: artifact.body ? null : undefined,
        headerVersion: artifact.headerVersion,
        bodyVersion: artifact.bodyVersion,
        seq: artifact.seq,
        createdAt: artifact.createdAt,
        updatedAt: artifact.updatedAt,
        isDecrypted: false,
      };
    }

    this.artifactDataKeyCache.set(artifact.id, dataKey);
    const cipher = artifactCipherFromKey(dataKey);
    const header = await cipher.decryptHeader(artifact.header);
    const body = artifact.body ? await cipher.decryptBody(artifact.body) : null;

    return {
      id: artifact.id,
      title: header?.title ?? null,
      sessions: header?.sessions,
      draft: header?.draft,
      body: artifact.body ? body?.body ?? null : undefined,
      headerVersion: artifact.headerVersion,
      bodyVersion: artifact.bodyVersion,
      seq: artifact.seq,
      createdAt: artifact.createdAt,
      updatedAt: artifact.updatedAt,
      isDecrypted: !!header,
    };
  }

  private async decodeMachineRecord(machine: RemoteMachineRecord): Promise<DesktopMachine> {
    const dataKey = machine.dataEncryptionKey
      ? await this.decryptDataEncryptionKey(machine.dataEncryptionKey)
      : null;
    const cipher = sessionCipherFromKeys(this.encryption.masterSecret, dataKey);
    const metadata = await cipher.decryptRecord(machine.metadata);
    const daemonState = machine.daemonState
      ? await cipher.decryptRecord(machine.daemonState)
      : null;

    return {
      id: machine.id,
      seq: machine.seq,
      createdAt: machine.createdAt,
      updatedAt: machine.updatedAt,
      active: machine.active,
      activeAt: machine.activeAt,
      metadata:
        metadata === null
          ? null
          : parseWithSchema(
              MachineMetadataSchema,
              metadata,
              `Decrypted machine metadata for ${machine.id}`,
            ),
      metadataVersion: machine.metadataVersion,
      daemonState:
        daemonState === null
          ? null
          : parseWithSchema(
              MachineDaemonStateSchema,
              daemonState,
              `Decrypted machine daemon state for ${machine.id}`,
            ),
      daemonStateVersion: machine.daemonStateVersion,
    };
  }

  private async getSessionCipher(
    sessionId: string,
    dataEncryptionKey: string | null,
  ): Promise<SessionCipher> {
    const cached = this.sessionCipherCache.get(sessionId);
    if (cached) {
      return cached;
    }

    let decryptedKey: Uint8Array | null = null;
    if (dataEncryptionKey) {
      decryptedKey = await this.decryptDataEncryptionKey(dataEncryptionKey);
    }

    const cipher = sessionCipherFromKeys(this.encryption.masterSecret, decryptedKey);
    this.sessionCipherCache.set(sessionId, cipher);
    return cipher;
  }

  private async decryptDataEncryptionKey(value: string): Promise<Uint8Array | null> {
    const encryptedKey = decodeBase64(value);
    if (encryptedKey[0] !== 0) {
      return null;
    }
    return decryptBox(encryptedKey.slice(1), this.encryption.contentPrivateKey);
  }

  private async encryptDataEncryptionKey(value: Uint8Array): Promise<string> {
    const encrypted = await encryptBox(value, this.encryption.contentPublicKey);
    const wrapped = new Uint8Array(encrypted.length + 1);
    wrapped[0] = 0;
    wrapped.set(encrypted, 1);
    return encodeBase64(wrapped, "base64");
  }
}

function slugify(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48) || "wave8-session";
}
