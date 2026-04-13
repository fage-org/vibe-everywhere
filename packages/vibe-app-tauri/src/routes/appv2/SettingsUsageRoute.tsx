import { useEffect, useState } from "react";
import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useDesktopState } from "../../useDesktopState";
import { useTranslation } from "react-i18next";
import { Body, Title3 } from "../../components/ui/Typography";
import { tokens } from "../../design-system/tokens";

/**
 * Usage bucket from the API
 */
interface UsageBucket {
  timestamp: number;
  tokens: Record<string, number>;
  cost: Record<string, number>;
  reportCount: number;
}

/**
 * SettingsUsageRoute - Usage statistics page
 *
 * Displays API usage and cost statistics.
 */
export function SettingsUsageRoute() {
  const { t } = useTranslation("routes");
  const { status, loadUsage } = useDesktopState();
  const [usage, setUsage] = useState<UsageBucket[]>([]);
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
        // Query last 30 days
        const result = await loadUsage("30days");
        setUsage(result.usage);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load usage");
      } finally {
        setLoading(false);
      }
    };

    fetchUsage();
  }, [status, loadUsage]);

  // Calculate totals
  const totalTokens = usage.reduce((sum, bucket) => {
    const bucketTotal = Object.values(bucket.tokens).reduce((s, v) => s + v, 0);
    return sum + bucketTotal;
  }, 0);

  const totalCost = usage.reduce((sum, bucket) => {
    const bucketTotal = Object.values(bucket.cost).reduce((s, v) => s + v, 0);
    return sum + bucketTotal;
  }, 0);

  const sections: SettingSection[] = [
    {
      id: "summary",
      title: t("settingsUsage.summary"),
      description: t("settingsUsage.summaryDescription"),
      settings: [
        {
          id: "totalTokens",
          label: t("settingsUsage.totalTokens"),
          type: "custom",
          value: totalTokens,
          render: () => (
            <Body style={{ fontFamily: tokens.typography.fontFamily.mono }}>
              {totalTokens.toLocaleString()}
            </Body>
          ),
        },
        {
          id: "estimatedCost",
          label: t("settingsUsage.estimatedCost"),
          type: "custom",
          value: totalCost,
          render: () => (
            <Body style={{ fontFamily: tokens.typography.fontFamily.mono }}>
              ${totalCost.toFixed(4)}
            </Body>
          ),
        },
        {
          id: "period",
          label: t("settingsUsage.period"),
          type: "custom",
          value: "30 days",
          render: () => <Body>Last 30 days</Body>,
        },
      ],
    },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <SettingsSurface
        sections={sections}
        description={t("settingsUsage.description")}
        loading={loading}
      />

      {/* Usage breakdown */}
      {usage.length > 0 && (
        <div
          style={{
            padding: tokens.spacing[6],
            borderTop: "1px solid var(--border-primary)",
          }}
        >
          <Title3 style={{ marginBottom: tokens.spacing[4] }}>
            {t("settingsUsage.breakdown")}
          </Title3>

          {error && (
            <Body color="tertiary" style={{ marginBottom: tokens.spacing[4] }}>
              {error}
            </Body>
          )}

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
              gap: tokens.spacing[4],
            }}
          >
            {usage.slice(0, 7).map((bucket, index) => (
              <UsageCard key={index} bucket={bucket} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Usage card component for displaying individual bucket stats
 */
function UsageCard({ bucket }: { bucket: UsageBucket }) {
  const date = new Date(bucket.timestamp * 1000);
  const dateStr = date.toLocaleDateString();

  const inputTokens = bucket.tokens["input"] || 0;
  const outputTokens = bucket.tokens["output"] || 0;
  const cost = Object.values(bucket.cost).reduce((s, v) => s + v, 0);

  return (
    <div
      style={{
        padding: tokens.spacing[4],
        backgroundColor: "var(--surface-secondary)",
        borderRadius: tokens.radii.lg,
        border: "1px solid var(--border-primary)",
      }}
    >
      <Body bold style={{ marginBottom: tokens.spacing[2] }}>
        {dateStr}
      </Body>
      <Body color="secondary" style={{ fontSize: tokens.typography.fontSize.sm }}>
        In: {inputTokens.toLocaleString()} | Out: {outputTokens.toLocaleString()}
      </Body>
      <Body color="tertiary" style={{ fontSize: tokens.typography.fontSize.xs }}>
        ${cost.toFixed(4)}
      </Body>
    </div>
  );
}