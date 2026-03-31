import type {
  ConversationDetailResponse,
  RelayEventEnvelope,
  TaskDetailResponse
} from "@/types";

export function shouldRefreshConversationDetail(
  event: RelayEventEnvelope,
  conversationId: string,
  detail: ConversationDetailResponse | null
) {
  if (event.eventType === "task_updated") {
    return event.task?.conversationId === conversationId;
  }

  if (event.eventType !== "task_event" || !event.taskEvent) {
    return false;
  }

  return currentConversationTaskIds(detail).has(event.taskEvent.taskId);
}

export function buildTaskFailureSummary(taskDetail: TaskDetailResponse) {
  const providerStderr = [...taskDetail.events]
    .reverse()
    .find((event) => event.kind === "provider_stderr" && event.message.trim());
  if (providerStderr) {
    return providerStderr.message.trim();
  }

  const systemError = [...taskDetail.events]
    .reverse()
    .find(
      (event) =>
        event.kind === "system" &&
        event.message.trim() &&
        /(turn\.failed|error)/i.test(event.message)
    );
  if (systemError) {
    return systemError.message.trim();
  }

  return taskDetail.task.error?.trim() || "";
}

function currentConversationTaskIds(detail: ConversationDetailResponse | null) {
  return new Set((detail?.tasks ?? []).map((task) => task.task.id));
}
