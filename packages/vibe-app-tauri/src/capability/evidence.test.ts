/**
 * Tests for Capability Evidence System
 *
 * These tests verify the evidence validation and requirements
 * functionality used throughout Wave 10.
 */

import { describe, expect, it } from "vitest";
import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityEvidence,
} from './types';
import {
  EVIDENCE_REQUIREMENTS,
  validateEvidence,
  generateEvidenceChecklist,
  getMinimumCapabilityClass,
} from './evidence';

describe('EVIDENCE_REQUIREMENTS', () => {
  it('should have requirements for all capability classes', () => {
    const classes = Object.values(CapabilityClass);
    for (const cls of classes) {
      expect(EVIDENCE_REQUIREMENTS[cls]).toBeDefined();
    }
  });

  it('should have higher coverage requirements for FullySupported', () => {
    expect(EVIDENCE_REQUIREMENTS[CapabilityClass.FullySupported].minCoverage).toBe(80);
    expect(EVIDENCE_REQUIREMENTS[CapabilityClass.Limited].minCoverage).toBe(60);
    expect(EVIDENCE_REQUIREMENTS[CapabilityClass.Internal].minCoverage).toBe(40);
  });

  it('should require all test types for FullySupported', () => {
    const reqs = EVIDENCE_REQUIREMENTS[CapabilityClass.FullySupported];
    expect(reqs.requiresUnitTests).toBe(true);
    expect(reqs.requiresIntegrationTests).toBe(true);
    expect(reqs.requiresE2ETests).toBe(true);
  });

  it('should not require E2E for Limited', () => {
    expect(EVIDENCE_REQUIREMENTS[CapabilityClass.Limited].requiresE2ETests).toBe(false);
  });

  it('should require minimal evidence for Unsupported', () => {
    const reqs = EVIDENCE_REQUIREMENTS[CapabilityClass.Unsupported];
    expect(reqs.minCoverage).toBe(0);
    expect(reqs.requiresUnitTests).toBe(false);
    expect(reqs.requiresCodePath).toBe(false);
    expect(reqs.requiresCustomerDescription).toBe(false);
  });
});

describe('validateEvidence', () => {
  const baseSurface: SurfaceCapability = {
    id: 'test-surface',
    name: 'Test Surface',
    capabilityClass: CapabilityClass.FullySupported,
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

  it('should validate FullySupported with valid evidence', () => {
    const result = validateEvidence(
      validEvidence,
      CapabilityClass.FullySupported,
      { ...baseSurface, customerDescription: 'A test surface' }
    );

    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('should fail if coverage is below minimum', () => {
    const lowCoverageEvidence: CapabilityEvidence = {
      ...validEvidence,
      tests: { ...validEvidence.tests, coverage: 70 },
    };

    const result = validateEvidence(
      lowCoverageEvidence,
      CapabilityClass.FullySupported,
      baseSurface
    );

    expect(result.valid).toBe(false);
    expect(result.errors).toContain(
      'Test coverage 70% is below minimum 80%'
    );
  });

  it('should fail if required tests are missing', () => {
    const missingE2E: CapabilityEvidence = {
      ...validEvidence,
      tests: { ...validEvidence.tests, e2e: false },
    };

    const result = validateEvidence(
      missingE2E,
      CapabilityClass.FullySupported,
      baseSurface
    );

    expect(result.valid).toBe(false);
    expect(result.errors).toContain('E2E tests are required but not present');
  });

  it('should fail if customer description is missing for FullySupported', () => {
    const result = validateEvidence(
      validEvidence,
      CapabilityClass.FullySupported,
      { ...baseSurface, customerDescription: undefined }
    );

    expect(result.valid).toBe(false);
    expect(result.errors).toContain(
      'Customer-facing description is required but not provided'
    );
  });

  it('should pass Unsupported with no evidence', () => {
    const result = validateEvidence(
      undefined,
      CapabilityClass.Unsupported,
      baseSurface
    );

    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('should allow platform downgrade', () => {
    const evidenceWithDowngrade: CapabilityEvidence = {
      ...validEvidence,
      platformScope: {
        desktop: CapabilityClass.FullySupported,
        android: CapabilityClass.HandoffOnly, // Downgrade
        browser: CapabilityClass.ReadOnly,
      },
    };

    const result = validateEvidence(
      evidenceWithDowngrade,
      CapabilityClass.FullySupported,
      { ...baseSurface, customerDescription: 'Test' }
    );

    expect(result.valid).toBe(true);
  });
});

describe('generateEvidenceChecklist', () => {
  it('should generate checklist for FullySupported', () => {
    const checklist = generateEvidenceChecklist(CapabilityClass.FullySupported);

    expect(checklist).toContain('## Evidence Requirements for fully_supported');
    expect(checklist).toContain('### Testing');
    expect(checklist).toContain('- [ ] Unit tests');
    expect(checklist).toContain('- [ ] Integration tests');
    expect(checklist).toContain('- [ ] E2E tests');
    expect(checklist).toContain('- [ ] Coverage >= 80%');
    expect(checklist).toContain('### Documentation');
  });

  it('should have different requirements for different classes', () => {
    const fullySupported = generateEvidenceChecklist(CapabilityClass.FullySupported);
    const limited = generateEvidenceChecklist(CapabilityClass.Limited);
    const unsupported = generateEvidenceChecklist(CapabilityClass.Unsupported);

    // FullySupported should require at least as much as Limited (both require all evidence)
    expect(fullySupported.filter(line => line.includes('- [ ]')).length).toBeGreaterThanOrEqual(
      limited.filter(line => line.includes('- [ ]')).length
    );

    // Unsupported should have minimal requirements
    expect(unsupported.filter(line => line.includes('- [ ]')).length).toBeLessThan(5);
  });
});

describe('getMinimumCapabilityClass', () => {
  it('should return Unsupported for empty array', () => {
    expect(getMinimumCapabilityClass([])).toBe(CapabilityClass.Unsupported);
  });

  it('should return the only capability for single surface', () => {
    const surfaces: SurfaceCapability[] = [
      { id: '1', name: 'Test', capabilityClass: CapabilityClass.FullySupported },
    ];
    expect(getMinimumCapabilityClass(surfaces)).toBe(CapabilityClass.FullySupported);
  });

  it('should return the minimum capability across surfaces', () => {
    const surfaces: SurfaceCapability[] = [
      { id: '1', name: 'Test1', capabilityClass: CapabilityClass.FullySupported },
      { id: '2', name: 'Test2', capabilityClass: CapabilityClass.Limited },
      { id: '3', name: 'Test3', capabilityClass: CapabilityClass.ReadOnly },
    ];
    // Minimum should be ReadOnly (lowest customer-facing)
    expect(getMinimumCapabilityClass(surfaces)).toBe(CapabilityClass.ReadOnly);
  });

  it('should handle all capability classes correctly', () => {
    const surfaces: SurfaceCapability[] = [
      { id: '1', name: 'Test1', capabilityClass: CapabilityClass.FullySupported },
      { id: '2', name: 'Test2', capabilityClass: CapabilityClass.Limited },
      { id: '3', name: 'Test3', capabilityClass: CapabilityClass.HandoffOnly },
      { id: '4', name: 'Test4', capabilityClass: CapabilityClass.ReadOnly },
      { id: '5', name: 'Test5', capabilityClass: CapabilityClass.Internal },
      { id: '6', name: 'Test6', capabilityClass: CapabilityClass.Unsupported },
    ];
    // Minimum should be Unsupported
    expect(getMinimumCapabilityClass(surfaces)).toBe(CapabilityClass.Unsupported);
  });
});
