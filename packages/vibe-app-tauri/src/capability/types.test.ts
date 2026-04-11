/**
 * Tests for Capability Classification System
 *
 * These tests verify the core types, enums, and helper functions
 * used throughout Wave 10 for capability classification.
 */

import { describe, expect, it } from "vitest";
import {
  CapabilityClass,
  isCustomerFacing,
  isMutable,
  getPlatformCapability,
  type SurfaceCapability,
  type CapabilityEvidence,
  type PlatformScope,
} from './types';

describe('CapabilityClass Enum', () => {
  it('should have all expected values', () => {
    expect(CapabilityClass.FullySupported).toBe('fully_supported');
    expect(CapabilityClass.Limited).toBe('limited');
    expect(CapabilityClass.HandoffOnly).toBe('handoff_only');
    expect(CapabilityClass.ReadOnly).toBe('read_only');
    expect(CapabilityClass.Internal).toBe('internal');
    expect(CapabilityClass.Unsupported).toBe('unsupported');
  });
});

describe('isCustomerFacing', () => {
  it('should return true for FullySupported', () => {
    expect(isCustomerFacing(CapabilityClass.FullySupported)).toBe(true);
  });

  it('should return true for Limited', () => {
    expect(isCustomerFacing(CapabilityClass.Limited)).toBe(true);
  });

  it('should return true for ReadOnly', () => {
    expect(isCustomerFacing(CapabilityClass.ReadOnly)).toBe(true);
  });

  it('should return false for HandoffOnly', () => {
    expect(isCustomerFacing(CapabilityClass.HandoffOnly)).toBe(false);
  });

  it('should return false for Internal', () => {
    expect(isCustomerFacing(CapabilityClass.Internal)).toBe(false);
  });

  it('should return false for Unsupported', () => {
    expect(isCustomerFacing(CapabilityClass.Unsupported)).toBe(false);
  });
});

describe('isMutable', () => {
  it('should return true for FullySupported', () => {
    expect(isMutable(CapabilityClass.FullySupported)).toBe(true);
  });

  it('should return true for Limited', () => {
    expect(isMutable(CapabilityClass.Limited)).toBe(true);
  });

  it('should return false for ReadOnly', () => {
    expect(isMutable(CapabilityClass.ReadOnly)).toBe(false);
  });

  it('should return false for HandoffOnly', () => {
    expect(isMutable(CapabilityClass.HandoffOnly)).toBe(false);
  });
});

describe('getPlatformCapability', () => {
  const baseSurface: SurfaceCapability = {
    id: 'test-surface',
    name: 'Test Surface',
    route: '/test',
    capabilityClass: CapabilityClass.FullySupported,
  };

  it('should return base capability when no platform overrides', () => {
    const result = getPlatformCapability(baseSurface, 'desktop');
    expect(result).toBe(CapabilityClass.FullySupported);
  });

  it('should return platform-specific capability when defined', () => {
    const surfaceWithPlatform: SurfaceCapability = {
      ...baseSurface,
      platformCapabilities: {
        android: CapabilityClass.Limited,
      },
    };

    const result = getPlatformCapability(surfaceWithPlatform, 'android');
    expect(result).toBe(CapabilityClass.Limited);
  });

  it('should allow platform downgrade from customer-facing to non-customer-facing', () => {
    const surfaceWithDowngrade: SurfaceCapability = {
      ...baseSurface,
      platformCapabilities: {
        browser: CapabilityClass.HandoffOnly,
      },
    };

    const result = getPlatformCapability(surfaceWithDowngrade, 'browser');
    expect(result).toBe(CapabilityClass.HandoffOnly);
  });

  it('should not allow platform upgrade from non-customer-facing to customer-facing', () => {
    const nonCustomerSurface: SurfaceCapability = {
      ...baseSurface,
      capabilityClass: CapabilityClass.Internal,
      platformCapabilities: {
        desktop: CapabilityClass.FullySupported,
      },
    };

    const result = getPlatformCapability(nonCustomerSurface, 'desktop');
    expect(result).toBe(CapabilityClass.Internal);
  });
});

describe('SurfaceCapability type', () => {
  it('should accept valid surface capability', () => {
    const surface: SurfaceCapability = {
      id: 'settings-account',
      name: 'Account Settings',
      route: '/settings/account',
      capabilityClass: CapabilityClass.FullySupported,
      platformCapabilities: {
        desktop: CapabilityClass.FullySupported,
        android: CapabilityClass.Limited,
        browser: CapabilityClass.ReadOnly,
      },
      customerDescription: 'Manage your account settings and preferences',
      internalNotes: 'Core settings surface, high priority',
    };

    expect(surface.id).toBe('settings-account');
    expect(surface.capabilityClass).toBe(CapabilityClass.FullySupported);
  });
});

describe('CapabilityEvidence type', () => {
  it('should accept valid capability evidence', () => {
    const evidence: CapabilityEvidence = {
      codePath: {
        files: ['src/settings/AccountSettings.tsx'],
        exports: ['AccountSettings', 'useAccountSettings'],
        linesOfCode: 150,
      },
      statePath: {
        store: 'settingsStore',
        schema: 'AccountSettingsSchema',
        persistence: 'localStorage',
      },
      tests: {
        unit: true,
        integration: true,
        e2e: false,
        coverage: 85,
      },
      platformScope: {
        desktop: CapabilityClass.FullySupported,
        android: CapabilityClass.Limited,
        browser: CapabilityClass.ReadOnly,
      },
    };

    expect(evidence.codePath.files).toHaveLength(1);
    expect(evidence.tests.coverage).toBe(85);
  });
});
