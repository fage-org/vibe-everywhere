<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import {
  Archive,
  ArrowLeft,
  Bot,
  FolderGit2,
  History,
  LoaderCircle,
  MessageSquarePlus,
  Send,
  Sparkles,
  UserRound,
  X,
} from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import type { ConversationInputOption, ConversationRecord, TaskDetailResponse } from "@/types";
import { useDashboardController } from "@/views/dashboard/controller";
import {
  collectProjectsForDevice,
  conversationMatchesProject,
  decodeProjectKey,
  normalizeProjectCwd,
} from "@/views/chat/projects";

const CUSTOM_INPUT_SENTINEL = "__custom_input__";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const dashboard = useDashboardController();
const store = dashboard.store;
const { devices, draft, pendingConversationInputDraft } = storeToRefs(store);

const drawerOpen = ref(false);

const routeDeviceId = computed(() => String(route.params.deviceId ?? ""));
const routeProjectCwd = computed(() => decodeProjectKey(route.params.projectKey));

const projectConversations = computed(() =>
  store.visibleConversations
    .filter((conversation) =>
      conversationMatchesProject(
        conversation,
        routeDeviceId.value,
        routeProjectCwd.value,
      ),
    )
    .sort((left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs),
);

const currentDevice = computed(
  () => devices.value.find((device) => device.id === routeDeviceId.value) ?? null,
);
const projectInfo = computed(() =>
  collectProjectsForDevice(store.visibleConversations, routeDeviceId.value).find(
    (project) => normalizeProjectCwd(project.cwd) === normalizeProjectCwd(routeProjectCwd.value),
  ) ?? null,
);
const projectTitle = computed(
  () => projectInfo.value?.name ?? t("chatProject.defaultWorkspaceTitle"),
);
const projectPath = computed(
  () => routeProjectCwd.value ?? t("chatProject.defaultWorkspacePath"),
);
const selectedConversation = computed(() => store.selectedConversation);
const pendingInputRequest = computed(() => store.conversationPendingInput);
const availableProviders = computed(() =>
  store.availableProviders.map((provider) => ({
    kind: provider.kind,
    label: dashboard.formatProviderKind(provider.kind),
  })),
);
const canSendPrompt = computed(() => {
  if (!draft.value.prompt.trim() || !store.relayBaseUrl) {
    return false;
  }

  if (selectedConversation.value) {
    return Boolean(currentDevice.value?.online);
  }

  return (
    Boolean(currentDevice.value?.online) &&
    Boolean(draft.value.provider)
  );
});
const transcriptTurns = computed(() =>
  (selectedConversation.value ? store.selectedConversationDetail?.tasks ?? [] : []).map((detail) => ({
    detail,
    assistantText: buildAssistantText(detail),
    traceEventCount: detail.events.filter(
      (event) =>
        event.kind === "tool_call" ||
        event.kind === "tool_output" ||
        event.kind === "system" ||
        event.kind === "status" ||
        event.kind === "provider_stderr",
    ).length,
  })),
);
const showPendingTextInput = computed(() => {
  if (!pendingInputRequest.value) {
    return false;
  }

  if (
    !pendingInputRequest.value.options.length &&
    pendingInputRequest.value.allowCustomInput
  ) {
    return true;
  }

  if (pendingConversationInputDraft.value.optionId === CUSTOM_INPUT_SENTINEL) {
    return true;
  }

  const selectedOption = pendingInputRequest.value.options.find(
    (option) => option.id === pendingConversationInputDraft.value.optionId,
  );
  return Boolean(selectedOption?.requiresTextInput);
});
const canSubmitPendingText = computed(
  () =>
    showPendingTextInput.value &&
    Boolean(pendingConversationInputDraft.value.text.trim()),
);

watch(
  [routeDeviceId, routeProjectCwd, () => devices.value.length, () => store.visibleConversations.length],
  async () => {
    if (!routeDeviceId.value || !currentDevice.value) {
      return;
    }

    if (store.selectedDeviceId !== routeDeviceId.value) {
      await store.selectDevice(routeDeviceId.value);
    }

    store.setProjectFolder(routeProjectCwd.value ?? "");

    const preferredConversation = projectConversations.value[0] ?? null;
    if (preferredConversation) {
      if (store.selectedConversationId !== preferredConversation.id) {
        await store.selectConversation(preferredConversation.id);
      }
      return;
    }

    store.startNewConversationDraft({
      deviceId: routeDeviceId.value,
      cwd: routeProjectCwd.value,
    });
  },
  { immediate: true },
);

function buildAssistantText(detail: TaskDetailResponse) {
  const assistant = detail.events
    .filter((event) => event.kind === "assistant_delta")
    .map((event) => event.message)
    .join("")
    .trim();

  return assistant;
}

function assistantFallbackText(detail: TaskDetailResponse) {
  switch (detail.task.status) {
    case "waiting_input":
      return t("chatProject.waitingInput");
    case "failed":
      return t("chatProject.traceOnlyFailed");
    case "succeeded":
      return t("chatProject.traceOnlyCompleted");
    case "canceled":
      return t("chatProject.traceOnlyCanceled");
    default:
      return t("chatProject.generating");
  }
}

function openConversation(conversation: ConversationRecord) {
  drawerOpen.value = false;
  void store.selectConversation(conversation.id);
}

function startNewTopic() {
  drawerOpen.value = false;
  store.startNewConversationDraft({
    deviceId: routeDeviceId.value,
    cwd: routeProjectCwd.value,
  });
}

function chooseProvider(provider: string) {
  draft.value.provider = provider as typeof draft.value.provider;
}

function choosePendingOption(option: ConversationInputOption) {
  pendingConversationInputDraft.value.optionId = option.id;
  if (!option.requiresTextInput) {
    pendingConversationInputDraft.value.text = "";
  }
}

function submitPendingInput() {
  const optionId =
    pendingConversationInputDraft.value.optionId === CUSTOM_INPUT_SENTINEL
      ? undefined
      : pendingConversationInputDraft.value.optionId ?? undefined;

  void store.respondToConversationInput({
    optionId,
    text: pendingConversationInputDraft.value.text.trim() || undefined,
  });
}

async function archiveConversation() {
  await store.archiveSelectedConversation();

  const nextConversation = projectConversations.value[0] ?? null;
  if (nextConversation) {
    await store.selectConversation(nextConversation.id);
    return;
  }

  store.startNewConversationDraft({
    deviceId: routeDeviceId.value,
    cwd: routeProjectCwd.value,
  });
}
</script>

<template>
  <section class="space-y-4">
    <header
      class="rounded-[30px] border border-border/70 bg-card/86 px-4 py-4 shadow-xl backdrop-blur-xl md:px-6"
    >
      <div class="flex items-start gap-3">
        <Button type="button" variant="outline" size="icon" @click="router.push({ name: 'chat-home' })">
          <ArrowLeft class="size-4" />
        </Button>
        <Button type="button" variant="outline" size="icon" @click="drawerOpen = true">
          <History class="size-4" />
        </Button>

        <div class="min-w-0 flex-1 space-y-2">
          <div class="flex flex-wrap items-center gap-2">
            <Badge
              variant="outline"
              class="border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
            >
              {{ t("chatProject.badge") }}
            </Badge>
            <Badge
              variant="outline"
              :class="
                currentDevice?.online
                  ? dashboard.statusBadgeClass('online')
                  : dashboard.statusBadgeClass('offline')
              "
            >
              {{ currentDevice?.name ?? t("dashboard.shell.noDeviceSelected") }}
            </Badge>
          </div>

          <div class="space-y-1">
            <h1 class="truncate text-2xl font-semibold tracking-tight text-foreground md:text-3xl">
              {{ projectTitle }}
            </h1>
            <p class="truncate font-mono text-xs text-muted-foreground md:text-sm">
              {{ projectPath }}
            </p>
          </div>
        </div>

        <div class="hidden shrink-0 items-center gap-2 md:flex">
          <Button type="button" variant="outline" @click="startNewTopic">
            <MessageSquarePlus class="size-4" />
            {{ t("chatProject.newTopic") }}
          </Button>
          <Button
            v-if="selectedConversation"
            type="button"
            variant="outline"
            @click="archiveConversation"
          >
            <Archive class="size-4" />
            {{ t("chatProject.archive") }}
          </Button>
        </div>
      </div>

      <div class="mt-4 flex flex-wrap gap-2">
        <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
          <FolderGit2 class="mr-1 size-3.5" />
          {{ t("chatProject.topicCount", { count: projectConversations.length }) }}
        </Badge>
        <Badge
          v-if="selectedConversation"
          variant="outline"
          class="border-border/70 bg-background/60 text-foreground"
        >
          {{ dashboard.formatProviderKind(selectedConversation.provider) }}
        </Badge>
        <Badge
          v-if="selectedConversation?.model"
          variant="outline"
          class="border-border/70 bg-background/60 text-foreground"
        >
          {{ selectedConversation.model }}
        </Badge>
      </div>
    </header>

    <Card class="border-border/70 bg-card/86 shadow-2xl backdrop-blur-xl">
      <CardContent class="flex min-h-[calc(100vh-17rem)] flex-col p-0">
        <ScrollArea class="min-h-0 flex-1">
          <div class="space-y-8 px-4 py-5 md:px-6">
            <template v-if="transcriptTurns.length">
              <article
                v-for="turn in transcriptTurns"
                :key="turn.detail.task.id"
                class="space-y-4"
              >
                <div class="flex justify-end">
                  <div
                    class="max-w-[92%] rounded-[28px] bg-slate-900 px-5 py-4 text-sm leading-7 text-slate-50 shadow-lg dark:bg-slate-50 dark:text-slate-900 md:max-w-[78%]"
                  >
                    <div class="mb-3 flex items-center justify-between gap-3">
                      <div class="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-slate-300 dark:text-slate-500">
                        <UserRound class="size-3.5" />
                        {{ t("chatProject.userTurn") }}
                      </div>
                      <span class="text-xs text-slate-300/80 dark:text-slate-500">
                        {{ dashboard.formatTimestamp(turn.detail.task.createdAtEpochMs) }}
                      </span>
                    </div>
                    <p class="whitespace-pre-wrap">{{ turn.detail.task.prompt }}</p>
                  </div>
                </div>

                <div class="flex gap-3">
                  <div
                    class="mt-1 flex size-10 shrink-0 items-center justify-center rounded-2xl bg-amber-500/12 text-amber-700 dark:text-amber-100"
                  >
                    <Bot class="size-5" />
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="rounded-[28px] border border-border/70 bg-background/72 px-5 py-4 shadow-sm">
                      <div class="mb-3 flex flex-wrap items-center gap-2">
                        <Badge variant="outline" :class="dashboard.taskStatusClass(turn.detail.task.status)">
                          {{ dashboard.formatTaskStatus(turn.detail.task.status) }}
                        </Badge>
                        <Badge
                          variant="outline"
                          class="border-border/70 bg-background/70 text-foreground"
                        >
                          {{ dashboard.formatExecutionProtocol(turn.detail.task.executionProtocol) }}
                        </Badge>
                        <Badge
                          v-if="turn.traceEventCount"
                          variant="outline"
                          class="border-border/70 bg-background/70 text-muted-foreground"
                        >
                          {{ t("chatProject.traceCount", { count: turn.traceEventCount }) }}
                        </Badge>
                      </div>

                      <p
                        v-if="turn.assistantText"
                        class="whitespace-pre-wrap text-sm leading-7 text-foreground"
                      >
                        {{ turn.assistantText }}
                      </p>
                      <p v-else class="text-sm leading-7 text-muted-foreground">
                        {{ assistantFallbackText(turn.detail) }}
                      </p>
                    </div>
                  </div>
                </div>
              </article>
            </template>

            <div
              v-else
              class="rounded-[32px] border border-dashed border-border/70 bg-gradient-to-br from-sky-500/8 via-background to-amber-500/8 px-6 py-14 text-center"
            >
              <div class="mx-auto max-w-xl space-y-3">
                <Badge
                  variant="outline"
                  class="border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
                >
                  {{ t("chatProject.emptyBadge") }}
                </Badge>
                <h2 class="text-2xl font-semibold tracking-tight text-foreground">
                  {{ t("chatProject.emptyTitle") }}
                </h2>
                <p class="text-sm leading-7 text-muted-foreground">
                  {{ t("chatProject.emptySummary") }}
                </p>
              </div>
            </div>
          </div>
        </ScrollArea>

        <div
          v-if="pendingInputRequest"
          class="border-t border-border/70 bg-amber-500/6 px-4 py-4 md:px-6"
        >
          <div class="space-y-4 rounded-[28px] border border-amber-500/25 bg-background/85 p-5 shadow-sm">
            <div class="space-y-1">
              <Badge
                variant="outline"
                class="border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-100"
              >
                {{ t("chatProject.inputRequestBadge") }}
              </Badge>
              <h3 class="text-lg font-semibold text-foreground">
                {{ pendingInputRequest.prompt }}
              </h3>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                v-for="option in pendingInputRequest.options"
                :key="option.id"
                type="button"
                variant="outline"
                :class="
                  pendingConversationInputDraft.optionId === option.id
                    ? 'border-primary/50 bg-primary/12 text-primary'
                    : ''
                "
                @click="choosePendingOption(option)"
              >
                {{ option.label }}
              </Button>
              <Button
                v-if="pendingInputRequest.allowCustomInput"
                type="button"
                variant="outline"
                :class="
                  pendingConversationInputDraft.optionId === CUSTOM_INPUT_SENTINEL
                    ? 'border-primary/50 bg-primary/12 text-primary'
                    : ''
                "
                @click="pendingConversationInputDraft.optionId = CUSTOM_INPUT_SENTINEL"
              >
                {{ t("chatProject.customReply") }}
              </Button>
            </div>

            <div v-if="showPendingTextInput" class="space-y-3">
              <Textarea
                :model-value="pendingConversationInputDraft.text"
                :placeholder="pendingInputRequest.customInputPlaceholder ?? t('chatProject.customReplyPlaceholder')"
                class="min-h-[100px] border-border/70 bg-background/80"
                @update:model-value="pendingConversationInputDraft.text = String($event)"
              />
              <Button type="button" :disabled="!canSubmitPendingText" @click="submitPendingInput">
                <Send class="size-4" />
                {{ t("chatProject.submitReply") }}
              </Button>
            </div>
          </div>
        </div>

        <div class="border-t border-border/70 px-4 py-4 md:px-6">
          <div class="space-y-3 rounded-[28px] border border-border/70 bg-background/70 p-4 shadow-sm">
            <div v-if="!selectedConversation" class="space-y-3">
              <div class="flex flex-wrap gap-2">
                <Button
                  v-for="provider in availableProviders"
                  :key="provider.kind"
                  type="button"
                  variant="outline"
                  size="sm"
                  :class="
                    draft.provider === provider.kind
                      ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                      : ''
                  "
                  @click="chooseProvider(provider.kind)"
                >
                  <Sparkles class="size-3.5" />
                  {{ provider.label }}
                </Button>
              </div>
              <Input
                :model-value="draft.model"
                :placeholder="t('chatProject.modelPlaceholder')"
                class="border-border/70 bg-background/80"
                @update:model-value="draft.model = String($event)"
              />
            </div>

            <Textarea
              :model-value="draft.prompt"
              :placeholder="t('chatProject.promptPlaceholder')"
              class="min-h-[120px] border-border/70 bg-background/80"
              @update:model-value="draft.prompt = String($event)"
            />

            <div class="flex items-center justify-between gap-3">
              <p class="truncate text-xs text-muted-foreground">
                {{
                  selectedConversation
                    ? t("chatProject.followupSummary")
                    : t("chatProject.newTopicSummary")
                }}
              </p>
              <Button type="button" :disabled="!canSendPrompt" @click="store.sendConversationPrompt">
                <LoaderCircle v-if="store.isBootstrapping" class="size-4 animate-spin" />
                <Send v-else class="size-4" />
                {{ t("chatProject.send") }}
              </Button>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>

    <div
      v-if="drawerOpen"
      class="fixed inset-0 z-50 bg-slate-950/45 backdrop-blur-[2px]"
      @click.self="drawerOpen = false"
    >
      <aside
        class="flex h-full w-[min(360px,92vw)] flex-col border-r border-border/70 bg-background/96 shadow-2xl"
      >
        <div class="flex items-start justify-between border-b border-border/70 px-5 py-4">
          <div class="space-y-1">
            <p class="text-lg font-semibold text-foreground">
              {{ t("chatProject.historyTitle") }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ t("chatProject.historySummary") }}
            </p>
          </div>
          <Button type="button" variant="outline" size="icon" @click="drawerOpen = false">
            <X class="size-4" />
          </Button>
        </div>

        <div class="space-y-3 border-b border-border/70 px-5 py-4">
          <Button type="button" class="w-full" @click="startNewTopic">
            <MessageSquarePlus class="size-4" />
            {{ t("chatProject.newTopic") }}
          </Button>
        </div>

        <ScrollArea class="min-h-0 flex-1">
          <div class="space-y-2 p-4">
            <button
              v-for="conversation in projectConversations"
              :key="conversation.id"
              type="button"
              class="w-full rounded-2xl border p-4 text-left transition"
              :class="
                store.selectedConversationId === conversation.id
                  ? 'border-primary/50 bg-primary/10'
                  : 'border-border/70 bg-background/60 hover:bg-accent/30'
              "
              @click="openConversation(conversation)"
            >
              <p class="truncate text-sm font-medium text-foreground">
                {{ conversation.title }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ dashboard.formatProviderKind(conversation.provider) }}
              </p>
              <p class="mt-2 text-xs text-muted-foreground">
                {{ dashboard.formatTimestamp(conversation.updatedAtEpochMs) }}
              </p>
            </button>

            <div
              v-if="!projectConversations.length"
              class="rounded-2xl border border-dashed border-border/70 px-4 py-8 text-center text-sm text-muted-foreground"
            >
              {{ t("chatProject.emptyHistory") }}
            </div>
          </div>
        </ScrollArea>
      </aside>
    </div>
  </section>
</template>
