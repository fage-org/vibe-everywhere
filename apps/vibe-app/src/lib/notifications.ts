import type { PortForwardRecord, TaskRecord } from "../types";

export type ActivitySeverity = "info" | "success" | "warning" | "error";
export type ActivityCategory =
  | "task_succeeded"
  | "task_failed"
  | "task_canceled"
  | "preview_ready"
  | "preview_failed";
export type ActivityResourceKind = "task" | "preview";

export type ActivityItem = {
  id: string;
  fingerprint: string;
  category: ActivityCategory;
  severity: ActivitySeverity;
  resourceKind: ActivityResourceKind;
  resourceId: string;
  deviceId: string;
  title: string;
  description: string;
  timestampEpochMs: number;
  unread: boolean;
};

type Translate = (key: string, params?: Record<string, unknown>) => string;

function createActivity(
  category: ActivityCategory,
  severity: ActivitySeverity,
  resourceKind: ActivityResourceKind,
  resourceId: string,
  deviceId: string,
  timestampEpochMs: number,
  title: string,
  description: string
): ActivityItem {
  return {
    id: `${resourceKind}:${resourceId}:${category}:${timestampEpochMs}`,
    fingerprint: `${resourceKind}:${resourceId}:${category}`,
    category,
    severity,
    resourceKind,
    resourceId,
    deviceId,
    title,
    description,
    timestampEpochMs,
    unread: true
  };
}

export function buildTaskActivity(
  previous: TaskRecord | null,
  next: TaskRecord,
  translate: Translate
): ActivityItem | null {
  if (!previous || previous.status === next.status) {
    return null;
  }

  if (next.status === "succeeded") {
    return createActivity(
      "task_succeeded",
      "success",
      "task",
      next.id,
      next.deviceId,
      next.finishedAtEpochMs ?? next.createdAtEpochMs,
      translate("dashboard.activity.categories.taskSucceeded"),
      translate("dashboard.activity.messages.taskSucceeded", {
        title: next.title,
        deviceId: next.deviceId
      })
    );
  }

  if (next.status === "failed") {
    return createActivity(
      "task_failed",
      "error",
      "task",
      next.id,
      next.deviceId,
      next.finishedAtEpochMs ?? next.createdAtEpochMs,
      translate("dashboard.activity.categories.taskFailed"),
      translate("dashboard.activity.messages.taskFailed", {
        title: next.title,
        deviceId: next.deviceId
      })
    );
  }

  if (next.status === "canceled") {
    return createActivity(
      "task_canceled",
      "warning",
      "task",
      next.id,
      next.deviceId,
      next.finishedAtEpochMs ?? next.createdAtEpochMs,
      translate("dashboard.activity.categories.taskCanceled"),
      translate("dashboard.activity.messages.taskCanceled", {
        title: next.title,
        deviceId: next.deviceId
      })
    );
  }

  return null;
}

export function buildPreviewActivity(
  previous: PortForwardRecord | null,
  next: PortForwardRecord,
  translate: Translate
): ActivityItem | null {
  if (!previous || previous.status === next.status) {
    return null;
  }

  if (next.status === "active") {
    return createActivity(
      "preview_ready",
      "success",
      "preview",
      next.id,
      next.deviceId,
      next.startedAtEpochMs ?? next.createdAtEpochMs,
      translate("dashboard.activity.categories.previewReady"),
      translate("dashboard.activity.messages.previewReady", {
        host: next.targetHost,
        port: next.targetPort,
        deviceId: next.deviceId
      })
    );
  }

  if (next.status === "failed") {
    return createActivity(
      "preview_failed",
      "error",
      "preview",
      next.id,
      next.deviceId,
      next.finishedAtEpochMs ?? next.createdAtEpochMs,
      translate("dashboard.activity.categories.previewFailed"),
      translate("dashboard.activity.messages.previewFailed", {
        host: next.targetHost,
        port: next.targetPort,
        deviceId: next.deviceId
      })
    );
  }

  return null;
}

export function readNotificationPermission(): NotificationPermission {
  if (typeof Notification === "undefined") {
    return "denied";
  }

  return Notification.permission;
}

export async function publishSystemActivity(
  activity: ActivityItem,
  enabled: boolean
): Promise<NotificationPermission> {
  if (!enabled || typeof Notification === "undefined") {
    return "denied";
  }

  let permission = Notification.permission;
  if (permission === "default") {
    permission = await Notification.requestPermission();
  }

  if (permission === "granted") {
    new Notification(activity.title, {
      body: activity.description,
      tag: activity.fingerprint
    });
  }

  return permission;
}
