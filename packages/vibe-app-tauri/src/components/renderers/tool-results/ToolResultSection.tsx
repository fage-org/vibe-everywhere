import type { ReactNode } from "react";
import { tokens } from "../../../design-system/tokens";
import { Caption1 } from "../../ui/Typography";

export interface ToolResultSectionProps {
  /** Optional section title */
  title?: string;
  /** Whether to use full width (no horizontal padding) */
  fullWidth?: boolean;
  /** Section content */
  children: ReactNode;
}

/**
 * ToolResultSection - Container for tool result sections
 *
 * Provides consistent padding and optional title for tool result content.
 * Similar to Happy's ToolSectionView component.
 */
export function ToolResultSection({
  title,
  fullWidth = false,
  children,
}: ToolResultSectionProps) {
  return (
    <div
      style={{
        marginBottom: tokens.spacing[3],
        overflow: "visible",
        ...(fullWidth && {
          marginLeft: `-${tokens.spacing[3]}`,
          marginRight: `-${tokens.spacing[3]}`,
        }),
      }}
    >
      {title && (
        <Caption1
          color="tertiary"
          style={{
            textTransform: "uppercase",
            letterSpacing: "0.05em",
            marginBottom: tokens.spacing[2],
            marginLeft: fullWidth ? tokens.spacing[3] : 0,
            marginRight: fullWidth ? tokens.spacing[3] : 0,
          }}
        >
          {title}
        </Caption1>
      )}
      <div style={fullWidth ? { overflow: "visible" } : undefined}>
        {children}
      </div>
    </div>
  );
}
