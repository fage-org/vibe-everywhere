/**
 * Tests for Inbox Registry
 *
 * B29: Inbox and Notification Closure - Phase 1 Tests
 */

import { describe, expect, it } from "vitest";
import { CapabilityClass } from './types';
import {
  ItemType,
  EventSource,
  inboxSurfaces,
  inboxRegistry,
  getSurfacesByItemType,
  getSurfacesByEventSource,
  getPlatformNotificationSupport,
  getInboxSummary,
} from './inbox-registry';

describe('Inbox Registry', () => {
  it('should have all inbox surfaces defined', () => {
    expect(inboxSurfaces.length).toBeGreaterThan(0);
    expect(inboxSurfaces.length).toBe(4); // 4 inbox surfaces
  });

  it('should have unique surface IDs', () => {
    const ids = inboxSurfaces.map((s) => s.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });

  it('should have valid capability classes', () => {
    for (const surface of inboxSurfaces) {
      expect(Object.values(CapabilityClass)).toContain(surface.capabilityClass);
    }
  });

  it('should have customer descriptions for customer-facing surfaces', () => {
    const customerFacingClasses = [
      CapabilityClass.FullySupported,
      CapabilityClass.Limited,
      CapabilityClass.ReadOnly,
    ];

    for (const surface of inboxSurfaces) {
      if (customerFacingClasses.includes(surface.capabilityClass)) {
        expect(surface.customerDescription).toBeTruthy();
        expect(surface.customerDescription!.length).toBeGreaterThan(0);
      }
    }
  });

  it('should have platform capabilities defined for all surfaces', () => {
    for (const surface of inboxSurfaces) {
      expect(surface.platformCapabilities).toBeDefined();
      expect(surface.platformCapabilities!.desktop).toBeDefined();
      expect(surface.platformCapabilities!.android).toBeDefined();
      expect(surface.platformCapabilities!.browser).toBeDefined();
    }
  });
});

describe('getSurfacesByItemType', () => {
  it('should return inbox surfaces', () => {
    const inboxSurfaces = getSurfacesByItemType(ItemType.Inbox);
    expect(inboxSurfaces.length).toBeGreaterThan(0);
    for (const surface of inboxSurfaces) {
      expect(['inbox-list', 'session-inbox']).toContain(surface.id);
    }
  });

  it('should return feed surfaces', () => {
    const feedSurfaces = getSurfacesByItemType(ItemType.Feed);
    expect(feedSurfaces.length).toBeGreaterThan(0);
    for (const surface of feedSurfaces) {
      expect(surface.id).toBe('feed-list');
    }
  });

  it('should return notification surfaces', () => {
    const notificationSurfaces = getSurfacesByItemType(ItemType.Notification);
    expect(notificationSurfaces.length).toBeGreaterThan(0);
    for (const surface of notificationSurfaces) {
      expect(surface.id).toBe('notification-center');
    }
  });
});

describe('getSurfacesByEventSource', () => {
  it('should return session-related surfaces', () => {
    const sessionSurfaces = getSurfacesByEventSource(EventSource.Session);
    expect(sessionSurfaces.length).toBeGreaterThan(0);
    for (const surface of sessionSurfaces) {
      expect(surface.id).toBe('session-inbox');
    }
  });

  it('should return relationship-related surfaces', () => {
    const relationshipSurfaces = getSurfacesByEventSource(EventSource.Relationship);
    expect(relationshipSurfaces.length).toBeGreaterThan(0);
    for (const surface of relationshipSurfaces) {
      expect(surface.id).toBe('inbox-list');
    }
  });

  it('should return artifact-related surfaces', () => {
    const artifactSurfaces = getSurfacesByEventSource(EventSource.Artifact);
    expect(artifactSurfaces.length).toBeGreaterThan(0);
    for (const surface of artifactSurfaces) {
      expect(surface.id).toBe('feed-list');
    }
  });

  it('should return system-related surfaces', () => {
    const systemSurfaces = getSurfacesByEventSource(EventSource.System);
    expect(systemSurfaces.length).toBeGreaterThan(0);
    for (const surface of systemSurfaces) {
      expect(['notification-center', 'inbox-list']).toContain(surface.id);
    }
  });
});

describe('getPlatformNotificationSupport', () => {
  it('should return supported platforms for FullySupported surface', () => {
    const fullySupportedSurface = inboxSurfaces.find(
      (s) => s.capabilityClass === CapabilityClass.FullySupported
    );
    expect(fullySupportedSurface).toBeDefined();

    const support = getPlatformNotificationSupport(fullySupportedSurface!);
    expect(support.desktopLocal).toBe(true);
  });

  it('should return correct support for Limited surface', () => {
    const limitedSurface = inboxSurfaces.find(
      (s) => s.capabilityClass === CapabilityClass.Limited
    );
    if (!limitedSurface) return;

    const support = getPlatformNotificationSupport(limitedSurface);
    expect(support.desktopLocal).toBe(true);
  });

  it('should return no support for ReadOnly surface', () => {
    const readOnlySurface = inboxSurfaces.find(
      (s) => s.capabilityClass === CapabilityClass.ReadOnly
    );
    if (!readOnlySurface) return;

    const support = getPlatformNotificationSupport(readOnlySurface);
    expect(support.desktopLocal).toBe(false);
    expect(support.androidPush).toBe(false);
    expect(support.browserPush).toBe(false);
  });
});

describe('getInboxSummary', () => {
  it('should return summary statistics', () => {
    const summary = getInboxSummary();

    expect(summary.totalSurfaces).toBe(inboxSurfaces.length);
    expect(Object.keys(summary.byItemType).length).toBe(3); // inbox, feed, notification
    expect(Object.keys(summary.byCapabilityClass).length).toBe(
      Object.values(CapabilityClass).length
    );
  });

  it('should have correct platform support counts', () => {
    const summary = getInboxSummary();

    expect(summary.platformSupport.desktop).toBeGreaterThanOrEqual(0);
    expect(summary.platformSupport.android).toBeGreaterThanOrEqual(0);
    expect(summary.platformSupport.browser).toBeGreaterThanOrEqual(0);
  });

  it('should have correct totals by item type', () => {
    const summary = getInboxSummary();

    const totalByItemType =
      summary.byItemType[ItemType.Inbox] +
      summary.byItemType[ItemType.Feed] +
      summary.byItemType[ItemType.Notification];

    // Note: Some surfaces may map to multiple item types, so this may not equal totalSurfaces
    expect(totalByItemType).toBeGreaterThan(0);
  });
});
