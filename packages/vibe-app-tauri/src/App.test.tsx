import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DesktopShell } from "./App";
import { DEFAULT_PATH } from "./router";

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
});
