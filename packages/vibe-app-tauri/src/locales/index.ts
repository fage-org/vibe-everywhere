import type { Resource } from 'i18next';
import enCommon from './en/common.json';
import enRoutes from './en/routes.json';
import enComponents from './en/components.json';
import enSettings from './en/settings.json';
import enUi from './en/ui.json';
import zhCommon from './zh-CN/common.json';
import zhRoutes from './zh-CN/routes.json';
import zhComponents from './zh-CN/components.json';
import zhSettings from './zh-CN/settings.json';
import zhUi from './zh-CN/ui.json';

export const resources: Resource = {
  en: {
    common: enCommon,
    routes: enRoutes,
    components: enComponents,
    settings: enSettings,
    ui: enUi,
  },
  'zh-CN': {
    common: zhCommon,
    routes: zhRoutes,
    components: zhComponents,
    settings: zhSettings,
    ui: zhUi,
  },
};

export type Resources = typeof resources;
