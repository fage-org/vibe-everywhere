import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1, Subheadline } from "../../components/ui/Typography";
import { Card, CardContent, Badge, ShimmerCard } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import { AI_PROVIDERS, type AIProvider } from "../../constants/ai-providers";
import type { VendorTokenInfo } from "../../desktop-client";

/**
 * SettingsAIProvidersRoute - AI Provider token management page
 *
 * Allows users to connect AI provider accounts (OpenAI, Anthropic, Gemini)
 * by storing their API tokens securely. Tokens are validated before being stored.
 *
 * IMPORTANT: Each provider uses different token formats:
 * - OpenAI: Bearer token auth (used by Codex)
 * - Anthropic: x-api-key header (NOT OpenAI compatible)
 * - Gemini: Query parameter auth
 */
export function SettingsAIProvidersRoute() {
  const { t } = useTranslation("ui");
  const { status, listVendorTokens, registerVendorToken, deleteVendorToken } = useDesktopState();

  const [vendorTokens, setVendorTokens] = useState<Record<string, VendorTokenInfo>>({});
  const [loading, setLoading] = useState(true);
  const [editingVendor, setEditingVendor] = useState<string | null>(null);
  const [inputToken, setInputToken] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTokens = useCallback(async () => {
    if (status !== "ready") return;
    setLoading(true);
    try {
      const list = await listVendorTokens();
      const map: Record<string, VendorTokenInfo> = {};
      for (const vendorToken of list) {
        map[vendorToken.vendor] = vendorToken;
      }
      setVendorTokens(map);
    } catch (err) {
      console.error("Failed to load vendor tokens:", err);
    } finally {
      setLoading(false);
    }
  }, [listVendorTokens, status]);

  useEffect(() => {
    loadTokens();
  }, [loadTokens]);

  const handleSave = async (provider: AIProvider) => {
    if (!inputToken.trim()) return;

    setSaving(true);
    setError(null);

    try {
      await registerVendorToken(provider.id, inputToken.trim());
      await loadTokens();
      setEditingVendor(null);
      setInputToken("");
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to save token";
      setError(message);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (vendor: string) => {
    try {
      await deleteVendorToken(vendor);
      const newVendorTokens = { ...vendorTokens };
      delete newVendorTokens[vendor];
      setVendorTokens(newVendorTokens);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to delete token";
      setError(message);
    }
  };

  const handleCancel = () => {
    setEditingVendor(null);
    setInputToken("");
    setError(null);
  };

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
        <Title3>{t("settings.aiProviders")}</Title3>
        <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
          {t("settings.aiProvidersDescription")}
        </Body>
      </div>

      {/* Content */}
      <div
        style={{
          flex: 1,
          padding: tokens.spacing[6],
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[4],
        }}
      >
        {/* Error */}
        {error && (
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {/* Loading */}
        {loading && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
            <ShimmerCard style={{ height: "100px" }} />
            <ShimmerCard style={{ height: "100px" }} />
            <ShimmerCard style={{ height: "100px" }} />
          </div>
        )}

        {/* Provider Cards */}
        {!loading && (
          <>
            {AI_PROVIDERS.map((provider) => {
              const vendorToken = vendorTokens[provider.id];

              return (
                <ProviderCard
                  key={provider.id}
                  provider={provider}
                  vendorToken={vendorToken}
                  editing={editingVendor === provider.id}
                  inputToken={inputToken}
                  saving={saving}
                  onEdit={() => {
                    setEditingVendor(provider.id);
                    setInputToken("");
                    setError(null);
                  }}
                  onInputChange={setInputToken}
                  onSave={() => handleSave(provider)}
                  onCancel={handleCancel}
                  onDelete={() => handleDelete(provider.id)}
                />
              );
            })}
          </>
        )}
      </div>
    </div>
  );
}

/**
 * Provider card component for each AI provider
 */
function ProviderCard({
  provider,
  vendorToken,
  editing,
  inputToken,
  saving,
  onEdit,
  onInputChange,
  onSave,
  onCancel,
  onDelete,
}: {
  provider: AIProvider;
  vendorToken?: VendorTokenInfo;
  editing: boolean;
  inputToken: string;
  saving: boolean;
  onEdit: () => void;
  onInputChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslation("ui");

  return (
    <Card variant="default" padding="md">
      <CardContent>
        {/* Header */}
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "flex-start",
            marginBottom: tokens.spacing[3],
          }}
        >
          <div>
            <Subheadline bold>{provider.name}</Subheadline>
            <Caption1 color="secondary">{provider.description}</Caption1>
          </div>
          {vendorToken && (
            <Badge variant="success">{t("settings.providerConnected")}</Badge>
          )}
        </div>

        {/* Connected state */}
        {vendorToken && (
          <div>
            <Caption1
              color="tertiary"
              style={{
                fontFamily: tokens.typography.fontFamily.mono,
                marginBottom: tokens.spacing[3],
              }}
            >
              {t("settings.tokenMasked")}: {vendorToken.maskedToken}
            </Caption1>
            <button
              onClick={onDelete}
              style={{
                padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                backgroundColor: "var(--color-error-muted)",
                border: "1px solid var(--color-error)",
                borderRadius: tokens.radii.md,
                color: "var(--color-error)",
                cursor: "pointer",
                fontSize: tokens.typography.fontSize.sm,
              }}
            >
              {t("settings.disconnectProvider")}
            </button>
          </div>
        )}

        {/* Editing state */}
        {!vendorToken && editing && (
          <div>
            <Caption1 color="tertiary" style={{ marginBottom: tokens.spacing[2] }}>
              {provider.validationHint}
            </Caption1>
            <input
              type="password"
              placeholder={provider.tokenPlaceholder}
              value={inputToken}
              onChange={(e) => onInputChange(e.target.value)}
              style={{
                width: "100%",
                padding: tokens.spacing[3],
                backgroundColor: "var(--surface-secondary)",
                border: "1px solid var(--border-primary)",
                borderRadius: tokens.radii.md,
                color: "var(--text-primary)",
                fontFamily: tokens.typography.fontFamily.mono,
                fontSize: tokens.typography.fontSize.sm,
                marginBottom: tokens.spacing[3],
                boxSizing: "border-box",
              }}
            />
            <div style={{ display: "flex", gap: tokens.spacing[2] }}>
              <button
                onClick={onSave}
                disabled={saving || !inputToken.trim()}
                style={{
                  flex: 1,
                  padding: tokens.spacing[3],
                  backgroundColor: saving || !inputToken.trim() ? "var(--surface-tertiary)" : "var(--color-primary)",
                  border: "none",
                  borderRadius: tokens.radii.md,
                  color: saving || !inputToken.trim() ? "var(--text-tertiary)" : "white",
                  cursor: saving || !inputToken.trim() ? "not-allowed" : "pointer",
                  fontSize: tokens.typography.fontSize.sm,
                  fontWeight: 600,
                }}
              >
                {saving ? t("settings.validatingToken") : t("settings.save")}
              </button>
              <button
                onClick={onCancel}
                disabled={saving}
                style={{
                  flex: 1,
                  padding: tokens.spacing[3],
                  backgroundColor: "var(--surface-secondary)",
                  border: "1px solid var(--border-primary)",
                  borderRadius: tokens.radii.md,
                  color: "var(--text-primary)",
                  cursor: saving ? "not-allowed" : "pointer",
                  fontSize: tokens.typography.fontSize.sm,
                }}
              >
                {t("settings.cancel")}
              </button>
            </div>
          </div>
        )}

        {/* Default state */}
        {!vendorToken && !editing && (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <button
              onClick={onEdit}
              style={{
                padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                backgroundColor: "var(--color-primary)",
                border: "none",
                borderRadius: tokens.radii.md,
                color: "white",
                cursor: "pointer",
                fontSize: tokens.typography.fontSize.sm,
                fontWeight: 600,
              }}
            >
              {t("settings.connectProvider")}
            </button>
            <a
              href={provider.tokenHelpUrl}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: "var(--color-primary)",
                fontSize: tokens.typography.fontSize.sm,
     textDecoration: "none",
              }}
            >
              {t("settings.getApiKey")}
            </a>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default SettingsAIProvidersRoute;