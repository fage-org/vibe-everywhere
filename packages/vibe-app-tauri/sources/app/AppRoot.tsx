import { AppRuntimeRoot } from "../../src/AppRuntimeRoot";
import { FeatureFlagPanel } from "../../src/feature-flags";

export function AppRoot() {
  return (
    <>
      <AppRuntimeRoot />
      {import.meta.env.DEV && <FeatureFlagPanel />}
    </>
  );
}
