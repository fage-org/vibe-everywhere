import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolveBootstrapProfile } from "./sources/shared/bootstrap-config";

const configDir = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig(({ mode }) => {
  const bootstrapProfile = resolveBootstrapProfile(mode);

  return {
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
    build: {
      chunkSizeWarningLimit: 900,
      outDir: path.resolve(configDir, bootstrapProfile.outDir),
      rolldownOptions: {
        output: {
          codeSplitting: true,
          manualChunks(id) {
            if (
              id.includes("/node_modules/react-syntax-highlighter/") ||
              id.includes("/node_modules/refractor/")
            ) {
              return "syntax-highlighter";
            }
            if (id.includes("/node_modules/libsodium-wrappers/")) {
              return "libsodium-wrapper";
            }
            if (id.includes("/node_modules/libsodium/")) {
              return "libsodium-core";
            }
            return undefined;
          },
        },
      },
    },
    define: {
      __VIBE_BOOTSTRAP_MODE__: JSON.stringify(mode),
      __VIBE_BOOTSTRAP_TARGET__: JSON.stringify(bootstrapProfile.runtimeTarget),
      __VIBE_BOOTSTRAP_ENV__: JSON.stringify(bootstrapProfile.appEnv),
    },
    server: {
      host: bootstrapProfile.devHost,
      port: bootstrapProfile.devPort,
      strictPort: true,
    },
    preview: {
      host: "127.0.0.1",
      port: 4173,
      strictPort: true,
    },
  };
});
