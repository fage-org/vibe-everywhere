import {
  createContext,
  useContext,
  useEffect,
  type PropsWithChildren,
} from "react";
import type { BootstrapProfile } from "../../shared/bootstrap-config";

export type RuntimeBootstrapProviderProps = PropsWithChildren<{
  profile: BootstrapProfile;
}>;

const RuntimeBootstrapContext = createContext<BootstrapProfile | null>(null);

export function useRuntimeBootstrapProfile(): BootstrapProfile | null {
  return useContext(RuntimeBootstrapContext);
}

export function RuntimeBootstrapProvider({
  children,
  profile,
}: RuntimeBootstrapProviderProps) {
  useEffect(() => {
    document.documentElement.dataset.appEnv = profile.appEnv;
    document.documentElement.dataset.runtimeTarget = profile.runtimeTarget;

    const splashScreen = document.getElementById("bootstrap-splash");
    splashScreen?.setAttribute("data-state", "ready");

    return () => {
      delete document.documentElement.dataset.appEnv;
      delete document.documentElement.dataset.runtimeTarget;
      splashScreen?.setAttribute("data-state", "idle");
    };
  }, [profile.appEnv, profile.runtimeTarget]);

  return (
    <RuntimeBootstrapContext.Provider value={profile}>
      {children}
    </RuntimeBootstrapContext.Provider>
  );
}
