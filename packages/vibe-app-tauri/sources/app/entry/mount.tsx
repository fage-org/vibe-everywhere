import React from "react";
import ReactDOM from "react-dom/client";
import { AppRoot } from "../AppRoot";
import { RuntimeBootstrapProvider } from "../providers/RuntimeBootstrapProvider";
import {
  resolveBootstrapProfile,
  type BootstrapProfile,
  type RuntimeTarget,
} from "../../shared/bootstrap-config";
import "../theme.css";
// Import i18n initialization
import "../../../src/i18n";

type MountApplicationOptions = {
  mode?: string;
  runtimeTarget?: RuntimeTarget;
};

function profileForOptions(options?: MountApplicationOptions): BootstrapProfile {
  const mode = options?.mode ?? import.meta.env.MODE;
  const profile = resolveBootstrapProfile(mode);
  if (!options?.runtimeTarget || options.runtimeTarget === profile.runtimeTarget) {
    return profile;
  }

  return {
    ...profile,
    runtimeTarget: options.runtimeTarget,
    surfaceKey:
      options.runtimeTarget === "browser"
        ? "browser"
        : options.runtimeTarget === "mobile"
          ? "mobileAndroid"
          : "desktop",
  };
}

export function mountApplication(options?: MountApplicationOptions) {
  const container = document.getElementById("root");
  if (!container) {
    throw new Error("Missing #root mount node for vibe-app-tauri bootstrap.");
  }

  const profile = profileForOptions(options);
  ReactDOM.createRoot(container).render(
    <React.StrictMode>
      <RuntimeBootstrapProvider profile={profile}>
        <AppRoot />
      </RuntimeBootstrapProvider>
    </React.StrictMode>,
  );
}
