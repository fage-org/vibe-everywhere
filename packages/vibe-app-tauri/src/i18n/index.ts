import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { resources } from '../locales';
import { DEFAULT_LANGUAGE, FALLBACK_LANGUAGE, type SupportedLanguage } from './types';

// Language detector options
const languageDetectorOptions = {
  order: ['localStorage', 'navigator', 'htmlTag'],
  lookupLocalStorage: 'vibe-language',
  caches: ['localStorage'],
  convertDetectedLanguage: (lng: string): string => {
    // Map browser language codes to our supported languages
    if (lng.startsWith('zh')) {
      return 'zh-CN';
    }
    return 'en';
  },
};

// Initialize i18n
void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: FALLBACK_LANGUAGE,
    supportedLngs: ['en', 'zh-CN'],
    defaultNS: 'common',
    ns: ['common', 'routes', 'components', 'settings', 'ui'],
    detection: languageDetectorOptions,
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    react: {
      useSuspense: false, // Disable suspense for SSR compatibility
    },
  });

// Export initialized instance
export default i18n;

// Helper functions
export function changeLanguage(lng: SupportedLanguage): Promise<void> {
  return i18n.changeLanguage(lng).then(() => undefined);
}

export function getCurrentLanguage(): SupportedLanguage {
  return (i18n.language as SupportedLanguage) || DEFAULT_LANGUAGE;
}

export function isSupportedLanguage(lng: string): lng is SupportedLanguage {
  return ['en', 'zh-CN'].includes(lng);
}
