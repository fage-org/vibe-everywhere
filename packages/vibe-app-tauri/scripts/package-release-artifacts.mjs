import fs from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { loadReleaseConfig, resolveProfileConfig } from "./release-config.mjs";

const execFileAsync = promisify(execFile);

const profileName = process.argv[2];
if (!profileName) {
  throw new Error("Usage: node ./scripts/package-release-artifacts.mjs <preview|production-candidate>");
}

const config = await loadReleaseConfig();
const profile = resolveProfileConfig(config, profileName);

const releaseRoot = path.join(config.packageRoot, "release", profile.profileName);
const packageVersion = profile.packageVersion;

const browserSourceDir = path.join(
  config.packageRoot,
  "dist",
  "browser",
  profile.profileName === "preview" ? "preview" : "production",
);
const desktopBundleDir = path.join(
  config.packageRoot,
  "src-tauri",
  "target",
  "release",
  "bundle",
);
const androidApkDir = path.join(
  config.packageRoot,
  "android",
  "app",
  "build",
  "outputs",
  "apk",
  "universal",
  profile.profileName === "preview" ? "debug" : "release",
);

await fs.rm(releaseRoot, { recursive: true, force: true });
await fs.mkdir(releaseRoot, { recursive: true });

async function assertPathExists(targetPath, description) {
  try {
    await fs.access(targetPath);
  } catch {
    throw new Error(`Missing ${description}: ${targetPath}`);
  }
}

async function createTarArchive(sourcePath, archivePath, sourceParent, sourceName) {
  await execFileAsync("tar", ["-czf", archivePath, "-C", sourceParent, sourceName]);
}

await assertPathExists(browserSourceDir, "browser export output");
await assertPathExists(desktopBundleDir, "desktop bundle output");
await assertPathExists(androidApkDir, "Android APK output directory");

const browserArchive = path.join(
  releaseRoot,
  `vibe-app-tauri-browser-${profile.label}-${packageVersion}.tar.gz`,
);
await createTarArchive(
  browserSourceDir,
  browserArchive,
  path.dirname(browserSourceDir),
  path.basename(browserSourceDir),
);

const desktopArchive = path.join(
  releaseRoot,
  `vibe-app-tauri-desktop-${profile.label}-${packageVersion}.tar.gz`,
);
await createTarArchive(
  desktopBundleDir,
  desktopArchive,
  path.dirname(desktopBundleDir),
  path.basename(desktopBundleDir),
);

const apkTarget = path.join(
  releaseRoot,
  `vibe-app-tauri-android-${profile.label}-${packageVersion}.apk`,
);
const androidApkEntries = (await fs.readdir(androidApkDir))
  .filter((entry) => entry.endsWith(".apk"))
  .sort();

if (androidApkEntries.length === 0) {
  throw new Error(`Missing Android APK output in ${androidApkDir}`);
}

const androidApkPath = path.join(androidApkDir, androidApkEntries[0]);
await fs.copyFile(androidApkPath, apkTarget);

const manifest = {
  packageVersion,
  profile: profile.profileName,
  githubReleaseName: profile.githubReleaseName,
  desktopProductName: profile.desktopProductName,
  desktopIdentifier: profile.desktopIdentifier,
  androidApplicationId: profile.androidApplicationId,
  androidAppName: profile.androidAppName,
  deepLinkScheme: profile.deepLinkScheme,
  updaterChannel: profile.updaterChannel,
  artifacts: {
    browserArchive: path.relative(config.packageRoot, browserArchive),
    desktopArchive: path.relative(config.packageRoot, desktopArchive),
    androidApk: path.relative(config.packageRoot, apkTarget),
  },
};

await fs.writeFile(
  path.join(releaseRoot, "release-manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8",
);

console.log(`Packaged release artifacts for ${profile.profileName} in ${releaseRoot}`);
