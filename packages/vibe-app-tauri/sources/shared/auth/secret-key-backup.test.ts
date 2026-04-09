import { describe, expect, it } from "vitest";
import { encodeBase64, decodeBase64 } from "../encryption/base64";
import {
  formatSecretKeyForBackup,
  isValidSecretKey,
  normalizeSecretKey,
  parseBackupSecretKey,
} from "./secret-key-backup";

function createSecret(seed: number): string {
  const bytes = new Uint8Array(32);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = (seed + index * 17) % 256;
  }
  return encodeBase64(bytes, "base64url");
}

describe("shared secret key backup helpers", () => {
  it("formats and parses backup keys without losing entropy", () => {
    const secret = createSecret(13);
    const formatted = formatSecretKeyForBackup(secret);

    expect(formatted).toMatch(/^[A-Z2-7]{5}(-[A-Z2-7]{1,5})+$/);
    expect(parseBackupSecretKey(formatted)).toBe(secret);
  });

  it("normalizes formatted keys with user input noise", () => {
    const secret = createSecret(29);
    const formatted = formatSecretKeyForBackup(secret).toLowerCase().replace(/-/g, " - ");

    expect(normalizeSecretKey(formatted)).toBe(secret);
  });

  it("validates only 32-byte secrets", () => {
    const validSecret = createSecret(7);
    const shortSecret = encodeBase64(new Uint8Array(16), "base64url");

    expect(isValidSecretKey(validSecret)).toBe(true);
    expect(isValidSecretKey(formatSecretKeyForBackup(validSecret))).toBe(true);
    expect(isValidSecretKey(shortSecret)).toBe(false);
    expect(() => normalizeSecretKey("not-a-secret")).toThrow();
  });

  it("round-trips decoded bytes exactly", () => {
    const secret = createSecret(91);
    const normalized = normalizeSecretKey(formatSecretKeyForBackup(secret));

    expect(decodeBase64(normalized, "base64url")).toEqual(
      decodeBase64(secret, "base64url"),
    );
  });
});
