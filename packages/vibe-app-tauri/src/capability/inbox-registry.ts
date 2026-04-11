/**
 * Inbox, Feed, and Notification Registry
 *
 * B29: Inbox and Notification Closure - Phase 1
 *
 * This module defines the taxonomy for inbox items, feed items, and notifications
 * across the Wave 10 capability contract.
 */

import {
  CapabilityClass,
  type SurfaceCapability,
  type CapabilityRegistry,
} from './types';

/**
 * Event source taxonomy for inbox/feed items
 */
export enum EventSource {
  /** Session-related events (messages, completions) */
  Session = 'session',
  /** Relationship events (invites, connections) */
  Relationship = 'relationship',
  /** Artifact events (files, outputs) */
  Artifact = 'artifact',
  /** Terminal/remote events */
  Terminal = 'terminal',
  /** System events (updates, notifications) */
  System = 'system',
}

/**
 * Item type taxonomy
 */
export enum ItemType {
  /** Inbox item - requires user attention/action */
  Inbox = 'inbox',
  /** Feed item - informational, no action required */
  Feed = 'feed',
  /** Notification - transient alert */
  Notification = 'notification',
}

/**
 * Unread semantics
 */
export enum UnreadState {
  /** Explicitly marked as unread */
  Unread = 'unread',
  /** Explicitly marked as read */
  Read = 'read',
  /** Auto-marked as read (viewed) */
  AutoRead = 'auto_read',
  /** New - never viewed */
  New = 'new',
}

/**
 * Platform notification support
 */
export interface PlatformNotificationSupport {
  /** Desktop local notifications */
  desktopLocal: boolean;
  /** Android push notifications */
  androidPush: boolean;
  /** Browser notifications */
  browserPush: boolean;
}

/**
 * Inbox/feed surface definitions
 */
export const inboxSurfaces: SurfaceCapability[] = [
  // Inbox List View
  {
    id: 'inbox-list',
    name: 'Inbox List',
    route: '/inbox',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.Limited,
    },
    customerDescription: 'View and manage your inbox items requiring attention',
    internalNotes: 'Primary inbox surface - session events, invites, system notifications',
  },

  // Feed View
  {
    id: 'feed-list',
    name: 'Activity Feed',
    route: '/feed',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.ReadOnly,
    },
    customerDescription: 'Browse your activity feed and recent events',
    internalNotes: 'Activity feed - informational, no action required',
  },

  // Notification Center
  {
    id: 'notification-center',
    name: 'Notification Center',
    route: '/notifications',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.Limited,
    },
    customerDescription: 'View and manage your notifications and alerts',
    internalNotes: 'Notification center - transient alerts, dismissible',
  },

  // Session Inbox (sub-surface)
  {
    id: 'session-inbox',
    name: 'Session Inbox',
    route: '/session/:id/inbox',
    capabilityClass: CapabilityClass.FullySupported,
    platformCapabilities: {
      desktop: CapabilityClass.FullySupported,
      android: CapabilityClass.FullySupported,
      browser: CapabilityClass.Limited,
    },
    customerDescription: 'Session-specific messages and events',
    internalNotes: 'Per-session inbox surface - session messages, completions',
  },
];

/**
 * Inbox registry
 */
export const inboxRegistry: CapabilityRegistry = {
  version: '1.0.0',
  surfaces: inboxSurfaces,
  lastUpdated: new Date(),
};

/**
 * Get surfaces by item type (derived from surface ID and capability)
 */
export function getSurfacesByItemType(itemType: ItemType): SurfaceCapability[] {
  const itemTypeToIds: Record<ItemType, string[]> = {
    [ItemType.Inbox]: ['inbox-list', 'session-inbox'],
    [ItemType.Feed]: ['feed-list'],
    [ItemType.Notification]: ['notification-center'],
  };

  return inboxSurfaces.filter((surface) =>
    itemTypeToIds[itemType].includes(surface.id)
  );
}

/**
 * Get surfaces by event source
 */
export function getSurfacesByEventSource(eventSource: EventSource): SurfaceCapability[] {
  // Map event sources to surface IDs
  const sourceToIds: Record<EventSource, string[]> = {
    [EventSource.Session]: ['session-inbox'],
    [EventSource.Relationship]: ['inbox-list'],
    [EventSource.Artifact]: ['feed-list'],
    [EventSource.Terminal]: ['inbox-list'],
    [EventSource.System]: ['notification-center', 'inbox-list'],
  };

  return inboxSurfaces.filter((surface) =>
    sourceToIds[eventSource].includes(surface.id)
  );
}

/**
 * Get notification support by platform
 */
export function getPlatformNotificationSupport(
  surface: SurfaceCapability
): PlatformNotificationSupport {
  const caps = surface.platformCapabilities;
  if (!caps) {
    return {
      desktopLocal: false,
      androidPush: false,
      browserPush: false,
    };
  }

  // Desktop local notifications supported for FullySupported and Limited
  const desktopLocal =
    caps.desktop === CapabilityClass.FullySupported ||
    caps.desktop === CapabilityClass.Limited;

  // Android push supported for FullySupported only
  const androidPush = caps.android === CapabilityClass.FullySupported;

  // Browser push supported for FullySupported only
  const browserPush = caps.browser === CapabilityClass.FullySupported;

  return {
    desktopLocal,
    androidPush,
    browserPush,
  };
}

/**
 * Get inbox summary statistics
 */
export function getInboxSummary(): {
  totalSurfaces: number;
  byItemType: Record<ItemType, number>;
  byCapabilityClass: Record<CapabilityClass, number>;
  platformSupport: {
    desktop: number;
    android: number;
    browser: number;
  };
} {
  return {
    totalSurfaces: inboxSurfaces.length,
    byItemType: {
      [ItemType.Inbox]: getSurfacesByItemType(ItemType.Inbox).length,
      [ItemType.Feed]: getSurfacesByItemType(ItemType.Feed).length,
      [ItemType.Notification]: getSurfacesByItemType(ItemType.Notification).length,
    },
    byCapabilityClass: {
      [CapabilityClass.FullySupported]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.FullySupported
      ).length,
      [CapabilityClass.Limited]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Limited
      ).length,
      [CapabilityClass.ReadOnly]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.ReadOnly
      ).length,
      [CapabilityClass.HandoffOnly]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.HandoffOnly
      ).length,
      [CapabilityClass.Internal]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Internal
      ).length,
      [CapabilityClass.Unsupported]: inboxSurfaces.filter(
        (s) => s.capabilityClass === CapabilityClass.Unsupported
      ).length,
    },
    platformSupport: {
      desktop: inboxSurfaces.filter(
        (s) =>
          s.platformCapabilities?.desktop === CapabilityClass.FullySupported
      ).length,
      android: inboxSurfaces.filter(
        (s) =>
          s.platformCapabilities?.android === CapabilityClass.FullySupported
      ).length,
      browser: inboxSurfaces.filter(
        (s) =>
          s.platformCapabilities?.browser === CapabilityClass.FullySupported
      ).length,
    },
  };
}

export default {
  inboxSurfaces,
  inboxRegistry,
  getSurfacesByItemType,
  getSurfacesByEventSource,
  getPlatformNotificationSupport,
  getInboxSummary,
};
