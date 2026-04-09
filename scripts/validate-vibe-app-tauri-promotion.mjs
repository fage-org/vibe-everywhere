import { readFile } from "node:fs/promises";
import path from "node:path";

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const modulePlanPath = path.join(
  repoRoot,
  "docs",
  "plans",
  "rebuild",
  "modules",
  "vibe-app-tauri",
  "promotion-and-vibe-app-deprecation.md",
);
const migrationPlanPath = path.join(
  repoRoot,
  "docs",
  "plans",
  "rebuild",
  "vibe-app-tauri-wave9-migration-and-release-plan.md",
);
const routeMatrixPath = path.join(
  repoRoot,
  "docs",
  "plans",
  "rebuild",
  "vibe-app-tauri-wave9-route-and-capability-matrix.md",
);
const promotionBaselineArtifactPath = path.join(
  repoRoot,
  "artifacts",
  "vibe-app-tauri",
  "promotion-baseline.md",
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
  const [modulePlan, migrationPlan, routeMatrix, promotionBaselineArtifact] = await Promise.all([
    readFile(modulePlanPath, "utf8"),
    readFile(migrationPlanPath, "utf8"),
    readFile(routeMatrixPath, "utf8"),
    readFile(promotionBaselineArtifactPath, "utf8"),
  ]);

  ensureIncludes(
    modulePlan,
    "promotion-and-vibe-app-deprecation.md",
    "# Module Plan: vibe-app-tauri/promotion-and-vibe-app-deprecation",
  );
  ensureIncludes(modulePlan, "promotion-and-vibe-app-deprecation.md", "## Status");
  ensureIncludes(modulePlan, "promotion-and-vibe-app-deprecation.md", "## Promotion Decision Record");
  ensureIncludes(
    modulePlan,
    "promotion-and-vibe-app-deprecation.md",
    "## Retention And Retirement Policy",
  );
  ensureIncludes(modulePlan, "promotion-and-vibe-app-deprecation.md", "## Current Execution Note");
  ensureIncludes(modulePlan, "promotion-and-vibe-app-deprecation.md", "default app path: `packages/vibe-app-tauri`");

  ensureIncludes(
    migrationPlan,
    "vibe-app-tauri-wave9-migration-and-release-plan.md",
    "# Wave 9 Migration And Release Plan: `vibe-app-tauri`",
  );
  ensureIncludes(
    migrationPlan,
    "vibe-app-tauri-wave9-migration-and-release-plan.md",
    "## Exact Rollback Mechanics",
  );
  ensureIncludes(
    migrationPlan,
    "vibe-app-tauri-wave9-migration-and-release-plan.md",
    "## Promotion Gate",
  );
  ensureIncludes(
    migrationPlan,
    "vibe-app-tauri-wave9-migration-and-release-plan.md",
    "`packages/vibe-app-tauri` becomes the default release owner",
  );

  ensureIncludes(
    routeMatrix,
    "vibe-app-tauri-wave9-route-and-capability-matrix.md",
    "# Wave 9 Route And Capability Matrix: `vibe-app-tauri`",
  );
  ensureIncludes(routeMatrix, "vibe-app-tauri-wave9-route-and-capability-matrix.md", "| `/(app)/session/[id]` | `P0` |");
  ensureIncludes(routeMatrix, "vibe-app-tauri-wave9-route-and-capability-matrix.md", "| auth credential storage | `C0` |");

  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "# Vibe App Tauri Promotion Baseline",
  );
  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "## Automated Snapshot",
  );
  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "## Manual Startup Validation",
  );
  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "## Manual Performance Review",
  );
  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "## Manual Memory Review",
  );
  ensureIncludes(
    promotionBaselineArtifact,
    "artifacts/vibe-app-tauri/promotion-baseline.md",
    "## Promotion Notes",
  );

  if (strict) {
    ensureNoPending(modulePlan, "promotion-and-vibe-app-deprecation.md", "promotion module plan");
    ensureNoPending(
      migrationPlan,
      "vibe-app-tauri-wave9-migration-and-release-plan.md",
      "migration and release plan",
    );
    ensureNoPending(
      routeMatrix,
      "vibe-app-tauri-wave9-route-and-capability-matrix.md",
      "route and capability matrix",
    );
    ensureNoPending(
      promotionBaselineArtifact,
      "artifacts/vibe-app-tauri/promotion-baseline.md",
      "promotion baseline artifact",
    );
  }

  process.stdout.write(
    `validated ${path.relative(repoRoot, modulePlanPath)}, ${path.relative(repoRoot, migrationPlanPath)}, ${path.relative(repoRoot, routeMatrixPath)}, and ${path.relative(repoRoot, promotionBaselineArtifactPath)}\n`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
});
