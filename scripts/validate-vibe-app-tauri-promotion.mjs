import { readFile } from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const baselinePath = path.join(
  repoRoot,
  "docs",
  "plans",
  "rebuild",
  "vibe-app-tauri-promotion-baseline.md",
);
const planPath = path.join(
  repoRoot,
  "docs",
  "plans",
  "rebuild",
  "vibe-app-tauri-promotion-plan.md",
);

const strict = process.argv.includes("--promotion-ready");

function ensureIncludes(content, fileLabel, needle) {
  if (!content.includes(needle)) {
    throw new Error(`${fileLabel} is missing required section: ${needle}`);
  }
}

function ensureNoPending(content, fileLabel, sectionLabel) {
  const normalized = content.toLowerCase();
  if (normalized.includes("pending") || normalized.includes("tbd")) {
    throw new Error(
      `${fileLabel} still contains pending placeholders; ${sectionLabel} is not promotion-ready`,
    );
  }
}

async function main() {
  const [baseline, plan] = await Promise.all([
    readFile(baselinePath, "utf8"),
    readFile(planPath, "utf8"),
  ]);

  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "# `vibe-app-tauri` Promotion Baseline");
  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "## Startup Validation");
  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "## Performance Review");
  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "## Memory Review");
  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "## Side-By-Side Parity Review");
  ensureIncludes(baseline, "vibe-app-tauri-promotion-baseline.md", "## Sign-Off");

  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "# `vibe-app-tauri` Promotion And Deprecation Plan");
  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "## Promotion Gate");
  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "## Rollout Stages");
  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "## Fallback Plan");
  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "## Deprecation Plan");
  ensureIncludes(plan, "vibe-app-tauri-promotion-plan.md", "## Approval");

  if (strict) {
    ensureNoPending(baseline, "vibe-app-tauri-promotion-baseline.md", "baseline review");
    ensureNoPending(plan, "vibe-app-tauri-promotion-plan.md", "promotion plan");
  }

  process.stdout.write(
    `validated ${path.relative(repoRoot, baselinePath)} and ${path.relative(repoRoot, planPath)}\n`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
