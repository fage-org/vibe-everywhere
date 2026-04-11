/**
 * Capability Validation Runner
 *
 * This module provides the validation functionality for B27 Phase 4.
 * It validates all surfaces in the capability registry and generates
 * validation reports.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityRegistry,
  type CapabilityEvidence,
} from './types';
import {
  validateEvidence,
  getMinimumCapabilityClass,
} from './evidence';

/**
 * Surface validation result
 */
export interface SurfaceValidationResult {
  /** Surface ID */
  surfaceId: string;
  /** Surface name */
  surfaceName: string;
  /** Whether the surface passed validation */
  valid: boolean;
  /** Validation errors */
  errors: string[];
  /** Validation warnings */
  warnings: string[];
  /** Test coverage percentage */
  coverage?: number;
}

/**
 * Registry validation result
 */
export interface RegistryValidationResult {
  /** Whether all surfaces passed validation */
  valid: boolean;
  /** Total number of surfaces */
  totalSurfaces: number;
  /** Number of valid surfaces */
  validSurfaces: number;
  /** Number of invalid surfaces */
  invalidSurfaces: number;
  /** Individual surface results */
  surfaceResults: SurfaceValidationResult[];
  /** Summary by capability class */
  classSummary: Record<CapabilityClass, {
    total: number;
    valid: number;
    invalid: number;
  }>;
  /** Registry-wide errors */
  registryErrors: string[];
  /** Registry-wide warnings */
  registryWarnings: string[];
}

/**
 * Validation options
 */
export interface ValidationOptions {
  /** Whether to include warnings in the result */
  includeWarnings?: boolean;
  /** Whether to validate evidence details */
  validateEvidenceDetails?: boolean;
  /** Minimum coverage threshold (overrides class requirement) */
  minCoverageOverride?: number;
  /** Specific surface IDs to validate (undefined = all) */
  surfaceIds?: string[];
}

/**
 * Default validation options
 */
const DEFAULT_VALIDATION_OPTIONS: ValidationOptions = {
  includeWarnings: true,
  validateEvidenceDetails: true,
  minCoverageOverride: undefined,
  surfaceIds: undefined,
};

/**
 * Validate a single surface
 */
export function validateSurface(
  surface: SurfaceCapability,
  evidence?: CapabilityEvidence,
  options: ValidationOptions = {}
): SurfaceValidationResult {
  const opts = { ...DEFAULT_VALIDATION_OPTIONS, ...options };

  // Run evidence validation if evidence is provided
  if (opts.validateEvidenceDetails && evidence) {
    const evidenceResult = validateEvidence(
      evidence,
      surface.capabilityClass,
      surface
    );

    return {
      surfaceId: surface.id,
      surfaceName: surface.name,
      valid: evidenceResult.valid,
      errors: evidenceResult.errors,
      warnings: opts.includeWarnings ? evidenceResult.warnings : [],
      coverage: evidenceResult.coverage,
    };
  }

  // Basic validation without evidence
  const errors: string[] = [];
  const warnings: string[] = [];

  // Check required fields
  if (!surface.id) {
    errors.push('Surface ID is required');
  }

  if (!surface.name) {
    errors.push('Surface name is required');
  }

  if (!surface.capabilityClass) {
    errors.push('Capability class is required');
  }

  // Check customer description for customer-facing surfaces
  const isCustomerFacing = [
    CapabilityClass.FullySupported,
    CapabilityClass.Limited,
    CapabilityClass.ReadOnly,
  ].includes(surface.capabilityClass);

  if (isCustomerFacing && !surface.customerDescription) {
    warnings.push('Customer-facing surface should have a customer description');
  }

  return {
    surfaceId: surface.id || 'unknown',
    surfaceName: surface.name || 'Unknown',
    valid: errors.length === 0,
    errors,
    warnings: opts.includeWarnings ? warnings : [],
  };
}

/**
 * Validate an entire capability registry
 */
export function validateRegistry(
  registry: CapabilityRegistry,
  evidenceMap: Map<string, CapabilityEvidence> = new Map(),
  options: ValidationOptions = {}
): RegistryValidationResult {
  const opts = { ...DEFAULT_VALIDATION_OPTIONS, ...options };
  const surfaceResults: SurfaceValidationResult[] = [];

  // Filter surfaces if specific IDs are requested
  const surfacesToValidate = opts.surfaceIds
    ? registry.surfaces.filter((s) => opts.surfaceIds!.includes(s.id))
    : registry.surfaces;

  // Validate each surface
  for (const surface of surfacesToValidate) {
    const evidence = evidenceMap.get(surface.id);
    const result = validateSurface(surface, evidence, opts);
    surfaceResults.push(result);
  }

  // Calculate summary statistics
  const validSurfaces = surfaceResults.filter((r) => r.valid).length;
  const invalidSurfaces = surfaceResults.filter((r) => !r.valid).length;

  // Calculate class summary
  const classSummary: Record<
    CapabilityClass,
    { total: number; valid: number; invalid: number }
  > = {
    [CapabilityClass.FullySupported]: { total: 0, valid: 0, invalid: 0 },
    [CapabilityClass.Limited]: { total: 0, valid: 0, invalid: 0 },
    [CapabilityClass.HandoffOnly]: { total: 0, valid: 0, invalid: 0 },
    [CapabilityClass.ReadOnly]: { total: 0, valid: 0, invalid: 0 },
    [CapabilityClass.Internal]: { total: 0, valid: 0, invalid: 0 },
    [CapabilityClass.Unsupported]: { total: 0, valid: 0, invalid: 0 },
  };

  for (const surface of surfacesToValidate) {
    const result = surfaceResults.find((r) => r.surfaceId === surface.id);
    if (result) {
      classSummary[surface.capabilityClass].total++;
      if (result.valid) {
        classSummary[surface.capabilityClass].valid++;
      } else {
        classSummary[surface.capabilityClass].invalid++;
      }
    }
  }

  // Collect registry-wide warnings
  const registryWarnings: string[] = [];
  const registryErrors: string[] = [];

  // Check for duplicate IDs
  const idCounts = new Map<string, number>();
  for (const surface of registry.surfaces) {
    const count = idCounts.get(surface.id) || 0;
    idCounts.set(surface.id, count + 1);
  }
  for (const [id, count] of idCounts.entries()) {
    if (count > 1) {
      registryErrors.push(`Duplicate surface ID: ${id} (${count} occurrences)`);
    }
  }

  // Check for surfaces without evidence
  const surfacesWithoutEvidence = registry.surfaces.filter(
    (s) => !evidenceMap.has(s.id)
  );
  if (surfacesWithoutEvidence.length > 0) {
    registryWarnings.push(
      `${surfacesWithoutEvidence.length} surfaces without evidence: ${surfacesWithoutEvidence
        .map((s) => s.id)
        .join(', ')}`
    );
  }

  // Calculate minimum capability class
  const minCapabilityClass = getMinimumCapabilityClass(registry.surfaces);
  if (minCapabilityClass === CapabilityClass.Unsupported) {
    registryWarnings.push('Registry contains unsupported surfaces');
  }

  return {
    valid: invalidSurfaces === 0 && registryErrors.length === 0,
    totalSurfaces: surfacesToValidate.length,
    validSurfaces,
    invalidSurfaces,
    surfaceResults,
    classSummary,
    registryErrors,
    registryWarnings: opts.includeWarnings ? registryWarnings : [],
  };
}

/**
 * Generate a validation report as a formatted string
 */
export function generateValidationReport(
  result: RegistryValidationResult
): string {
  const lines: string[] = [];

  lines.push('# Capability Registry Validation Report');
  lines.push('');
  lines.push(`**Status:** ${result.valid ? '✅ PASSED' : '❌ FAILED'}`);
  lines.push(`**Date:** ${new Date().toISOString()}`);
  lines.push('');

  // Summary
  lines.push('## Summary');
  lines.push('');
  lines.push(`- **Total Surfaces:** ${result.totalSurfaces}`);
  lines.push(`- **Valid:** ${result.validSurfaces} ✅`);
  lines.push(`- **Invalid:** ${result.invalidSurfaces} ❌`);
  lines.push('');

  // Class summary
  lines.push('## By Capability Class');
  lines.push('');
  lines.push('| Class | Total | Valid | Invalid |');
  lines.push('|-------|-------|-------|---------|');
  for (const [cls, stats] of Object.entries(result.classSummary)) {
    lines.push(
      `| ${cls} | ${stats.total} | ${stats.valid} ✅ | ${stats.invalid} ❌ |`
    );
  }
  lines.push('');

  // Invalid surfaces
  if (result.invalidSurfaces > 0) {
    lines.push('## Invalid Surfaces');
    lines.push('');
    for (const surfaceResult of result.surfaceResults.filter((r) => !r.valid)) {
      lines.push(`### ${surfaceResult.surfaceName} (${surfaceResult.surfaceId})`);
      lines.push('');
      for (const error of surfaceResult.errors) {
        lines.push(`- ❌ ${error}`);
      }
      if (surfaceResult.warnings.length > 0) {
        lines.push('');
        lines.push('**Warnings:**');
        for (const warning of surfaceResult.warnings) {
          lines.push(`- ⚠️ ${warning}`);
        }
      }
      lines.push('');
    }
  }

  // Registry warnings
  if (result.registryWarnings.length > 0) {
    lines.push('## Registry Warnings');
    lines.push('');
    for (const warning of result.registryWarnings) {
      lines.push(`- ⚠️ ${warning}`);
    }
    lines.push('');
  }

  // Registry errors
  if (result.registryErrors.length > 0) {
    lines.push('## Registry Errors');
    lines.push('');
    for (const error of result.registryErrors) {
      lines.push(`- ❌ ${error}`);
    }
    lines.push('');
  }

  // Footer
  lines.push('---');
  lines.push('');
  lines.push('*Generated by Capability Classification System v1.0.0*');

  return lines.join('\n');
}

export default {
  validateSurface,
  validateRegistry,
  generateValidationReport,
};
