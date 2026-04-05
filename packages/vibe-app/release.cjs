#!/usr/bin/env node

const { spawnSync } = require("child_process");

const workspaceRoot = __dirname;

const actions = [
  {
    id: "all",
    label: "All",
    description: "Developer build plus production store build",
    scripts: [
      "release:build:developer",
      "release:build:appstore",
    ],
  },
  {
    id: "all-interactive",
    label: "All interactive",
    description: "All build steps with interactive prompts when needed",
    scripts: [
      "release:build:developer:interactive",
      "release:build:appstore:interactive",
    ],
  },
  {
    id: "developer-build",
    label: "Developer build",
    description: "Run development and preview builds; iOS store auto-submit is opt-in via env",
    scripts: ["release:build:developer"],
  },
  {
    id: "appstore-build",
    label: "App Store build",
    description: "Run production store builds; auto-submit is opt-in via env",
    scripts: ["release:build:appstore"],
  },
  {
    id: "ota-preview",
    label: "OTA (preview)",
    description: "Publish an update to the preview channel",
    scripts: ["release:ota:preview"],
  },
  {
    id: "ota-release",
    label: "OTA (release)",
    description: "Publish an update to the production channel",
    scripts: ["release:ota:release"],
  },
];

const actionAliases = {
  all: "all",
  interactive: "all-interactive",
  "all-interactive": "all-interactive",
  "all interactive": "all-interactive",
  developer: "developer-build",
  dev: "developer-build",
  "developer-build": "developer-build",
  appstore: "appstore-build",
  store: "appstore-build",
  "appstore-build": "appstore-build",
  "ota-preview": "ota-preview",
  "ota:preview": "ota-preview",
  preview: "ota-preview",
  "ota-release": "ota-release",
  "ota:release": "ota-release",
  production: "appstore-build",
};

function findAction(input) {
  const normalized = String(input || "").trim().toLowerCase();
  const actionId = actionAliases[normalized] || normalized;
  return actions.find((action) => action.id === actionId);
}

function printAvailableOptions() {
  console.error("Available vibe-app release options:");
  for (const action of actions) {
    console.error(`- ${action.id}: ${action.description}`);
  }
}

function runScript(scriptName) {
  console.log(`> yarn run ${scriptName}`);
  const result = spawnSync("yarn", ["run", scriptName], {
    cwd: workspaceRoot,
    stdio: "inherit",
    env: process.env,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

async function promptForAction() {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    console.error("Interactive vibe-app release selection requires a TTY.");
    printAvailableOptions();
    console.error("Run `yarn release -- <option>` in non-interactive mode.");
    process.exit(1);
  }

  // Keep the release entrypoint self-contained so it still works even when
  // optional prompt packages are not installed in the workspace.
  console.error("What should be released for vibe-app?");
  actions.forEach((action, index) => {
    console.error(`${index + 1}. ${action.label} - ${action.description}`);
  });
  console.error("Enter a number:");

  const actionId = await new Promise((resolve) => {
    let buffer = "";
    process.stdin.setEncoding("utf8");
    process.stdin.resume();
    process.stdin.once("data", (chunk) => {
      buffer += chunk;
      const trimmed = buffer.trim();
      const index = Number.parseInt(trimmed, 10);
      if (Number.isInteger(index) && index >= 1 && index <= actions.length) {
        resolve(actions[index - 1].id);
        return;
      }
      resolve(trimmed);
    });
  });

  const action = findAction(actionId);
  if (!action) {
    console.error(`Invalid selection: ${actionId}`);
    printAvailableOptions();
    process.exit(1);
  }
  return action;
}

async function main() {
  const input = process.argv[2];
  if (input === "--help" || input === "-h") {
    console.log("Usage: yarn release -- <option>");
    printAvailableOptions();
    return;
  }

  let action = input ? findAction(input) : null;
  if (input && !action) {
    console.error(`Unknown vibe-app release option: ${input}`);
    printAvailableOptions();
    process.exit(1);
  }

  if (!action) {
    action = await promptForAction();
  }

  console.log(`Running vibe-app release option: ${action.label}`);
  for (const scriptName of action.scripts) {
    runScript(scriptName);
  }
}

module.exports = {
  actions,
  actionAliases,
  findAction,
};

if (require.main === module) {
  main().catch((error) => {
    console.error(error.message || String(error));
    process.exit(1);
  });
}
