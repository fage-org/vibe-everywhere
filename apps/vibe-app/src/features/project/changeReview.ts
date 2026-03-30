import type { GitChangedFile, GitDiffFileResponse, GitInspectResponse } from "@/types";

export type ChangeReviewSummary = {
  titleKey: string;
  detailKey: string;
  detailParams?: Record<string, string | number>;
  tone: "default" | "warning" | "danger";
};

export function buildChangeReviewSummaries(
  gitInspect: GitInspectResponse | null
): ChangeReviewSummary[] {
  if (!gitInspect || gitInspect.state !== "ready") {
    return [
      {
        titleKey: "changes.reviewUnavailableTitle",
        detailKey: "changes.reviewUnavailableSummary",
        tone: "warning"
      }
    ];
  }

  if (!gitInspect.changedFiles.length) {
    return [
      {
        titleKey: "changes.reviewCleanTitle",
        detailKey: "changes.reviewCleanSummary",
        tone: "default"
      }
    ];
  }

  const summaries: ChangeReviewSummary[] = [
    {
      titleKey: "changes.reviewScopeTitle",
      detailKey: "changes.reviewScopeSummary",
      detailParams: {
        count: gitInspect.diffStats.changedFiles,
        branch: gitInspect.branchName ?? "HEAD"
      },
      tone: "default"
    }
  ];

  if (gitInspect.diffStats.conflictedFiles > 0) {
    summaries.push({
      titleKey: "changes.reviewConflictTitle",
      detailKey: "changes.reviewConflictSummary",
      detailParams: { count: gitInspect.diffStats.conflictedFiles },
      tone: "danger"
    });
  } else if (gitInspect.changedFiles.some((file) => file.status === "deleted")) {
    summaries.push({
      titleKey: "changes.reviewDeleteTitle",
      detailKey: "changes.reviewDeleteSummary",
      tone: "warning"
    });
  } else if (gitInspect.diffStats.untrackedFiles > 0) {
    summaries.push({
      titleKey: "changes.reviewNewFilesTitle",
      detailKey: "changes.reviewNewFilesSummary",
      detailParams: { count: gitInspect.diffStats.untrackedFiles },
      tone: "warning"
    });
  }

  if (gitInspect.diffStats.stagedFiles > 0 && gitInspect.diffStats.unstagedFiles > 0) {
    summaries.push({
      titleKey: "changes.reviewMixedTitle",
      detailKey: "changes.reviewMixedSummary",
      tone: "default"
    });
  }

  return summaries;
}

export function deriveChangeRiskTone(file: GitChangedFile) {
  if (file.status === "updated_but_unmerged" || file.status === "deleted") {
    return "danger";
  }
  if (file.status === "untracked" || file.status === "type_changed") {
    return "warning";
  }
  return "default";
}

export function deriveChangeRiskLabelKey(file: GitChangedFile) {
  if (file.status === "updated_but_unmerged") {
    return "changes.riskConflict";
  }
  if (file.status === "deleted") {
    return "changes.riskDelete";
  }
  if (file.status === "untracked") {
    return "changes.riskNewFile";
  }
  if (file.status === "type_changed") {
    return "changes.riskTypeChange";
  }
  return "changes.riskStandard";
}

export function buildDiffSections(diff: GitDiffFileResponse | null) {
  if (!diff || diff.state !== "ready") {
    return [];
  }

  return [
    diff.stagedPatch
      ? { id: "staged", titleKey: "changes.diffStaged", patch: diff.stagedPatch }
      : null,
    diff.unstagedPatch
      ? { id: "unstaged", titleKey: "changes.diffUnstaged", patch: diff.unstagedPatch }
      : null
  ].filter((value): value is { id: string; titleKey: string; patch: string } => Boolean(value));
}
