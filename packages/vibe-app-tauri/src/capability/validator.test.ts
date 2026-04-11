/**
 * Tests for Capability Validation System
 *
 * These tests verify the validation functionality used in B27 Phase 4
 * to validate surfaces and registries.
 */

import { describe, expect, it } from "vitest";
import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityEvidence,
  type CapabilityRegistry,
} from './types';
import {
  validateSurface,
  validateRegistry,
  generateValidationReport,
  type RegistryValidationResult,
} from './validator';

describe('validateSurface', () => {
  const baseSurface: SurfaceCapability = {
    id: 'test-surface',
    name: 'Test Surface',
    route: '/test',
    capabilityClass: CapabilityClass.FullySupported,
    customerDescription: 'A test surface for validation',
  };

  const validEvidence: CapabilityEvidence = {
    codePath: {
      files: ['src/test.tsx'],
      exports: ['TestComponent'],
      linesOfCode: 100,
    },
    statePath: {
      store: 'testStore',
      schema: 'TestSchema',
      persistence: 'localStorage',
    },
    tests: {
      unit: true,
      integration: true,
      e2e: true,
      coverage: 85,
    },
    platformScope: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.ReadOnly,
    },
  };

  it('should validate a fully supported surface with valid evidence', () => {
    const result = validateSurface(baseSurface, validEvidence);

    expect(result.valid).toBe(true);
    expect(result.surfaceId).toBe('test-surface');
    expect(result.surfaceName).toBe('Test Surface');
    expect(result.errors).toHaveLength(0);
  });

  it('should fail validation for missing required fields', () => {
    const invalidSurface: SurfaceCapability = {
      id: '',
      name: '',
      capabilityClass: CapabilityClass.FullySupported,
    };

    const result = validateSurface(invalidSurface);

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('Surface ID is required');
    expect(result.errors).toContain('Surface name is required');
  });

  it('should warn about missing customer description for customer-facing surfaces', () => {
    const surfaceWithoutDescription: SurfaceCapability = {
      ...baseSurface,
      customerDescription: undefined,
    };

    const result = validateSurface(surfaceWithoutDescription, validEvidence, {
      includeWarnings: true,
    });

    expect(result.errors).toContain(
      'Customer-facing description is required but not provided'
    );
  });

  it('should not include warnings when includeWarnings is false', () => {
    const surfaceWithoutDescription: SurfaceCapability = {
      ...baseSurface,
      customerDescription: undefined,
    };

    const result = validateSurface(surfaceWithoutDescription, validEvidence, {
      includeWarnings: false,
    });

    expect(result.warnings).toHaveLength(0);
  });

  it('should handle surfaces without evidence', () => {
    const result = validateSurface(baseSurface, undefined);

    // Should pass basic validation but may have warnings
    expect(result.surfaceId).toBe('test-surface');
    expect(result.valid).toBe(true); // Basic validation passes
  });
});

describe('validateRegistry', () => {
  const createTestRegistry = (): CapabilityRegistry => ({
    version: '1.0.0',
    surfaces: [
      {
        id: 'surface-1',
        name: 'Surface 1',
        capabilityClass: CapabilityClass.FullySupported,
        customerDescription: 'Test surface 1',
      },
      {
        id: 'surface-2',
        name: 'Surface 2',
        capabilityClass: CapabilityClass.Limited,
        customerDescription: 'Test surface 2',
      },
      {
        id: 'surface-3',
        name: 'Surface 3',
        capabilityClass: CapabilityClass.ReadOnly,
        customerDescription: 'Test surface 3',
      },
    ],
    lastUpdated: new Date(),
  });

  const createTestEvidenceMap = (): Map<string, CapabilityEvidence> => {
    const map = new Map();
    map.set('surface-1', {
      codePath: { files: ['src/s1.tsx'], exports: ['S1'], linesOfCode: 100 },
      statePath: { store: 's1Store', schema: 'S1Schema', persistence: 'localStorage' },
      tests: { unit: true, integration: true, e2e: true, coverage: 85 },
      platformScope: {
        desktop: CapabilityClass.FullySupported,
        android: CapabilityClass.Limited,
        browser: CapabilityClass.ReadOnly,
      },
    });
    return map;
  };

  it('should validate a registry with all valid surfaces', () => {
    const registry = createTestRegistry();
    const evidenceMap = createTestEvidenceMap();

    const result = validateRegistry(registry, evidenceMap);

    expect(result.totalSurfaces).toBe(3);
    expect(result.valid).toBe(true);
    expect(result.invalidSurfaces).toBe(0);
  });

  it('should detect duplicate surface IDs', () => {
    const registry = createTestRegistry();
    registry.surfaces[1].id = 'surface-1'; // Duplicate

    const result = validateRegistry(registry);

    expect(result.valid).toBe(false);
    expect(result.registryErrors).toContain(
      'Duplicate surface ID: surface-1 (2 occurrences)'
    );
  });

  it('should warn about surfaces without evidence', () => {
    const registry = createTestRegistry();

    const result = validateRegistry(registry, new Map(), {
      includeWarnings: true,
    });

    expect(result.registryWarnings.length).toBeGreaterThan(0);
    expect(result.registryWarnings[0]).toContain('surfaces without evidence');
  });

  it('should calculate class summary correctly', () => {
    const registry = createTestRegistry();

    const result = validateRegistry(registry);

    expect(result.classSummary[CapabilityClass.FullySupported].total).toBe(1);
    expect(result.classSummary[CapabilityClass.Limited].total).toBe(1);
    expect(result.classSummary[CapabilityClass.ReadOnly].total).toBe(1);
  });

  it('should respect surfaceIds filter option', () => {
    const registry = createTestRegistry();

    const result = validateRegistry(registry, new Map(), {
      surfaceIds: ['surface-1', 'surface-2'],
    });

    expect(result.totalSurfaces).toBe(2);
  });
});

describe('generateValidationReport', () => {
  const createMockResult = (): RegistryValidationResult => ({
    valid: true,
    totalSurfaces: 3,
    validSurfaces: 3,
    invalidSurfaces: 0,
    surfaceResults: [
      {
        surfaceId: 'surface-1',
        surfaceName: 'Surface 1',
        valid: true,
        errors: [],
        warnings: [],
        coverage: 85,
      },
      {
        surfaceId: 'surface-2',
        surfaceName: 'Surface 2',
        valid: true,
        errors: [],
        warnings: ['Consider adding E2E tests'],
        coverage: 70,
      },
      {
        surfaceId: 'surface-3',
        surfaceName: 'Surface 3',
        valid: true,
        errors: [],
        warnings: [],
      },
    ],
    classSummary: {
      [CapabilityClass.FullySupported]: { total: 1, valid: 1, invalid: 0 },
      [CapabilityClass.Limited]: { total: 1, valid: 1, invalid: 0 },
      [CapabilityClass.ReadOnly]: { total: 1, valid: 1, invalid: 0 },
      [CapabilityClass.HandoffOnly]: { total: 0, valid: 0, invalid: 0 },
      [CapabilityClass.Internal]: { total: 0, valid: 0, invalid: 0 },
      [CapabilityClass.Unsupported]: { total: 0, valid: 0, invalid: 0 },
    },
    registryErrors: [],
    registryWarnings: [],
  });

  it('should generate a valid markdown report', () => {
    const result = createMockResult();
    const report = generateValidationReport(result);

    expect(report).toContain('# Capability Registry Validation Report');
    expect(report).toContain('✅ PASSED');
    expect(report).toContain('**Total Surfaces:** 3');
    expect(report).toContain('**Valid:** 3 ✅');
  });

  it('should include class summary in the report', () => {
    const result = createMockResult();
    const report = generateValidationReport(result);

    expect(report).toContain('## By Capability Class');
    expect(report).toContain('| Class | Total | Valid | Invalid |');
    expect(report).toContain('fully_supported');
    expect(report).toContain('limited');
    expect(report).toContain('read_only');
  });

  it('should include invalid surfaces section when there are failures', () => {
    const result = createMockResult();
    result.valid = false;
    result.invalidSurfaces = 1;
    result.surfaceResults[0].valid = false;
    result.surfaceResults[0].errors = ['Missing evidence'];

    const report = generateValidationReport(result);

    expect(report).toContain('❌ FAILED');
    expect(report).toContain('## Invalid Surfaces');
    expect(report).toContain('Surface 1 (surface-1)');
    expect(report).toContain('- ❌ Missing evidence');
  });

  it('should include registry warnings when present', () => {
    const result = createMockResult();
    result.registryWarnings = ['3 surfaces without evidence'];

    const report = generateValidationReport(result);

    expect(report).toContain('## Registry Warnings');
    expect(report).toContain('- ⚠️ 3 surfaces without evidence');
  });

  it('should include registry errors when present', () => {
    const result = createMockResult();
    result.valid = false;
    result.registryErrors = ['Duplicate surface ID: surface-1 (2 occurrences)'];

    const report = generateValidationReport(result);

    expect(report).toContain('## Registry Errors');
    expect(report).toContain('- ❌ Duplicate surface ID: surface-1 (2 occurrences)');
  });

  it('should include surface warnings when present', () => {
    const result = createMockResult();
    result.surfaceResults[1].warnings = ['Consider adding E2E tests'];

    const report = generateValidationReport(result);

    // Valid surfaces with warnings should still be listed
    expect(report).toContain('**Valid:** 3 ✅');
  });
});

describe('Integration: Complete validation workflow', () => {
  it('should validate a complete workflow from registry to report', () => {
    // 1. Create a registry with multiple surfaces
    const registry: CapabilityRegistry = {
      version: '1.0.0',
      surfaces: [
        {
          id: 'settings-account',
          name: 'Account Settings',
          route: '/settings/account',
          capabilityClass: CapabilityClass.FullySupported,
          customerDescription: 'Manage your account settings',
        },
        {
          id: 'settings-appearance',
          name: 'Appearance Settings',
          route: '/settings/appearance',
          capabilityClass: CapabilityClass.FullySupported,
          customerDescription: 'Customize the app appearance',
        },
        {
          id: 'terminal-connect',
          name: 'Terminal Connect',
          route: '/terminal/connect',
          capabilityClass: CapabilityClass.HandoffOnly,
          customerDescription: 'Connect to terminal via external tool',
        },
      ],
      lastUpdated: new Date(),
    };

    // 2. Create evidence map for some surfaces
    const evidenceMap = new Map<string, CapabilityEvidence>([
      [
        'settings-account',
        {
          codePath: {
            files: ['src/settings/AccountSettings.tsx'],
            exports: ['AccountSettings'],
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
            e2e: true,
            coverage: 85,
          },
          platformScope: {
            desktop: CapabilityClass.FullySupported,
            android: CapabilityClass.Limited,
            browser: CapabilityClass.ReadOnly,
          },
        },
      ],
      [
        'settings-appearance',
        {
          codePath: {
            files: ['src/settings/AppearanceSettings.tsx'],
            exports: ['AppearanceSettings'],
            linesOfCode: 200,
          },
          statePath: {
            store: 'settingsStore',
            schema: 'AppearanceSettingsSchema',
            persistence: 'localStorage',
          },
          tests: {
            unit: true,
            integration: true,
            e2e: false,
            coverage: 75,
          },
          platformScope: {
            desktop: CapabilityClass.FullySupported,
            android: CapabilityClass.Limited,
            browser: CapabilityClass.Unsupported,
          },
        },
      ],
    ]);

    // 3. Validate the registry
    const validationResult = validateRegistry(registry, evidenceMap, {
      includeWarnings: true,
    });

    // 4. Assertions
    expect(validationResult.totalSurfaces).toBe(3);
    expect(validationResult.validSurfaces).toBe(2); // settings-account and settings-appearance
    expect(validationResult.invalidSurfaces).toBe(1); // terminal-connect (no evidence)
    expect(validationResult.valid).toBe(false); // Not all surfaces are valid

    // 5. Generate and verify report
    const report = generateValidationReport(validationResult);
    expect(report).toContain('# Capability Registry Validation Report');
    expect(report).toContain('❌ FAILED');
    expect(report).toContain('**Total Surfaces:** 3');
    expect(report).toContain('settings-appearance');
    expect(report).toContain('terminal-connect');
  });
});
