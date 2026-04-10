import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Body, Caption1 } from "../ui/Typography";

export type NavItemState = "default" | "active" | "disabled";

export interface NavItem {
  /** Unique identifier */
  id: string;
  /** Display label */
  label: string;
  /** Icon component */
  icon?: ReactNode;
  /** Navigation path */
  href?: string;
  /** Click handler */
  onClick?: () => void;
  /** Current state */
  state?: NavItemState;
  /** Badge count or text */
  badge?: string | number;
}

export interface NavSection {
  /** Section title */
  title?: string;
  /** Navigation items */
  items: NavItem[];
}

export interface SidebarProps {
  /** Brand/logo element */
  brand?: ReactNode;
  /** Primary navigation sections */
  primarySections?: NavSection[];
  /** Secondary navigation (bottom) */
  secondarySections?: NavSection[];
  /** Connection status element */
  connectionStatus?: ReactNode;
  /** Whether sidebar is collapsed */
  collapsed?: boolean;
  /** Callback when item is clicked */
  onItemClick?: (item: NavItem) => void;
}

/**
 * Sidebar - Navigation sidebar
 *
 * Matches Happy's SidebarNavigator:
 * - Brand block at top
 * - Primary navigation with sections
 * - Secondary navigation at bottom
 * - Connection status indicator
 * - Active state highlighting
 */
export function Sidebar({
  brand,
  primarySections = [],
  secondarySections = [],
  connectionStatus,
  collapsed = false,
  onItemClick,
}: SidebarProps) {
  const handleItemClick = (item: NavItem) => {
    if (item.state !== "disabled") {
      item.onClick?.();
      onItemClick?.(item);
    }
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        padding: collapsed ? tokens.spacing[2] : tokens.spacing[3],
        gap: tokens.spacing[4],
      }}
    >
      {/* Brand */}
      {brand && (
        <div
          style={{
            flexShrink: 0,
            padding: collapsed ? tokens.spacing[2] : `${tokens.spacing[3]} ${tokens.spacing[2]}`,
          }}
        >
          {brand}
        </div>
      )}

      {/* Primary Navigation */}
      <nav
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[6],
          overflowY: "auto",
          overflowX: "hidden",
        }}
      >
        {primarySections.map((section, sectionIndex) => (
          <NavSectionComponent
            key={section.title || `section-${sectionIndex}`}
            section={section}
            collapsed={collapsed}
            onItemClick={handleItemClick}
          />
        ))}
      </nav>

      {/* Secondary Navigation */}
      {secondarySections.length > 0 && (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[4],
            paddingTop: tokens.spacing[4],
            borderTop: "1px solid var(--border-primary)",
          }}
        >
          {secondarySections.map((section, sectionIndex) => (
            <NavSectionComponent
              key={`secondary-${section.title || sectionIndex}`}
              section={section}
              collapsed={collapsed}
              onItemClick={handleItemClick}
            />
          ))}
        </div>
      )}

      {/* Connection Status */}
      {connectionStatus && (
        <div
          style={{
            flexShrink: 0,
            paddingTop: tokens.spacing[3],
            borderTop: "1px solid var(--border-primary)",
          }}
        >
          {connectionStatus}
        </div>
      )}
    </div>
  );
}

interface NavSectionComponentProps {
  section: NavSection;
  collapsed: boolean;
  onItemClick: (item: NavItem) => void;
}

function NavSectionComponent({ section, collapsed, onItemClick }: NavSectionComponentProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[1],
      }}
    >
      {section.title && !collapsed && (
        <Caption1 color="tertiary" style={{ padding: `0 ${tokens.spacing[3]}` }}>
          {section.title}
        </Caption1>
      )}

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[0.5],
        }}
      >
        {section.items.map((item) => (
          <NavItemComponent
            key={item.id}
            item={item}
            collapsed={collapsed}
            onClick={() => onItemClick(item)}
          />
        ))}
      </div>
    </div>
  );
}

interface NavItemComponentProps {
  item: NavItem;
  collapsed: boolean;
  onClick: () => void;
}

function NavItemComponent({ item, collapsed, onClick }: NavItemComponentProps) {
  const isActive = item.state === "active";
  const isDisabled = item.state === "disabled";

  const baseStyles: React.CSSProperties = {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacing[3],
    padding: collapsed
      ? `${tokens.spacing[3]} ${tokens.spacing[2]}`
      : `${tokens.spacing[2]} ${tokens.spacing[3]}`,
    borderRadius: tokens.radii.md,
    cursor: isDisabled ? "not-allowed" : "pointer",
    opacity: isDisabled ? 0.5 : 1,
    backgroundColor: isActive ? "var(--surface-secondary)" : "transparent",
    color: isActive ? "var(--text-primary)" : "var(--text-secondary)",
    transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
    justifyContent: collapsed ? "center" : "flex-start",
  };

  const hoverStyles: React.CSSProperties =
    !isActive && !isDisabled
      ? {
          backgroundColor: "var(--surface-hover)",
          color: "var(--text-primary)",
        }
      : {};

  return (
    <button
      onClick={onClick}
      disabled={isDisabled}
      style={baseStyles}
      onMouseEnter={(e) => {
        if (!isActive && !isDisabled) {
          Object.assign(e.currentTarget.style, hoverStyles);
        }
      }}
      onMouseLeave={(e) => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = "transparent";
          e.currentTarget.style.color = "var(--text-secondary)";
        }
      }}
    >
      {item.icon && (
        <span
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            width: "20px",
            height: "20px",
            flexShrink: 0,
          }}
        >
          {item.icon}
        </span>
      )}

      {!collapsed && (
        <>
          <Body
            color={isActive ? "primary" : "secondary"}
            truncate
            style={{ flex: 1, textAlign: "left" }}
          >
            {item.label}
          </Body>

          {item.badge !== undefined && (
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                justifyContent: "center",
                minWidth: "18px",
                height: "18px",
                padding: "0 6px",
                fontSize: tokens.typography.fontSize.xs,
                fontWeight: tokens.typography.fontWeight.medium,
                backgroundColor: isActive ? "var(--color-primary)" : "var(--surface-tertiary)",
                color: isActive ? "#ffffff" : "var(--text-secondary)",
                borderRadius: tokens.radii.full,
              }}
            >
              {item.badge}
            </span>
          )}
        </>
      )}
    </button>
  );
}
