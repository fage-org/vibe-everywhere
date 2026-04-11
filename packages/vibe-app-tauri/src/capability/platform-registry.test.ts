/**
 * Tests for Platform Registry
 *
 * B31: Platform Parity and Browser Contract
 */

import { describe, expect, it } from "vitest";
import { CapabilityClass } from './types';
import {
  PlatformSupportLevel,
  BrowserContractType,
  type PlatformSupportMatrixEntry,
  capabilityToSupportLevel,
  getAllSurfaces,
  getPlatformSupportMatrix,
  getPlatformParitySummary,
  isSupportedOnPlatform,
  getBrowserContractDescription,
} from './platform-registry';

// Test data
const testSurfaces = [
  {
    id: 'test-fully-supported',
    name: 'Test Fully Supported',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.ReadOnly,
    },
  },
  {
    id: 'test-limited',
    name: 'Test Limited',
    capabilityClass: CapabilityClass.Limited,
    platformCapabilities: {
      desktop: CapabilityClass.Limited,
      android: CapabilityClass.HandoffOnly,
      browser: CapabilityClass.Unsupported,
    },
  },
  {
    id: 'test-internal',
    name: 'Test Internal',
    capabilityClass: CapabilityClass.Internal,
    // No platformCapabilities - should use base capabilityClass
  },
];

describe('PlatformSupportLevel Enum', () => {
  it('should have all expected values', () => {
    expect(PlatformSupportLevel.Complete).toBe('complete');
    expect(PlatformSupportLevel.Limited).toBe('limited');
    expect(PlatformSupportLevel.ReadOnly).toBe('read_only');
    expect(PlatformSupportLevel.HandoffOnly).toBe('handoff_only');
    expect(PlatformSupportLevel.Unsupported).toBe('unsupported');
  });
});

describe('BrowserContractType Enum', () => {
  it('should have all expected values', () => {
    expect(BrowserContractType.FullApplication).toBe('full_application');
    expect(BrowserContractType.LimitedFeatures).toBe('limited_features');
    expect(BrowserContractType.ReadOnlyView).toBe('read_only_view');
    expect(BrowserContractType.StaticExport).toBe('static_export');
    expect(BrowserContractType.NotApplicable).toBe('not_applicable');
  });
});

describe('capabilityToSupportLevel', () => {
  it('should map FullySupported to Complete', () => {
    expect(capabilityToSupportLevel(CapabilityClass.FullySupported)).toBe(
      PlatformSupportLevel.Complete
    );
  });

  it('should map Limited to Limited', () => {
    expect(capabilityToSupportLevel(CapabilityClass.Limited)).toBe(
      PlatformSupportLevel.Limited
    );
  });

  it('should map ReadOnly to ReadOnly', () => {
    expect(capabilityToSupportLevel(CapabilityClass.ReadOnly)).toBe(
      PlatformSupportLevel.ReadOnly
    );
  });

  it('should map HandoffOnly to HandoffOnly', () => {
    expect(capabilityToSupportLevel(CapabilityClass.HandoffOnly)).toBe(
      PlatformSupportLevel.HandoffOnly
    );
  });

  it('should map Internal to Unsupported', () => {
    expect(capabilityToSupportLevel(CapabilityClass.Internal)).toBe(
      PlatformSupportLevel.Unsupported
    );
  });

  it('should map Unsupported to Unsupported', () => {
    expect(capabilityToSupportLevel(CapabilityClass.Unsupported)).toBe(
      PlatformSupportLevel.Unsupported
    );
  });
});

describe('getPlatformSupportMatrix', () => {
  it('should return matrix for all surfaces', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);

    expect(matrix.length).toBe(testSurfaces.length);
  });

  it('should correctly map capability classes to support levels', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);
    const fullySupportedEntry = matrix.find(
      (e) => e.surfaceId === 'test-fully-supported'
    );

    expect(fullySupportedEntry).toBeDefined();
    expect(fullySupportedEntry!.desktop).toBe(PlatformSupportLevel.Complete);
    expect(fullySupportedEntry!.android).toBe(PlatformSupportLevel.Limited);
    expect(fullySupportedEntry!.browser).toBe(PlatformSupportLevel.ReadOnly);
  });

  it('should use base capabilityClass when platformCapabilities not defined', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);
    const internalEntry = matrix.find((e) => e.surfaceId === 'test-internal');

    expect(internalEntry).toBeDefined();
    expect(internalEntry!.desktop).toBe(PlatformSupportLevel.Unsupported);
    expect(internalEntry!.android).toBe(PlatformSupportLevel.Unsupported);
    expect(internalEntry!.browser).toBe(PlatformSupportLevel.Unsupported);
  });

  it('should determine browser contract type correctly', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);

    const fullySupportedEntry = matrix.find(
      (e) => e.surfaceId === 'test-fully-supported'
    );
    expect(fullySupportedEntry!.browserContract).toBe(
      BrowserContractType.ReadOnlyView
    );

    const limitedEntry = matrix.find((e) => e.surfaceId === 'test-limited');
    expect(limitedEntry!.browserContract).toBe(
      BrowserContractType.NotApplicable
    );
  });
});

describe('getPlatformParitySummary', () => {
  it('should return summary with all fields', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);
    const summary = getPlatformParitySummary(matrix);

    expect(summary.totalSurfaces).toBe(testSurfaces.length);
    expect(summary.desktopComplete).toBeDefined();
    expect(summary.desktopLimited).toBeDefined();
    expect(summary.androidComplete).toBeDefined();
    expect(summary.androidLimited).toBeDefined();
    expect(summary.browserComplete).toBeDefined();
    expect(summary.browserLimited).toBeDefined();
    expect(summary.browserReadOnly).toBeDefined();
    expect(summary.browserHandoff).toBeDefined();
    expect(summary.mobileUnsupported).toBeDefined();
    expect(summary.browserUnsupported).toBeDefined();
  });

  it('should calculate correct counts', () => {
    const matrix = getPlatformSupportMatrix(testSurfaces);
    const summary = getPlatformParitySummary(matrix);

    // test-fully-supported: Complete on desktop, Limited on android, ReadOnly on browser
    // test-limited: Limited on desktop, HandoffOnly on android, Unsupported on browser
    // test-internal: Unsupported on all platforms

    expect(summary.desktopComplete).toBe(1); // test-fully-supported
    expect(summary.desktopLimited).toBe(1); // test-limited
    expect(summary.androidLimited).toBe(1); // test-fully-supported
    expect(summary.browserReadOnly).toBe(1); // test-fully-supported
    expect(summary.browserUnsupported).toBe(2); // test-limited, test-internal
  });
});

describe('isSupportedOnPlatform', () => {
  it('should return true for Complete support level', () => {
    const entry: PlatformSupportMatrixEntry = {
      surfaceId: 'test',
      desktop: PlatformSupportLevel.Complete,
      android: PlatformSupportLevel.Unsupported,
      browser: PlatformSupportLevel.Unsupported,
      browserContract: BrowserContractType.NotApplicable,
    };

    expect(isSupportedOnPlatform(entry, 'desktop')).toBe(true);
  });

  it('should return true for Limited support level', () => {
    const entry: PlatformSupportMatrixEntry = {
      surfaceId: 'test',
      desktop: PlatformSupportLevel.Limited,
      android: PlatformSupportLevel.Unsupported,
      browser: PlatformSupportLevel.Unsupported,
      browserContract: BrowserContractType.NotApplicable,
    };

    expect(isSupportedOnPlatform(entry, 'desktop')).toBe(true);
  });

  it('should return true for ReadOnly support level', () => {
    const entry: PlatformSupportMatrixEntry = {
      surfaceId: 'test',
      desktop: PlatformSupportLevel.ReadOnly,
      android: PlatformSupportLevel.Unsupported,
      browser: PlatformSupportLevel.Unsupported,
      browserContract: BrowserContractType.NotApplicable,
    };

    expect(isSupportedOnPlatform(entry, 'desktop')).toBe(true);
  });

  it('should return false for HandoffOnly support level', () => {
    const entry: PlatformSupportMatrixEntry = {
      surfaceId: 'test',
      desktop: PlatformSupportLevel.HandoffOnly,
      android: PlatformSupportLevel.Unsupported,
      browser: PlatformSupportLevel.Unsupported,
      browserContract: BrowserContractType.NotApplicable,
    };

    expect(isSupportedOnPlatform(entry, 'desktop')).toBe(false);
  });

  it('should return false for Unsupported support level', () => {
    const entry: PlatformSupportMatrixEntry = {
      surfaceId: 'test',
      desktop: PlatformSupportLevel.Unsupported,
      android: PlatformSupportLevel.Unsupported,
      browser: PlatformSupportLevel.Unsupported,
      browserContract: BrowserContractType.NotApplicable,
    };

    expect(isSupportedOnPlatform(entry, 'desktop')).toBe(false);
  });
});

describe('getBrowserContractDescription', () => {
  it('should return correct description for FullApplication', () => {
    expect(getBrowserContractDescription(BrowserContractType.FullApplication)).toBe(
      'Full interactive application with all features'
    );
  });

  it('should return correct description for LimitedFeatures', () => {
    expect(getBrowserContractDescription(BrowserContractType.LimitedFeatures)).toBe(
      'Limited feature set with core functionality'
    );
  });

  it('should return correct description for ReadOnlyView', () => {
    expect(getBrowserContractDescription(BrowserContractType.ReadOnlyView)).toBe(
      'Read-only view with no editing capabilities'
    );
  });

  it('should return correct description for StaticExport', () => {
    expect(getBrowserContractDescription(BrowserContractType.StaticExport)).toBe(
      'Static export with limited interactivity'
    );
  });

  it('should return correct description for NotApplicable', () => {
    expect(getBrowserContractDescription(BrowserContractType.NotApplicable)).toBe(
      'Not applicable for browser platform'
    );
  });
});

describe('getAllSurfaces', () => {
  it('should return all surfaces from all registries', () => {
    const allSurfaces = getAllSurfaces();

    // Should have surfaces from all registries
    expect(allSurfaces.length).toBeGreaterThan(0);

    // Should include settings surfaces
    const settingsSurface = allSurfaces.find((s) => s.id === 'settings-account');
    expect(settingsSurface).toBeDefined();

    // Should include inbox surfaces
    const inboxSurface = allSurfaces.find((s) => s.id === 'inbox-list');
    expect(inboxSurface).toBeDefined();

    // Should include remote surfaces
    const remoteSurface = allSurfaces.find((s) => s.id === 'terminal-connect');
    expect(remoteSurface).toBeDefined();
  });
});

describe('Integration: Complete platform matrix', () => {
  it('should generate complete platform support matrix for all surfaces', () => {
    const allSurfaces = getAllSurfaces();
    const matrix = getPlatformSupportMatrix(allSurfaces);
    const summary = getPlatformParitySummary(matrix);

    // Verify summary has all required fields
    expect(summary.totalSurfaces).toBe(allSurfaces.length);
    expect(summary.desktopComplete).toBeGreaterThanOrEqual(0);
    expect(summary.desktopLimited).toBeGreaterThanOrEqual(0);
    expect(summary.androidComplete).toBeGreaterThanOrEqual(0);
    expect(summary.androidLimited).toBeGreaterThanOrEqual(0);
    expect(summary.browserComplete).toBeGreaterThanOrEqual(0);
    expect(summary.browserLimited).toBeGreaterThanOrEqual(0);
    expect(summary.browserHandoff).toBeGreaterThanOrEqual(0);
    expect(summary.mobileUnsupported).toBeGreaterThanOrEqual(0);
    expect(summary.browserUnsupported).toBeGreaterThanOrEqual(0);
  });

  it('should have valid platform support matrix entries for all surfaces', () => {
    const allSurfaces = getAllSurfaces();
    const matrix = getPlatformSupportMatrix(allSurfaces);

    for (const entry of matrix) {
      // All entries should have required fields
      expect(entry.surfaceId).toBeDefined();
      expect(entry.desktop).toBeDefined();
      expect(entry.android).toBeDefined();
      expect(entry.browser).toBeDefined();
      expect(entry.browserContract).toBeDefined();

      // All support levels should be valid
      expect(Object.values(PlatformSupportLevel)).toContain(entry.desktop);
      expect(Object.values(PlatformSupportLevel)).toContain(entry.android);
      expect(Object.values(PlatformSupportLevel)).toContain(entry.browser);

      // Browser contract should be valid
      expect(Object.values(BrowserContractType)).toContain(entry.browserContract);
    }
  });
});
