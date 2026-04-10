/**
 * i18n TypeScript types for type-safe translations
 */

// Supported languages
export type SupportedLanguage = 'en' | 'zh-CN';

// Translation namespaces
export type TranslationNamespace =
  | 'common'
  | 'routes'
  | 'components'
  | 'settings'
  | 'ui';

// Translation keys structure
export interface TranslationKeys {
  // Common
  'common.app.name': string;
  'common.app.tagline': string;
  'common.loading': string;
  'common.error': string;
  'common.retry': string;
  'common.cancel': string;
  'common.save': string;
  'common.delete': string;
  'common.edit': string;
  'common.close': string;
  'common.back': string;
  'common.next': string;
  'common.done': string;
  'common.search': string;
  'common.filter': string;
  'common.sort': string;
  'common.empty': string;
  'common.noResults': string;

  // Routes - Home
  'routes.home.title': string;
  'routes.home.subtitle': string;
  'routes.home.welcome': string;
  'routes.home.description': string;
  'routes.home.actions.newSession': string;
  'routes.home.actions.resume': string;
  'routes.home.actions.settings': string;
  'routes.home.sections.quickActions': string;
  'routes.home.sections.recentSessions': string;
  'routes.home.sections.stats': string;
  'routes.home.actions.viewAll': string;

  // Routes - Session
  'routes.session.header.eyebrow': string;
  'routes.session.emptyState.title': string;
  'routes.session.emptyState.subtitle': string;
  'routes.session.timeline.empty.title': string;
  'routes.session.timeline.empty.subtitle': string;
  'routes.session.composer.placeholder': string;
  'routes.session.composer.hint': string;
  'routes.session.models.selectPlaceholder': string;
  'routes.session.actions.share': string;
  'routes.session.actions.export': string;
  'routes.session.actions.rename': string;
  'routes.session.actions.delete': string;

  // Routes - Inbox
  'routes.inbox.header.eyebrow': string;
  'routes.inbox.title': string;
  'routes.inbox.unreadCount': string;
  'routes.inbox.actions.markAllRead': string;
  'routes.inbox.filters.all': string;
  'routes.inbox.filters.unread': string;
  'routes.inbox.empty.title': string;
  'routes.inbox.empty.subtitle': string;

  // Routes - Settings
  'routes.settings.title': string;
  'routes.settings.sections.general': string;
  'routes.settings.sections.language': string;
  'routes.settings.sections.appearance': string;
  'routes.settings.sections.notifications': string;
  'routes.settings.sections.account': string;
  'routes.settings.sections.about': string;
  'routes.settings.actions.reset': string;
  'routes.settings.actions.saveChanges': string;
  'routes.settings.language.title': string;
  'routes.settings.language.description': string;
  'routes.settings.language.auto': string;

  // Components - SessionList
  'components.sessionList.status.active': string;
  'components.sessionList.status.idle': string;
  'components.sessionList.status.completed': string;
  'components.sessionList.status.error': string;
  'components.sessionList.empty.title': string;
  'components.sessionList.empty.subtitle': string;
  'components.sessionList.search.placeholder': string;

  // Components - Timeline
  'components.timeline.date.today': string;
  'components.timeline.date.yesterday': string;

  // Components - Composer
  'components.composer.placeholder': string;
  'components.composer.charCount': string;
  'components.composer.send': string;
  'components.composer.stop': string;

  // Components - Navigation
  'components.nav.home': string;
  'components.nav.sessions': string;
  'components.nav.inbox': string;
  'components.nav.settings': string;

  // Settings
  'settings.theme.title': string;
  'settings.theme.light': string;
  'settings.theme.dark': string;
  'settings.theme.system': string;
  'settings.notifications.title': string;
  'settings.notifications.desktop': string;
  'settings.notifications.sound': string;

  // UI
  'ui.badges.beta': string;
  'ui.badges.new': string;
  'ui.badges.pro': string;
  'ui.buttons.save': string;
  'ui.buttons.cancel': string;
  'ui.buttons.confirm': string;
  'ui.buttons.delete': string;
  'ui.connection.connected': string;
  'ui.connection.disconnected': string;
  'ui.connection.connecting': string;
}

// Type for translation function
export type TFunction = (key: keyof TranslationKeys, options?: Record<string, unknown>) => string;

// Language metadata
export interface Language {
  code: SupportedLanguage;
  name: string;
  nativeName: string;
  flag: string;
}

export const SUPPORTED_LANGUAGES: Language[] = [
  { code: 'en', name: 'English', nativeName: 'English', flag: '🇺🇸' },
  { code: 'zh-CN', name: 'Chinese (Simplified)', nativeName: '简体中文', flag: '🇨🇳' },
];

export const DEFAULT_LANGUAGE: SupportedLanguage = 'en';
export const FALLBACK_LANGUAGE: SupportedLanguage = 'en';
