/**
 * Registry Validation
 *
 * Provides registry-level validation functionality.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityRegistry,
} from './types';
import { validateSurface, type SurfaceValidationResult } from './surface-validator';

/**
 * Registry validation options
 */
export interface RegistryValidationOptions {
  /** Whether to validate evidence details */
  validateEvidenceDetails?: boolean;
  /** Whether to include warnings in output */
  includeWarnings?: boolean;
  /** Filter to specific surface IDs */
  surfaceIds?: string[];
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
  /** Results for each surface */
  surfaceResults: SurfaceValidationResult[];
  /** Summary by capability class */
  classSummary: {
    [key in CapabilityClass]?: {
      total: number;
      valid: number;
      invalid: number;
    };
  };
  /** Warnings at the registry level */
  warnings: string[];
}

/**
 * Get minimum capability class from a set of surfaces
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

/**
 * Validate a capability registry
 */
export function validateRegistry(
  registry: CapabilityRegistry,
  opts: RegistryValidationOptions = {}
): RegistryValidationResult {
  const {
    validateEvidenceDetails = true,
    includeWarnings = true,
    surfaceIds,
  } = opts;

  const warnings: string[] = [];
  const surfaceResults: SurfaceValidationResult[] = [];

  // Filter surfaces if surfaceIds provided
  const surfacesToValidate = surfaceIds
    ? registry.surfaces.filter((s) => surfaceIds.includes(s.id))
    : registry.surfaces;

  // Check for missing requested surfaces
  if (surfaceIds) {
    const foundIds = new Set(surfacesToValidate.map((s) => s.id));
    const missingIds = surfaceIds.filter((id) => !foundIds.has(id));
    if (missingIds.length > 0) {
      warnings.push(`Requested surfaces not found in registry: ${missingIds.join(', ')}`);
    }
  }

  // Validate each surface
  for (const surface of surfacesToValidate) {
    const result = validateSurface(surface, {
      validateEvidenceDetails,
      includeWarnings,
    });
    surfaceResults.push(result);
  }

  // Check for duplicate IDs
  const idCounts = new Map<string, number>();
  for (const surface of registry.surfaces) {
    idCounts.set(surface.id, (idCounts.get(surface.id) || 0) + 1);
  }
  for (const [id, count] of idCounts) {
    if (count > 1) {
      warnings.push(`Duplicate surface ID: ${id} (${count} occurrences)`);
    }
  }

  // Calculate minimum capability class
  const minCapabilityClass = getMinimumCapabilityClass(registry.surfaces);
  if (minCapabilityClass === CapabilityClass.Unsupported) {
    warnings.push('Registry contains unsupported surfaces');
  }

  // Calculate class summary
  const classSummary: RegistryValidationResult['classSummary'] = {};
  for (const cls of Object.values(CapabilityClass)) {
    const classResults = surfaceResults.filter((r) => {
      const surface = registry.surfaces.find((s) => s.id === r.surfaceId);
      return surface?.capabilityClass === cls;
    });
    classSummary[cls] = {
      total: classResults.length,
      valid: classResults.filter((r) => r.valid).length,
      invalid: classResults.filter((r) => !r.valid).length,
    };
  }

  return {
    valid: surfaceResults.every((r) => r.valid),
    totalSurfaces: surfaceResults.length,
    validSurfaces: surfaceResults.filter((r) => r.valid).length,
    invalidSurfaces: surfaceResults.filter((r) => !r.valid).length,
    surfaceResults,
    classSummary,
    warnings,
  };
}

/**
 * Generate validation report
 */
export function generateValidationReport(
  result: RegistryValidationResult
): string {
  const lines: string[] = [];

  lines.push('# Capability Validation Report');
  lines.push('');
  lines.push(`**Overall Status**: ${result.valid ? '✅ PASSED' : '❌ FAILED'}`);
  lines.push('');
  lines.push('## Summary');
  lines.push('');
  lines.push(`- **Total Surfaces**: ${result.totalSurfaces}`);
  lines.push(`- **Valid**: ${result.validSurfaces}`);
  lines.push(`- **Invalid**: ${result.invalidSurfaces}`);
  lines.push('');

  if (result.warnings.length > 0) {
    lines.push('## Warnings');
    lines.push('');
    for (const warning of result.warnings) {
      lines.push(`- ⚠️ ${warning}`);
    }
    lines.push('');
  }

  if (result.invalidSurfaces > 0) {
    lines.push('## Failed Validations');
    lines.push('');

    for (const surfaceResult of result.surfaceResults.filter((r) => !r.valid)) {
      lines.push(`### ${surfaceResult.surfaceName} (${surfaceResult.surfaceId})`);
      lines.push('');
      lines.push(`**Class**: ${surfaceResult.capabilityClass}`);
      lines.push('');
      lines.push('**Errors**:');
      for (const error of surfaceResult.errors) {
        lines.push(`- ❌ ${error}`);
      }
      if (surfaceResult.warnings.length > 0) {
        lines.push('');
        lines.push('**Warnings**:');
        for (const warning of surfaceResult.warnings) {
          lines.push(`- ⚠️ ${warning}`);
        }
      }
      lines.push('');
    }
  }

  lines.push('## Class Summary');
  lines.push('');
  lines.push('| Class | Total | Valid | Invalid |');
  lines.push('|-------|-------|-------|---------|');

  for (const [cls, summary] of Object.entries(result.classSummary)) {
    lines.push(
      `| ${cls} | ${summary.total} | ${summary.valid} | ${summary.invalid} |`
    );
  }

  lines.push('');
  lines.push('---');
  lines.push('*Generated by Capability Validation System*');

  return lines.join('\n');
}

export default {
  validateRegistry,
  generateValidationReport,
};
