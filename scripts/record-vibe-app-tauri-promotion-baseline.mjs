import { mkdir, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const distDir = path.join(repoRoot, "packages", "vibe-app-tauri", "dist");
const debugBinary = path.join(
  repoRoot,
  "packages",
  "vibe-app-tauri",
  "src-tauri",
  "target",
  "debug",
  process.platform === "win32" ? "vibe-app-tauri-desktop.exe" : "vibe-app-tauri-desktop",
);
const releaseBundleDir = path.join(
  repoRoot,
  "packages",
  "vibe-app-tauri",
  "src-tauri",
  "target",
  "release",
  "bundle",
);
const defaultOutput = path.join(
  repoRoot,
  "artifacts",
  "vibe-app-tauri",
  "promotion-baseline.md",
);

function formatBytes(bytes) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const units = ["KiB", "MiB", "GiB"];
  let value = bytes;
  let unitIndex = -1;
  while (value >= 1024 && unitIndex + 1 < units.length) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(2)} ${units[unitIndex]}`;
}

async function collectFiles(rootDir) {
  const entries = await readdir(rootDir, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const absolutePath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(absolutePath)));
      continue;
    }

    const info = await stat(absolutePath);
    files.push({
      path: absolutePath,
      sizeBytes: info.size,
    });
  }

  return files;
}

async function collectOptionalFile(targetPath) {
  try {
    const info = await stat(targetPath);
    return {
      path: targetPath,
      sizeBytes: info.size,
    };
  } catch {
    return null;
  }
}

async function collectOptionalDirectory(rootDir) {
  try {
    return await collectFiles(rootDir);
  } catch {
    return [];
  }
}

function relativeToRepo(targetPath) {
  return path.relative(repoRoot, targetPath) || ".";
}

function renderTableRows(files) {
  if (files.length === 0) {
    return "| not available | n/a |\n";
  }

  return files
    .sort((left, right) => right.sizeBytes - left.sizeBytes)
    .map(
      (file) => `| \`${relativeToRepo(file.path)}\` | ${formatBytes(file.sizeBytes)} |`,
    )
    .join("\n")
    .concat("\n");
}

async function main() {
  const outputArg = process.argv[2] ?? null;
  const outputPath = outputArg
    ? path.isAbsolute(outputArg)
      ? outputArg
      : path.resolve(repoRoot, outputArg)
    : defaultOutput;

  const [distFiles, debugExecutable, releaseBundles] = await Promise.all([
    collectOptionalDirectory(distDir),
    collectOptionalFile(debugBinary),
    collectOptionalDirectory(releaseBundleDir),
  ]);

  const timestamp = new Date().toISOString();
  const markdown = `# Vibe App Tauri Promotion Baseline

Generated at: \`${timestamp}\`

## Automated Snapshot

### Dist Assets

| File | Size |
| --- | --- |
${renderTableRows(distFiles)}

### Debug Desktop Binary

| File | Size |
| --- | --- |
${renderTableRows(debugExecutable ? [debugExecutable] : [])}

### Release Bundle Outputs

| File | Size |
| --- | --- |
${renderTableRows(releaseBundles)}

## Manual Startup Validation

| Platform | Package built | App launched | Notes |
| --- | --- | --- | --- |
| Linux | pending | pending | |
| macOS | pending | pending | |
| Windows | pending | pending | |

## Manual Performance Review

Use a realistic session load before promotion. Record the measured values here instead of replacing
this file with prose.

| Platform | Session dataset | Startup time | First interaction time | Notes |
| --- | --- | --- | --- | --- |
| Linux | pending | pending | pending | |
| macOS | pending | pending | pending | |
| Windows | pending | pending | pending | |

## Manual Memory Review

| Platform | Session dataset | Peak memory | Steady-state memory | Notes |
| --- | --- | --- | --- | --- |
| Linux | pending | pending | pending | |
| macOS | pending | pending | pending | |
| Windows | pending | pending | pending | |

## Promotion Notes

- \`packages/vibe-app\` is now legacy reference-only and must not be restored as the default desktop path without an explicit plan update.
- Attach cross-platform startup evidence and realistic session-load measurements before marking the
  active release-and-promotion slice complete.
`;

  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, markdown);
  process.stdout.write(`${path.relative(repoRoot, outputPath)}\n`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
