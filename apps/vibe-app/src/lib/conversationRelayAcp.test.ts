import { randomUUID } from "node:crypto";
import {
  spawn,
  spawnSync,
  type ChildProcessByStdio
} from "node:child_process";
import { promises as fs } from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import type { Readable } from "node:stream";
import { fileURLToPath } from "node:url";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  createConversation,
  fetchDevices,
  fetchHealth,
  fetchTaskDetail,
  sendConversationMessage
} from "@/lib/api";
import type {
  DeviceRecord,
  TaskStatus
} from "@/types";

type RunningProcess = {
  child: ChildProcessByStdio<null, Readable, Readable>;
  name: string;
  logLines: string[];
};

type TestHarness = {
  tempDir: string;
  relayBaseUrl: string;
  controlToken: string;
  enrollmentToken: string;
  deviceId: string;
  workspaceDir: string;
  extraPathsToRemove: string[];
  relay: RunningProcess;
  agent: RunningProcess;
};

type OpenCodeCommandConfig = {
  command: string;
  label: string;
};

const REPO_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../../"
);
const RELAY_BINARY = path.join(REPO_ROOT, "target", "debug", "vibe-relay");
const AGENT_BINARY = path.join(REPO_ROOT, "target", "debug", "vibe-agent");

let harness: TestHarness | null = null;
let realHarness: TestHarness | null = null;
const realOpenCode = detectRealOpenCode();

describe("app relay ACP conversation integration", () => {
  beforeAll(async () => {
    await ensureBinariesBuilt();
    const fakeOpenCodeDir = await fs.mkdtemp(
      path.join(os.tmpdir(), "vibe-app-relay-acp-fake-opencode-")
    );
    harness = await startHarness({
      command: await writeFakeOpenCodeBinary(fakeOpenCodeDir),
      label: "fake-opencode"
    }, [fakeOpenCodeDir]);
  }, 120_000);

  afterAll(async () => {
    if (harness) {
      await stopHarness(harness);
      harness = null;
    }
  }, 30_000);

  it(
    "creates a helloworld file through an app conversation over relay and agent ACP",
    async () => {
      if (!harness) {
        throw new Error("test harness failed to start");
      }

      await assertHelloWorldConversation(harness, {
        title: "ACP helloworld integration",
        prompt:
          "Please create a file named helloworld in the current working directory and confirm once it exists.",
        requireAssistantDeltaIncludes: "Created helloworld"
      });
    },
    60_000
  );
});

describe.runIf(realOpenCode.available)(
  "app relay ACP conversation integration with local opencode",
  () => {
    beforeAll(async () => {
      await ensureBinariesBuilt();
      realHarness = await startHarness({
        command: realOpenCode.command,
        label: `opencode ${realOpenCode.version ?? "unknown"}`
      });
    }, 120_000);

    afterAll(async () => {
      if (realHarness) {
        await stopHarness(realHarness);
        realHarness = null;
      }
    }, 30_000);

    it(
      "runs a two-turn time conversation through the real local opencode ACP server",
      async () => {
        if (!realHarness) {
          throw new Error("real opencode test harness failed to start");
        }

        const reportContent = await assertRealOpenCodeTimeConversation(realHarness);
        console.log("real-opencode-time-report");
        console.log(reportContent);
      },
      180_000
    );
  }
);

async function ensureBinariesBuilt() {
  if ((await fileExists(RELAY_BINARY)) && (await fileExists(AGENT_BINARY))) {
    return;
  }

  await runCommand("cargo", ["build", "-p", "vibe-relay", "-p", "vibe-agent"], REPO_ROOT);
}

async function startHarness(
  openCode: OpenCodeCommandConfig,
  extraPathsToRemove: string[] = []
): Promise<TestHarness> {
  const tempDir = await fs.mkdtemp(
    path.join(os.tmpdir(), "vibe-app-relay-acp-integration-")
  );
  const controlToken = `control-${randomUUID()}`;
  const enrollmentToken = `enroll-${randomUUID()}`;
  const deviceId = `device-${randomUUID()}`;
  const workspaceDir = path.join(tempDir, "workspace");
  const relayPort = await reservePort();
  const relayBaseUrl = `http://127.0.0.1:${relayPort}`;

  await fs.mkdir(workspaceDir, { recursive: true });

  const relay = spawnLoggedProcess(
    "relay",
    RELAY_BINARY,
    [],
    {
      ...process.env,
      VIBE_RELAY_HOST: "127.0.0.1",
      VIBE_RELAY_PORT: String(relayPort),
      VIBE_PUBLIC_RELAY_BASE_URL: relayBaseUrl,
      VIBE_RELAY_STATE_FILE: path.join(tempDir, "relay-state.json"),
      VIBE_RELAY_ACCESS_TOKEN: controlToken,
      VIBE_RELAY_ENROLLMENT_TOKEN: enrollmentToken,
      VIBE_RELAY_FORWARD_HOST: "127.0.0.1",
      VIBE_RELAY_FORWARD_BIND_HOST: "127.0.0.1"
    },
    REPO_ROOT
  );

  await waitFor(async () => {
    await fetchHealth(relayBaseUrl);
    return true;
  }, 20_000);

  const agent = spawnLoggedProcess(
    "agent",
    AGENT_BINARY,
    [],
    {
      ...process.env,
      VIBE_RELAY_URL: relayBaseUrl,
      VIBE_RELAY_ENROLLMENT_TOKEN: enrollmentToken,
      VIBE_DEVICE_ID: deviceId,
      VIBE_DEVICE_NAME: "App ACP Integration Agent",
      VIBE_WORKING_ROOT: workspaceDir,
      VIBE_POLL_INTERVAL_MS: "200",
      VIBE_HEARTBEAT_INTERVAL_MS: "500",
      VIBE_OPENCODE_COMMAND: openCode.command
    },
    REPO_ROOT
  );

  return {
    tempDir,
    relayBaseUrl,
    controlToken,
    enrollmentToken,
    deviceId,
    workspaceDir,
    extraPathsToRemove,
    relay,
    agent
  };
}

async function assertHelloWorldConversation(
  activeHarness: TestHarness,
  input: {
    title: string;
    prompt: string;
    requireAssistantDeltaIncludes?: string;
  }
) {
  const devices = await waitFor(async () => {
    const registeredDevices = await fetchDevices(
      activeHarness.relayBaseUrl,
      activeHarness.controlToken
    );
    const device = registeredDevices.find(
      (entry) =>
        entry.id === activeHarness.deviceId &&
        entry.online &&
        entry.providers.some(
          (provider) => provider.kind === "open_code" && provider.available
        )
    );
    return device ? registeredDevices : null;
  }, 20_000);

  const device = devices.find(
    (entry) => entry.id === activeHarness.deviceId
  ) as DeviceRecord;
  expect(
    device.providers.some(
      (provider) => provider.kind === "open_code" && provider.available
    )
  ).toBe(true);

  const response = await createConversation(
    activeHarness.relayBaseUrl,
    {
      deviceId: activeHarness.deviceId,
      provider: "open_code",
      executionMode: "workspace_write",
      cwd: activeHarness.workspaceDir,
      title: input.title,
      prompt: input.prompt
    },
    activeHarness.controlToken
  );

  expect(response.conversation.executionProtocol).toBe("acp");
  expect(response.task.executionProtocol).toBe("acp");
  expect(response.conversation.provider).toBe("open_code");

  const detail = await waitForTaskTerminal(
    activeHarness.relayBaseUrl,
    activeHarness.controlToken,
    response.task.id
  );
  expect(detail.task.status).toBe("succeeded");
  expect(detail.task.error).toBeNull();
  if (input.requireAssistantDeltaIncludes) {
    expect(
      detail.events.some(
        (event) =>
          event.kind === "assistant_delta" &&
          event.message.includes(input.requireAssistantDeltaIncludes!)
      )
    ).toBe(true);
  }

  const helloWorldPath = path.join(activeHarness.workspaceDir, "helloworld");
  const content = await fs.readFile(helloWorldPath, "utf8");
  expect(content).toBe("hello world\n");
}

async function assertRealOpenCodeTimeConversation(activeHarness: TestHarness) {
  const fileName = "time-report.txt";

  await waitForOpenCodeDevice(activeHarness);

  const firstTurn = await createConversation(
    activeHarness.relayBaseUrl,
    {
      deviceId: activeHarness.deviceId,
      provider: "open_code",
      executionMode: "workspace_write",
      cwd: activeHarness.workspaceDir,
      title: "Real OpenCode ACP time integration",
      prompt: [
        "Tell me the current local time with hour, minute, and second included.",
        "Use the exact format HH:MM:SS in your reply.",
        "Do not write any files yet."
      ].join(" ")
    },
    activeHarness.controlToken
  );

  expect(firstTurn.conversation.executionProtocol).toBe("acp");
  expect(firstTurn.task.executionProtocol).toBe("acp");
  const firstDetail = await waitForTaskTerminal(
    activeHarness.relayBaseUrl,
    activeHarness.controlToken,
    firstTurn.task.id
  );
  expect(firstDetail.task.status).toBe("succeeded");
  expect(findHmsTime(firstDetail.events.map((event) => event.message).join("\n"))).toBeTruthy();

  await sleep(10_000);

  const secondTurn = await sendConversationMessage(
    activeHarness.relayBaseUrl,
    firstTurn.conversation.id,
    {
      title: "Real OpenCode ACP time follow-up",
      prompt: [
        "Now tell me the current local time again with hour, minute, and second included.",
        "Use the exact format HH:MM:SS.",
        `Compare it with your previous time reply from this same conversation and calculate how many seconds elapsed.`,
        `Write the first time, second time, and elapsed seconds into a file named ${fileName} in the current working directory.`,
        "Reply after the file has been written."
      ].join(" ")
    },
    activeHarness.controlToken
  );

  expect(secondTurn.conversation.executionProtocol).toBe("acp");
  expect(secondTurn.task.executionProtocol).toBe("acp");
  const secondDetail = await waitForTaskTerminal(
    activeHarness.relayBaseUrl,
    activeHarness.controlToken,
    secondTurn.task.id
  );
  expect(secondDetail.task.status).toBe("succeeded");

  const reportPath = path.join(activeHarness.workspaceDir, fileName);
  const reportContent = await fs.readFile(reportPath, "utf8");
  expect(findHmsTime(reportContent)).toBeTruthy();
  expect(reportContent.toLowerCase()).toContain("elapsed");

  return reportContent;
}

async function waitForOpenCodeDevice(activeHarness: TestHarness) {
  const devices = await waitFor(async () => {
    const registeredDevices = await fetchDevices(
      activeHarness.relayBaseUrl,
      activeHarness.controlToken
    );
    const device = registeredDevices.find(
      (entry) =>
        entry.id === activeHarness.deviceId &&
        entry.online &&
        entry.providers.some(
          (provider) => provider.kind === "open_code" && provider.available
        )
    );
    return device ? registeredDevices : null;
  }, 20_000);

  const device = devices.find(
    (entry) => entry.id === activeHarness.deviceId
  ) as DeviceRecord;
  expect(
    device.providers.some(
      (provider) => provider.kind === "open_code" && provider.available
    )
  ).toBe(true);

  return device;
}

function findHmsTime(input: string) {
  return input.match(/\b\d{2}:\d{2}:\d{2}\b/);
}

async function stopHarness(activeHarness: TestHarness) {
  await stopProcess(activeHarness.agent);
  await stopProcess(activeHarness.relay);
  for (const extraPath of activeHarness.extraPathsToRemove) {
    await fs.rm(extraPath, { recursive: true, force: true });
  }
  await fs.rm(activeHarness.tempDir, { recursive: true, force: true });
}

async function waitForTaskTerminal(
  relayBaseUrl: string,
  controlToken: string,
  taskId: string
) {
  return waitFor(async () => {
    const detail = await fetchTaskDetail(relayBaseUrl, taskId, controlToken);
    if (isTerminalTaskStatus(detail.task.status)) {
      if (detail.task.status !== "succeeded") {
        throw new Error(
          `task ${taskId} reached ${detail.task.status}\n${JSON.stringify(detail, null, 2)}`
        );
      }
      return detail;
    }
    return null;
  }, 30_000);
}

function isTerminalTaskStatus(status: TaskStatus) {
  return status === "succeeded" || status === "failed" || status === "canceled";
}

async function writeFakeOpenCodeBinary(tempDir: string) {
  const binaryPath = path.join(tempDir, "fake-opencode.mjs");
  const script = `#!/usr/bin/env node
import { promises as fs } from "node:fs";
import path from "node:path";
import readline from "node:readline";

const args = process.argv.slice(2);

if (args[0] === "--version") {
  process.stdout.write("fake-opencode 0.1.0\\n");
  process.exit(0);
}

if (args[0] !== "acp") {
  process.stderr.write(\`unsupported invocation: \${args.join(" ")}\\n\`);
  process.exit(2);
}

let launchCwd = process.cwd();
for (let index = 1; index < args.length; index += 1) {
  if (args[index] === "--cwd" && args[index + 1]) {
    launchCwd = args[index + 1];
    index += 1;
  }
}

let nextSessionId = 1;
const sessions = new Map();
const rl = readline.createInterface({
  input: process.stdin,
  crlfDelay: Infinity
});

function send(message) {
  process.stdout.write(\`\${JSON.stringify(message)}\\n\`);
}

for await (const rawLine of rl) {
  const line = rawLine.trim();
  if (!line) {
    continue;
  }

  const message = JSON.parse(line);
  const method = message.method;
  const requestId = message.id;

  if (method === "initialize") {
    send({
      jsonrpc: "2.0",
      id: requestId,
      result: {
        protocolVersion: 1,
        agentCapabilities: {
          sessionCapabilities: {}
        }
      }
    });
    continue;
  }

  if (method === "session/new") {
    const sessionId = \`session_\${nextSessionId++}\`;
    sessions.set(sessionId, {
      cwd: message.params?.cwd || launchCwd
    });
    send({
      jsonrpc: "2.0",
      id: requestId,
      result: {
        sessionId
      }
    });
    continue;
  }

  if (method === "session/prompt") {
    const sessionId = message.params?.sessionId;
    const session = sessions.get(sessionId);
    if (!session) {
      send({
        jsonrpc: "2.0",
        id: requestId,
        error: {
          code: -32000,
          message: "unknown session"
        }
      });
      continue;
    }

    const helloWorldPath = path.join(session.cwd, "helloworld");
    await fs.mkdir(session.cwd, { recursive: true });
    await fs.writeFile(helloWorldPath, "hello world\\n", "utf8");

    send({
      jsonrpc: "2.0",
      method: "session/update",
      params: {
        sessionId,
        update: {
          sessionUpdate: "agent_message_chunk",
          content: [
            {
              type: "text",
              text: \`Created helloworld in \${session.cwd}\`
            }
          ]
        }
      }
    });

    send({
      jsonrpc: "2.0",
      id: requestId,
      result: {
        stopReason: "end_turn",
        usage: {
          inputTokens: 4,
          outputTokens: 6,
          totalTokens: 10
        }
      }
    });
    continue;
  }

  if (method === "session/cancel") {
    continue;
  }

  if (requestId !== undefined) {
    send({
      jsonrpc: "2.0",
      id: requestId,
      error: {
        code: -32601,
        message: \`unsupported method: \${method}\`
      }
    });
  }
}
`;

  await fs.writeFile(binaryPath, script, "utf8");
  await fs.chmod(binaryPath, 0o755);
  return binaryPath;
}

function detectRealOpenCode() {
  if (process.env.VIBE_APP_SKIP_REAL_OPENCODE_TEST === "1") {
    return {
      available: false,
      command: "opencode",
      version: null as string | null
    };
  }

  const which = spawnSync("bash", ["-lc", "command -v opencode"], {
    cwd: REPO_ROOT,
    encoding: "utf8"
  });
  const command = which.status === 0 ? which.stdout.trim() : "opencode";
  if (which.status !== 0 || !command) {
    return {
      available: false,
      command,
      version: null as string | null
    };
  }

  const version = spawnSync(command, ["--version"], {
    cwd: REPO_ROOT,
    encoding: "utf8"
  });
  if (version.status !== 0) {
    return {
      available: false,
      command,
      version: null as string | null
    };
  }

  return {
    available: true,
    command,
    version: [version.stdout, version.stderr]
      .map((value) => value.trim())
      .find((value) => value.length > 0) ?? null
  };
}

function spawnLoggedProcess(
  name: string,
  command: string,
  args: string[],
  env: Record<string, string | undefined>,
  cwd: string
): RunningProcess {
  const child: ChildProcessByStdio<null, Readable, Readable> = spawn(
    command,
    args,
    {
    cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"]
    }
  );
  const logLines: string[] = [];
  child.stdout.on("data", (chunk: Uint8Array | string) => {
    const text =
      typeof chunk === "string" ? chunk : Buffer.from(chunk).toString("utf8");
    logLines.push(`[stdout] ${text.trimEnd()}`);
  });
  child.stderr.on("data", (chunk: Uint8Array | string) => {
    const text =
      typeof chunk === "string" ? chunk : Buffer.from(chunk).toString("utf8");
    logLines.push(`[stderr] ${text.trimEnd()}`);
  });
  child.on("exit", (code: number | null, signal: NodeJS.Signals | null) => {
    logLines.push(`[exit] code=${code ?? "null"} signal=${signal ?? "null"}`);
  });
  return {
    child,
    name,
    logLines
  };
}

async function stopProcess(processHandle: RunningProcess) {
  if (processHandle.child.exitCode !== null || processHandle.child.signalCode !== null) {
    return;
  }

  processHandle.child.kill("SIGINT");
  await new Promise<void>((resolve) => {
    const timeout = setTimeout(() => {
      if (
        processHandle.child.exitCode === null &&
        processHandle.child.signalCode === null
      ) {
        processHandle.child.kill("SIGTERM");
      }
      resolve();
    }, 5_000);

    processHandle.child.once("exit", () => {
      clearTimeout(timeout);
      resolve();
    });
  });
}

async function reservePort() {
  return await new Promise<number>((resolve, reject) => {
    const server = net.createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("failed to reserve a TCP port"));
        return;
      }
      server.close((error?: Error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(address.port);
      });
    });
  });
}

async function waitFor<T>(
  operation: () => Promise<T | null>,
  timeoutMs: number,
  intervalMs = 200
): Promise<T> {
  const startedAt = Date.now();
  let lastError: unknown = null;

  while (Date.now() - startedAt < timeoutMs) {
    try {
      const result = await operation();
      if (result !== null) {
        return result;
      }
    } catch (error) {
      lastError = error;
    }
    await sleep(intervalMs);
  }

  throw lastError instanceof Error
    ? lastError
    : new Error(`operation did not complete within ${timeoutMs}ms`);
}

async function fileExists(targetPath: string) {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

async function runCommand(command: string, args: string[], cwd: string) {
  await new Promise<void>((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      stdio: "inherit"
    });
    child.on("error", reject);
    child.on("exit", (code: number | null) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${command} ${args.join(" ")} exited with code ${code}`));
    });
  });
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
