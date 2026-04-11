/**
 * Capability Evidence Requirements
 *
 * This module defines the evidence requirements for each capability class
 * and provides utilities for evidence validation and collection.
 */

import {
  CapabilityClass,
  type CapabilityEvidence,
  type SurfaceCapability,
} from './types';

/**
 * Evidence requirements for each capability class
 */
export interface EvidenceRequirements {
  /** Minimum test coverage percentage */
  minCoverage: number;
  /** Whether unit tests are required */
  requiresUnitTests: boolean;
  /** Whether integration tests are required */
  requiresIntegrationTests: boolean;
  /** Whether E2E tests are required */
  requiresE2ETests: boolean;
  /** Whether code path documentation is required */
  requiresCodePath: boolean;
  /** Whether state path documentation is required */
  requiresStatePath: boolean;
  /** Whether platform scope definition is required */
  requiresPlatformScope: boolean;
  /** Whether customer-facing description is required */
  requiresCustomerDescription: boolean;
  /** Whether internal notes are required */
  requiresInternalNotes: boolean;
}

/**
 * Evidence requirements for each capability class
 */
export const EVIDENCE_REQUIREMENTS: Record<CapabilityClass, EvidenceRequirements> = {
  [CapabilityClass.FullySupported]: {
    minCoverage: 80,
    requiresUnitTests: true,
    requiresIntegrationTests: true,
    requiresE2ETests: true,
    requiresCodePath: true,
    requiresStatePath: true,
    requiresPlatformScope: true,
    requiresCustomerDescription: true,
    requiresInternalNotes: false,
  },
  [CapabilityClass.Limited]: {
    minCoverage: 60,
    requiresUnitTests: true,
    requiresIntegrationTests: true,
    requiresE2ETests: false,
    requiresCodePath: true,
    requiresStatePath: true,
    requiresPlatformScope: true,
    requiresCustomerDescription: true,
    requiresInternalNotes: true,
  },
  [CapabilityClass.HandoffOnly]: {
    minCoverage: 40,
    requiresUnitTests: true,
    requiresIntegrationTests: false,
    requiresE2ETests: false,
    requiresCodePath: true,
    requiresStatePath: false,
    requiresPlatformScope: true,
    requiresCustomerDescription: true,
    requiresInternalNotes: true,
  },
  [CapabilityClass.ReadOnly]: {
    minCoverage: 60,
    requiresUnitTests: true,
    requiresIntegrationTests: true,
    requiresE2ETests: false,
    requiresCodePath: true,
    requiresStatePath: true,
    requiresPlatformScope: true,
    requiresCustomerDescription: true,
    requiresInternalNotes: false,
  },
  [CapabilityClass.Internal]: {
    minCoverage: 40,
    requiresUnitTests: true,
    requiresIntegrationTests: false,
    requiresE2ETests: false,
    requiresCodePath: true,
    requiresStatePath: false,
    requiresPlatformScope: false,
    requiresCustomerDescription: false,
    requiresInternalNotes: true,
  },
  [CapabilityClass.Unsupported]: {
    minCoverage: 0,
    requiresUnitTests: false,
    requiresIntegrationTests: false,
    requiresE2ETests: false,
    requiresCodePath: false,
    requiresStatePath: false,
    requiresPlatformScope: false,
    requiresCustomerDescription: false,
    requiresInternalNotes: true,
  },
};

/**
 * Evidence validation result
 */
export interface EvidenceValidationResult {
  /** Whether the evidence meets all requirements */
  valid: boolean;
  /** List of validation errors */
  errors: string[];
  /** List of validation warnings */
  warnings: string[];
  /** Coverage percentage (if applicable) */
  coverage?: number;
}

/**
 * Validate evidence against capability class requirements
 */
export function validateEvidence(
  evidence: CapabilityEvidence | undefined,
  capabilityClass: CapabilityClass,
  surface: SurfaceCapability
): EvidenceValidationResult {
  const requirements = EVIDENCE_REQUIREMENTS[capabilityClass];
  const errors: string[] = [];
  const warnings: string[] = [];

  // Check if evidence exists
  if (!evidence) {
    // For unsupported class, no evidence is required
    if (capabilityClass === CapabilityClass.Unsupported) {
      return { valid: true, errors: [], warnings: [] };
    }

    // For internal class, evidence is optional
    if (capabilityClass === CapabilityClass.Internal) {
      return { valid: true, errors: [], warnings: ['No evidence provided for internal surface'] };
    }

    return {
      valid: false,
      errors: [`No evidence provided for ${capabilityClass} surface`],
      warnings: [],
    };
  }

  // Validate test requirements
  if (requirements.requiresUnitTests && !evidence.tests.unit) {
    errors.push('Unit tests are required but not present');
  }

  if (requirements.requiresIntegrationTests && !evidence.tests.integration) {
    errors.push('Integration tests are required but not present');
  }

  if (requirements.requiresE2ETests && !evidence.tests.e2e) {
    errors.push('E2E tests are required but not present');
  }

  // Validate coverage
  if (evidence.tests.coverage < requirements.minCoverage) {
    errors.push(
      `Test coverage ${evidence.tests.coverage}% is below minimum ${requirements.minCoverage}%`
    );
  }

  // Validate code path
  if (requirements.requiresCodePath) {
    if (!evidence.codePath.files || evidence.codePath.files.length === 0) {
      errors.push('Code path files are required but not documented');
    }
    if (!evidence.codePath.exports || evidence.codePath.exports.length === 0) {
      warnings.push('Code path exports should be documented');
    }
  }

  // Validate state path
  if (requirements.requiresStatePath) {
    if (!evidence.statePath.store) {
      errors.push('State store is required but not documented');
    }
  }

  // Validate platform scope
  if (requirements.requiresPlatformScope) {
    if (!evidence.platformScope) {
      errors.push('Platform scope is required but not documented');
    } else {
      const { desktop, android, browser } = evidence.platformScope;
      if (!desktop) warnings.push('Desktop platform scope should be defined');
      if (!android) warnings.push('Android platform scope should be defined');
      if (!browser) warnings.push('Browser platform scope should be defined');
    }
  }

  // Validate customer description
  if (requirements.requiresCustomerDescription && !surface.customerDescription) {
    errors.push('Customer-facing description is required but not provided');
  }

  // Validate internal notes
  if (requirements.requiresInternalNotes && !surface.internalNotes) {
    warnings.push('Internal notes are recommended for this capability class');
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    coverage: evidence.tests.coverage,
  };
}

/**
 * Generate evidence requirements checklist for a capability class
 */
export function generateEvidenceChecklist(
  capabilityClass: CapabilityClass
): string[] {
  const requirements = EVIDENCE_REQUIREMENTS[capabilityClass];
  const checklist: string[] = [];

  checklist.push(`## Evidence Requirements for ${capabilityClass}`);
  checklist.push('');

  // Test requirements
  checklist.push('### Testing');
  if (requirements.requiresUnitTests) checklist.push('- [ ] Unit tests');
  if (requirements.requiresIntegrationTests) checklist.push('- [ ] Integration tests');
  if (requirements.requiresE2ETests) checklist.push('- [ ] E2E tests');
  checklist.push(`- [ ] Coverage >= ${requirements.minCoverage}%`);
  checklist.push('');

  // Documentation requirements
  checklist.push('### Documentation');
  if (requirements.requiresCodePath) checklist.push('- [ ] Code path documented');
  if (requirements.requiresStatePath) checklist.push('- [ ] State path documented');
  if (requirements.requiresPlatformScope) checklist.push('- [ ] Platform scope defined');
  if (requirements.requiresCustomerDescription)
    checklist.push('- [ ] Customer description provided');
  if (requirements.requiresInternalNotes) checklist.push('- [ ] Internal notes added');
  checklist.push('');

  return checklist;
}

/**
 * Get the minimum capability class from a set of surfaces
 * Used to determine the overall capability level of a feature area
 */
export function getMinimumCapabilityClass(
  surfaces: SurfaceCapability[]
): CapabilityClass {
  const order = [
    CapabilityClass.Unsupported,
    CapabilityClass.Internal,
    CapabilityClass.HandoffOnly,
    CapabilityClass.ReadOnly,
    CapabilityClass.Limited,
    CapabilityClass.FullySupported,
  ];

  if (surfaces.length === 0) {
    return CapabilityClass.Unsupported;
  }

  let minIndex = order.length - 1;
  for (const surface of surfaces) {
    const index = order.indexOf(surface.capabilityClass);
    if (index < minIndex) {
      minIndex = index;
    }
  }

  return order[minIndex];
}

export default {
  EVIDENCE_REQUIREMENTS,
  validateEvidence,
  generateEvidenceChecklist,
  getMinimumCapabilityClass,
};
