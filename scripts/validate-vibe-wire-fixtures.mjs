#!/usr/bin/env node

import { access, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import * as ts from 'typescript';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const fixtureDir = path.join(repoRoot, 'crates', 'vibe-wire', 'fixtures');

const fixtureFiles = {
  messageMeta: 'message-meta.json',
  legacyMessages: 'legacy-messages.json',
  sessionEnvelopes: 'session-envelopes.json',
  messageContent: 'message-content.json',
  updateContainers: 'update-containers.json',
  voiceResponses: 'voice-responses.json',
  invalidSessionEnvelopes: 'session-invalid-envelopes.json',
};

async function main() {
  const happyRoot = await resolveHappyRoot();
  const bridgeSources = bridgeSourcePaths(happyRoot);
  const bridgeBaseDir = path.join(scriptDir, '.tmp');
  await mkdir(bridgeBaseDir, { recursive: true });
  const bridgeDir = await mkdtemp(path.join(bridgeBaseDir, 'vibe-wire-happy-bridge-'));

  try {
    await buildBridgeModules(bridgeDir, bridgeSources);
    const modules = await loadBridgeModules(bridgeDir, bridgeSources);
    await validatePublishedFixtures(modules);
    validateOptionalNullRejections(modules);

    console.log(`Validated vibe-wire fixtures against Happy schemas from ${happyRoot}`);
  } finally {
    await rm(bridgeDir, { recursive: true, force: true });
  }
}

async function resolveHappyRoot() {
  const configuredRoot = process.env.HAPPY_ROOT?.trim();
  const happyRoot = path.resolve(configuredRoot || '/root/happy');
  const requiredPaths = Object.values(bridgeSourcePaths(happyRoot));

  for (const requiredPath of requiredPaths) {
    try {
      await access(requiredPath);
    } catch {
      const prefix = configuredRoot
        ? `HAPPY_ROOT=${configuredRoot}`
        : 'default HAPPY_ROOT=/root/happy';
      throw new Error(
        `${prefix} does not contain the required Happy source file: ${requiredPath}\n` +
          'Set HAPPY_ROOT to a local Happy checkout before running this validator.',
      );
    }
  }

  return happyRoot;
}

function bridgeSourcePaths(happyRoot) {
  const happyWireDir = path.join(happyRoot, 'packages', 'happy-wire', 'src');
  const happyAppMessageMetaPath = path.join(
    happyRoot,
    'packages',
    'happy-app',
    'sources',
    'sync',
    'typesMessageMeta.ts',
  );

  return {
    messageMeta: path.join(happyWireDir, 'messageMeta.ts'),
    legacyProtocol: path.join(happyWireDir, 'legacyProtocol.ts'),
    sessionProtocol: path.join(happyWireDir, 'sessionProtocol.ts'),
    messages: path.join(happyWireDir, 'messages.ts'),
    voice: path.join(happyWireDir, 'voice.ts'),
    appMessageMeta: happyAppMessageMetaPath,
  };
}

async function buildBridgeModules(bridgeDir, bridgeSources) {
  for (const [moduleName, sourcePath] of Object.entries(bridgeSources)) {
    const source = await readFile(sourcePath, 'utf8');
    const transformed = transpileTypeScriptModule(source, sourcePath);
    await writeFile(path.join(bridgeDir, `${moduleName}.mjs`), transformed);
  }
}

async function loadBridgeModules(bridgeDir, bridgeSources) {
  const modules = {};

  for (const moduleName of Object.keys(bridgeSources)) {
    const moduleUrl = pathToFileURL(path.join(bridgeDir, `${moduleName}.mjs`)).href;
    modules[moduleName] = await import(moduleUrl);
  }

  return modules;
}

function transpileTypeScriptModule(source, sourcePath) {
  const result = ts.transpileModule(source, {
    fileName: sourcePath,
    reportDiagnostics: true,
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      esModuleInterop: true,
      skipLibCheck: true,
    },
  });

  if (result.diagnostics?.length) {
    const diagnostics = ts.formatDiagnosticsWithColorAndContext(result.diagnostics, {
      getCanonicalFileName: (fileName) => fileName,
      getCurrentDirectory: () => process.cwd(),
      getNewLine: () => '\n',
    });
    throw new Error(`failed to transpile ${sourcePath}\n${diagnostics}`);
  }

  return rewriteRelativeModuleSpecifiers(result.outputText);
}

function rewriteRelativeModuleSpecifiers(source) {
  return source
    .replace(
      /((?:import|export)\s[^'"]*from\s*['"])(\.\.?\/[^'"]+?)(['"])/g,
      '$1$2.mjs$3',
    )
    .replace(/(import\s*\(\s*['"])(\.\.?\/[^'"]+?)(['"]\s*\))/g, '$1$2.mjs$3');
}

async function validatePublishedFixtures(modules) {
  const messageMetaFixtures = await readFixtureFile(fixtureFiles.messageMeta);
  const legacyMessageFixtures = await readFixtureFile(fixtureFiles.legacyMessages);
  const sessionEnvelopeFixtures = await readFixtureFile(fixtureFiles.sessionEnvelopes);
  const messageContentFixtures = await readFixtureFile(fixtureFiles.messageContent);
  const updateContainerFixtures = await readFixtureFile(fixtureFiles.updateContainers);
  const voiceResponseFixtures = await readFixtureFile(fixtureFiles.voiceResponses);
  const invalidSessionEnvelopeFixtures = await readFixtureFile(
    fixtureFiles.invalidSessionEnvelopes,
  );

  validateFixtures(
    messageMetaFixtures,
    modules.appMessageMeta.MessageMetaSchema,
    'happy-app MessageMetaSchema',
  );
  validateFixtures(
    messageMetaFixtures.filter(({ name }) => name !== 'message-meta-custom-permission-mode'),
    modules.messageMeta.MessageMetaSchema,
    'happy-wire MessageMetaSchema',
  );

  const customPermissionFixture = messageMetaFixtures.find(
    ({ name }) => name === 'message-meta-custom-permission-mode',
  );
  assert(customPermissionFixture, 'missing message-meta-custom-permission-mode fixture');
  expectReject(
    modules.messageMeta.MessageMetaSchema,
    customPermissionFixture.value,
    'happy-wire MessageMetaSchema custom permissionMode fixture',
  );

  validateFixtures(
    legacyMessageFixtures,
    modules.legacyProtocol.LegacyMessageContentSchema,
    'happy-wire LegacyMessageContentSchema',
  );
  validateFixtures(
    sessionEnvelopeFixtures,
    modules.sessionProtocol.sessionEnvelopeSchema,
    'happy-wire sessionEnvelopeSchema',
  );
  validateFixtures(
    messageContentFixtures,
    modules.messages.MessageContentSchema,
    'happy-wire MessageContentSchema',
  );
  validateFixtures(
    updateContainerFixtures,
    modules.messages.CoreUpdateContainerSchema,
    'happy-wire CoreUpdateContainerSchema',
  );
  validateFixtures(
    voiceResponseFixtures,
    modules.voice.VoiceTokenResponseSchema,
    'happy-wire VoiceTokenResponseSchema',
  );

  for (const fixture of invalidSessionEnvelopeFixtures) {
    expectReject(
      modules.sessionProtocol.sessionEnvelopeSchema,
      fixture.value,
      `happy-wire sessionEnvelopeSchema invalid fixture ${fixture.name}`,
    );
  }
}

function validateOptionalNullRejections(modules) {
  expectReject(
    modules.appMessageMeta.MessageMetaSchema,
    { sentFrom: null },
    'happy-app MessageMetaSchema sentFrom=null',
  );
  expectReject(
    modules.appMessageMeta.MessageMetaSchema,
    { permissionMode: null },
    'happy-app MessageMetaSchema permissionMode=null',
  );
  expectReject(
    modules.legacyProtocol.UserMessageSchema,
    {
      role: 'user',
      content: {
        type: 'text',
        text: 'hello',
      },
      localKey: null,
    },
    'happy-wire UserMessageSchema localKey=null',
  );
  expectReject(
    modules.legacyProtocol.AgentMessageSchema,
    {
      role: 'agent',
      content: {
        type: 'output',
      },
      meta: null,
    },
    'happy-wire AgentMessageSchema meta=null',
  );
  expectReject(
    modules.sessionProtocol.sessionEventSchema,
    {
      t: 'text',
      text: 'hello',
      thinking: null,
    },
    'happy-wire sessionEventSchema text.thinking=null',
  );
  expectReject(
    modules.sessionProtocol.sessionEventSchema,
    {
      t: 'file',
      ref: 'upload-1',
      name: 'report.txt',
      size: 1,
      mimeType: null,
    },
    'happy-wire sessionEventSchema file.mimeType=null',
  );
  expectReject(
    modules.sessionProtocol.sessionEventSchema,
    {
      t: 'start',
      title: null,
    },
    'happy-wire sessionEventSchema start.title=null',
  );
  expectReject(
    modules.sessionProtocol.sessionEnvelopeSchema,
    {
      id: 'msg-1',
      time: 1,
      role: 'agent',
      turn: null,
      ev: {
        t: 'text',
        text: 'hello',
      },
    },
    'happy-wire sessionEnvelopeSchema turn=null',
  );
  expectReject(
    modules.sessionProtocol.sessionEnvelopeSchema,
    {
      id: 'msg-2',
      time: 1,
      role: 'agent',
      subagent: null,
      ev: {
        t: 'text',
        text: 'hello',
      },
    },
    'happy-wire sessionEnvelopeSchema subagent=null',
  );
  expectReject(
    modules.messages.SessionProtocolMessageSchema,
    {
      role: 'session',
      content: {
        id: 'msg-3',
        time: 1,
        role: 'agent',
        ev: {
          t: 'text',
          text: 'hello',
        },
      },
      meta: null,
    },
    'happy-wire SessionProtocolMessageSchema meta=null',
  );
  expectReject(
    modules.messages.UpdateMachineBodySchema,
    {
      t: 'update-machine',
      machineId: 'machine-1',
      active: null,
    },
    'happy-wire UpdateMachineBodySchema active=null',
  );
  expectReject(
    modules.messages.UpdateMachineBodySchema,
    {
      t: 'update-machine',
      machineId: 'machine-1',
      activeAt: null,
    },
    'happy-wire UpdateMachineBodySchema activeAt=null',
  );
}

async function readFixtureFile(fileName) {
  const raw = await readFile(path.join(fixtureDir, fileName), 'utf8');
  const parsed = JSON.parse(raw);

  if (!Array.isArray(parsed)) {
    throw new Error(`${fileName} must contain an array of named fixtures`);
  }

  return parsed;
}

function validateFixtures(fixtures, schema, schemaName) {
  for (const fixture of fixtures) {
    parseFixture(schema, fixture.value, `${schemaName} fixture ${fixture.name}`);
  }
}

function parseFixture(schema, value, description) {
  const result = schema.safeParse(value);
  if (!result.success) {
    throw new Error(`${description} failed: ${formatIssues(result.error.issues)}`);
  }
}

function expectReject(schema, value, description) {
  const result = schema.safeParse(value);
  if (result.success) {
    throw new Error(`${description} unexpectedly passed`);
  }
}

function formatIssues(issues) {
  return issues
    .map((issue) => `${issue.path.length === 0 ? '<root>' : issue.path.join('.')}: ${issue.message}`)
    .join('; ');
}

function assert(value, message) {
  if (!value) {
    throw new Error(message);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
