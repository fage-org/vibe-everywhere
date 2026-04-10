/**
 * Feature flag system for gradual migration to Happy-aligned UI
 *
 * This allows toggling between legacy and new UI during development
 * and can be controlled via localStorage, URL params, or build flags.
 */

// Feature flag configuration
interface FeatureFlags {
  /** Enable new Happy-aligned UI */
  enableHappyUI: boolean;
  /** Use new component library */
  useNewComponents: boolean;
  /** Use new theme system */
  useNewTheme: boolean;
  /** Debug mode for development */
  debug: boolean;
}

// Default flags
const defaultFlags: FeatureFlags = {
  enableHappyUI: true,  // Enable new Happy UI by default
  useNewComponents: true,
  useNewTheme: true,
  debug: false,
};

// Storage key with namespace to avoid collisions
const STORAGE_KEY = "vibe:feature-flags:v1";

/**
 * Get current feature flags
 * Priority: URL params > localStorage > defaults
 */
export function getFeatureFlags(): FeatureFlags {
  const flags = { ...defaultFlags };

  // Check localStorage
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      Object.assign(flags, parsed);
    }
  } catch {
    // Ignore storage errors
  }

  // Check URL params (highest priority)
  if (typeof window !== "undefined") {
    const params = new URLSearchParams(window.location.search);

    if (params.has("happy-ui")) {
      flags.enableHappyUI = params.get("happy-ui") === "true";
    }
    if (params.has("new-components")) {
      flags.useNewComponents = params.get("new-components") === "true";
    }
    if (params.has("new-theme")) {
      flags.useNewTheme = params.get("new-theme") === "true";
    }
    if (params.has("debug")) {
      flags.debug = params.get("debug") === "true";
    }
  }

  // Check build-time flag
  if (import.meta.env.VITE_ENABLE_HAPPY_UI === "true") {
    flags.enableHappyUI = true;
  }

  return flags;
}

/**
 * Set feature flags in localStorage
 */
export function setFeatureFlags(flags: Partial<FeatureFlags>): void {
  try {
    const current = getFeatureFlags();
    const updated = { ...current, ...flags };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));

    // Reload to apply changes
    window.location.reload();
  } catch {
    // Ignore storage errors
  }
}

/**
 * Reset feature flags to defaults
 */
export function resetFeatureFlags(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
    window.location.reload();
  } catch {
    // Ignore storage errors
  }
}

/**
 * Check if Happy UI is enabled
 */
export function isHappyUIEnabled(): boolean {
  return getFeatureFlags().enableHappyUI;
}

/**
 * Feature flag hook for React components
 */
export function useFeatureFlags(): FeatureFlags {
  return getFeatureFlags();
}

/**
 * Toggle Happy UI on/off
 */
export function toggleHappyUI(): void {
  const flags = getFeatureFlags();
  setFeatureFlags({ enableHappyUI: !flags.enableHappyUI });
}

/**
 * Enable Happy UI (for development)
 */
export function enableHappyUI(): void {
  setFeatureFlags({
    enableHappyUI: true,
    useNewComponents: true,
    useNewTheme: true,
  });
}

/**
 * Development helper to show feature flag UI
 */
export function FeatureFlagPanel(): JSX.Element {
  const flags = useFeatureFlags();

  return (
    <div
      style={{
        position: "fixed",
        bottom: "20px",
        right: "20px",
        padding: "16px",
        backgroundColor: "var(--surface-primary)",
        border: "1px solid var(--border-primary)",
        borderRadius: "12px",
        boxShadow: "0 4px 16px rgba(0,0,0,0.2)",
        zIndex: 9999,
        fontSize: "14px",
        maxWidth: "300px",
      }}
    >
      <h3 style={{ margin: "0 0 12px 0", fontSize: "16px" }}>Feature Flags</h3>

      <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
        <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <input
            type="checkbox"
            checked={flags.enableHappyUI}
            onChange={(e) => setFeatureFlags({ enableHappyUI: e.target.checked })}
          />
          Enable Happy UI
        </label>

        <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <input
            type="checkbox"
            checked={flags.useNewComponents}
            onChange={(e) => setFeatureFlags({ useNewComponents: e.target.checked })}
          />
          Use New Components
        </label>

        <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <input
            type="checkbox"
            checked={flags.useNewTheme}
            onChange={(e) => setFeatureFlags({ useNewTheme: e.target.checked })}
          />
          Use New Theme
        </label>

        <button
          onClick={resetFeatureFlags}
          style={{
            marginTop: "8px",
            padding: "8px 16px",
            backgroundColor: "var(--surface-secondary)",
            border: "1px solid var(--border-primary)",
            borderRadius: "6px",
            cursor: "pointer",
          }}
        >
          Reset to Defaults
        </button>
      </div>

      <div style={{ marginTop: "12px", fontSize: "12px", color: "var(--text-tertiary)" }}>
        URL params: ?happy-ui=true&new-components=true
      </div>
    </div>
  );
}
