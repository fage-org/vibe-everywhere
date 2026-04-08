import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const configDir = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
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
    rolldownOptions: {
      output: {
        codeSplitting: true,
        manualChunks(id) {
          if (id.includes("/node_modules/react-syntax-highlighter/") || id.includes("/node_modules/refractor/")) {
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
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
    port: 4173,
    strictPort: true,
  },
});
