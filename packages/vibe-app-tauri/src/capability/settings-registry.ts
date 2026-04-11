/**
 * Settings Surface Registry
 *
 * B28: Settings and Connection Center - Phase 1
 *
 * This module defines all settings surfaces with their capability classifications
 * according to the Wave 10 capability contract.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityRegistry,
} from './types';

/**
 * Settings surface definitions
 */
export const settingsSurfaces: SurfaceCapability[] = [
  // Account Settings
  {
    id: 'settings-account',
    name: 'Account Settings',
    route: '/settings/account',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'Manage your account information, profile, and security settings',
    internalNotes: 'Core settings surface - account/profile management',
  },

  // Appearance Settings
  {
    id: 'settings-appearance',
    name: 'Appearance Settings',
    route: '/settings/appearance',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'Customize the look and feel of the application',
    internalNotes: 'Theme, density, and visual preference settings',
  },

  // Language Settings
  {
    id: 'settings-language',
    name: 'Language Settings',
    route: '/settings/language',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.FullySupported,
    },
    customerDescription: 'Set your preferred language for the application interface',
    internalNotes: 'App language/locale settings',
  },

  // Connection Center (formerly vendor/connect settings)
  {
    id: 'settings-connections',
    name: 'Connection Center',
    route: '/settings/connections',
    capabilityClass: CapabilityClass.Limited,
    platformCapabilities: {
      desktop: CapabilityClass.Limited,
      android: CapabilityClass.HandoffOnly,
      browser: CapabilityClass.HandoffOnly,
    },
    customerDescription: 'Manage connections to external services and integrations',
    internalNotes: 'Connection management with handoff to external auth flows',
  },

  // Voice Settings
  {
    id: 'settings-voice',
    name: 'Voice Settings',
    route: '/settings/voice',
    capabilityClass: CapabilityClass.Limited,
    platformCapabilities: {
      desktop: CapabilityClass.Limited,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.Unsupported,
    },
    customerDescription: 'Configure voice assistant and audio settings',
    internalNotes: 'Voice assistant configuration - mobile-first feature',
  },

  // Usage/Plan Settings
  {
    id: 'settings-usage',
    name: 'Usage & Plan',
    route: '/settings/usage',
    capabilityClass: CapabilityClass.ReadOnly,
    platformCapabilities: {
      desktop: CapabilityClass.ReadOnly,
      android: CapabilityClass.ReadOnly,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'View your usage statistics and subscription plan',
    internalNotes: 'Usage dashboard - read-only analytics',
  },

  // Developer Settings (internal)
  {
    id: 'settings-developer',
    name: 'Developer Settings',
    route: '/settings/developer',
    capabilityClass: CapabilityClass.Internal,
    platformCapabilities: {
      desktop: CapabilityClass.Internal,
      android: CapabilityClass.Unsupported,
      browser: CapabilityClass.Unsupported,
    },
    customerDescription: '', // Internal - no customer-facing description
    internalNotes: 'Developer tools and diagnostics - not for end users',
  },
];

/**
 * Settings registry
 */
export const settingsRegistry: CapabilityRegistry = {
  version: '1.0.0',
  surfaces: settingsSurfaces,
  lastUpdated: new Date(),
};

/**
 * Get settings surfaces by capability class
 */
export function getSettingsByClass(
  capabilityClass: CapabilityClass
): SurfaceCapability[] {
  return settingsSurfaces.filter(
    (surface) => surface.capabilityClass === capabilityClass
  );
}

/**
 * Get settings surfaces by platform capability
 */
export function getSettingsByPlatformCapability(
  platform: 'desktop' | 'android' | 'browser',
  capabilityClass: CapabilityClass
): SurfaceCapability[] {
  return settingsSurfaces.filter((surface) => {
    const platformCapability = surface.platformCapabilities?.[platform];
    return platformCapability === capabilityClass;
  });
}

/**
 * Get customer-facing settings surfaces
 */
export function getCustomerFacingSettings(): SurfaceCapability[] {
  return settingsSurfaces.filter((surface) => {
    const customerFacingClasses = [
      CapabilityClass.FullySupported,
      CapabilityClass.Limited,
      CapabilityClass.ReadOnly,
    ];
    return customerFacingClasses.includes(surface.capabilityClass);
  });
}

/**
 * Get settings summary statistics
 */
export function getSettingsSummary(): {
  total: number;
  byClass: Record<CapabilityClass, number>;
  customerFacing: number;
  internal: number;
} {
  const byClass = {} as Record<CapabilityClass, number>;

  for (const cls of Object.values(CapabilityClass)) {
    byClass[cls] = settingsSurfaces.filter(
      (s) => s.capabilityClass === cls
    ).length;
  }

  return {
    total: settingsSurfaces.length,
    byClass,
    customerFacing: getCustomerFacingSettings().length,
    internal: settingsSurfaces.filter(
      (s) => s.capabilityClass === CapabilityClass.Internal
    ).length,
  };
}

export default {
  settingsSurfaces,
  settingsRegistry,
  getSettingsByClass,
  getSettingsByPlatformCapability,
  getCustomerFacingSettings,
  getSettingsSummary,
};
