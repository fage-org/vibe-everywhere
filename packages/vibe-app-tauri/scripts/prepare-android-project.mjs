import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const packageRoot = path.resolve(import.meta.dirname, "..");
const canonicalAndroidRoot = path.join(packageRoot, "android");
const tauriGenRoot = path.join(packageRoot, "src-tauri", "gen");
const tauriAndroidBridge = path.join(tauriGenRoot, "android");
const bridgeTarget = path.relative(tauriGenRoot, canonicalAndroidRoot);
const canonicalGradleBuildFile = path.join(
  canonicalAndroidRoot,
  "app",
  "build.gradle.kts",
);
const canonicalBuildTaskFile = path.join(
  canonicalAndroidRoot,
  "buildSrc",
  "src",
  "main",
  "java",
  "engineering",
  "vibe",
  "app",
  "next",
  "kotlin",
  "BuildTask.kt",
);
const normalizedRootDirRel = 'rootDirRel = "../.."';

function pathExists(targetPath) {
  return fs.existsSync(targetPath);
}

function ensureGenRoot() {
  fs.mkdirSync(tauriGenRoot, { recursive: true });
}

function ensureBridgeSymlink() {
  ensureGenRoot();
  if (pathExists(tauriAndroidBridge)) {
    const stats = fs.lstatSync(tauriAndroidBridge);
    if (stats.isSymbolicLink()) {
      const linkedTarget = fs.readlinkSync(tauriAndroidBridge);
      if (linkedTarget === bridgeTarget) {
        return;
      }
      fs.unlinkSync(tauriAndroidBridge);
    } else {
      throw new Error(
        `Expected ${tauriAndroidBridge} to be a symlink or absent, found a real path instead.`,
      );
    }
  }

  fs.symlinkSync(bridgeTarget, tauriAndroidBridge, "dir");
}

function normalizeAndroidBuildTaskRoot() {
  if (!pathExists(canonicalGradleBuildFile)) {
    return;
  }

  const fileContent = fs.readFileSync(canonicalGradleBuildFile, "utf8");
  const normalizedContent = fileContent.replace(
    /rootDirRel\s*=\s*"[^"]+"/,
    normalizedRootDirRel,
  );

  if (normalizedContent !== fileContent) {
    fs.writeFileSync(canonicalGradleBuildFile, normalizedContent);
  }
}

function normalizeAndroidBuildTaskCommand() {
  if (!pathExists(canonicalBuildTaskFile)) {
    return;
  }

  const fileContent = fs.readFileSync(canonicalBuildTaskFile, "utf8");
  const normalizedContent = fileContent.replace(
    'val args = listOf("tauri", "android", "android-studio-script");',
    'val args = listOf("./node_modules/.bin/tauri", "android", "android-studio-script");',
  );

  if (normalizedContent !== fileContent) {
    fs.writeFileSync(canonicalBuildTaskFile, normalizedContent);
  }
}

function moveGeneratedProjectIntoCanonicalRoot() {
  if (!pathExists(tauriAndroidBridge)) {
    return;
  }

  const stats = fs.lstatSync(tauriAndroidBridge);
  if (stats.isSymbolicLink()) {
    return;
  }

  fs.mkdirSync(canonicalAndroidRoot, { recursive: true });

  for (const entry of fs.readdirSync(tauriAndroidBridge)) {
    const sourcePath = path.join(tauriAndroidBridge, entry);
    const targetPath = path.join(canonicalAndroidRoot, entry);
    if (pathExists(targetPath)) {
      throw new Error(
        `Cannot move generated Android project into canonical root because ${targetPath} already exists.`,
      );
    }
    fs.renameSync(sourcePath, targetPath);
  }

  fs.rmSync(tauriAndroidBridge, { force: true, recursive: true });
}

function main() {
  moveGeneratedProjectIntoCanonicalRoot();

  if (!pathExists(canonicalAndroidRoot)) {
    throw new Error(
      "Android project not found. Run `tauri android init --ci` before preparing the canonical android path.",
    );
  }

  normalizeAndroidBuildTaskRoot();
  normalizeAndroidBuildTaskCommand();
  ensureBridgeSymlink();
  process.stdout.write(
    `Android project ready: ${canonicalAndroidRoot} (bridge: ${tauriAndroidBridge} -> ${bridgeTarget})\n`,
  );
}

main();
