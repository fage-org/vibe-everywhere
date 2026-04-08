import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(),
}));

import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  clearStoredCredentials,
  loadStoredCredentials,
  saveStoredCredentials,
} from "./wave8-client";

function createLocalStorageMock() {
  const values = new Map<string, string>();
  return {
    getItem: vi.fn((key: string) => values.get(key) ?? null),
    setItem: vi.fn((key: string, value: string) => values.set(key, value)),
    removeItem: vi.fn((key: string) => values.delete(key)),
  };
}

describe("wave8 secure storage bridge", () => {
  beforeEach(() => {
    const localStorage = createLocalStorageMock();
    vi.stubGlobal("window", { localStorage });
    vi.mocked(isTauri).mockReturnValue(false);
    vi.mocked(invoke).mockReset();
  });

  it("persists credentials to localStorage outside Tauri", async () => {
    await saveStoredCredentials({ token: "token-a", secret: "secret-a" });
    const stored = await loadStoredCredentials();

    expect(stored).toEqual({ token: "token-a", secret: "secret-a" });
  });

  it("persists credentials through invoke inside Tauri", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "secure_store_get_credentials") {
        return JSON.stringify({ token: "token-b", secret: "secret-b" });
      }
      return null;
    });

    await saveStoredCredentials({ token: "token-b", secret: "secret-b" });
    const stored = await loadStoredCredentials();
    await clearStoredCredentials();

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("secure_store_set_credentials", {
      value: JSON.stringify({ token: "token-b", secret: "secret-b" }),
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("secure_store_get_credentials");
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("secure_store_clear_credentials");
    expect(stored).toEqual({ token: "token-b", secret: "secret-b" });
  });

  it("drops malformed stored credentials instead of trusting unchecked JSON", async () => {
    const localStorage = createLocalStorageMock();
    localStorage.getItem.mockReturnValueOnce("{\"token\":123}");
    vi.stubGlobal("window", { localStorage });

    await expect(loadStoredCredentials()).resolves.toBeNull();
  });
});
