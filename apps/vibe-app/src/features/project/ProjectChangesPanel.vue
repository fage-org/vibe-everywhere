<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDateTime } from "@/lib/format";
import { buildChangeReviewSummaries, buildDiffSections, deriveChangeRiskLabelKey, deriveChangeRiskTone } from "./changeReview";
import type { GitDiffFileResponse, GitInspectResponse } from "@/types";

const props = defineProps<{
  gitInspect: GitInspectResponse | null;
  gitDiff: GitDiffFileResponse | null;
  activeRepoPath: string | null;
}>();

const emit = defineEmits<{
  selectFile: [repoPath: string];
}>();

const { t } = useI18n();

const reviewSummaries = computed(() => buildChangeReviewSummaries(props.gitInspect));
const diffSections = computed(() => buildDiffSections(props.gitDiff));
</script>

<template>
  <div class="space-y-5">
    <section class="grid gap-4 lg:grid-cols-4">
      <article class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.branch") }}</p>
        <p class="mt-2 text-lg font-semibold">{{ gitInspect?.branchName ?? "—" }}</p>
      </article>
      <article class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.changedFiles") }}</p>
        <p class="mt-2 text-lg font-semibold">{{ gitInspect?.diffStats.changedFiles ?? 0 }}</p>
      </article>
      <article class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.aheadBehind") }}</p>
        <p class="mt-2 text-lg font-semibold">
          {{ gitInspect?.aheadCount ?? 0 }} / {{ gitInspect?.behindCount ?? 0 }}
        </p>
      </article>
      <article class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.untracked") }}</p>
        <p class="mt-2 text-lg font-semibold">{{ gitInspect?.diffStats.untrackedFiles ?? 0 }}</p>
      </article>
    </section>

    <section class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.reviewTitle") }}</h3>
          <p class="mt-2 text-sm text-muted-foreground">{{ t("changes.reviewSummary") }}</p>
        </div>
        <StatusBadge :tone="gitInspect?.diffStats.conflictedFiles ? 'danger' : gitInspect?.diffStats.changedFiles ? 'warning' : 'success'">
          {{
            gitInspect?.diffStats.changedFiles
              ? t("changes.reviewNeedsAttention")
              : t("changes.reviewClean")
          }}
        </StatusBadge>
      </div>
      <div class="mt-4 grid gap-3 lg:grid-cols-2">
        <article
          v-for="summary in reviewSummaries"
          :key="summary.titleKey"
          class="rounded-2xl bg-background/75 p-4"
        >
          <div class="flex items-center gap-2">
            <StatusBadge :tone="summary.tone">{{ t(summary.titleKey) }}</StatusBadge>
          </div>
          <p class="mt-3 text-sm text-muted-foreground">
            {{ t(summary.detailKey, summary.detailParams ?? {}) }}
          </p>
        </article>
      </div>
    </section>

    <div class="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
      <section class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.changedFiles") }}</h3>
        <div v-if="gitInspect?.changedFiles.length" class="mt-4 space-y-2">
          <button
            v-for="file in gitInspect.changedFiles"
            :key="file.path"
            class="w-full rounded-2xl border px-4 py-3 text-left"
            :class="
              activeRepoPath === file.repoPath
                ? 'border-primary bg-primary/10'
                : 'border-border bg-background/75'
            "
            @click="emit('selectFile', file.repoPath)"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-medium text-foreground">{{ file.repoPath }}</p>
                <p class="mt-1 text-xs text-muted-foreground">{{ file.path }}</p>
                <div class="mt-2 flex flex-wrap gap-2 text-[11px] text-muted-foreground">
                  <span v-if="file.staged" class="rounded-full bg-background px-2 py-1">{{ t("changes.staged") }}</span>
                  <span v-if="file.unstaged" class="rounded-full bg-background px-2 py-1">{{ t("changes.unstaged") }}</span>
                </div>
              </div>
              <div class="flex flex-col items-end gap-2">
                <StatusBadge :tone="deriveChangeRiskTone(file)">
                  {{ t(deriveChangeRiskLabelKey(file)) }}
                </StatusBadge>
                <StatusBadge :tone="file.status === 'deleted' ? 'danger' : file.status === 'untracked' ? 'warning' : 'default'">
                  {{ file.status }}
                </StatusBadge>
              </div>
            </div>
          </button>
        </div>
        <p v-else class="mt-4 text-sm text-muted-foreground">{{ t("changes.noChanges") }}</p>
      </section>

      <section class="space-y-5">
        <div class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.diffTitle") }}</h3>
              <p class="mt-2 text-sm text-muted-foreground">
                {{ activeRepoPath ? activeRepoPath : t("changes.diffEmpty") }}
              </p>
            </div>
            <StatusBadge v-if="gitDiff?.isBinary" tone="warning">{{ t("changes.binary") }}</StatusBadge>
          </div>
          <p v-if="gitDiff?.truncated" class="mt-3 text-xs text-muted-foreground">
            {{ t("changes.diffTruncated") }}
          </p>
          <div v-if="diffSections.length" class="mt-4 space-y-4">
            <article
              v-for="section in diffSections"
              :key="section.id"
              class="overflow-hidden rounded-2xl border border-border bg-background/80"
            >
              <div class="border-b border-border px-4 py-3 text-sm font-medium text-foreground">
                {{ t(section.titleKey) }}
              </div>
              <pre class="overflow-x-auto px-4 py-4 text-xs leading-6 text-foreground"><code>{{ section.patch }}</code></pre>
            </article>
          </div>
          <p v-else class="mt-4 text-sm text-muted-foreground">{{ t("changes.diffUnavailable") }}</p>
        </div>

        <div class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
          <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("changes.recentCommits") }}</h3>
          <div v-if="gitInspect?.recentCommits.length" class="mt-4 space-y-2">
            <article
              v-for="commit in gitInspect.recentCommits"
              :key="commit.id"
              class="rounded-2xl border border-border bg-background/75 px-4 py-3"
            >
              <div class="flex items-center justify-between gap-4">
                <p class="text-sm font-medium">{{ commit.summary }}</p>
                <span class="text-xs text-muted-foreground">{{ commit.shortId }}</span>
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ commit.authorName }} · {{ formatDateTime(commit.committedAtEpochMs) }}
              </p>
            </article>
          </div>
          <p v-else class="mt-4 text-sm text-muted-foreground">{{ t("changes.noCommits") }}</p>
        </div>
      </section>
    </div>
  </div>
</template>
