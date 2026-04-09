import { describe, expect, it } from "vitest";
import { resolveRuntimeNativeCapabilities } from "./native-capabilities";

describe("native capability matrix", () => {
  it("keeps desktop file and notification capabilities enabled", () => {
    const capabilities = resolveRuntimeNativeCapabilities("desktop");

    expect(capabilities.fileImport.available).toBe(true);
    expect(capabilities.fileExport.available).toBe(true);
    expect(capabilities.notificationRouting.available).toBe(true);
  });

  it("marks Android promotion blockers as explicitly deferred", () => {
    const capabilities = resolveRuntimeNativeCapabilities("mobile");

    expect(capabilities.pushRegistration.decision).toBe("deferred");
    expect(capabilities.notificationRouting.decision).toBe("deferred");
    expect(capabilities.cameraQrScan.decision).toBe("deferred");
    expect(capabilities.voiceCapture.decision).toBe("deferred");
    expect(capabilities.fileImport.available).toBe(false);
    expect(capabilities.fileExport.available).toBe(false);
  });

  it("keeps browser export on DOM-backed file handoff", () => {
    const capabilities = resolveRuntimeNativeCapabilities("browser");

    expect(capabilities.fileImport.available).toBe(true);
    expect(capabilities.fileExport.available).toBe(true);
    expect(capabilities.clipboard.available).toBe(true);
  });
});
