import { describe, expect, it } from "vitest";
import {
  DEFAULT_PATH,
  matchRoutePattern,
  normalizeAppPath,
  resolveRoute,
  routeInventoryByClass,
} from "./router";

describe("desktop router", () => {
  it("normalizes empty paths to the default desktop entry", () => {
    expect(normalizeAppPath("")).toBe(DEFAULT_PATH);
    expect(normalizeAppPath("#/(app)/index")).toBe(DEFAULT_PATH);
  });

  it("matches dynamic session routes", () => {
    expect(matchRoutePattern("/(app)/session/[id]", "/(app)/session/demo-ship-review")).toEqual({
      id: "demo-ship-review",
    });
  });

  it("resolves a session detail path to the P0 session route", () => {
    const resolved = resolveRoute("/(app)/session/demo-ship-review");

    expect(resolved.definition.key).toBe("session-detail");
    expect(resolved.definition.promotionClass).toBe("P0");
    expect(resolved.params.id).toBe("demo-ship-review");
  });

  it("prefers static session routes over the dynamic session detail route", () => {
    const resolved = resolveRoute("/(app)/session/recent");

    expect(resolved.definition.key).toBe("session-recent");
    expect(resolved.params).toEqual({});
  });

  it("preserves query parameters for deep-linkable session file routes", () => {
    const resolved = resolveRoute(
      "/(app)/session/demo-ship-review/file?path=src%2Fmain.ts",
    );

    expect(resolved.definition.key).toBe("session-file");
    expect(resolved.params.id).toBe("demo-ship-review");
    expect(resolved.searchParams.get("path")).toBe("src/main.ts");
  });

  it("resolves deep-linkable session message routes", () => {
    const resolved = resolveRoute("/(app)/session/demo-ship-review/message/msg-1");

    expect(resolved.definition.key).toBe("session-message");
    expect(resolved.params.id).toBe("demo-ship-review");
    expect(resolved.params.messageId).toBe("msg-1");
  });

  it("keeps promotion route counts explicit", () => {
    expect(routeInventoryByClass.P0).toHaveLength(8);
    expect(routeInventoryByClass.P1.length).toBeGreaterThan(routeInventoryByClass.P2.length);
  });

  it("marks desktop-backed P1 routes as wired once they gain real state or command flows", () => {
    expect(routeInventoryByClass.P1.every((route) => route.status === "wired")).toBe(true);
    expect(resolveRoute("/(app)/artifacts/index").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/account").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/appearance").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/usage").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/features").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/language").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/voice").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/voice/language").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/settings/connect/claude").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/user/demo-user").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/machine/demo-workstation").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/terminal/index").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/terminal/connect").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/session/demo-ship-review/files").definition.status).toBe("wired");
    expect(resolveRoute("/(app)/session/demo-ship-review/message/msg-1").definition.status).toBe("wired");
  });

  it("keeps deferred social and developer routes explicit as P2 planned surfaces", () => {
    expect(resolveRoute("/(app)/friends/index").definition.promotionClass).toBe("P2");
    expect(resolveRoute("/(app)/friends/index").definition.status).toBe("planned");
    expect(resolveRoute("/(app)/friends/search").definition.status).toBe("planned");
    expect(resolveRoute("/(app)/dev/index").definition.status).toBe("planned");
  });
});
