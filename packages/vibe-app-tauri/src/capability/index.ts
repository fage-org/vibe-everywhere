/**
 * Capability Classification System - Main Entry Point
 *
 * This module provides the capability classification system used
 * throughout Wave 10 to categorize app surfaces by their support level.
 *
 * @example
 * ```typescript
 * import {
 *   CapabilityClass,
 *   validateEvidence,
 *   isCustomerFacing,
 * } from '@/capability';
 *
 * const surface: SurfaceCapability = {
 *   id: 'settings-account',
 *   name: 'Account Settings',
 *   capabilityClass: CapabilityClass.FullySupported,
 *   customerDescription: 'Manage your account settings',
 * };
 *
 * if (isCustomerFacing(surface.capabilityClass)) {
 *   console.log('This surface is customer-facing');
 * }
 * ```
 */

import {
  CapabilityClass,
  getPlatformCapability,
  isCustomerFacing,
  isMutable,
} from './types';
import {
  EVIDENCE_REQUIREMENTS,
  generateEvidenceChecklist,
  getMinimumCapabilityClass,
  validateEvidence,
} from './evidence';

// Re-export all types from types.ts
export {
  CapabilityClass,
  isCustomerFacing,
  isMutable,
  getPlatformCapability,
} from './types';

export type {
  CapabilityEvidence,
  CapabilityRegistry,
  PlatformScope,
  SurfaceCapability,
} from './types';

// Re-export all functions from evidence.ts
export {
  EVIDENCE_REQUIREMENTS,
  validateEvidence,
  generateEvidenceChecklist,
  getMinimumCapabilityClass,
} from './evidence';

export type {
  EvidenceRequirements,
  EvidenceValidationResult,
} from './evidence';

// Version of the capability system
export const CAPABILITY_SYSTEM_VERSION = '1.0.0';

// Default exports
export default {
  CapabilityClass,
  isCustomerFacing,
  isMutable,
  getPlatformCapability,
  EVIDENCE_REQUIREMENTS,
  validateEvidence,
  generateEvidenceChecklist,
  getMinimumCapabilityClass,
  CAPABILITY_SYSTEM_VERSION,
};
