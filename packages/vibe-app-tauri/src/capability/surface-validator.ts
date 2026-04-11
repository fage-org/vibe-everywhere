/**
 * Surface Validation
 *
 * Provides surface-level validation functionality.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityEvidence,
} from './types';

/**
 * Surface validation options
 */
export interface SurfaceValidationOptions {
  /** Whether to validate evidence details */
  validateEvidenceDetails?: boolean;
  /** Whether to include warnings in output */
  includeWarnings?: boolean;
}

/**
 * Surface validation result
 */
export interface SurfaceValidationResult {
  /** Surface ID */
  surfaceId: string;
  /** Surface name */
  surfaceName: string;
  /** Capability class */
  capabilityClass: CapabilityClass;
  /** Whether validation passed */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Coverage percentage */
  coverage?: number;
}

/**
 * Validate evidence for a surface
 */
function validateEvidenceForSurface(
  evidence: CapabilityEvidence | undefined,
  capabilityClass: CapabilityClass,
  surface: SurfaceCapability
): { valid: boolean; errors: string[]; warnings: string[]; coverage?: number } {
  const errors: string[] = [];
  const warnings: string[] = [];

  // For unsupported class, no evidence is required
  if (capabilityClass === CapabilityClass.Unsupported) {
    return { valid: true, errors: [], warnings: [] };
  }

  // For internal class, evidence is optional but recommended
  if (capabilityClass === CapabilityClass.Internal) {
    if (!evidence) {
      warnings.push('No evidence provided for internal surface (optional but recommended)');
      return { valid: true, errors: [], warnings };
    }
  }

  // For other classes, evidence is required
  if (!evidence) {
    errors.push(`No evidence provided for ${capabilityClass} surface`);
    return { valid: false, errors, warnings };
  }

  // Validate coverage
  if (evidence.tests.coverage < 40) {
    errors.push(`Test coverage ${evidence.tests.coverage}% is below minimum 40%`);
  } else if (evidence.tests.coverage < 60) {
    warnings.push(`Test coverage ${evidence.tests.coverage}% is below recommended 60%`);
  }

  // Validate test types based on capability class
  if (capabilityClass === CapabilityClass.FullySupported) {
    if (!evidence.tests.unit) {
      errors.push('Unit tests are required for FullySupported surfaces');
    }
    if (!evidence.tests.integration) {
      errors.push('Integration tests are required for FullySupported surfaces');
    }
    if (!evidence.tests.e2e) {
      errors.push('E2E tests are required for FullySupported surfaces');
    }
    if (evidence.tests.coverage < 80) {
      errors.push(`Test coverage ${evidence.tests.coverage}% is below minimum 80% for FullySupported`);
    }
  }

  // Validate customer description for customer-facing surfaces
  const customerFacing = [
    CapabilityClass.FullySupported,
    CapabilityClass.Limited,
    CapabilityClass.ReadOnly,
  ];
  if (customerFacing.includes(capabilityClass) && !surface.customerDescription) {
    errors.push('Customer-facing description is required for customer-facing surfaces');
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    coverage: evidence.tests.coverage,
  };
}

/**
 * Validate a single surface
 */
export function validateSurface(
  surface: SurfaceCapability,
  opts: SurfaceValidationOptions = {}
): SurfaceValidationResult {
  const { validateEvidenceDetails = true, includeWarnings = true } = opts;

  const errors: string[] = [];
  const warnings: string[] = [];
  let coverage: number | undefined;

  // Validate evidence if requested and present
  if (validateEvidenceDetails && surface.evidence) {
    const evidenceResult = validateEvidenceForSurface(
      surface.evidence,
      surface.capabilityClass,
      surface
    );

    errors.push(...evidenceResult.errors);
    if (includeWarnings) {
      warnings.push(...evidenceResult.warnings);
    }
    coverage = evidenceResult.coverage;
  }

  return {
    surfaceId: surface.id,
    surfaceName: surface.name,
    capabilityClass: surface.capabilityClass,
    valid: errors.length === 0,
    errors,
    warnings: includeWarnings ? warnings : [],
    coverage,
  };
}

export default {
  validateSurface,
};
