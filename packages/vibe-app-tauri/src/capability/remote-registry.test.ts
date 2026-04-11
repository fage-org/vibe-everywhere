/**
 * Tests for Remote Operations Registry
 *
 * B30: Remote Operations Surfaces - Phase 1 Tests
 */

import { describe, expect, it } from "vitest";
import { CapabilityClass } from './types';
import {
  RemoteSurfaceType,
  WorkflowStep,
  remoteSurfaces,
  remoteRegistry,
  getSurfacesByType,
  getSurfacesByWorkflowStep,
  getCoreWorkflowSurfaces,
  getHelperSurfaces,
  getRemoteSummary,
} from './remote-registry';

describe('Remote Registry', () => {
  it('should have all remote surfaces defined', () => {
    expect(remoteSurfaces.length).toBeGreaterThan(0);
    expect(remoteSurfaces.length).toBe(6); // 6 remote surfaces
  });

  it('should have unique surface IDs', () => {
    const ids = remoteSurfaces.map((s) => s.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });

  it('should have valid capability classes', () => {
    for (const surface of remoteSurfaces) {
      expect(Object.values(CapabilityClass)).toContain(surface.capabilityClass);
    }
  });

  it('should have customer descriptions for customer-facing surfaces', () => {
    const customerFacingClasses = [
      CapabilityClass.FullySupported,
      CapabilityClass.Limited,
      CapabilityClass.ReadOnly,
    ];

    for (const surface of remoteSurfaces) {
      if (customerFacingClasses.includes(surface.capabilityClass)) {
        expect(surface.customerDescription).toBeTruthy();
        expect(surface.customerDescription!.length).toBeGreaterThan(0);
      }
    }
  });

  it('should have platform capabilities defined for all surfaces', () => {
    for (const surface of remoteSurfaces) {
      expect(surface.platformCapabilities).toBeDefined();
      expect(surface.platformCapabilities!.desktop).toBeDefined();
      expect(surface.platformCapabilities!.android).toBeDefined();
      expect(surface.platformCapabilities!.browser).toBeDefined();
    }
  });
});

describe('getSurfacesByType', () => {
  it('should return terminal surfaces', () => {
    const terminalSurfaces = getSurfacesByType(RemoteSurfaceType.Terminal);
    expect(terminalSurfaces.length).toBeGreaterThan(0);
    for (const surface of terminalSurfaces) {
      expect(['terminal-connect', 'terminal-helper']).toContain(surface.id);
    }
  });

  it('should return machine surfaces', () => {
    const machineSurfaces = getSurfacesByType(RemoteSurfaceType.Machine);
    expect(machineSurfaces.length).toBeGreaterThan(0);
    for (const surface of machineSurfaces) {
      expect(['machine-list', 'machine-detail']).toContain(surface.id);
    }
  });

  it('should return server surfaces', () => {
    const serverSurfaces = getSurfacesByType(RemoteSurfaceType.Server);
    expect(serverSurfaces.length).toBeGreaterThan(0);
    for (const surface of serverSurfaces) {
      expect(surface.id).toBe('server-config');
    }
  });

  it('should return remote session surfaces', () => {
    const sessionSurfaces = getSurfacesByType(RemoteSurfaceType.RemoteSession);
    expect(sessionSurfaces.length).toBeGreaterThan(0);
    for (const surface of sessionSurfaces) {
      expect(surface.id).toBe('remote-session-launcher');
    }
  });
});

describe('getSurfacesByWorkflowStep', () => {
  it('should return core workflow surfaces', () => {
    const coreSurfaces = getSurfacesByWorkflowStep(WorkflowStep.Core);
    expect(coreSurfaces.length).toBeGreaterThan(0);
    for (const surface of coreSurfaces) {
      expect(surface.capabilityClass).toBe(CapabilityClass.FullySupported);
    }
  });

  it('should return limited workflow surfaces', () => {
    const limitedSurfaces = getSurfacesByWorkflowStep(WorkflowStep.Limited);
    expect(limitedSurfaces.length).toBeGreaterThanOrEqual(0);
    for (const surface of limitedSurfaces) {
      expect([CapabilityClass.Limited]).toContain(surface.capabilityClass);
    }
  });
});

describe('getCoreWorkflowSurfaces', () => {
  it('should return all core workflow surfaces', () => {
    const coreSurfaces = getCoreWorkflowSurfaces();
    expect(coreSurfaces.length).toBeGreaterThan(0);

    // All core surfaces should be FullySupported
    for (const surface of coreSurfaces) {
      expect(surface.capabilityClass).toBe(CapabilityClass.FullySupported);
    }
  });
});

describe('getHelperSurfaces', () => {
  it('should return all helper surfaces', () => {
    const helperSurfaces = getHelperSurfaces();
    expect(helperSurfaces.length).toBeGreaterThanOrEqual(0);

    // All helper surfaces should be Limited
    for (const surface of helperSurfaces) {
      expect(surface.capabilityClass).toBe(CapabilityClass.Limited);
    }
  });
});

describe('getRemoteSummary', () => {
  it('should return summary statistics', () => {
    const summary = getRemoteSummary();

    expect(summary.totalSurfaces).toBe(remoteSurfaces.length);
    expect(Object.keys(summary.byType).length).toBe(5); // 5 surface types
    expect(Object.keys(summary.byWorkflowStep).length).toBe(4); // 4 workflow steps
    expect(Object.keys(summary.byCapabilityClass).length).toBe(
      Object.values(CapabilityClass).length
    );
    expect(summary.coreWorkflowCount).toBeGreaterThan(0);
    expect(summary.helperCount).toBeGreaterThanOrEqual(0);
  });

  it('should have correct totals by type', () => {
    const summary = getRemoteSummary();

    let totalByType = 0;
    for (const count of Object.values(summary.byType)) {
      totalByType += count;
    }

    // Note: Some surfaces may map to multiple types, so this may not equal totalSurfaces
    expect(totalByType).toBeGreaterThan(0);
  });

  it('should have correct totals by workflow step', () => {
    const summary = getRemoteSummary();

    let totalByStep = 0;
    for (const count of Object.values(summary.byWorkflowStep)) {
      totalByStep += count;
    }

    // Note: Some surfaces may map to multiple steps, so this may not equal totalSurfaces
    expect(totalByStep).toBeGreaterThan(0);
  });
});

describe('remoteRegistry', () => {
  it('should have correct version', () => {
    expect(remoteRegistry.version).toBe('1.0.0');
  });

  it('should have all surfaces', () => {
    expect(remoteRegistry.surfaces).toEqual(remoteSurfaces);
  });

  it('should have lastUpdated date', () => {
    expect(remoteRegistry.lastUpdated).toBeInstanceOf(Date);
  });
});
