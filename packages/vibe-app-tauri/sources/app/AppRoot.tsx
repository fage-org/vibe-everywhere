import { App } from "../../src/App";
import { AppV2 } from "../../src/AppV2";
import { isHappyUIEnabled, FeatureFlagPanel } from "../../src/feature-flags";

export function AppRoot() {
  const useHappyUI = isHappyUIEnabled();

  return (
    <>
      {useHappyUI ? <AppV2 /> : <App />}
      {import.meta.env.DEV && <FeatureFlagPanel />}
    </>
  );
}
