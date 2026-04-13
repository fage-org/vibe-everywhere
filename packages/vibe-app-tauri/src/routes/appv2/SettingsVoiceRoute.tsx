import { useTranslation } from "react-i18next";
import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useLocalSettings } from "../../local-settings";
import { useSpeechSynthesis } from "../../hooks/useSpeechSynthesis";

/**
 * SettingsVoiceRoute - Voice settings page
 *
 * Manages voice input/output settings stored in local localStorage.
 * Includes speech recognition and synthesis configuration.
 */
export function SettingsVoiceRoute() {
  const { t } = useTranslation("ui");
  const { t: tRoutes } = useTranslation("routes");
  const { settings, updateSettings } = useLocalSettings();
  const { voices, isSupported: ttsSupported } = useSpeechSynthesis({
    language: settings.voiceLanguage,
  });

  // Map voice language options
  const languageOptions = [
    { label: "English (US)", value: "en-US" },
    { label: "English (UK)", value: "en-GB" },
    { label: "中文（简体）", value: "zh-CN" },
    { label: "中文（繁體）", value: "zh-TW" },
    { label: "日本語", value: "ja-JP" },
    { label: "한국어", value: "ko-KR" },
    { label: "Français", value: "fr-FR" },
    { label: "Deutsch", value: "de-DE" },
    { label: "Español", value: "es-ES" },
    { label: "Italiano", value: "it-IT" },
    { label: "Português", value: "pt-BR" },
    { label: "Русский", value: "ru-RU" },
  ];

  // Map available voices for the selected language
  const voiceOptions = voices
    .filter((v) => v.lang.startsWith(settings.voiceLanguage.split("-")[0]))
    .map((v) => ({
      label: v.name,
      value: v.name,
    }));

  const sections: SettingSection[] = [
    {
      id: "voice-input",
      title: t("voice.settings.inputEnabled"),
      description: t("voice.settings.inputEnabledDescription"),
      settings: [
        {
          id: "voiceInputEnabled",
          label: t("voice.settings.inputEnabled"),
          description: t("voice.settings.inputEnabledDescription"),
          type: "toggle",
          value: settings.voiceInputEnabled,
          onChange: (value) => updateSettings({ voiceInputEnabled: value as boolean }),
        },
      ],
    },
    {
      id: "voice-output",
      title: t("voice.settings.outputEnabled"),
      description: t("voice.settings.outputEnabledDescription"),
      settings: [
        {
          id: "voiceOutputEnabled",
          label: t("voice.settings.outputEnabled"),
          description: t("voice.settings.outputEnabledDescription"),
          type: "toggle",
          value: settings.voiceOutputEnabled,
          onChange: (value) => updateSettings({ voiceOutputEnabled: value as boolean }),
        },
        {
          id: "voiceAutoPlay",
          label: t("voice.settings.autoPlay"),
          description: t("voice.settings.autoPlayDescription"),
          type: "toggle",
          value: settings.voiceAutoPlay,
          onChange: (value) => updateSettings({ voiceAutoPlay: value as boolean }),
          disabled: !settings.voiceOutputEnabled,
        },
      ],
    },
    {
      id: "voice-language",
      title: t("voice.settings.language"),
      description: t("voice.settings.languageDescription"),
      settings: [
        {
          id: "voiceLanguage",
          label: t("voice.settings.language"),
          description: t("voice.settings.languageDescription"),
          type: "select",
          value: settings.voiceLanguage,
          options: languageOptions,
          onChange: (value) => updateSettings({ voiceLanguage: value as string }),
        },
      ],
    },
    {
      id: "voice-synthesis",
      title: tRoutes("settingsAppearance.display"),
      description: tRoutes("settingsAppearance.displayDescription"),
      settings: [
        {
          id: "voiceRate",
          label: t("voice.settings.rate"),
          description: t("voice.settings.rateDescription"),
          type: "number",
          value: settings.voiceRate,
          onChange: (value) => updateSettings({ voiceRate: value as number }),
          disabled: !settings.voiceOutputEnabled,
        },
        {
          id: "voicePitch",
          label: t("voice.settings.pitch"),
          description: t("voice.settings.pitchDescription"),
          type: "number",
          value: settings.voicePitch,
          onChange: (value) => updateSettings({ voicePitch: value as number }),
          disabled: !settings.voiceOutputEnabled,
        },
      ],
    },
  ];

  // Add voice selection if TTS is supported and voices are available
  if (ttsSupported && voiceOptions.length > 0) {
    sections[3].settings.push({
      id: "voiceName",
      label: t("voice.settings.voice"),
      description: t("voice.settings.voiceDescription"),
      type: "select",
      value: "",
      options: voiceOptions,
      onChange: (value) => {
        // Voice name is handled separately in useSpeechSynthesis
        // This just provides UI for selection
      },
      disabled: !settings.voiceOutputEnabled,
    });
  }

  return (
    <SettingsSurface
      sections={sections}
      description={t("voice.settings.title")}
    />
  );
}

export default SettingsVoiceRoute;