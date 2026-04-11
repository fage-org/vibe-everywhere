/**
 * Capability Classification System for Wave 10
 *
 * This module defines the capability classification model used throughout
 * Wave 10 to categorize app surfaces by their support level.
 */

/**
 * CapabilityClass enum defines the support level for each app surface
 */
export enum CapabilityClass {
  /**
   * Fully supported product feature
   * - Complete functionality
   * - Full test coverage
   * - Active maintenance
   * - Customer-facing documentation
   */
  FullySupported = 'fully_supported',

  /**
   * Limited support feature
   * - Core functionality works
   * - Some edge cases not handled
   * - Basic test coverage
   * - Minimal documentation
   */
  Limited = 'limited',

  /**
   * Handoff-only feature
   * - UI exists but delegates to external tools
   * - No internal implementation
   * - Copy-to-clipboard or redirect only
   */
  HandoffOnly = 'handoff_only',

  /**
   * Read-only feature
   * - Can view data but not modify
   * - No write operations supported
   * - Display/analytics only
   */
  ReadOnly = 'read_only',

  /**
   * Internal feature
   * - Not exposed to end users
   * - Developer/diagnostic tools only
   * - May lack polish/documentation
   */
  Internal = 'internal',

  /**
   * Unsupported feature
   * - Code may exist but not maintained
   * - No testing or documentation
   * - May be removed in future
   */
  Unsupported = 'unsupported',
}

/**
 * Platform scope for a capability
 */
export interface PlatformScope {
  /** Desktop (Windows/Mac/Linux) support level */
  desktop: CapabilityClass;
  /** Android support level */
  android: CapabilityClass;
  /** Browser export support level */
  browser: CapabilityClass;
}

/**
 * Evidence required to claim a capability
 */
export interface CapabilityEvidence {
  /** Code path evidence - the actual implementation */
  codePath: {
    /** Path to implementation files */
    files: string[];
    /** Key functions/classes implemented */
    exports: string[];
    /** Lines of code (approximate) */
    linesOfCode: number;
  };

  /** State path evidence - state management */
  statePath: {
    /** State store implementation */
    store: string;
    /** State shape/schema */
    schema: string;
    /** Persistence mechanism */
    persistence: 'localStorage' | 'indexedDB' | 'remote' | 'none';
  };

  /** Test evidence - test coverage */
  tests: {
    /** Unit tests exist */
    unit: boolean;
    /** Integration tests exist */
    integration: boolean;
    /** E2E tests exist */
    e2e: boolean;
    /** Coverage percentage */
    coverage: number;
  };

  /** Platform scope evidence */
  platformScope: PlatformScope;
}

/**
 * Surface capability definition
 * Associates a route/surface with its capability class and evidence
 */
export interface SurfaceCapability {
  /** Unique identifier for the surface */
  id: string;

  /** Human-readable name */
  name: string;

  /** Route path (if applicable) */
  route?: string;

  /** Capability class - the support level */
  capabilityClass: CapabilityClass;

  /** Platform-specific capability classes */
  platformCapabilities?: Partial<PlatformScope>;

  /** Evidence supporting the capability claim */
  evidence?: CapabilityEvidence;

  /** Customer-facing description */
  customerDescription?: string;

  /** Internal notes */
  internalNotes?: string;

  /** Last reviewed date */
  lastReviewed?: Date;

  /** Reviewer name */
  reviewedBy?: string;
}

/**
 * Capability registry - holds all surface capabilities
 */
export interface CapabilityRegistry {
  /** Version of the registry schema */
  version: string;

  /** All surface capabilities */
  surfaces: SurfaceCapability[];

  /** Last updated timestamp */
  lastUpdated: Date;

  /** Global defaults for platform scope */
  defaultPlatformScope?: Partial<PlatformScope>;
}

/**
 * Helper function to check if a capability class allows customer use
 */
export function isCustomerFacing(capabilityClass: CapabilityClass): boolean {
  return [
    CapabilityClass.FullySupported,
    CapabilityClass.Limited,
    CapabilityClass.ReadOnly,
  ].includes(capabilityClass);
}

/**
 * Helper function to check if a capability class allows modification
 */
export function isMutable(capabilityClass: CapabilityClass): boolean {
  return [
    CapabilityClass.FullySupported,
    CapabilityClass.Limited,
  ].includes(capabilityClass);
}

/**
 * Helper function to get the effective capability class for a platform
 */
export function getPlatformCapability(
  surface: SurfaceCapability,
  platform: 'desktop' | 'android' | 'browser'
): CapabilityClass {
  const platformCapabilities = surface.platformCapabilities;
  if (!platformCapabilities) {
    return surface.capabilityClass;
  }

  const platformClass = platformCapabilities[platform];
  if (!platformClass) {
    return surface.capabilityClass;
  }

  // The platform-specific class cannot be better than the base class
  // in terms of customer-facing capability
  const baseIsCustomerFacing = isCustomerFacing(surface.capabilityClass);
  const platformIsCustomerFacing = isCustomerFacing(platformClass);

  if (baseIsCustomerFacing && !platformIsCustomerFacing) {
    // Platform downgrades are allowed
    return platformClass;
  }

  if (!baseIsCustomerFacing && platformIsCustomerFacing) {
    // Platform cannot upgrade a non-customer-facing base
    return surface.capabilityClass;
  }

  return platformClass;
}

export default {
  CapabilityClass,
  isCustomerFacing,
  isMutable,
  getPlatformCapability,
};
