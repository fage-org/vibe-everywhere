export type SupportedLanguage =
  | "en"
  | "ru"
  | "pl"
  | "es"
  | "it"
  | "pt"
  | "ca"
  | "zh-Hans"
  | "zh-Hant"
  | "ja";

export interface LanguageInfo {
  code: SupportedLanguage;
  nativeName: string;
  englishName: string;
}

export const SUPPORTED_LANGUAGES: Record<SupportedLanguage, LanguageInfo> = {
  en: { code: "en", nativeName: "English", englishName: "English" },
  ru: { code: "ru", nativeName: "Русский", englishName: "Russian" },
  pl: { code: "pl", nativeName: "Polski", englishName: "Polish" },
  es: { code: "es", nativeName: "Español", englishName: "Spanish" },
  it: { code: "it", nativeName: "Italiano", englishName: "Italian" },
  pt: { code: "pt", nativeName: "Português", englishName: "Portuguese" },
  ca: { code: "ca", nativeName: "Català", englishName: "Catalan" },
  "zh-Hans": {
    code: "zh-Hans",
    nativeName: "中文(简体)",
    englishName: "Chinese (Simplified)",
  },
  "zh-Hant": {
    code: "zh-Hant",
    nativeName: "中文(繁體)",
    englishName: "Chinese (Traditional)",
  },
  ja: { code: "ja", nativeName: "日本語", englishName: "Japanese" },
} as const;

export const SUPPORTED_LANGUAGE_CODES = Object.keys(SUPPORTED_LANGUAGES) as SupportedLanguage[];
export const DEFAULT_LANGUAGE: SupportedLanguage = "en";

export function getLanguageNativeName(code: SupportedLanguage): string {
  return SUPPORTED_LANGUAGES[code].nativeName;
}

export function getLanguageEnglishName(code: SupportedLanguage): string {
  return SUPPORTED_LANGUAGES[code].englishName;
}
