import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useLocalSettings } from "../../local-settings";
import { useTranslation } from "react-i18next";

/**
 * SettingsFeaturesRoute - Features settings page
 *
 * Manages experimental features and input preferences
 * stored in local localStorage (not synced to server).
 */
export function SettingsFeaturesRoute() {
  const { t } = useTranslation("routes");
  const { settings, updateSettings } = useLocalSettings();

  const sections: SettingSection[] = [
    {
      id: "experimental",
      title: t("settingsFeatures.experimental"),
      description: t("settingsFeatures.experimentalDescription"),
      settings: [
        {
          id: "devModeEnabled",
          label: t("settingsFeatures.developerMode"),
          description: t("settingsFeatures.developerModeDescription"),
          type: "toggle",
          value: settings.devModeEnabled,
          onChange: (value) => updateSettings({ devModeEnabled: value as boolean }),
          experimental: true,
        },
        {
          id: "debugMode",
          label: t("settingsFeatures.debugMode"),
          description: t("settingsFeatures.debugModeDescription"),
          type: "toggle",
          value: settings.debugMode,
          onChange: (value) => updateSettings({ debugMode: value as boolean }),
        },
      ],
    },
    {
      id: "input",
      title: t("settingsFeatures.input"),
      description: t("settingsFeatures.inputDescription"),
      settings: [
        {
          id: "agentInputEnterToSend",
          label: t("settingsFeatures.enterToSend"),
          description: t("settingsFeatures.enterToSendDescription"),
          type: "toggle",
          value: settings.agentInputEnterToSend,
          onChange: (value) => updateSettings({ agentInputEnterToSend: value as boolean }),
        },
      ],
    },
    {
      id: "sessions",
      title: t("settingsFeatures.sessions"),
      description: t("settingsFeatures.sessionsDescription"),
      settings: [
        {
          id: "hideInactiveSessions",
          label: t("settingsFeatures.hideInactiveSessions"),
          description: t("settingsFeatures.hideInactiveSessionsDescription"),
          type: "toggle",
          value: settings.hideInactiveSessions,
          onChange: (value) => updateSettings({ hideInactiveSessions: value as boolean }),
        },
      ],
    },
  ];

  return (
    <SettingsSurface
      sections={sections}
      description={t("settingsFeatures.description")}
    />
  );
}