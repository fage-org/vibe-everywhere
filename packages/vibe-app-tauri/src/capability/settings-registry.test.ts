/**
 * Tests for Settings Registry
 *
 * B28: Settings and Connection Center - Phase 1 Tests
 */

import { describe, expect, it } from "vitest";
import { CapabilityClass } from './types';
import {
  settingsSurfaces,
  settingsRegistry,
  getSettingsByClass,
  getSettingsByPlatformCapability,
  getCustomerFacingSettings,
  getSettingsSummary,
} from './settings-registry';

describe('Settings Registry', () => {
  it('should have all settings surfaces defined', () => {
    expect(settingsSurfaces.length).toBeGreaterThan(0);
    expect(settingsSurfaces.length).toBe(7); // 7 settings surfaces
  });

  it('should have unique surface IDs', () => {
    const ids = settingsSurfaces.map((s) => s.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });

  it('should have valid capability classes', () => {
    for (const surface of settingsSurfaces) {
      expect(Object.values(CapabilityClass)).toContain(surface.capabilityClass);
    }
  });

  it('should have customer descriptions for customer-facing surfaces', () => {
    const customerFacingClasses = [
      CapabilityClass.FullySupported,
      CapabilityClass.Limited,
      CapabilityClass.ReadOnly,
    ];

    for (const surface of settingsSurfaces) {
      if (customerFacingClasses.includes(surface.capabilityClass)) {
        expect(surface.customerDescription).toBeTruthy();
        expect(surface.customerDescription!.length).toBeGreaterThan(0);
      }
    }
  });

  it('should have platform capabilities defined for all surfaces', () => {
    for (const surface of settingsSurfaces) {
      expect(surface.platformCapabilities).toBeDefined();
      expect(surface.platformCapabilities!.desktop).toBeDefined();
      expect(surface.platformCapabilities!.android).toBeDefined();
      expect(surface.platformCapabilities!.browser).toBeDefined();
    }
  });
});

describe('getSettingsByClass', () => {
  it('should return only FullySupported settings', () => {
    const fullySupported = getSettingsByClass(CapabilityClass.FullySupported);
    expect(fullySupported.length).toBeGreaterThan(0);
    for (const setting of fullySupported) {
      expect(setting.capabilityClass).toBe(CapabilityClass.FullySupported);
    }
  });

  it('should return only Limited settings', () => {
    const limited = getSettingsByClass(CapabilityClass.Limited);
    expect(limited.length).toBeGreaterThanOrEqual(0);
    for (const setting of limited) {
      expect(setting.capabilityClass).toBe(CapabilityClass.Limited);
    }
  });

  it('should return empty array for unsupported class if none exist', () => {
    const unsupported = getSettingsByClass(CapabilityClass.Unsupported);
    expect(unsupported).toEqual([]);
  });
});

describe('getSettingsByPlatformCapability', () => {
  it('should return desktop FullySupported settings', () => {
    const desktopFullySupported = getSettingsByPlatformCapability(
      'desktop',
      CapabilityClass.FullySupported
    );
    expect(desktopFullySupported.length).toBeGreaterThan(0);
  });

  it('should return android Limited settings', () => {
    const androidLimited = getSettingsByPlatformCapability(
      'android',
      CapabilityClass.Limited
    );
    // May be 0 or more depending on configuration
    expect(androidLimited.length).toBeGreaterThanOrEqual(0);
  });
});

describe('getCustomerFacingSettings', () => {
  it('should return only customer-facing settings', () => {
    const customerFacing = getCustomerFacingSettings();
    expect(customerFacing.length).toBeGreaterThan(0);

    const customerFacingClasses = [
      CapabilityClass.FullySupported,
      CapabilityClass.Limited,
      CapabilityClass.ReadOnly,
    ];

    for (const setting of customerFacing) {
      expect(customerFacingClasses).toContain(setting.capabilityClass);
    }
  });

  it('should not include internal or unsupported settings', () => {
    const customerFacing = getCustomerFacingSettings();
    const ids = customerFacing.map((s) => s.id);

    // Should not include developer settings (internal)
    expect(ids).not.toContain('settings-developer');
  });
});

describe('getSettingsSummary', () => {
  it('should return summary statistics', () => {
    const summary = getSettingsSummary();

    expect(summary.total).toBe(settingsSurfaces.length);
    expect(summary.customerFacing).toBeGreaterThan(0);
    expect(summary.internal).toBeGreaterThanOrEqual(0);
    expect(Object.keys(summary.byClass).length).toBe(
      Object.values(CapabilityClass).length
    );
  });

  it('should have correct totals in byClass', () => {
    const summary = getSettingsSummary();
    let totalFromByClass = 0;

    for (const count of Object.values(summary.byClass)) {
      totalFromByClass += count;
    }

    expect(totalFromByClass).toBe(summary.total);
  });
});

describe('settingsRegistry', () => {
  it('should have correct version', () => {
    expect(settingsRegistry.version).toBe('1.0.0');
  });

  it('should have all surfaces', () => {
    expect(settingsRegistry.surfaces).toEqual(settingsSurfaces);
  });

  it('should have lastUpdated date', () => {
    expect(settingsRegistry.lastUpdated).toBeInstanceOf(Date);
  });
});
