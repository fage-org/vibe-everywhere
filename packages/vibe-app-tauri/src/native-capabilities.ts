import type { RuntimeTarget } from "../sources/shared/bootstrap-config";

export type NativeCapabilityDecision = "implemented" | "deferred" | "waived";

export type NativeCapability = {
  available: boolean;
  decision: NativeCapabilityDecision;
  summary: string;
};

export type RuntimeNativeCapabilities = {
  runtimeTarget: RuntimeTarget;
  notificationRouting: NativeCapability;
  pushRegistration: NativeCapability;
  purchases: NativeCapability;
  cameraQrScan: NativeCapability;
  voiceCapture: NativeCapability;
  fileImport: NativeCapability;
  fileExport: NativeCapability;
  shareSheet: NativeCapability;
  clipboard: NativeCapability;
};

export function resolveRuntimeNativeCapabilities(
  runtimeTarget: RuntimeTarget,
): RuntimeNativeCapabilities {
  if (runtimeTarget === "desktop") {
    return {
      runtimeTarget,
      notificationRouting: {
        available: true,
        decision: "implemented",
        summary:
          "Desktop local notifications are available for informational handoffs; route restoration remains owned outside the notification channel.",
      },
      pushRegistration: {
        available: false,
        decision: "waived",
        summary: "Push registration is not part of the desktop promotion scope.",
      },
      purchases: {
        available: false,
        decision: "waived",
        summary: "Desktop purchases and entitlement refresh are not part of the current replacement slice.",
      },
      cameraQrScan: {
        available: false,
        decision: "waived",
        summary: "Desktop camera and QR scanning are not required for the current promotion gate.",
      },
      voiceCapture: {
        available: false,
        decision: "deferred",
        summary:
          "Voice and microphone capture stay deferred while settings continuity remains available.",
      },
      fileImport: {
        available: true,
        decision: "implemented",
        summary: "Desktop file import uses native file-open dialogs where restore and export flows require it.",
      },
      fileExport: {
        available: true,
        decision: "implemented",
        summary: "Desktop file export uses native save dialogs for artifacts and utility routes.",
      },
      shareSheet: {
        available: false,
        decision: "waived",
        summary: "Desktop share-sheet integration is not required for Wave 9 promotion.",
      },
      clipboard: {
        available: true,
        decision: "implemented",
        summary: "Clipboard copy flows are implemented for desktop parity-critical routes.",
      },
    };
  }

  if (runtimeTarget === "browser") {
    return {
      runtimeTarget,
      notificationRouting: {
        available: true,
        decision: "implemented",
        summary:
          "Browser notifications are best-effort only and do not act as a route restoration contract.",
      },
      pushRegistration: {
        available: false,
        decision: "waived",
        summary: "Browser push registration is out of scope for the retained Wave 9 export.",
      },
      purchases: {
        available: false,
        decision: "waived",
        summary: "Browser purchase continuity is deferred until a route requires it.",
      },
      cameraQrScan: {
        available: false,
        decision: "waived",
        summary: "Browser QR scanning is not required for the retained export path.",
      },
      voiceCapture: {
        available: false,
        decision: "deferred",
        summary: "Browser voice capture remains deferred while language/settings continuity stays available.",
      },
      fileImport: {
        available: true,
        decision: "implemented",
        summary: "Browser file import falls back to DOM file selection for restore flows.",
      },
      fileExport: {
        available: true,
        decision: "implemented",
        summary: "Browser file export uses download handoff instead of desktop save dialogs.",
      },
      shareSheet: {
        available: false,
        decision: "waived",
        summary: "Browser share-sheet integration is not part of the retained export gate.",
      },
      clipboard: {
        available: true,
        decision: "implemented",
        summary: "Browser clipboard copy remains available for export and auth helpers.",
      },
    };
  }

  return {
    runtimeTarget,
    notificationRouting: {
      available: false,
      decision: "deferred",
      summary:
        "Android notification routing and permission-backed route restoration remain deferred until native delivery is implemented and validated on hardware.",
    },
    pushRegistration: {
      available: false,
      decision: "deferred",
      summary:
        "Android push registration is explicitly deferred; the current Wave 9 candidate does not register device tokens.",
    },
    purchases: {
      available: false,
      decision: "deferred",
      summary:
        "Android purchases and entitlement refresh are deferred until the replacement owns a purchase-gated route.",
    },
    cameraQrScan: {
      available: false,
      decision: "deferred",
      summary:
        "Android QR scanning and camera permissions are deferred; current device-link flows rely on QR display plus fallback link/paste paths instead.",
    },
    voiceCapture: {
      available: false,
      decision: "deferred",
      summary:
        "Android live voice capture and microphone permissions are deferred while preference continuity remains available.",
    },
    fileImport: {
      available: false,
      decision: "deferred",
      summary:
        "Android file import is deferred; manual restore currently relies on paste-based backup key entry instead of native file pickers.",
    },
    fileExport: {
      available: false,
      decision: "deferred",
      summary:
        "Android file export is deferred; desktop/browser remain the supported export owners for Wave 9.",
    },
    shareSheet: {
      available: false,
      decision: "deferred",
      summary: "Android share-sheet integration is deferred until a route requires native handoff.",
    },
    clipboard: {
      available: true,
      decision: "implemented",
      summary: "Clipboard copy remains available for Android fallback links and parity helpers.",
    },
  };
}
