import fs from "node:fs/promises";
import path from "node:path";

const packageRoot = path.resolve(import.meta.dirname, "..");
const releaseConfigPath = path.join(packageRoot, "release.config.json");
const packageJsonPath = path.join(packageRoot, "package.json");

export async function loadReleaseConfig() {
  const [releaseConfigRaw, packageJsonRaw] = await Promise.all([
    fs.readFile(releaseConfigPath, "utf8"),
    fs.readFile(packageJsonPath, "utf8"),
  ]);

  const releaseConfig = JSON.parse(releaseConfigRaw);
  const packageJson = JSON.parse(packageJsonRaw);
  return {
    packageRoot,
    packageVersion: packageJson.version,
    profiles: releaseConfig.profiles,
  };
}

export function resolveProfileConfig(config, profileName) {
  const profile = config.profiles[profileName];
  if (!profile) {
    throw new Error(
      `Unknown release profile '${profileName}'. Expected one of: ${Object.keys(config.profiles).join(", ")}`,
    );
  }

  return {
    ...profile,
    profileName,
    packageVersion: config.packageVersion,
  };
}
