import fs from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const packageRoot = path.join(repoRoot, "packages", "vibe-app-tauri");

async function readFile(relativePath) {
  return fs.readFile(path.join(repoRoot, relativePath), "utf8");
}

async function assertExists(relativePath) {
  try {
    await fs.access(path.join(repoRoot, relativePath));
  } catch {
    throw new Error(`Missing required release input: ${relativePath}`);
  }
}

function assertIncludes(content, needle, label) {
  if (!content.includes(needle)) {
    throw new Error(`${label} is missing required content: ${needle}`);
  }
}

async function main() {
  const requiredFiles = [
    "packages/vibe-app-tauri/release.cjs",
    "packages/vibe-app-tauri/release-dev.sh",
    "packages/vibe-app-tauri/release-production.sh",
    "packages/vibe-app-tauri/release.config.json",
    "packages/vibe-app-tauri/src-tauri/tauri.preview.conf.json",
    "packages/vibe-app-tauri/src-tauri/tauri.production-candidate.conf.json",
    "packages/vibe-app-tauri/scripts/write-release-tauri-properties.mjs",
    "packages/vibe-app-tauri/scripts/package-release-artifacts.mjs",
  ];

  await Promise.all(requiredFiles.map(assertExists));

  const [packageJson, releaseConfig, workflow, migrationPlan] = await Promise.all([
    readFile("packages/vibe-app-tauri/package.json"),
    readFile("packages/vibe-app-tauri/release.config.json"),
    readFile(".github/workflows/app-release.yml"),
    readFile("docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md"),
  ]);

  assertIncludes(packageJson, "\"release\": \"node ./release.cjs\"", "packages/vibe-app-tauri/package.json");
  assertIncludes(packageJson, "\"release:build:preview\": \"sh ./release-dev.sh\"", "packages/vibe-app-tauri/package.json");
  assertIncludes(packageJson, "\"release:build:production-candidate\": \"sh ./release-production.sh\"", "packages/vibe-app-tauri/package.json");

  assertIncludes(releaseConfig, "\"preview\"", "packages/vibe-app-tauri/release.config.json");
  assertIncludes(releaseConfig, "\"production-candidate\"", "packages/vibe-app-tauri/release.config.json");

  assertIncludes(workflow, "APP_DIR: packages/vibe-app-tauri", ".github/workflows/app-release.yml");
  assertIncludes(workflow, "app-release-browser", ".github/workflows/app-release.yml");
  assertIncludes(workflow, "app-release-android", ".github/workflows/app-release.yml");
  assertIncludes(workflow, "yarn workspace vibe-app-tauri", ".github/workflows/app-release.yml");

  assertIncludes(
    migrationPlan,
    "no legacy `packages/vibe-app` upgrade-validation lane remains in scope",
    "docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md",
  );
  assertIncludes(
    migrationPlan,
    "analytics/tracking continuity decision",
    "docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md",
  );

  console.log("vibe-app-tauri release inputs and workflow references are present.");
}

main().catch((error) => {
  console.error(error.message || String(error));
  process.exit(1);
});
