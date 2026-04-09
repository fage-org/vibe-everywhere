import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  bootstrapSurfaces,
  resolveBootstrapProfile,
  rootAssetPaths,
  rootProviderOrder,
} from "./bootstrap-config";

const packageRoot = path.resolve(import.meta.dirname, "../..");

describe("bootstrap config", () => {
  it("maps desktop, mobile, and browser modes to explicit profiles", () => {
    expect(resolveBootstrapProfile("desktop-development")).toMatchObject({
      appEnv: "development",
      devHost: "127.0.0.1",
      outDir: "dist/desktop",
      runtimeTarget: "desktop",
      surfaceKey: "desktop",
    });

    expect(resolveBootstrapProfile("mobile-preview")).toMatchObject({
      appEnv: "preview",
      devHost: "0.0.0.0",
      outDir: "dist/mobile",
      runtimeTarget: "mobile",
      surfaceKey: "mobileAndroid",
    });

    expect(resolveBootstrapProfile("browser-production")).toMatchObject({
      appEnv: "production",
      devHost: "127.0.0.1",
      outDir: "dist/browser/production",
      runtimeTarget: "browser",
      surfaceKey: "browser",
    });
  });

  it("resolves package-local output paths", () => {
    expect(path.resolve(packageRoot, resolveBootstrapProfile("browser-preview").outDir)).toBe(
      path.resolve(packageRoot, "dist/browser/preview"),
    );
  });

  it("keeps the three B19 bootstrap surfaces explicit", () => {
    expect(Object.keys(bootstrapSurfaces)).toEqual([
      "desktop",
      "mobileAndroid",
      "browser",
    ]);
    expect(rootProviderOrder).toEqual([
      "StrictMode",
      "RuntimeBootstrapProvider",
      "ThemeBootstrap",
      "SplashScreenBridge",
    ]);
  });

  it("owns bootstrap assets inside the package", () => {
    for (const assetPath of rootAssetPaths) {
      expect(fs.existsSync(path.resolve(packageRoot, assetPath))).toBe(true);
    }
  });
});
