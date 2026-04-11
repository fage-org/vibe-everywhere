/**
 * Platform Parity and Browser Contract Registry
 *
 * B31: Platform Parity and Browser Contract
 *
 * This module defines the per-surface desktop/Android/browser support contracts
 * and provides utilities for platform capability checking and validation.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
} from './types';
import { settingsSurfaces } from './settings-registry';
import { inboxSurfaces } from './inbox-registry';
import { remoteSurfaces } from './remote-registry';

/**
 * Platform support level definitions
 */
export enum PlatformSupportLevel {
  /** Complete feature parity */
  Complete = 'complete',
  /** Limited functionality */
  Limited = 'limited',
  /** Read-only access */
  ReadOnly = 'read_only',
  /** Handoff to external tool */
  HandoffOnly = 'handoff_only',
  /** Not supported */
  Unsupported = 'unsupported',
}

/**
 * Browser export contract types
 */
export enum BrowserContractType {
  /** Full interactive application */
  FullApplication = 'full_application',
  /** Limited feature set */
  LimitedFeatures = 'limited_features',
  /** Read-only/export view */
  ReadOnlyView = 'read_only_view',
  /** Static export only */
  StaticExport = 'static_export',
  /** Not applicable */
  NotApplicable = 'not_applicable',
}

/**
 * Platform support matrix entry
 */
export interface PlatformSupportMatrixEntry {
  /** Surface ID */
  surfaceId: string;
  /** Desktop support level */
  desktop: PlatformSupportLevel;
  /** Android support level */
  android: PlatformSupportLevel;
  /** Browser support level */
  browser: PlatformSupportLevel;
  /** Browser contract type */
  browserContract: BrowserContractType;
  /** Customer-facing description */
  customerNote?: string;
  /** Internal notes */
  internalNote?: string;
}

/**
 * Platform parity summary
 */
export interface PlatformParitySummary {
  /** Total surfaces */
  totalSurfaces: number;
  /** Desktop-complete surfaces */
  desktopComplete: number;
  /** Desktop-limited surfaces */
  desktopLimited: number;
  /** Android-complete surfaces */
  androidComplete: number;
  /** Android-limited surfaces */
  androidLimited: number;
  /** Browser-complete surfaces */
  browserComplete: number;
  /** Browser-limited surfaces */
  browserLimited: number;
  /** Browser read-only surfaces */
  browserReadOnly: number;
  /** Browser-handoff surfaces */
  browserHandoff: number;
  /** Unsupported on mobile */
  mobileUnsupported: number;
  /** Unsupported on browser */
  browserUnsupported: number;
}

/**
 * Convert CapabilityClass to PlatformSupportLevel
 */
export function capabilityToSupportLevel(
  capabilityClass: CapabilityClass
): PlatformSupportLevel {
  const mapping: Record<CapabilityClass, PlatformSupportLevel> = {
    [CapabilityClass.FullySupported]: PlatformSupportLevel.Complete,
    [CapabilityClass.Limited]: PlatformSupportLevel.Limited,
    [CapabilityClass.ReadOnly]: PlatformSupportLevel.ReadOnly,
    [CapabilityClass.HandoffOnly]: PlatformSupportLevel.HandoffOnly,
    [CapabilityClass.Internal]: PlatformSupportLevel.Unsupported,
    [CapabilityClass.Unsupported]: PlatformSupportLevel.Unsupported,
  };
  return mapping[capabilityClass];
}

/**
 * Get platform support matrix for all surfaces
 */
export function getPlatformSupportMatrix(
  surfaces: SurfaceCapability[]
): PlatformSupportMatrixEntry[] {
  return surfaces.map((surface) => {
    const desktop = surface.platformCapabilities?.desktop ?? surface.capabilityClass;
    const android = surface.platformCapabilities?.android ?? surface.capabilityClass;
    const browser = surface.platformCapabilities?.browser ?? surface.capabilityClass;

    // Determine browser contract type
    let browserContract: BrowserContractType;
    switch (browser) {
      case CapabilityClass.FullySupported:
        browserContract = BrowserContractType.FullApplication;
        break;
      case CapabilityClass.Limited:
        browserContract = BrowserContractType.LimitedFeatures;
        break;
      case CapabilityClass.ReadOnly:
        browserContract = BrowserContractType.ReadOnlyView;
        break;
      case CapabilityClass.HandoffOnly:
        browserContract = BrowserContractType.StaticExport;
        break;
      default:
        browserContract = BrowserContractType.NotApplicable;
    }

    return {
      surfaceId: surface.id,
      desktop: capabilityToSupportLevel(desktop),
      android: capabilityToSupportLevel(android),
      browser: capabilityToSupportLevel(browser),
      browserContract,
      customerNote: surface.customerDescription,
      internalNote: surface.internalNotes,
    };
  });
}

/**
 * Get platform parity summary
 */
export function getPlatformParitySummary(
  matrix: PlatformSupportMatrixEntry[]
): PlatformParitySummary {
  return {
    totalSurfaces: matrix.length,
    desktopComplete: matrix.filter(
      (e) => e.desktop === PlatformSupportLevel.Complete
    ).length,
    desktopLimited: matrix.filter(
      (e) => e.desktop === PlatformSupportLevel.Limited
    ).length,
    androidComplete: matrix.filter(
      (e) => e.android === PlatformSupportLevel.Complete
    ).length,
    androidLimited: matrix.filter(
      (e) => e.android === PlatformSupportLevel.Limited
    ).length,
    browserComplete: matrix.filter(
      (e) => e.browser === PlatformSupportLevel.Complete
    ).length,
    browserLimited: matrix.filter(
      (e) => e.browser === PlatformSupportLevel.Limited
    ).length,
    browserReadOnly: matrix.filter(
      (e) => e.browser === PlatformSupportLevel.ReadOnly
    ).length,
    browserHandoff: matrix.filter(
      (e) => e.browser === PlatformSupportLevel.HandoffOnly
    ).length,
    mobileUnsupported: matrix.filter(
      (e) => e.android === PlatformSupportLevel.Unsupported
    ).length,
    browserUnsupported: matrix.filter(
      (e) => e.browser === PlatformSupportLevel.Unsupported
    ).length,
  };
}

/**
 * Check if surface is supported on platform
 */
export function isSupportedOnPlatform(
  matrixEntry: PlatformSupportMatrixEntry,
  platform: 'desktop' | 'android' | 'browser'
): boolean {
  const level = matrixEntry[platform];
  return (
    level === PlatformSupportLevel.Complete ||
    level === PlatformSupportLevel.Limited ||
    level === PlatformSupportLevel.ReadOnly
  );
}

/**
 * Get browser contract description
 */
export function getBrowserContractDescription(
  contractType: BrowserContractType
): string {
  const descriptions: Record<BrowserContractType, string> = {
    [BrowserContractType.FullApplication]:
      'Full interactive application with all features',
    [BrowserContractType.LimitedFeatures]:
      'Limited feature set with core functionality',
    [BrowserContractType.ReadOnlyView]:
      'Read-only view with no editing capabilities',
    [BrowserContractType.StaticExport]:
      'Static export with limited interactivity',
    [BrowserContractType.NotApplicable]:
      'Not applicable for browser platform',
  };
  return descriptions[contractType];
}

/**
 * Export all surfaces from all registries
 */
export function getAllSurfaces(): SurfaceCapability[] {
  return [
    ...settingsSurfaces,
    ...inboxSurfaces,
    ...remoteSurfaces,
  ];
}

export default {
  PlatformSupportLevel,
  BrowserContractType,
  capabilityToSupportLevel,
  getPlatformSupportMatrix,
  getPlatformParitySummary,
  isSupportedOnPlatform,
  getBrowserContractDescription,
  getAllSurfaces,
};
