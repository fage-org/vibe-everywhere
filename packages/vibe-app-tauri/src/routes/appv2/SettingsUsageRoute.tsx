import { useEffect, useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Eyebrow, Caption1, Subheadline } from "../../components/ui/Typography";
import { Card, CardContent, Badge, ShimmerCard } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import { UsageChart, UsagePieChart, PIE_COLORS, type PieSlice, type UsageDataPoint } from "../../components/usage";
import type { UsageBucket } from "../../desktop-client";
import type { UsagePeriod } from "../../desktop-client";

/**
 * Usage bucket from the API
 */
interface UsageData {
  usage: UsageBucket[];
  groupBy: "hour" | "day";
  totalReports: number;
}

/**
 * SettingsUsageRoute - Usage statistics page
 *
 * Displays API usage and cost statistics with:
 * - Bar chart for daily/hourly usage
 * - Pie chart for model distribution
 * - Period selector (today/7days/30days)
 * - Cost projection
 */
export function SettingsUsageRoute() {
  const { t } = useTranslation("routes");
  const { status, loadUsage } = useDesktopState();
  const [usageData, setUsageData] = useState<UsageData | null>(null);
  const [period, setPeriod] = useState<UsagePeriod>("30days");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchUsage = async () => {
      if (status !== "ready") {
        setError("Not connected");
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        const result = await loadUsage(period);
        setUsageData(result);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load usage");
      } finally {
        setLoading(false);
      }
    };

    fetchUsage();
  }, [status, loadUsage, period]);

  // Calculate totals
  const totals = useMemo(() => {
    if (!usageData?.usage) return { totalTokens: 0, totalCost: 0, tokensByModel: {}, costByModel: {} };

    return calculateTotals(usageData.usage);
  }, [usageData]);

  // Prepare chart data
  const chartData = useMemo((): UsageDataPoint[] => {
    if (!usageData?.usage) return [];

    return usageData.usage
      .filter((bucket) => {
        const totalTokens = Object.values(bucket.tokens).reduce((s, v) => s + v, 0);
        return totalTokens > 0;
      })
      .slice(-14) // Show last 14 data points
      .map((bucket) => {
        const date = new Date(bucket.timestamp * 1000);
        const label = usageData.groupBy === "hour"
          ? date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
          : date.toLocaleDateString([], { month: "short", day: "numeric" });

        const totalTokens = Object.values(bucket.tokens).reduce((s, v) => s + v, 0);

        return {
          timestamp: bucket.timestamp,
          value: totalTokens,
          label,
        };
      });
  }, [usageData]);

  // Prepare pie chart data (model distribution)
  const pieData = useMemo((): PieSlice[] => {
    const entries = Object.entries(totals.tokensByModel);
    if (entries.length === 0) return [];

    return entries
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10) // Top 10 models
      .map(([model, tokens], index) => ({
        label: formatModelName(model),
        value: tokens,
        color: PIE_COLORS[index % PIE_COLORS.length],
      }));
  }, [totals]);

  // Cost projection (simple linear projection)
  const projection = useMemo(() => {
    if (!usageData?.usage || usageData.usage.length === 0) return null;

    const dailyCost = totals.totalCost / Math.max(usageData.usage.length, 1);
    const daysRemaining = period === "today" ? 1 : period === "7days" ? 7 : 30;
    const projectedMonthlyCost = dailyCost * 30;

    return {
      dailyCost,
      projectedMonthlyCost,
      daysRemaining,
    };
  }, [usageData, totals, period]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
        overflow: "auto",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: `${tokens.spacing[6]} ${tokens.spacing[6]} ${tokens.spacing[4]}`,
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            gap: tokens.spacing[4],
          }}
        >
          <div>
            <Title3>{t("settingsUsage.title")}</Title3>
            <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
              {t("settingsUsage.description")}
            </Body>
          </div>

          {/* Period Selector */}
          <select
            value={period}
            onChange={(e) => setPeriod(e.target.value as UsagePeriod)}
            disabled={loading}
            style={{
              padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
              backgroundColor: "var(--surface-secondary)",
              border: "1px solid var(--border-primary)",
              borderRadius: tokens.radii.md,
              color: "var(--text-primary)",
              fontSize: tokens.typography.fontSize.sm,
              cursor: "pointer",
            }}
          >
            <option value="today">{t("settingsUsage.periodOptions.today")}</option>
            <option value="7days">{t("settingsUsage.periodOptions.7days")}</option>
            <option value="30days">{t("settingsUsage.periodOptions.30days")}</option>
          </select>
        </div>
      </div>

      {/* Content */}
      <div
        style={{
          flex: 1,
          padding: tokens.spacing[6],
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[6],
        }}
      >
        {error && (
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {loading && !error && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
            <ShimmerCard style={{ height: "120px" }} />
            <ShimmerCard style={{ height: "250px" }} />
            <ShimmerCard style={{ height: "200px" }} />
          </div>
        )}

        {!loading && !error && (
          <>
            {/* Summary Stats */}
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "repeat(auto-fit, minmax(150px, 1fr))",
                gap: tokens.spacing[4],
              }}
            >
              <SummaryCard
                label={t("settingsUsage.totalTokens")}
                value={formatNumber(totals.totalTokens)}
              />
              <SummaryCard
                label={t("settingsUsage.estimatedCost")}
                value={`$${totals.totalCost.toFixed(4)}`}
              />
              {projection && (
                <SummaryCard
                  label={t("settingsUsage.projectedMonthly")}
                  value={`$${projection.projectedMonthlyCost.toFixed(2)}`}
                  secondary
                />
              )}
            </div>

            {/* Usage Chart */}
            {chartData.length > 0 && (
              <Card variant="default" padding="md">
                <CardContent>
                  <Eyebrow style={{ marginBottom: tokens.spacing[4] }}>
                    {t("settingsUsage.chart.title")}
                  </Eyebrow>
                  <UsageChart data={chartData} height={180} />
                </CardContent>
              </Card>
            )}

            {/* Model Distribution */}
            {pieData.length > 0 && (
              <Card variant="default" padding="md">
                <CardContent>
                  <Eyebrow style={{ marginBottom: tokens.spacing[4] }}>
                    {t("settingsUsage.distribution.title")}
                  </Eyebrow>
                  <div style={{ display: "flex", justifyContent: "center" }}>
                    <UsagePieChart
                      slices={pieData}
                      size={200}
                      centerText={t("settingsUsage.distribution.tokens")}
                      centerValue={totals.totalTokens}
                    />
                  </div>
                </CardContent>
              </Card>
            )}

            {/* Empty State */}
            {chartData.length === 0 && pieData.length === 0 && (
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: tokens.spacing[8],
                  textAlign: "center",
                }}
              >
                <Title3 color="secondary">{t("settingsUsage.empty.title")}</Title3>
                <Body color="tertiary" style={{ marginTop: tokens.spacing[2] }}>
                  {t("settingsUsage.empty.description")}
                </Body>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

/**
 * Summary card for quick stats
 */
function SummaryCard({
  label,
  value,
  secondary,
}: {
  label: string;
  value: string;
  secondary?: boolean;
}) {
  return (
    <div
      style={{
        padding: tokens.spacing[4],
        backgroundColor: secondary ? "var(--surface-tertiary)" : "var(--surface-secondary)",
        borderRadius: tokens.radii.lg,
        border: "1px solid var(--border-primary)",
      }}
    >
      <Caption1 color="tertiary">{label}</Caption1>
      <Subheadline
        bold
        style={{
          marginTop: tokens.spacing[2],
          fontFamily: tokens.typography.fontFamily.mono,
        }}
      >
        {value}
      </Subheadline>
    </div>
  );
}

/**
 * Calculate totals from usage buckets
 */
function calculateTotals(usage: UsageBucket[]) {
  const result = {
    totalTokens: 0,
    totalCost: 0,
    tokensByModel: {} as Record<string, number>,
    costByModel: {} as Record<string, number>,
  };

  for (const bucket of usage) {
    for (const [key, tokens] of Object.entries(bucket.tokens)) {
      result.totalTokens += tokens;
      result.tokensByModel[key] = (result.tokensByModel[key] || 0) + tokens;
    }
    for (const [key, cost] of Object.entries(bucket.cost)) {
      result.totalCost += cost;
      result.costByModel[key] = (result.costByModel[key] || 0) + cost;
    }
  }

  return result;
}

/**
 * Format large numbers
 */
function formatNumber(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return value.toLocaleString();
}

/**
 * Format model name for display
 */
function formatModelName(model: string): string {
  // Common model name mappings
  const modelNames: Record<string, string> = {
    input: "Input Tokens",
    output: "Output Tokens",
    "claude-3-5-sonnet": "Claude 3.5 Sonnet",
    "claude-3-opus": "Claude 3 Opus",
    "claude-3-sonnet": "Claude 3 Sonnet",
    "claude-3-haiku": "Claude 3 Haiku",
    "gpt-4o": "GPT-4o",
    "gpt-4-turbo": "GPT-4 Turbo",
    "gpt-4": "GPT-4",
    "gpt-3.5-turbo": "GPT-3.5 Turbo",
  };

  return modelNames[model] || model;
}