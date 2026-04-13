import { useMemo, useState } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1, Subheadline } from "../ui/Typography";

export interface PieSlice {
  label: string;
  value: number;
  color: string;
}

export interface UsagePieChartProps {
  /** Pie slices */
  slices: PieSlice[];
  /** Chart size in pixels */
  size?: number;
  /** Show legend */
  showLegend?: boolean;
  /** Center text */
  centerText?: string;
  /** Center value */
  centerValue?: number;
}

/**
 * UsagePieChart - Pie chart for model distribution
 *
 * Features:
 * - CSS conic-gradient implementation
 * - Hover tooltips
 * - Interactive legend
 */
export function UsagePieChart({
  slices,
  size = 200,
  showLegend = true,
  centerText,
  centerValue,
}: UsagePieChartProps) {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  const total = useMemo(() => {
    return slices.reduce((sum, slice) => sum + slice.value, 0);
  }, [slices]);

  // Calculate conic-gradient stops
  const gradientStops = useMemo(() => {
    let currentAngle = 0;
    return slices.map((slice) => {
      const percentage = (slice.value / total) * 100;
      const startAngle = currentAngle;
      const endAngle = currentAngle + percentage * 3.6; // 360deg / 100
      currentAngle = endAngle;
      return {
        color: slice.color,
        start: startAngle,
        end: endAngle,
        percentage,
      };
    });
  }, [slices, total]);

  // Build gradient string
  const gradientString = useMemo(() => {
    return gradientStops
      .map((stop) => `${stop.color} ${stop.start}deg ${stop.end}deg`)
      .join(", ");
  }, [gradientStops]);

  // If only one slice or empty, show a different style
  if (slices.length === 0 || total === 0) {
    return (
      <div style={{ textAlign: "center" }}>
        <div
          style={{
            width: size,
            height: size,
            borderRadius: "50%",
            backgroundColor: "var(--surface-tertiary)",
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
          }}
        >
          <Caption1 color="tertiary">No data</Caption1>
        </div>
      </div>
    );
  }

  return (
    <div>
      {/* Pie chart */}
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          position: "relative",
        }}
      >
        <div
          style={{
            width: size,
            height: size,
            borderRadius: "50%",
            background: `conic-gradient(${gradientString})`,
            transition: "transform 0.2s ease",
            transform: hoveredIndex !== null ? "scale(1.02)" : "scale(1)",
            boxShadow:
              hoveredIndex !== null
                ? "0 4px 12px rgba(0, 0, 0, 0.15)"
                : "0 2px 8px rgba(0, 0, 0, 0.1)",
          }}
          role="img"
          aria-label={`Pie chart: ${slices.map((s) => `${s.label}: ${s.value}`).join(", ")}`}
        />

        {/* Center overlay for donut effect */}
        <div
          style={{
            position: "absolute",
            top: "50%",
            left: "50%",
            transform: "translate(-50%, -50%)",
            width: size * 0.5,
            height: size * 0.5,
            borderRadius: "50%",
            backgroundColor: "var(--bg-primary)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexDirection: "column",
          }}
        >
          {centerValue !== undefined && (
            <Subheadline bold>{formatValue(centerValue)}</Subheadline>
          )}
          {centerText && (
            <Caption1 color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
              {centerText}
            </Caption1>
          )}
        </div>
      </div>

      {/* Legend */}
      {showLegend && (
        <div
          style={{
            marginTop: tokens.spacing[4],
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[2],
          }}
        >
          {slices.map((slice, index) => (
            <div
              key={index}
              onMouseEnter={() => setHoveredIndex(index)}
              onMouseLeave={() => setHoveredIndex(null)}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
                borderRadius: tokens.radii.md,
                backgroundColor:
                  hoveredIndex === index
                    ? "var(--surface-secondary)"
                    : "transparent",
                transition: "background-color 0.15s ease",
                cursor: "pointer",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[2] }}>
                <span
                  style={{
                    width: 12,
                    height: 12,
                    borderRadius: tokens.radii.sm,
                    backgroundColor: slice.color,
                    flexShrink: 0,
                  }}
                />
                <Caption1>{slice.label}</Caption1>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <Caption1 color="tertiary">
                  {((slice.value / total) * 100).toFixed(1)}%
                </Caption1>
                <Caption1 style={{ fontFamily: tokens.typography.fontFamily.mono }}>
                  {formatValue(slice.value)}
                </Caption1>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Format large numbers for display
 */
function formatValue(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return value.toLocaleString();
}

/**
 * Default color palette for pie charts
 */
export const PIE_COLORS = [
  "var(--color-primary)",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#8b5cf6",
  "#06b6d4",
  "#ec4899",
  "#f97316",
  "#84cc16",
  "#6366f1",
];