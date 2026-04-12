import { useMemo } from "react";
import type { SettingSection } from "./components/routes";
import type { SupportedLanguage } from "./i18n/types";
import { SUPPORTED_LANGUAGES } from "./i18n/types";
import type { DesktopAppearanceSettings } from "./desktop-preferences";

type SettingsSectionOptions = {
  appearance: DesktopAppearanceSettings;
  sidebarCollapsed: boolean;
  language: SupportedLanguage;
  setLanguage: (value: SupportedLanguage) => void | Promise<void>;
  setAppearance: (value: DesktopAppearanceSettings) => void;
  persistAppearance: (value: DesktopAppearanceSettings) => void;
  setSidebarCollapsed: (value: boolean) => void;
  t: (key: string) => string;
};

export function useAppV2SettingsSections({
  appearance,
  sidebarCollapsed,
  language,
  setLanguage,
  setAppearance,
  persistAppearance,
  setSidebarCollapsed,
  t,
}: SettingsSectionOptions): SettingSection[] {
  return useMemo((): SettingSection[] => {
    return [
      {
        id: "language",
        title: t("routes:settings.language.title"),
        description: t("routes:settings.language.description"),
        settings: [
          {
            id: "language-select",
            label: t("routes:settings.language.title"),
            type: "select",
            value: language,
            options: SUPPORTED_LANGUAGES.map((lang) => ({
              label: `${lang.flag} ${lang.nativeName}`,
              value: lang.code,
            })),
            onChange: (value) => {
              void setLanguage(value as SupportedLanguage);
            },
          },
        ],
      },
      {
        id: "appearance",
        title: t("routes:settings.sections.appearance"),
        description: t("routes:settings.appearanceDescription"),
        settings: [
          {
            id: "theme",
            label: t("settings:theme.title"),
            description: t("settings:theme.description"),
            type: "select",
            value: appearance.themePreference,
            options: [
              { label: t("settings:theme.system"), value: "adaptive" },
              { label: t("settings:theme.light"), value: "light" },
              { label: t("settings:theme.dark"), value: "dark" },
            ],
            onChange: (value) => {
              const nextAppearance = {
                ...appearance,
                themePreference: value as DesktopAppearanceSettings["themePreference"],
              };
              setAppearance(nextAppearance);
              persistAppearance(nextAppearance);
            },
          },
          {
            id: "sidebar",
            label: t("routes:settings.sidebar.label"),
            description: t("routes:settings.sidebar.description"),
            type: "toggle",
            value: sidebarCollapsed,
            onChange: (value) => setSidebarCollapsed(value as boolean),
          },
        ],
      },
    ];
  }, [
    appearance,
    language,
    persistAppearance,
    setAppearance,
    setLanguage,
    setSidebarCollapsed,
    sidebarCollapsed,
    t,
  ]);
}
