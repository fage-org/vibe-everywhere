import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const packageRoot = path.resolve(import.meta.dirname, "..");

const ownedBootstrapFiles = [
  "index.html",
  "src/main.tsx",
  "sources/app/entry/browser.tsx",
  "sources/app/entry/desktop.tsx",
  "sources/app/entry/mobile.tsx",
  "sources/app/entry/mount.tsx",
  "sources/app/providers/RuntimeBootstrapProvider.tsx",
  "sources/app/theme.css",
] as const;

describe("B19 bootstrap ownership", () => {
  it("keeps bootstrap files package-local", () => {
    for (const relativePath of ownedBootstrapFiles) {
      const fileContent = fs.readFileSync(
        path.resolve(packageRoot, relativePath),
        "utf8",
      );
      expect(fileContent.includes("../../vibe-app")).toBe(false);
      expect(fileContent.includes("../vibe-app")).toBe(false);
    }
  });

  it("points the package root html entry at the normalized app bootstrap", () => {
    const html = fs.readFileSync(path.resolve(packageRoot, "index.html"), "utf8");
    expect(html).toContain('/sources/app/entry/browser.tsx');
    expect(html).toContain('id="bootstrap-splash"');
  });

  it("keeps android project ownership at the package root with a tauri-compatible bridge", () => {
    const androidRoot = path.resolve(packageRoot, "android");
    const tauriAndroidBridge = path.resolve(packageRoot, "src-tauri/gen/android");
    const androidBuildGradle = fs.readFileSync(
      path.join(androidRoot, "app/build.gradle.kts"),
      "utf8",
    );
    const androidBuildTask = fs.readFileSync(
      path.join(
        androidRoot,
        "buildSrc/src/main/java/engineering/vibe/app/next/kotlin/BuildTask.kt",
      ),
      "utf8",
    );

    expect(fs.existsSync(androidRoot)).toBe(true);
    expect(fs.existsSync(path.join(androidRoot, "app/build.gradle.kts"))).toBe(true);
    expect(fs.existsSync(tauriAndroidBridge)).toBe(true);
    expect(androidBuildGradle).toContain('rootDirRel = "../.."');
    expect(androidBuildTask).toContain('./node_modules/.bin/tauri');
  });
});
