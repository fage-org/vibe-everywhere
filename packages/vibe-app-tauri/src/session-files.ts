import {
  createDiffStatsMap,
  parseNumStat,
} from "../sources/shared/sync/git";
import {
  getCurrentBranchV2,
  parseStatusSummaryV2,
} from "../sources/shared/sync/git";

export type SessionWorkspaceFile = {
  fileName: string;
  filePath: string;
  relativePath: string;
  absolutePath: string;
  status: "modified" | "added" | "deleted" | "renamed" | "untracked";
  isStaged: boolean;
  linesAdded: number;
  linesRemoved: number;
  oldPath?: string;
};

export type SessionWorkspaceFileInventory = {
  branch: string | null;
  files: SessionWorkspaceFile[];
  totalStaged: number;
  totalUnstaged: number;
};

export type SessionWorkspaceFileContent = {
  relativePath: string;
  absolutePath: string;
  content: string;
  diff: string | null;
  isBinary: boolean;
  language: string | null;
};

function shellEscapeForPosix(argument: string): string {
  return `'${argument.replace(/'/g, `'\"'\"'`)}'`;
}

function joinPath(rootPath: string, relativePath: string): string {
  const root = rootPath.replace(/\/+$/, "");
  const relative = relativePath.replace(/^\/+/, "");
  return relative ? `${root}/${relative}` : root;
}

export function normalizeSessionRelativePath(relativePath: string): string {
  if (relativePath.length === 0) {
    throw new Error("Session file path is required");
  }

  if (
    relativePath.startsWith("/") ||
    relativePath.startsWith("~/") ||
    /^[A-Za-z]:[\\/]/.test(relativePath)
  ) {
    throw new Error("Session file path must stay relative to the workspace root");
  }

  if (relativePath.includes("\0") || relativePath.includes("\\")) {
    throw new Error("Session file path contains unsupported characters");
  }

  const segments = relativePath.split("/");
  if (segments.some((segment) => segment.length === 0 || segment === "." || segment === "..")) {
    throw new Error("Session file path must not contain traversal segments");
  }

  return segments.join("/");
}

export function buildWorkspaceFilePath(
  workspaceRoot: string,
  relativePath: string,
): string {
  return joinPath(workspaceRoot, normalizeSessionRelativePath(relativePath));
}

export function buildGitDiffCommand(relativePath: string): string {
  return `git diff --no-ext-diff -- ${shellEscapeForPosix(
    normalizeSessionRelativePath(relativePath),
  )}`;
}

function mapStatusCode(code: string): SessionWorkspaceFile["status"] {
  switch (code) {
    case "A":
      return "added";
    case "D":
      return "deleted";
    case "R":
    case "C":
      return "renamed";
    case "?":
      return "untracked";
    default:
      return "modified";
  }
}

export function parseSessionWorkspaceFiles(
  statusOutput: string,
  combinedDiffOutput: string,
  workspaceRoot: string,
): SessionWorkspaceFileInventory {
  const summary = parseStatusSummaryV2(statusOutput);
  const branch = getCurrentBranchV2(summary);
  const [unstagedOutput = "", stagedOutput = ""] = combinedDiffOutput.split("---STAGED---");
  const unstagedStats = createDiffStatsMap(parseNumStat(unstagedOutput.trim()));
  const stagedStats = createDiffStatsMap(parseNumStat(stagedOutput.trim()));

  const stagedFiles: SessionWorkspaceFile[] = [];
  const unstagedFiles: SessionWorkspaceFile[] = [];

  for (const file of summary.files) {
    const pathParts = file.path.split("/");
    const fileName = pathParts[pathParts.length - 1] || file.path;
    const filePath = pathParts.slice(0, -1).join("/");

    if (file.index !== " " && file.index !== "." && file.index !== "?") {
      const stats = stagedStats[file.path] || { added: 0, removed: 0, binary: false };
      stagedFiles.push({
        fileName,
        filePath,
        relativePath: file.path,
        absolutePath: joinPath(workspaceRoot, file.path),
        status: mapStatusCode(file.index),
        isStaged: true,
        linesAdded: stats.added,
        linesRemoved: stats.removed,
        oldPath: file.from,
      });
    }

    if (file.working_dir !== " " && file.working_dir !== ".") {
      const stats = unstagedStats[file.path] || { added: 0, removed: 0, binary: false };
      unstagedFiles.push({
        fileName,
        filePath,
        relativePath: file.path,
        absolutePath: joinPath(workspaceRoot, file.path),
        status: mapStatusCode(file.working_dir),
        isStaged: false,
        linesAdded: stats.added,
        linesRemoved: stats.removed,
        oldPath: file.from,
      });
    }
  }

  for (const untracked of summary.not_added) {
    if (untracked.endsWith("/")) {
      continue;
    }
    const pathParts = untracked.split("/");
    const fileName = pathParts[pathParts.length - 1] || untracked;
    const filePath = pathParts.slice(0, -1).join("/");
    unstagedFiles.push({
      fileName,
      filePath,
      relativePath: untracked,
      absolutePath: joinPath(workspaceRoot, untracked),
      status: "untracked",
      isStaged: false,
      linesAdded: 0,
      linesRemoved: 0,
    });
  }

  return {
    branch,
    files: [...stagedFiles, ...unstagedFiles],
    totalStaged: stagedFiles.length,
    totalUnstaged: unstagedFiles.length,
  };
}

export function detectFileLanguage(path: string): string | null {
  const extension = path.split(".").pop()?.toLowerCase();
  switch (extension) {
    case "js":
    case "jsx":
      return "javascript";
    case "ts":
    case "tsx":
      return "typescript";
    case "py":
      return "python";
    case "rs":
      return "rust";
    case "go":
      return "go";
    case "json":
      return "json";
    case "md":
      return "markdown";
    case "yml":
    case "yaml":
      return "yaml";
    case "sh":
    case "bash":
      return "bash";
    case "css":
      return "css";
    case "html":
      return "html";
    default:
      return null;
  }
}

export function decodeBase64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

export function decodeWorkspaceFileContent(
  relativePath: string,
  workspaceRoot: string,
  encodedContent: string,
  diff: string | null,
): SessionWorkspaceFileContent {
  const bytes = decodeBase64ToBytes(encodedContent);
  const decodedContent = new TextDecoder().decode(bytes);
  const hasNullBytes = bytes.some((byte) => byte === 0);
  const nonPrintable = decodedContent
    .split("")
    .filter((char) => {
      const code = char.charCodeAt(0);
      return code < 32 && code !== 9 && code !== 10 && code !== 13;
    }).length;
  const isBinary = hasNullBytes || (decodedContent.length > 0 && nonPrintable / decodedContent.length > 0.1);

  return {
    relativePath,
    absolutePath: joinPath(workspaceRoot, relativePath),
    content: isBinary ? "" : decodedContent,
    diff,
    isBinary,
    language: detectFileLanguage(relativePath),
  };
}
