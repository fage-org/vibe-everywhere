import { useState, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1 } from "../ui/Typography";

export type MobileTab = {
  /** Unique identifier */
  id: string;
  /** Icon component */
  icon: ReactNode;
  /** Active icon component (optional, falls back to icon) */
  activeIcon?: ReactNode;
  /** Tab label */
  label: string;
  /** Badge count */
  badge?: number;
};

export interface MobileShellProps {
  /** Top header content */
  header?: ReactNode;
  /** Main content */
  children: ReactNode;
  /** Bottom tab bar tabs */
  tabs?: MobileTab[];
  /** Currently active tab */
  activeTab?: string;
  /** Callback when tab is selected */
  onTabChange?: (tabId: string) => void;
  /** Safe area insets (for notched devices) */
  safeAreaInsets?: {
    top?: number;
    bottom?: number;
    left?: number;
    right?: number;
  };
}

/**
 * MobileShell - Mobile-specific layout with bottom tab bar
 *
 * Matches Happy's mobile navigation patterns:
 * - Fixed header at top
 * - Scrollable main content
 * - Bottom tab bar for primary navigation
 * - Safe area support for notched devices
 */
export function MobileShell({
  header,
  children,
  tabs,
  activeTab,
  onTabChange,
  safeAreaInsets = {},
}: MobileShellProps) {
  const { top = 0, bottom = 0 } = safeAreaInsets;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        width: "100vw",
        overflow: "hidden",
        backgroundColor: "var(--bg-primary)",
        paddingTop: top,
        paddingBottom: bottom,
      }}
    >
      {/* Header */}
      {header && (
        <header
          style={{
            flexShrink: 0,
            backgroundColor: "var(--bg-primary)",
            borderBottom: "1px solid var(--border-primary)",
          }}
        >
          {header}
        </header>
      )}

      {/* Main Content */}
      <main
        style={{
          flex: 1,
          overflow: "auto",
          WebkitOverflowScrolling: "touch",
        }}
      >
        {children}
      </main>

      {/* Tab Bar */}
      {tabs && tabs.length > 0 && (
        <MobileTabBar tabs={tabs} activeTab={activeTab} onTabChange={onTabChange} />
      )}
    </div>
  );
}

interface MobileTabBarProps {
  tabs: MobileTab[];
  activeTab?: string;
  onTabChange?: (tabId: string) => void;
}

/**
 * MobileTabBar - Bottom tab bar for mobile navigation
 *
 * iOS-style tab bar with:
 * - Icon and label for each tab
 * - Active state highlighting
 * - Badge support
 * - Safe area padding
 */
function MobileTabBar({ tabs, activeTab, onTabChange }: MobileTabBarProps) {
  return (
    <nav
      style={{
        flexShrink: 0,
        display: "flex",
        justifyContent: "space-around",
        alignItems: "center",
        height: `calc(${tokens.spacing[14]} + env(safe-area-inset-bottom, 0px))`,
        paddingBottom: "env(safe-area-inset-bottom, 0px)",
        backgroundColor: "var(--surface-primary)",
        borderTop: "1px solid var(--border-primary)",
      }}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTab;
        const icon = isActive && tab.activeIcon ? tab.activeIcon : tab.icon;

        return (
          <button
            key={tab.id}
            onClick={() => onTabChange?.(tab.id)}
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              gap: tokens.spacing[0.5],
              flex: 1,
              height: "100%",
              padding: `${tokens.spacing[2]} 0`,
              backgroundColor: "transparent",
              border: "none",
              cursor: "pointer",
              color: isActive ? "var(--color-primary)" : "var(--text-tertiary)",
              transition: `color ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
              position: "relative",
            }}
          >
            {/* Icon */}
            <span
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: "24px",
                height: "24px",
              }}
            >
              {icon}
            </span>

            {/* Label */}
            <Caption1>{tab.label}</Caption1>

            {/* Badge */}
            {tab.badge !== undefined && tab.badge > 0 && (
              <span
                style={{
                  position: "absolute",
                  top: tokens.spacing[1.5],
                  right: "calc(50% - 16px)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  minWidth: "18px",
                  height: "18px",
                  padding: "0 5px",
                  fontSize: "11px",
                  fontWeight: tokens.typography.fontWeight.semibold,
                  backgroundColor: "var(--color-danger)",
                  color: "#ffffff",
                  borderRadius: tokens.radii.full,
                  border: "2px solid var(--surface-primary)",
                }}
              >
                {tab.badge > 99 ? "99+" : tab.badge}
              </span>
            )}
          </button>
        );
      })}
    </nav>
  );
}

interface MobileNavBarProps {
  /** Title text */
  title: string;
  /** Leading action (back button, etc.) */
  leading?: ReactNode;
  /** Trailing actions */
  trailing?: ReactNode;
}

/**
 * MobileNavBar - iOS-style navigation bar for mobile
 *
 * Centered title with leading and trailing actions
 */
export function MobileNavBar({ title, leading, trailing }: MobileNavBarProps) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        height: tokens.layout.header.default,
        padding: `0 ${tokens.spacing[4]}`,
      }}
    >
      {/* Leading */}
      <div
        style={{
          flex: 1,
          display: "flex",
          justifyContent: "flex-start",
          minWidth: "44px",
        }}
      >
        {leading}
      </div>

      {/* Title */}
      <div
        style={{
          flex: 2,
          display: "flex",
          justifyContent: "center",
          textAlign: "center",
        }}
      >
        <span
          style={{
            fontSize: tokens.typography.fontSize.lg,
            fontWeight: tokens.typography.fontWeight.semibold,
            color: "var(--text-primary)",
          }}
        >
          {title}
        </span>
      </div>

      {/* Trailing */}
      <div
        style={{
          flex: 1,
          display: "flex",
          justifyContent: "flex-end",
          minWidth: "44px",
        }}
      >
        {trailing}
      </div>
    </div>
  );
}
