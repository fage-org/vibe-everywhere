import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useLocalSettings } from "../../local-settings";
import { useTranslation } from "react-i18next";

/**
 * SettingsAppearanceRoute - Appearance settings page
 *
 * Manages theme, display preferences, and visual settings
 * stored in local localStorage (not synced to server).
 */
export function SettingsAppearanceRoute() {
  const { t } = useTranslation("routes");
  const { settings, updateSettings } = useLocalSettings();

  const sections: SettingSection[] = [
    {
      id: "theme",
      title: t("settingsAppearance.theme"),
      description: t("settingsAppearance.themeDescription"),
      settings: [
        {
          id: "themePreference",
          label: t("settings.appearance"),
          type: "select",
          value: settings.themePreference,
          options: [
            { label: t("settingsAppearance.themeOptions.adaptive"), value: "adaptive" },
            { label: t("settingsAppearance.themeOptions.light"), value: "light" },
            { label: t("settingsAppearance.themeOptions.dark"), value: "dark" },
          ],
          onChange: (value) => updateSettings({ themePreference: value as "adaptive" | "light" | "dark" }),
        },
      ],
    },
    {
      id: "display",
      title: t("settingsAppearance.display"),
      description: t("settingsAppearance.displayDescription"),
      settings: [
        {
          id: "compactSessionView",
          label: t("settingsAppearance.compactSessionView"),
          description: t("settingsAppearance.compactSessionViewDescription"),
          type: "toggle",
          value: settings.compactSessionView,
          onChange: (value) => updateSettings({ compactSessionView: value as boolean }),
        },
        {
          id: "viewInline",
          label: t("settingsAppearance.inlineToolCalls"),
          description: t("settingsAppearance.inlineToolCallsDescription"),
          type: "toggle",
          value: settings.viewInline,
          onChange: (value) => updateSettings({ viewInline: value as boolean }),
        },
        {
          id: "expandTodos",
          label: t("settingsAppearance.expandTodoLists"),
          description: t("settingsAppearance.expandTodoListsDescription"),
          type: "toggle",
          value: settings.expandTodos,
          onChange: (value) => updateSettings({ expandTodos: value as boolean }),
        },
        {
          id: "showLineNumbers",
          label: t("settingsAppearance.showLineNumbersInDiffs"),
          description: t("settingsAppearance.showLineNumbersInDiffsDescription"),
          type: "toggle",
          value: settings.showLineNumbers,
          onChange: (value) => updateSettings({ showLineNumbers: value as boolean }),
        },
        {
          id: "showLineNumbersInToolViews",
          label: t("settingsAppearance.showLineNumbersInToolViews"),
          description: t("settingsAppearance.showLineNumbersInToolViewsDescription"),
          type: "toggle",
          value: settings.showLineNumbersInToolViews,
          onChange: (value) => updateSettings({ showLineNumbersInToolViews: value as boolean }),
        },
        {
          id: "wrapLinesInDiffs",
          label: t("settingsAppearance.wrapLinesInDiffs"),
          description: t("settingsAppearance.wrapLinesInDiffsDescription"),
          type: "toggle",
          value: settings.wrapLinesInDiffs,
          onChange: (value) => updateSettings({ wrapLinesInDiffs: value as boolean }),
        },
        {
          id: "alwaysShowContextSize",
          label: t("settingsAppearance.alwaysShowContextSize"),
          description: t("settingsAppearance.alwaysShowContextSizeDescription"),
          type: "toggle",
          value: settings.alwaysShowContextSize,
          onChange: (value) => updateSettings({ alwaysShowContextSize: value as boolean }),
        },
      ],
    },
    {
      id: "avatars",
      title: t("settingsAppearance.avatarStyle"),
      description: t("settingsAppearance.avatarStyleDescription"),
      settings: [
        {
          id: "avatarStyle",
          label: t("settingsAppearance.avatarStyle"),
          type: "select",
          value: settings.avatarStyle,
          options: [
            { label: t("settingsAppearance.avatarOptions.pixelated"), value: "pixelated" },
            { label: t("settingsAppearance.avatarOptions.gradient"), value: "gradient" },
            { label: t("settingsAppearance.avatarOptions.brutalist"), value: "brutalist" },
          ],
          onChange: (value) => updateSettings({ avatarStyle: value as "pixelated" | "gradient" | "brutalist" }),
        },
        {
          id: "showFlavorIcons",
          label: t("settingsAppearance.showFlavorIcons"),
          description: t("settingsAppearance.showFlavorIconsDescription"),
          type: "toggle",
          value: settings.showFlavorIcons,
          onChange: (value) => updateSettings({ showFlavorIcons: value as boolean }),
        },
      ],
    },
  ];

  return (
    <SettingsSurface
      sections={sections}
      description={t("settingsAppearance.description")}
    />
  );
}