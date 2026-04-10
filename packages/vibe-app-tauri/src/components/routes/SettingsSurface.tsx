import { useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, Button, Input, Badge } from "../ui";
import { Title2, Body, Subheadline, Eyebrow } from "../ui/Typography";

export interface SettingSection {
  /** Section ID */
  id: string;
  /** Section title */
  title: string;
  /** Section description */
  description?: string;
  /** Settings in this section */
  settings: SettingItem[];
}

export interface SettingItem {
  /** Setting ID */
  id: string;
  /** Setting label */
  label: string;
  /** Setting description */
  description?: string;
  /** Setting type */
  type: "toggle" | "select" | "text" | "number" | "custom";
  /** Current value */
  value: unknown;
  /** Options for select type */
  options?: { label: string; value: string }[];
  /** Custom renderer */
  render?: () => ReactNode;
  /** Callback when value changes */
  onChange?: (value: unknown) => void;
  /** Whether setting is disabled */
  disabled?: boolean;
  /** Beta/Experimental badge */
  experimental?: boolean;
}

export interface SettingsSurfaceProps {
  /** Setting sections */
  sections: SettingSection[];
  /** Page title */
  title?: string;
  /** Page description */
  description?: string;
  /** Save button handler */
  onSave?: () => void;
  /** Reset button handler */
  onReset?: () => void;
  /** Whether settings have unsaved changes */
  hasChanges?: boolean;
  /** Loading state */
  loading?: boolean;
}

/**
 * SettingsSurface - Settings route surface
 *
 * Matches Happy's SettingsView:
 * - Grouped sections with clear headings
 * - Row-based settings layout
 * - Toggle, select, and input types
 * - Beta/experimental badges
 * - Save/Reset actions
 */
export function SettingsSurface({
  sections,
  description,
  onSave,
  onReset,
  hasChanges = false,
  loading,
}: SettingsSurfaceProps) {
  const { t } = useTranslation(['routes', 'ui']);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
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
            <Title2>{t('routes:settings.title')}</Title2>
            {description && (
              <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
                {description}
              </Body>
            )}
          </div>

          {(onSave || onReset) && (
            <div style={{ display: "flex", gap: tokens.spacing[2] }}>
              {onReset && (
                <Button variant="ghost" size="sm" onClick={onReset} disabled={loading}>
                  {t('routes:settings.actions.reset')}
                </Button>
              )}
              {onSave && (
                <Button
                  variant="primary"
                  size="sm"
                  onClick={onSave}
                  disabled={!hasChanges || loading}
                  loading={loading}
                >
                  {t('routes:settings.actions.saveChanges')}
                </Button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Settings Content */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          padding: tokens.spacing[6],
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[8],
            maxWidth: "800px",
          }}
        >
          {sections.map((section) => (
            <SettingSectionView key={section.id} section={section} />
          ))}
        </div>
      </div>
    </div>
  );
}

interface SettingSectionViewProps {
  section: SettingSection;
}

function SettingSectionView({ section }: SettingSectionViewProps) {
  return (
    <section>
      <div style={{ marginBottom: tokens.spacing[4] }}>
        <Eyebrow>{section.title}</Eyebrow>
        {section.description && (
          <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
            {section.description}
          </Body>
        )}
      </div>

      <Card variant="default" padding="none">
        {section.settings.map((setting, index) => (
          <SettingRow
            key={setting.id}
            setting={setting}
            isLast={index === section.settings.length - 1}
          />
        ))}
      </Card>
    </section>
  );
}

interface SettingRowProps {
  setting: SettingItem;
  isLast: boolean;
}

function SettingRow({ setting, isLast }: SettingRowProps) {
  const { t } = useTranslation(['ui']);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: tokens.spacing[4],
        padding: `${tokens.spacing[4]}`,
        borderBottom: isLast ? undefined : "1px solid var(--border-primary)",
        opacity: setting.disabled ? 0.5 : 1,
      }}
    >
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[2] }}>
          <Subheadline>{setting.label}</Subheadline>
          {setting.experimental && (
            <Badge variant="warning" size="sm">{t('ui:badges.beta')}</Badge>
          )}
        </div>
        {setting.description && (
          <Body color="tertiary" style={{ marginTop: tokens.spacing[1], fontSize: tokens.typography.fontSize.sm }}>
            {setting.description}
          </Body>
        )}
      </div>

      <div style={{ flexShrink: 0 }}>
        <SettingControl setting={setting} />
      </div>
    </div>
  );
}

function SettingControl({ setting }: { setting: SettingItem }) {
  if (setting.render) {
    return <>{setting.render()}</>;
  }

  switch (setting.type) {
    case "toggle":
      return (
        <Toggle
          checked={setting.value as boolean}
          onChange={(checked) => setting.onChange?.(checked)}
          disabled={setting.disabled}
        />
      );

    case "select":
      return (
        <select
          value={setting.value as string}
          onChange={(e) => setting.onChange?.(e.target.value)}
          disabled={setting.disabled}
          style={{
            padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
            backgroundColor: "var(--surface-secondary)",
            border: "1px solid var(--border-primary)",
            borderRadius: tokens.radii.md,
            color: "var(--text-primary)",
            fontSize: tokens.typography.fontSize.sm,
            cursor: setting.disabled ? "not-allowed" : "pointer",
          }}
        >
          {setting.options?.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      );

    case "text":
      return (
        <Input
          type="text"
          value={setting.value as string}
          onChange={(e) => setting.onChange?.(e.target.value)}
          disabled={setting.disabled}
          size="sm"
        />
      );

    case "number":
      return (
        <Input
          type="number"
          value={setting.value as number}
          onChange={(e) => setting.onChange?.(Number(e.target.value))}
          disabled={setting.disabled}
          size="sm"
        />
      );

    default:
      return null;
  }
}

interface ToggleProps {
  checked: boolean;
  onChange?: (checked: boolean) => void;
  disabled?: boolean;
}

function Toggle({ checked, onChange, disabled }: ToggleProps) {
  return (
    <button
      onClick={() => onChange?.(!checked)}
      disabled={disabled}
      style={{
        position: "relative",
        width: "48px",
        height: "28px",
        borderRadius: tokens.radii.full,
        backgroundColor: checked ? "var(--color-primary)" : "var(--surface-tertiary)",
        border: "none",
        cursor: disabled ? "not-allowed" : "pointer",
        transition: `background-color ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      }}
    >
      <span
        style={{
          position: "absolute",
          top: "2px",
          left: checked ? "22px" : "2px",
          width: "24px",
          height: "24px",
          borderRadius: tokens.radii.full,
          backgroundColor: "#ffffff",
          boxShadow: tokens.shadows.sm,
          transition: `left ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
        }}
      />
    </button>
  );
}
