import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export interface ShellProps {
  /** Sidebar navigation component */
  sidebar?: ReactNode;
  /** Header component */
  header?: ReactNode;
  /** Main content */
  children: ReactNode;
  /** Whether sidebar is collapsed */
  sidebarCollapsed?: boolean;
}

/**
 * Shell - Top-level application layout
 *
 * Matches Happy's MainView structure:
 * - Fixed sidebar on desktop
 * - Scrollable main content area
 * - Optional header above main content
 * - Responsive grid layout
 */
export function Shell({
  sidebar,
  header,
  children,
  sidebarCollapsed = false,
}: ShellProps) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: sidebar
          ? `${sidebarCollapsed ? tokens.layout.sidebar.collapsed : tokens.layout.sidebar.default} 1fr`
          : "1fr",
        gridTemplateRows: "1fr",
        height: "100vh",
        width: "100vw",
        overflow: "hidden",
        backgroundColor: "var(--bg-primary)",
      }}
    >
      {sidebar && (
        <aside
          style={{
            gridColumn: "1",
            gridRow: "1 / -1",
            display: "flex",
            flexDirection: "column",
            backgroundColor: "var(--surface-primary)",
            borderRight: "1px solid var(--border-primary)",
            overflow: "hidden",
          }}
        >
          {sidebar}
        </aside>
      )}

      <main
        style={{
          gridColumn: sidebar ? "2" : "1",
          gridRow: "1 / -1",
          display: "flex",
          flexDirection: "column",
          minWidth: 0,
          overflow: "hidden",
        }}
      >
        {header && (
          <header
            style={{
              flexShrink: 0,
              borderBottom: "1px solid var(--border-primary)",
            }}
          >
            {header}
          </header>
        )}

        <div
          style={{
            flex: 1,
            overflow: "auto",
            minHeight: 0,
          }}
        >
          {children}
        </div>
      </main>
    </div>
  );
}
