import { describe, expect, it } from "vitest";
import {
  normalizeTerminalPublicKeyInput,
  readTerminalConnectKey,
} from "./terminal-connect";

describe("terminal connect helpers", () => {
  it("accepts a plain terminal public key", () => {
    expect(normalizeTerminalPublicKeyInput("demo-terminal-key")).toBe("demo-terminal-key");
  });

  it("extracts the public key from a terminal deep link", () => {
    expect(normalizeTerminalPublicKeyInput("vibe:///terminal?key=demo-terminal-key")).toBe(
      "demo-terminal-key",
    );
  });

  it("supports naked query-style terminal deep links for compatibility", () => {
    expect(normalizeTerminalPublicKeyInput("vibe:///terminal?demo-terminal-key")).toBe(
      "demo-terminal-key",
    );
  });

  it("prefers the explicit key search param", () => {
    expect(
      readTerminalConnectKey(new URLSearchParams("key=demo-terminal-key&source=desktop")),
    ).toBe("demo-terminal-key");
  });
});
