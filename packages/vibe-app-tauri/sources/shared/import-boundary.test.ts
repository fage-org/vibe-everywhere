import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const packageRoot = path.resolve(import.meta.dirname, "..", "..");
const scanRoots = [
  path.join(packageRoot, "src"),
  path.join(packageRoot, "sources"),
] as const;
const bannedPatterns = [
  "../../vibe-app/sources",
  "../vibe-app/sources",
  "/packages/vibe-app/sources/",
  "/packages/happy-app/sources/",
] as const;

function collectSourceFiles(rootPath: string): string[] {
  const entries = fs.readdirSync(rootPath, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const entryPath = path.join(rootPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectSourceFiles(entryPath));
      continue;
    }
    if (/\.(ts|tsx)$/.test(entry.name)) {
      files.push(entryPath);
    }
  }
  return files;
}

describe("shared core import boundaries", () => {
  it("does not depend on packages/vibe-app or happy-app source files at runtime", () => {
    const offenders: string[] = [];

    for (const scanRoot of scanRoots) {
      for (const filePath of collectSourceFiles(scanRoot)) {
        if (filePath.endsWith("sources/shared/import-boundary.test.ts")) {
          continue;
        }
        const fileContent = fs.readFileSync(filePath, "utf8");
        if (bannedPatterns.some((pattern) => fileContent.includes(pattern))) {
          offenders.push(path.relative(packageRoot, filePath));
        }
      }
    }

    expect(offenders).toEqual([]);
  });
});
