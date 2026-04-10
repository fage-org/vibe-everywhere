import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { SupportedLanguage, Language } from '../i18n/types';
import { SUPPORTED_LANGUAGES, DEFAULT_LANGUAGE } from '../i18n/types';
import { changeLanguage, getCurrentLanguage, isSupportedLanguage } from '../i18n';

export interface UseLanguageReturn {
  /** Current language code */
  language: SupportedLanguage;
  /** Full language metadata */
  currentLanguage: Language;
  /** All supported languages */
  languages: Language[];
  /** Change language */
  setLanguage: (lng: SupportedLanguage) => Promise<void>;
  /** Check if language is supported */
  isSupported: (lng: string) => boolean;
  /** Translation function */
  t: (key: string, options?: Record<string, unknown>) => string;
  /** i18n instance */
  i18n: ReturnType<typeof useTranslation>['i18n'];
}

/**
 * Hook for managing language settings
 */
export function useLanguage(): UseLanguageReturn {
  const { t, i18n } = useTranslation();

  const language = getCurrentLanguage();

  const currentLanguage = SUPPORTED_LANGUAGES.find(l => l.code === language) ||
    SUPPORTED_LANGUAGES.find(l => l.code === DEFAULT_LANGUAGE)!;

  const setLanguage = useCallback(async (lng: SupportedLanguage) => {
    if (isSupportedLanguage(lng)) {
      await changeLanguage(lng);
    }
  }, []);

  return {
    language,
    currentLanguage,
    languages: SUPPORTED_LANGUAGES,
    setLanguage,
    isSupported: isSupportedLanguage,
    t,
    i18n,
  };
}
