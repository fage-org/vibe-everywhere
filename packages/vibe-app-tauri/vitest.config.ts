import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

const configDir = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@app": path.resolve(configDir, "sources/app"),
      "@desktop": path.resolve(configDir, "sources/desktop"),
      "@mobile": path.resolve(configDir, "sources/mobile"),
      "@shared": path.resolve(configDir, "sources/shared"),
      libsodium: path.resolve(
        configDir,
        "../../node_modules/libsodium/dist/modules/libsodium.js",
      ),
      "libsodium-wrappers": path.resolve(
        configDir,
        "../../node_modules/libsodium-wrappers/dist/modules/libsodium-wrappers.js",
      ),
    },
  },
  test: {
    globals: false,
    environment: "node",
    include: [
      "src/**/*.{test,spec}.ts",
      "src/**/*.{test,spec}.tsx",
      "sources/**/*.{test,spec}.ts",
      "sources/**/*.{test,spec}.tsx",
    ],
    exclude: [
      "node_modules/**",
      "dist/**",
      "release/**",
      "src-tauri/target/**",
      "tests/e2e/**",
    ],
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html"],
      include: [
        "src/**/*.ts",
        "src/**/*.tsx",
        "sources/**/*.ts",
        "sources/**/*.tsx",
      ],
      exclude: [
        "node_modules/**",
        "dist/**",
        "release/**",
        "src/**/*.d.ts",
        "src/**/*.test.*",
        "sources/**/*.test.*",
        "src/main.tsx",
        "src/i18n/**",
        "src/locales/**",
        "src-tauri/**",
        "tests/**",
        "**/*.config.*",
      ],
      thresholds: {
        lines: 35,
        functions: 30,
        branches: 25,
        statements: 35,
      },
    },
  },
});
