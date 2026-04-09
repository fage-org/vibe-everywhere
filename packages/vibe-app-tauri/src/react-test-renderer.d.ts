declare module "react-test-renderer" {
  import type { ReactElement } from "react";

  export type ReactTestRenderer = {
    unmount(): void;
  };

  export function create(element: ReactElement): ReactTestRenderer;
  export function act<T>(callback: () => T | Promise<T>): Promise<T>;
}
