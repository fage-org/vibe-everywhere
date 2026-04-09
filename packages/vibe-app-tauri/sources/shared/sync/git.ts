export type LineParseHandler<T> = (
  target: T,
  matches: (string | undefined)[],
  lines: string[],
  index: number,
) => void;

class LineParser<T> {
  private readonly regexes: readonly RegExp[];
  private readonly handler: LineParseHandler<T>;

  constructor(regexes: RegExp | RegExp[], handler: LineParseHandler<T>) {
    this.regexes = Array.isArray(regexes) ? regexes : [regexes];
    this.handler = handler;
  }

  parse(target: T, lines: string[]): T {
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      for (const regex of this.regexes) {
        const match = regex.exec(line);
        if (match) {
          this.handler(target, match, lines, index);
          break;
        }
      }
    }
    return target;
  }
}

export interface DiffFileStat {
  file: string;
  changes: number;
  insertions: number;
  deletions: number;
  binary: boolean;
}

export interface DiffSummary {
  files: DiffFileStat[];
  insertions: number;
  deletions: number;
  changes: number;
  changed: number;
}

const NUMSTAT_REGEX = /^(\d+|-)\t(\d+|-)\t(.*)$/;
const BRANCH_OID_REGEX = /^# branch\.oid (.+)$/;
const BRANCH_HEAD_REGEX = /^# branch\.head (.+)$/;
const BRANCH_UPSTREAM_REGEX = /^# branch\.upstream (.+)$/;
const BRANCH_AB_REGEX = /^# branch\.ab \+(\d+) -(\d+)$/;
const STASH_REGEX = /^# stash (\d+)$/;
const ORDINARY_CHANGE_REGEX =
  /^1 (.)(.) (.{4}) (\d{6}) (\d{6}) (\d{6}) ([0-9a-f]+) ([0-9a-f]+) (.+)$/;
const RENAME_COPY_REGEX =
  /^2 (.)(.) (.{4}) (\d{6}) (\d{6}) (\d{6}) ([0-9a-f]+) ([0-9a-f]+) ([RC])(\d{1,3}) (.+)\t(.+)$/;
const UNMERGED_REGEX =
  /^u (.)(.) (.{4}) (\d{6}) (\d{6}) (\d{6}) (\d{6}) ([0-9a-f]+) ([0-9a-f]+) ([0-9a-f]+) (.+)$/;
const UNTRACKED_REGEX = /^\? (.+)$/;
const IGNORED_REGEX = /^! (.+)$/;

export function parseNumStat(numStatOutput: string): DiffSummary {
  const lines = numStatOutput.trim().split("\n").filter((line) => line.length > 0);
  const result: DiffSummary = {
    files: [],
    insertions: 0,
    deletions: 0,
    changes: 0,
    changed: 0,
  };

  return new LineParser(
    NUMSTAT_REGEX,
    (target: DiffSummary, matches: (string | undefined)[]) => {
      const insertionsStr = matches[1];
      const deletionsStr = matches[2];
      const file = matches[3];

      if (!file || !insertionsStr || !deletionsStr) {
        return;
      }

      const binary = insertionsStr === "-" || deletionsStr === "-";
      const insertions = binary ? 0 : Number.parseInt(insertionsStr, 10);
      const deletions = binary ? 0 : Number.parseInt(deletionsStr, 10);

      target.files.push({
        file,
        changes: insertions + deletions,
        insertions,
        deletions,
        binary,
      });
      target.insertions += insertions;
      target.deletions += deletions;
      target.changes += insertions + deletions;
      target.changed += 1;
    },
  ).parse(result, lines);
}

export function createDiffStatsMap(
  summary: DiffSummary,
): Record<string, { added: number; removed: number; binary: boolean }> {
  const stats: Record<string, { added: number; removed: number; binary: boolean }> = {};

  for (const file of summary.files) {
    stats[file.file] = {
      added: file.insertions,
      removed: file.deletions,
      binary: file.binary,
    };
  }

  return stats;
}

export interface GitFileEntryV2 {
  path: string;
  index: string;
  working_dir: string;
  from?: string;
  submoduleState?: string;
  modeHead?: string;
  modeIndex?: string;
  modeWorktree?: string;
  hashHead?: string;
  hashIndex?: string;
  renameScore?: number;
}

export interface GitBranchInfo {
  oid?: string;
  head?: string;
  upstream?: string;
  ahead?: number;
  behind?: number;
}

export interface GitStatusSummaryV2 {
  files: GitFileEntryV2[];
  staged: string[];
  modified: string[];
  created: string[];
  deleted: string[];
  renamed: string[];
  conflicted: string[];
  not_added: string[];
  ignored: string[];
  branch: GitBranchInfo;
  stashCount?: number;
}

export function parseStatusSummaryV2(statusOutput: string): GitStatusSummaryV2 {
  const lines = statusOutput.trim().split("\n").filter((line) => line.length > 0);
  const result: GitStatusSummaryV2 = {
    files: [],
    staged: [],
    modified: [],
    created: [],
    deleted: [],
    renamed: [],
    conflicted: [],
    not_added: [],
    ignored: [],
    branch: {},
  };

  for (const line of lines) {
    if (line.startsWith("# branch.oid ")) {
      const match = BRANCH_OID_REGEX.exec(line);
      if (match) {
        result.branch.oid = match[1];
      }
      continue;
    }
    if (line.startsWith("# branch.head ")) {
      const match = BRANCH_HEAD_REGEX.exec(line);
      if (match) {
        result.branch.head = match[1];
      }
      continue;
    }
    if (line.startsWith("# branch.upstream ")) {
      const match = BRANCH_UPSTREAM_REGEX.exec(line);
      if (match) {
        result.branch.upstream = match[1];
      }
      continue;
    }
    if (line.startsWith("# branch.ab ")) {
      const match = BRANCH_AB_REGEX.exec(line);
      if (match) {
        result.branch.ahead = Number.parseInt(match[1], 10);
        result.branch.behind = Number.parseInt(match[2], 10);
      }
      continue;
    }
    if (line.startsWith("# stash ")) {
      const match = STASH_REGEX.exec(line);
      if (match) {
        result.stashCount = Number.parseInt(match[1], 10);
      }
      continue;
    }
    if (line.startsWith("1 ")) {
      const match = ORDINARY_CHANGE_REGEX.exec(line);
      const entry = match ? parseOrdinaryChange(match) : null;
      if (entry) {
        result.files.push(entry);
        categorizeFileV2(result, entry);
      }
      continue;
    }
    if (line.startsWith("2 ")) {
      const match = RENAME_COPY_REGEX.exec(line);
      const entry = match ? parseRenameCopy(match) : null;
      if (entry) {
        result.files.push(entry);
        categorizeFileV2(result, entry);
      }
      continue;
    }
    if (line.startsWith("u ")) {
      const match = UNMERGED_REGEX.exec(line);
      const entry = match ? parseUnmerged(match) : null;
      if (entry) {
        result.files.push(entry);
        categorizeFileV2(result, entry);
      }
      continue;
    }
    if (line.startsWith("? ")) {
      const match = UNTRACKED_REGEX.exec(line);
      if (match) {
        result.not_added.push(match[1]);
      }
      continue;
    }
    if (line.startsWith("! ")) {
      const match = IGNORED_REGEX.exec(line);
      if (match) {
        result.ignored.push(match[1]);
      }
    }
  }

  return result;
}

function parseOrdinaryChange(matches: (string | undefined)[]): GitFileEntryV2 | null {
  if (!matches[1] || !matches[2] || !matches[9]) {
    return null;
  }

  return {
    index: matches[1],
    working_dir: matches[2],
    submoduleState: matches[3],
    modeHead: matches[4],
    modeIndex: matches[5],
    modeWorktree: matches[6],
    hashHead: matches[7],
    hashIndex: matches[8],
    path: matches[9],
  };
}

function parseRenameCopy(matches: (string | undefined)[]): GitFileEntryV2 | null {
  if (!matches[1] || !matches[2] || !matches[11] || !matches[12]) {
    return null;
  }

  return {
    index: matches[1],
    working_dir: matches[2],
    submoduleState: matches[3],
    modeHead: matches[4],
    modeIndex: matches[5],
    modeWorktree: matches[6],
    hashHead: matches[7],
    hashIndex: matches[8],
    renameScore: Number.parseInt(matches[10] || "0", 10),
    from: matches[11],
    path: matches[12],
  };
}

function parseUnmerged(matches: (string | undefined)[]): GitFileEntryV2 | null {
  if (!matches[1] || !matches[2] || !matches[11]) {
    return null;
  }

  return {
    index: matches[1],
    working_dir: matches[2],
    submoduleState: matches[3],
    modeHead: matches[4],
    modeIndex: matches[5],
    modeWorktree: matches[7],
    hashHead: matches[8],
    hashIndex: matches[9],
    path: matches[11],
  };
}

function categorizeFileV2(summary: GitStatusSummaryV2, entry: GitFileEntryV2): void {
  const { index, working_dir, path } = entry;

  if (index !== " " && index !== "." && index !== "?") {
    summary.staged.push(path);
    switch (index) {
      case "A":
        summary.created.push(path);
        break;
      case "D":
        summary.deleted.push(path);
        break;
      case "R":
      case "C":
        summary.renamed.push(path);
        break;
      case "M":
        if (!summary.created.includes(path)) {
          summary.modified.push(path);
        }
        break;
      case "U":
        summary.conflicted.push(path);
        break;
      default:
        break;
    }
  }

  if (working_dir !== " " && working_dir !== ".") {
    switch (working_dir) {
      case "M":
        if (!summary.modified.includes(path)) {
          summary.modified.push(path);
        }
        break;
      case "D":
        if (!summary.deleted.includes(path)) {
          summary.deleted.push(path);
        }
        break;
      case "R":
      case "C":
        if (!summary.renamed.includes(path)) {
          summary.renamed.push(path);
        }
        break;
      case "?":
        summary.not_added.push(path);
        break;
      case "U":
        if (!summary.conflicted.includes(path)) {
          summary.conflicted.push(path);
        }
        break;
      default:
        break;
    }
  }
}

export function getCurrentBranchV2(summary: GitStatusSummaryV2): string | null {
  const head = summary.branch.head;
  return head && head !== "(detached)" && head !== "(initial)" ? head : null;
}
