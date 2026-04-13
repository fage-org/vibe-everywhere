import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useLocalSettings } from "../../local-settings";
import { useTranslation } from "react-i18next";

/**
 * Supported languages for the application
 */
const SUPPORTED_LANGUAGES = [
  { code: null, nativeName: "Auto-detect", englishName: "Auto-detect" },
  { code: "en", nativeName: "English", englishName: "English" },
  { code: "zh-CN", nativeName: "简体中文", englishName: "Simplified Chinese" },
];

/**
 * SettingsLanguageRoute - Language settings page
 *
 * Manages language preferences stored in local localStorage.
 */
export function SettingsLanguageRoute() {
  const { t, i18n } = useTranslation();
  const { settings, updateSettings } = useLocalSettings();

  const handleLanguageChange = (langCode: string | null) => {
    // Update local settings
    updateSettings({ preferredLanguage: langCode });

    // Change i18n language
    if (langCode === null) {
      // Auto-detect from browser
      const browserLang = navigator.language.split("-")[0];
      const supportedLang = SUPPORTED_LANGUAGES.find(
        (l) => l.code === browserLang || l.code === navigator.language
      );
      i18n.changeLanguage(supportedLang?.code || "en");
    } else {
      i18n.changeLanguage(langCode);
    }
  };

  const currentLanguageLabel =
    SUPPORTED_LANGUAGES.find((l) => l.code === settings.preferredLanguage)
      ?.nativeName || SUPPORTED_LANGUAGES[0].nativeName;

  const sections: SettingSection[] = [
    {
      id: "language",
      title: t("routes:settingsLanguage.title"),
      description: t("routes:settingsLanguage.description"),
      settings: [
        {
          id: "preferredLanguage",
          label: t("routes:settingsLanguage.currentLanguage"),
          type: "select",
          value: settings.preferredLanguage || "",
          options: SUPPORTED_LANGUAGES.map((lang) => ({
            label: lang.nativeName,
            value: lang.code || "",
          })),
          onChange: (value) => handleLanguageChange(value === "" ? null : (value as string)),
        },
      ],
    },
  ];

  return (
    <SettingsSurface
      sections={sections}
      description={t("routes:settingsLanguage.description")}
    />
  );
}