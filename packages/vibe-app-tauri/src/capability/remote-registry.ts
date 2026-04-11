/**
 * Remote Operations Registry
 *
 * B30: Remote Operations Surfaces
 *
 * This module defines the remote operations workflow surfaces including
 * terminal, machine, server, and related helper routes.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityRegistry,
} from './types';

/**
 * Remote operation surface types
 */
export enum RemoteSurfaceType {
  /** Terminal connection and management */
  Terminal = 'terminal',
  /** Machine management */
  Machine = 'machine',
  /** Server configuration */
  Server = 'server',
  /** Remote session control */
  RemoteSession = 'remote_session',
  /** Helper/utility surfaces */
  Helper = 'helper',
}

/**
 * Workflow step classification
 */
export enum WorkflowStep {
  /** Core workflow step - fully supported */
  Core = 'core',
  /** Limited helper - partial functionality */
  Limited = 'limited',
  /** Internal utility - not customer-facing */
  Internal = 'internal',
  /** Handoff-only - delegates to external */
  Handoff = 'handoff',
}

/**
 * Remote operations surface definitions
 */
export const remoteSurfaces: SurfaceCapability[] = [
  // Terminal Connect (Core workflow)
  {
    id: 'terminal-connect',
    name: 'Terminal Connect',
    route: '/terminal/connect',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.Unsupported,
    },
    customerDescription: 'Connect to remote terminals and execute commands',
    internalNotes: 'Core remote operation - terminal connection and command execution. Desktop-first feature.',
  },

  // Terminal Helper (Limited helper)
  {
    id: 'terminal-helper',
    name: 'Terminal Helper',
    route: '/terminal/helper',
    capabilityClass: CapabilityClass.Limited,
    platformCapabilities: {
      desktop: CapabilityClass.Limited,
      android: CapabilityClass.HandoffOnly,
      browser: CapabilityClass.HandoffOnly,
    },
    customerDescription: 'Terminal setup helper - copy commands to clipboard',
    internalNotes: 'Limited helper - provides command copy, opens external terminal. Handoff on mobile/browser.',
  },

  // Machine List (Core workflow)
  {
    id: 'machine-list',
    name: 'Machine List',
    route: '/machines',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'View and manage your remote machines',
    internalNotes: 'Core machine management - list, status, basic control. Full support desktop/mobile, read-only browser.',
  },

  // Machine Detail (Core workflow)
  {
    id: 'machine-detail',
    name: 'Machine Detail',
    route: '/machines/:id',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'Detailed machine information and diagnostics',
    internalNotes: 'Machine diagnostics and detail view. Full desktop, limited mobile (view only), read-only browser.',
  },

  // Server Configuration (Limited)
  {
    id: 'server-config',
    name: 'Server Configuration',
    route: '/settings/server',
    capabilityClass: CapabilityClass.Limited,
    platformCapabilities: {
      desktop: CapabilityClass.Limited,
      android: CapabilityClass.Unsupported,
      browser: CapabilityClass.Unsupported,
    },
    customerDescription: 'Configure server connection settings',
    internalNotes: 'Server/operator configuration - limited support, desktop only. Advanced feature.',
  },

  // Remote Session Launcher (Core workflow)
  {
    id: 'remote-session-launcher',
    name: 'Remote Session Launcher',
    route: '/remote/launch',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.Limited,
      browser: CapabilityClass.Unsupported,
    },
    customerDescription: 'Launch remote sessions on connected machines',
    internalNotes: 'Remote session initiation - spawn sessions on remote machines. Desktop primary, limited mobile.',
  },
];

/**
 * Remote operations registry
 */
export const remoteRegistry: CapabilityRegistry = {
  version: '1.0.0',
  surfaces: remoteSurfaces,
  lastUpdated: new Date(),
};

/**
 * Get surfaces by type
 */
export function getSurfacesByType(type: RemoteSurfaceType): SurfaceCapability[] {
  const typeToIds: Record<RemoteSurfaceType, string[]> = {
    [RemoteSurfaceType.Terminal]: ['terminal-connect', 'terminal-helper'],
    [RemoteSurfaceType.Machine]: ['machine-list', 'machine-detail'],
    [RemoteSurfaceType.Server]: ['server-config'],
    [RemoteSurfaceType.RemoteSession]: ['remote-session-launcher'],
    [RemoteSurfaceType.Helper]: ['terminal-helper'],
  };

  return remoteSurfaces.filter((surface) =>
    typeToIds[type].includes(surface.id)
  );
}

/**
 * Get surfaces by workflow step classification
 */
export function getSurfacesByWorkflowStep(step: WorkflowStep): SurfaceCapability[] {
  const stepToClasses: Record<WorkflowStep, CapabilityClass[]> = {
    [WorkflowStep.Core]: [CapabilityClass.FullySupported],
    [WorkflowStep.Limited]: [CapabilityClass.Limited],
    [WorkflowStep.Internal]: [CapabilityClass.Internal],
    [WorkflowStep.Handoff]: [CapabilityClass.HandoffOnly],
  };

  return remoteSurfaces.filter((surface) =>
    stepToClasses[step].includes(surface.capabilityClass)
  );
}

/**
 * Get core workflow surfaces (customer-facing primary flows)
 */
export function getCoreWorkflowSurfaces(): SurfaceCapability[] {
  return getSurfacesByWorkflowStep(WorkflowStep.Core);
}

/**
 * Get helper surfaces (limited functionality)
 */
export function getHelperSurfaces(): SurfaceCapability[] {
  return getSurfacesByWorkflowStep(WorkflowStep.Limited);
}

/**
 * Get remote operations summary
 */
export function getRemoteSummary(): {
  totalSurfaces: number;
  byType: Record<RemoteSurfaceType, number>;
  byWorkflowStep: Record<WorkflowStep, number>;
  byCapabilityClass: Record<CapabilityClass, number>;
  coreWorkflowCount: number;
  helperCount: number;
} {
  return {
    totalSurfaces: remoteSurfaces.length,
    byType: {
      [RemoteSurfaceType.Terminal]: getSurfacesByType(RemoteSurfaceType.Terminal).length,
      [RemoteSurfaceType.Machine]: getSurfacesByType(RemoteSurfaceType.Machine).length,
      [RemoteSurfaceType.Server]: getSurfacesByType(RemoteSurfaceType.Server).length,
      [RemoteSurfaceType.RemoteSession]: getSurfacesByType(RemoteSurfaceType.RemoteSession).length,
      [RemoteSurfaceType.Helper]: getSurfacesByType(RemoteSurfaceType.Helper).length,
    },
    byWorkflowStep: {
      [WorkflowStep.Core]: getSurfacesByWorkflowStep(WorkflowStep.Core).length,
      [WorkflowStep.Limited]: getSurfacesByWorkflowStep(WorkflowStep.Limited).length,
      [WorkflowStep.Internal]: getSurfacesByWorkflowStep(WorkflowStep.Internal).length,
      [WorkflowStep.Handoff]: getSurfacesByWorkflowStep(WorkflowStep.Handoff).length,
    },
    byCapabilityClass: {
      [CapabilityClass.FullySupported]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.FullySupported
      ).length,
      [CapabilityClass.Limited]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Limited
      ).length,
      [CapabilityClass.ReadOnly]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.ReadOnly
      ).length,
      [CapabilityClass.HandoffOnly]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.HandoffOnly
      ).length,
      [CapabilityClass.Internal]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Internal
      ).length,
      [CapabilityClass.Unsupported]: remoteSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Unsupported
      ).length,
    },
    coreWorkflowCount: getCoreWorkflowSurfaces().length,
    helperCount: getHelperSurfaces().length,
  };
}

export default {
  remoteSurfaces,
  remoteRegistry,
  getSurfacesByType,
  getSurfacesByWorkflowStep,
  getCoreWorkflowSurfaces,
  getHelperSurfaces,
  getRemoteSummary,
};
