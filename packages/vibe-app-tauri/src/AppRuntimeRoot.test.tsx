import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { afterEach, describe, expect, it, vi } from "vitest";

const rootMocks = vi.hoisted(() => ({
  resolved: {
    definition: {
      key: "home",
    },
  },
}));

vi.mock("./router", async () => {
  const actual = await vi.importActual<typeof import("./router")>("./router");
  return {
    ...actual,
    useDesktopRouter: () => ({
      path: "/(app)/index",
      navigate: vi.fn(),
      resolved: rootMocks.resolved,
    }),
  };
});

vi.mock("./App", () => ({
  App: () => <div data-shell="legacy">legacy</div>,
}));

vi.mock("./AppV2", () => ({
  AppV2: () => <div data-shell="app-v2">app-v2</div>,
}));

import { AppRuntimeRoot, shouldUseAppV2Root } from "./AppRuntimeRoot";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

describe("AppRuntimeRoot", () => {
  let renderer: ReactTestRenderer | null = null;

  afterEach(async () => {
    if (renderer) {
      await act(async () => {
        renderer?.unmount();
      });
    }
    renderer = null;
  });

  it("routes restore and session deep links to the legacy shell", () => {
    expect(
      shouldUseAppV2Root({
        definition: { key: "restore-index" } as never,
      } as never),
    ).toBe(false);
    expect(
      shouldUseAppV2Root({
        definition: { key: "session-file" } as never,
      } as never),
    ).toBe(false);
  });

  it("keeps home and unsupported deferred routes on the AppV2 root", () => {
    expect(
      shouldUseAppV2Root({
        definition: { key: "home" } as never,
      } as never),
    ).toBe(true);
    expect(
      shouldUseAppV2Root({
        definition: { key: "artifacts-index" } as never,
      } as never),
    ).toBe(true);
  });

  it("renders the legacy shell for routes gated out of AppV2", async () => {
    rootMocks.resolved = {
      definition: {
        key: "restore-manual",
      },
    };

    await act(async () => {
      renderer = create(<AppRuntimeRoot />);
    });

    const mountedRenderer = renderer as ReactTestRenderer & {
      root: { findByProps: (props: Record<string, string>) => unknown };
    };
    expect(mountedRenderer.root.findByProps({ "data-shell": "legacy" })).toBeTruthy();
  });

  it("renders AppV2 for routes owned by the new shell", async () => {
    rootMocks.resolved = {
      definition: {
        key: "settings-index",
      },
    };

    await act(async () => {
      renderer = create(<AppRuntimeRoot />);
    });

    const mountedRenderer = renderer as ReactTestRenderer & {
      root: { findByProps: (props: Record<string, string>) => unknown };
    };
    expect(mountedRenderer.root.findByProps({ "data-shell": "app-v2" })).toBeTruthy();
  });
});
