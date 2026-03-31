import { describe, expect, it } from "vitest";
import {
  buildTaskFailureSummary,
  shouldRefreshConversationDetail
} from "@/lib/conversationRealtime";
import type {
  ConversationDetailResponse,
  RelayEventEnvelope,
  TaskDetailResponse
} from "@/types";

function taskDetail(overrides?: Partial<TaskDetailResponse>): TaskDetailResponse {
  return {
    task: {
      id: "task-1",
      deviceId: "device-1",
      conversationId: "conversation-1",
      title: "title",
      provider: "codex",
      executionProtocol: "acp",
      executionMode: "workspace_write",
      prompt: "prompt",
      cwd: "/root/vibe-remote",
      model: "gpt-5.4",
      providerSessionId: null,
      pendingInputRequestId: null,
      status: "failed",
      cancelRequested: false,
      createdAtEpochMs: 1,
      startedAtEpochMs: 2,
      finishedAtEpochMs: 3,
      exitCode: null,
      error: "generic failure",
      lastEventSeq: 3
    },
    events: [],
    pendingInputRequest: null,
    ...overrides
  };
}

function conversationDetail(): ConversationDetailResponse {
  return {
    conversation: {
      id: "conversation-1",
      deviceId: "device-1",
      title: "title",
      provider: "codex",
      executionProtocol: "acp",
      executionMode: "workspace_write",
      cwd: "/root/vibe-remote",
      model: "gpt-5.4",
      providerSessionId: null,
      latestTaskId: "task-1",
      pendingInputRequestId: null,
      archived: false,
      createdAtEpochMs: 1,
      updatedAtEpochMs: 2
    },
    tasks: [taskDetail()],
    pendingInputRequest: null
  };
}

describe("shouldRefreshConversationDetail", () => {
  it("refreshes when the task update belongs to the current conversation", () => {
    const event: RelayEventEnvelope = {
      eventType: "task_updated",
      device: null,
      task: { ...taskDetail().task, conversationId: "conversation-1" },
      taskEvent: null
    };

    expect(
      shouldRefreshConversationDetail(event, "conversation-1", conversationDetail())
    ).toBe(true);
  });

  it("refreshes when the task event belongs to a task in the current detail", () => {
    const event: RelayEventEnvelope = {
      eventType: "task_event",
      device: null,
      task: null,
      taskEvent: {
        seq: 4,
        taskId: "task-1",
        deviceId: "device-1",
        kind: "system",
        message: "error",
        timestampEpochMs: 10
      }
    };

    expect(
      shouldRefreshConversationDetail(event, "conversation-1", conversationDetail())
    ).toBe(true);
  });
});

describe("buildTaskFailureSummary", () => {
  it("prefers provider stderr over generic task error", () => {
    const detail = taskDetail({
      events: [
        {
          seq: 1,
          taskId: "task-1",
          deviceId: "device-1",
          kind: "provider_stderr",
          message: "model not supported",
          timestampEpochMs: 1
        }
      ]
    });

    expect(buildTaskFailureSummary(detail)).toBe("model not supported");
  });

  it("falls back to system error events when stderr is absent", () => {
    const detail = taskDetail({
      events: [
        {
          seq: 2,
          taskId: "task-1",
          deviceId: "device-1",
          kind: "system",
          message: "Codex event turn.failed: invalid model",
          timestampEpochMs: 2
        }
      ]
    });

    expect(buildTaskFailureSummary(detail)).toContain("invalid model");
  });
});
