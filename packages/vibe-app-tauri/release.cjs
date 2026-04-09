#!/usr/bin/env node

const { spawnSync } = require("node:child_process");

const workspaceRoot = __dirname;

const actions = [
  {
    id: "preview",
    label: "Preview artifacts",
    description: "Build and package preview desktop, Android APK, and browser export artifacts",
    scripts: ["release:build:preview"],
  },
  {
    id: "production-candidate",
    label: "Production candidate artifacts",
    description:
      "Build and package production-candidate desktop, Android APK, and browser export artifacts",
    scripts: ["release:build:production-candidate"],
  },
  {
    id: "all",
    label: "All release artifacts",
    description: "Run both preview and production-candidate packaging flows",
    scripts: ["release:build:preview", "release:build:production-candidate"],
  },
];

const aliases = {
  preview: "preview",
  dev: "preview",
  candidate: "production-candidate",
  production: "production-candidate",
  "production-candidate": "production-candidate",
  all: "all",
};

function findAction(input) {
  const normalized = String(input || "").trim().toLowerCase();
  const id = aliases[normalized] || normalized;
  return actions.find((action) => action.id === id);
}

function printOptions() {
  console.error("Available vibe-app-tauri release options:");
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
    console.error("Interactive release selection requires a TTY.");
    printOptions();
    process.exit(1);
  }

  let select;
  try {
    ({ select } = await import("@inquirer/prompts"));
  } catch {
    console.error("Missing interactive prompt dependency: @inquirer/prompts");
    console.error("Run `yarn install` from the repo root and retry.");
    process.exit(1);
  }

  const actionId = await select({
    message: "What should be packaged for vibe-app-tauri?",
    choices: actions.map((action) => ({
      name: action.label,
      value: action.id,
      description: action.description,
    })),
  });

  return findAction(actionId);
}

async function main() {
  const input = process.argv[2];
  if (input === "--help" || input === "-h") {
    console.log("Usage: yarn release -- <preview|production-candidate|all>");
    printOptions();
    return;
  }

  let action = input ? findAction(input) : null;
  if (input && !action) {
    console.error(`Unknown vibe-app-tauri release option: ${input}`);
    printOptions();
    process.exit(1);
  }

  if (!action) {
    action = await promptForAction();
  }

  console.log(`Running vibe-app-tauri release option: ${action.label}`);
  for (const scriptName of action.scripts) {
    runScript(scriptName);
  }
}

main().catch((error) => {
  console.error(error.message || String(error));
  process.exit(1);
});
