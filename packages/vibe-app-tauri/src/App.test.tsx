import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DesktopShell } from "./App";
import { DEFAULT_PATH } from "./router";
import { RuntimeBootstrapProvider } from "../sources/app/providers/RuntimeBootstrapProvider";

function renderWithRuntimeTarget(
  runtimeTarget: "desktop" | "mobile" | "browser",
  path: string,
) {
  const surfaceKey =
    runtimeTarget === "mobile"
      ? "mobileAndroid"
      : runtimeTarget === "browser"
        ? "browser"
        : "desktop";
  return renderToStaticMarkup(
    <RuntimeBootstrapProvider
      profile={{
        appEnv: "development",
        devHost: runtimeTarget === "mobile" ? "0.0.0.0" : "127.0.0.1",
        devPort: 1420,
        mode: `test-${runtimeTarget}`,
        outDir: `dist/${runtimeTarget}`,
        runtimeTarget,
        surfaceKey,
      }}
    >
      <DesktopShell
        path={path}
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
        runtimeTarget={runtimeTarget}
        hostMode={runtimeTarget === "mobile" ? "mobile" : "desktop"}
      />
    </RuntimeBootstrapProvider>,
  );
}

describe("DesktopShell", () => {
  it("renders the desktop entry surface on the home route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path={DEFAULT_PATH}
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Desktop entry");
    expect(html).toContain("Create or restore a Vibe desktop account");
    expect(html).toContain("Open Palette");
  });

  it("renders the signed-out fallback for protected session routes without credentials", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/session/demo-session"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Sign in required");
    expect(html).toContain("Restore or link account");
  });

  it("renders the stronger settings hub structure on the settings route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/index"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Connected Accounts");
    expect(html).toContain("Desktop Configuration");
    expect(html).toContain("About");
  });

  it("renders Android-specific entry copy and hides keyboard-only chrome on the mobile home route", () => {
    const html = renderWithRuntimeTarget("mobile", DEFAULT_PATH);

    expect(html).toContain("mobile-app-shell");
    expect(html).toContain("Android entry");
    expect(html).toContain("Create or restore a Vibe account");
    expect(html).not.toContain("Open Palette");
    expect(html).not.toContain("Keyboard shortcuts");
    expect(html).not.toContain('class="sidebar"');
    expect(html).not.toContain("Create or restore a Vibe desktop account");
  });

  it("renders Android inbox through the mobile shell instead of the desktop review inbox", () => {
    const html = renderWithRuntimeTarget("mobile", "/(app)/inbox/index");

    expect(html).toContain("mobile-app-shell");
    expect(html).toContain("Android entry");
    expect(html).not.toContain('class="sidebar"');
    expect(html).not.toContain("Session inventory is loaded from `/v1/sessions`");
  });

  it("renders Android settings through the mobile shell instead of the desktop settings dashboard", () => {
    const html = renderWithRuntimeTarget("mobile", "/(app)/settings/index");

    expect(html).toContain("mobile-app-shell");
    expect(html).toContain("Android entry");
    expect(html).not.toContain("Desktop Configuration");
    expect(html).not.toContain('class="sidebar"');
  });

  it("renders browser-specific restore copy on the retained browser route", () => {
    const html = renderWithRuntimeTarget("browser", "/(app)/restore/index");

    expect(html).toContain("Happy-aligned browser export shell");
    expect(html).toContain("retained browser route keeps the same create, link, and restore semantics");
    expect(html).toContain("Device link request");
    expect(html).not.toContain("desktop restore entry");
  });

  it("renders browser-specific settings configuration labels", () => {
    const html = renderWithRuntimeTarget("browser", "/(app)/settings/index");

    expect(html).toContain("Browser Configuration");
    expect(html).toContain("browser configuration");
    expect(html).toContain("Browser shell preview build");
    expect(html).not.toContain("Desktop Configuration");
  });

  it("renders the manual restore route with the desktop file-load action", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/restore/manual"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Manual restore");
    expect(html).toContain("Load key file");
  });

  it("hides desktop-only file import affordances on the Android manual restore route", () => {
    const html = renderWithRuntimeTarget("mobile", "/(app)/restore/manual");

    expect(html).toContain("Manual restore");
    expect(html).toContain("Restore with your secret key");
    expect(html).toContain("Restore account");
    expect(html).not.toContain("Load key file");
  });

  it("renders Android restore as a QR-first device-link flow", () => {
    const html = renderWithRuntimeTarget("mobile", "/(app)/restore/index");

    expect(html).toContain("Link this Android device");
    expect(html).toContain("Preparing QR code");
    expect(html).toContain("Restore with secret key instead");
    expect(html).not.toContain("Device link request");
  });

  it("renders the retained changelog route instead of the planned placeholder", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/changelog"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Version 6");
    expect(html).toContain("2026-03-19");
  });

  it("renders the retained server configuration route with validated input guidance", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/server"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Server configuration");
    expect(html).toContain("must use HTTPS unless they target localhost");
  });

  it("renders the text selection utility route instead of a planned surface", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/text-selection"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Text selection utility");
    expect(html).toContain("Selection stats");
    expect(html).toContain("Save selection to file");
  });

  it("renders deferred friends routes as explicit planned surfaces instead of silently dropping them", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/friends/index"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Friends");
    expect(html).toContain("Social surface explicitly deferred from the current promotion gate");
    expect(html).toContain("Planned surface");
  });

  it("renders the artifacts index route with retained artifact actions", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/artifacts/index"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Artifacts Index");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Examples in this slice");
  });

  it("renders the retained user detail route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/user/demo-user"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("User Detail");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Avery Stone");
  });

  it("renders the retained machine detail route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/machine/demo-workstation"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Machine Detail");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Desktop Review Workstation");
  });

  it("renders the retained session info route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/session/demo-ship-review/info"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Session Info");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Examples in this slice");
  });

  it("renders the retained session files route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/session/demo-ship-review/files"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Session Files");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Examples in this slice");
  });

  it("renders the retained session file route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/session/demo-ship-review/file?path=src%2FApp.tsx"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Session File Viewer");
    expect(html).toContain("Sign in required");
    expect(html).not.toContain("Examples in this slice");
  });

  it("renders the desktop-backed appearance settings route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/appearance"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Persisted");
    expect(html).toContain("Reset appearance settings");
    expect(html).toContain("Compact session view");
  });

  it("renders the desktop-backed language settings route", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/language"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Reset language preference");
    expect(html).toContain("Current stored desktop preference");
    expect(html).toContain("English");
  });

  it("renders runtime-specific language preference labels for Android", () => {
    const html = renderWithRuntimeTarget("mobile", "/(app)/settings/language");

    expect(html).toContain("Current stored Android preference");
    expect(html).toContain("Preferred Android language");
    expect(html).not.toContain("Current stored desktop preference");
  });
});
