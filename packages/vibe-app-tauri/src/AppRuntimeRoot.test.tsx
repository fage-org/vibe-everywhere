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

vi.mock("./AppV2", () => ({
  AppV2: () => <div data-shell="app-v2">app-v2</div>,
}));

import { AppRuntimeRoot } from "./AppRuntimeRoot";

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

  it("always renders AppV2 for home route", async () => {
    rootMocks.resolved = {
      definition: {
        key: "home",
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

  it("renders AppV2 for settings routes", async () => {
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

  it("renders AppV2 for restore routes (now migrated to AppV2)", async () => {
    rootMocks.resolved = {
      definition: {
        key: "restore-index",
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

  it("renders AppV2 for session deep-link routes (now migrated to AppV2)", async () => {
    rootMocks.resolved = {
      definition: {
        key: "session-files",
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
