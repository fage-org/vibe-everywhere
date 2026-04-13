import { useMemo } from "react";
import { tokens } from "../../design-system/tokens";
import { Caption1 } from "../ui/Typography";

export interface UsageDataPoint {
  timestamp: number;
  value: number;
  label: string;
}

export interface UsageChartProps {
  /** Data points to display */
  data: UsageDataPoint[];
  /** Chart height in pixels */
  height?: number;
  /** Bar color */
  color?: string;
  /** Show values on hover */
  showValues?: boolean;
  /** Maximum value (auto-calculated if not provided) */
  maxValue?: number;
}

/**
 * UsageChart - SVG bar chart for usage visualization
 *
 * Features:
 * - Simple CSS/SVG implementation (no external libraries)
 * - Responsive scaling
 * - Hover tooltips
 * - Gradient fill
 */
export function UsageChart({
  data,
  height = 200,
  color = "var(--color-primary)",
  showValues = true,
  maxValue: propMaxValue,
}: UsageChartProps) {
  const maxValue = useMemo(() => {
    if (propMaxValue) return propMaxValue;
    const values = data.map((d) => d.value);
    return Math.max(...values, 1);
  }, [data, propMaxValue]);

  const chartWidth = data.length * 40;
  const barWidth = 28;
  const barGap = 12;
  const padding = { top: 20, bottom: 40, left: 10, right: 10 };

  // Scale value to chart height
  const scaleValue = (value: number) => {
    const chartHeight = height - padding.top - padding.bottom;
    return (value / maxValue) * chartHeight;
  };

  return (
    <div style={{ width: "100%", overflow: "auto" }}>
      <svg
        viewBox={`0 0 ${chartWidth} ${height}`}
        style={{
          width: chartWidth > 400 ? `${chartWidth}px` : "100%",
          minWidth: "100%",
          height,
        }}
        role="img"
        aria-label="Usage chart"
      >
        {/* Gradient definition */}
        <defs>
          <linearGradient id="usageGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor="var(--color-primary)" stopOpacity="1" />
            <stop offset="100%" stopColor="var(--color-primary)" stopOpacity="0.6" />
          </linearGradient>
        </defs>

        {/* Grid lines */}
        {[0, 0.25, 0.5, 0.75, 1].map((ratio, index) => {
          const y = padding.top + (height - padding.top - padding.bottom) * (1 - ratio);
          return (
            <line
              key={index}
              x1={padding.left}
              y1={y}
              x2={chartWidth - padding.right}
              y2={y}
              stroke="var(--border-tertiary)"
              strokeWidth="1"
              strokeDasharray={ratio === 0 ? undefined : "4, 4"}
            />
          );
        })}

        {/* Bars */}
        {data.map((point, index) => {
          const x = padding.left + index * (barWidth + barGap) + barGap / 2;
          const barHeight = scaleValue(point.value);
          const y = height - padding.bottom - barHeight;

          return (
            <g key={index}>
              {/* Bar */}
              <rect
                x={x}
                y={y}
                width={barWidth}
                height={barHeight}
                fill="url(#usageGradient)"
                rx={tokens.radii.sm}
                ry={tokens.radii.sm}
                role="graphics-symbol"
                aria-label={`${point.label}: ${point.value.toLocaleString()}`}
              />

              {/* Value label */}
              {showValues && point.value > 0 && (
                <text
                  x={x + barWidth / 2}
                  y={y - 5}
                  textAnchor="middle"
                  fontSize={tokens.typography.fontSize.xs}
                  fill="var(--text-secondary)"
                  fontFamily={tokens.typography.fontFamily.mono}
                >
                  {formatValue(point.value)}
                </text>
              )}

              {/* X-axis label */}
              <text
                x={x + barWidth / 2}
                y={height - padding.bottom + 20}
                textAnchor="middle"
                fontSize={tokens.typography.fontSize.xs}
                fill="var(--text-tertiary)"
              >
                {point.label}
              </text>
            </g>
          );
        })}
      </svg>
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
  return value.toString();
}

/**
 * UsageChartLegend - Legend for the chart
 */
export function UsageChartLegend({
  items,
}: {
  items: { label: string; color: string; value: number }[];
}) {
  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        gap: tokens.spacing[4],
        marginTop: tokens.spacing[4],
      }}
    >
      {items.map((item, index) => (
        <div
          key={index}
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[2],
          }}
        >
          <span
            style={{
              width: 12,
              height: 12,
              borderRadius: tokens.radii.sm,
              backgroundColor: item.color,
            }}
          />
          <Caption1>
            {item.label}: {item.value.toLocaleString()}
          </Caption1>
        </div>
      ))}
    </div>
  );
}