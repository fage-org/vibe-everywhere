import { randomBytes } from 'node:crypto';
import { mkdtemp, rm } from 'node:fs/promises';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

import { afterAll, afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('expo-crypto', async () => {
    const crypto = await import('node:crypto');
    return {
        getRandomBytes(length: number) {
            return new Uint8Array(crypto.randomBytes(length));
        },
    };
});

vi.mock('@/encryption/libsodium.lib', async () => {
    const naclModule = await import('tweetnacl');
    const nacl = naclModule.default;

    return {
        default: {
            crypto_box_NONCEBYTES: nacl.box.nonceLength,
            crypto_box_PUBLICKEYBYTES: nacl.box.publicKeyLength,
            crypto_box_seed_keypair(seed: Uint8Array) {
                const pair = nacl.box.keyPair.fromSecretKey(seed);
                return { publicKey: pair.publicKey, privateKey: pair.secretKey };
            },
            crypto_box_keypair() {
                const pair = nacl.box.keyPair();
                return { publicKey: pair.publicKey, privateKey: pair.secretKey };
            },
            crypto_box_easy(
                data: Uint8Array,
                nonce: Uint8Array,
                recipientPublicKey: Uint8Array,
                secretKey: Uint8Array,
            ) {
                const ciphertext = nacl.box(data, nonce, recipientPublicKey, secretKey);
                if (!ciphertext) {
                    throw new Error('failed to encrypt box payload');
                }
                return ciphertext;
            },
            crypto_box_open_easy(
                ciphertext: Uint8Array,
                nonce: Uint8Array,
                senderPublicKey: Uint8Array,
                secretKey: Uint8Array,
            ) {
                const plaintext = nacl.box.open(ciphertext, nonce, senderPublicKey, secretKey);
                if (!plaintext) {
                    throw new Error('failed to decrypt box payload');
                }
                return plaintext;
            },
            crypto_sign_seed_keypair(seed: Uint8Array) {
                const pair = nacl.sign.keyPair.fromSeed(seed);
                return { publicKey: pair.publicKey, privateKey: pair.secretKey };
            },
            crypto_sign_detached(message: Uint8Array, secretKey: Uint8Array) {
                return nacl.sign.detached(message, secretKey);
            },
        },
    };
});

vi.mock('@/sync/serverConfig', () => ({
    getServerUrl() {
        const serverUrl = process.env.TEST_VIBE_SERVER_URL;
        if (!serverUrl) {
            throw new Error('TEST_VIBE_SERVER_URL is not set');
        }
        return serverUrl;
    },
}));

import nacl from 'tweetnacl';

import { decodeBase64, encodeBase64 } from '../encryption/base64';
import { authAccountApprove } from './authAccountApprove';
import { authGetToken } from './authGetToken';
import { authQRStart, generateAuthKeyPair } from './authQRStart';
import { authQRWait } from './authQRWait';

type ChildCapture = {
    process: ReturnType<typeof spawn>;
    stdout: string;
    stderr: string;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../../');
const childProcesses = new Set<ReturnType<typeof spawn>>();

function concatBytes(...parts: Uint8Array[]): Uint8Array {
    const size = parts.reduce((sum, part) => sum + part.length, 0);
    const result = new Uint8Array(size);
    let offset = 0;
    for (const part of parts) {
        result.set(part, offset);
        offset += part.length;
    }
    return result;
}

function allocatePort(): Promise<number> {
    return new Promise((resolve, reject) => {
        const server = net.createServer();
        server.listen(0, '127.0.0.1', () => {
            const address = server.address();
            if (!address || typeof address === 'string') {
                reject(new Error('failed to allocate test port'));
                return;
            }
            server.close((error) => {
                if (error) {
                    reject(error);
                    return;
                }
                resolve(address.port);
            });
        });
        server.on('error', reject);
    });
}

function spawnProcess(command: string, args: string[], env: NodeJS.ProcessEnv): ChildCapture {
    const child = spawn(command, args, {
        cwd: repoRoot,
        env,
        stdio: ['ignore', 'pipe', 'pipe'],
    });
    childProcesses.add(child);

    const capture: ChildCapture = {
        process: child,
        stdout: '',
        stderr: '',
    };

    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');
    child.stdout.on('data', (chunk: string) => {
        capture.stdout += chunk;
    });
    child.stderr.on('data', (chunk: string) => {
        capture.stderr += chunk;
    });

    child.on('exit', () => {
        childProcesses.delete(child);
    });

    return capture;
}

async function waitForServer(serverUrl: string): Promise<void> {
    const startedAt = Date.now();
    while (Date.now() - startedAt < 60_000) {
        try {
            const response = await fetch(serverUrl);
            if (response.ok) {
                const text = await response.text();
                if (text.includes('Welcome to Vibe Server!')) {
                    return;
                }
            }
        } catch {
            // Retry until the server is ready.
        }
        await new Promise((resolve) => setTimeout(resolve, 250));
    }
    throw new Error('timed out waiting for vibe-server');
}

async function waitForMatch(capture: ChildCapture, pattern: RegExp, label: string): Promise<RegExpMatchArray> {
    const startedAt = Date.now();
    while (Date.now() - startedAt < 60_000) {
        const match = capture.stderr.match(pattern) ?? capture.stdout.match(pattern);
        if (match) {
            return match;
        }
        if (capture.process.exitCode !== null) {
            throw new Error(`${label} exited early\nstdout:\n${capture.stdout}\nstderr:\n${capture.stderr}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error(`timed out waiting for ${label}`);
}

async function waitForExit(capture: ChildCapture, label: string): Promise<number> {
    if (capture.process.exitCode !== null) {
        return capture.process.exitCode;
    }

    return await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
            reject(new Error(`timed out waiting for ${label} to exit`));
        }, 60_000);
        capture.process.once('exit', (code) => {
            clearTimeout(timeout);
            resolve(code ?? 0);
        });
        capture.process.once('error', (error) => {
            clearTimeout(timeout);
            reject(error);
        });
    });
}

function encryptAccountSecret(accountSecret: Uint8Array, recipientPublicKey: Uint8Array): Uint8Array {
    const ephemeral = nacl.box.keyPair();
    const nonce = randomBytes(nacl.box.nonceLength);
    const ciphertext = nacl.box(accountSecret, nonce, recipientPublicKey, ephemeral.secretKey);
    if (!ciphertext) {
        throw new Error('failed to encrypt account secret for account-link response');
    }
    return concatBytes(ephemeral.publicKey, nonce, ciphertext);
}

afterEach(() => {
    delete process.env.TEST_VIBE_SERVER_URL;
});

afterAll(() => {
    for (const child of childProcesses) {
        child.kill('SIGTERM');
    }
});

describe('app auth helper real chains', () => {
    it('links a real vibe-agent through app auth helpers and the account QR flow', async () => {
        const port = await allocatePort();
        const serverUrl = `http://127.0.0.1:${port}`;
        const tempRoot = await mkdtemp(path.join(os.tmpdir(), 'vibe-app-chain-'));
        const agentHome = path.join(tempRoot, 'agent-home');
        const accountSecret = new Uint8Array(randomBytes(32));
        process.env.TEST_VIBE_SERVER_URL = serverUrl;

        const server = spawnProcess(
            'cargo',
            ['run', '-q', '-p', 'vibe-server', '--', '--host', '127.0.0.1', '--port', String(port)],
            {
                ...process.env,
                VIBE_MASTER_SECRET: 'wave6-integration-secret',
                VIBE_SERVER_HOST: '127.0.0.1',
                VIBE_SERVER_PORT: String(port),
                VIBE_WEBAPP_URL: 'https://app.vibe.engineering',
            }
        );

        try {
            await waitForServer(serverUrl);
            const appToken = await authGetToken(accountSecret);

            const agent = spawnProcess(
                'cargo',
                ['run', '-q', '-p', 'vibe-agent', '--', 'auth', 'login', '--json'],
                {
                    ...process.env,
                    VIBE_SERVER_URL: serverUrl,
                    VIBE_HOME_DIR: agentHome,
                }
            );

            const urlMatch = await waitForMatch(
                agent,
                /vibe:\/\/\/account\?[A-Za-z0-9_-]+/,
                'vibe-agent auth login'
            );
            const deepLinkUrl = urlMatch[0];
            const publicKey = decodeBase64(
                deepLinkUrl.slice('vibe:///account?'.length),
                'base64url'
            );
            const responseBundle = encryptAccountSecret(accountSecret, publicKey);

            await authAccountApprove(appToken, publicKey, responseBundle);

            const exitCode = await waitForExit(agent, 'vibe-agent auth login');
            expect(exitCode).toBe(0);
            const loginPayload = JSON.parse(agent.stdout.trim()) as { status: string };
            expect(loginPayload.status).toBe('authenticated');

            const status = spawnProcess(
                'cargo',
                ['run', '-q', '-p', 'vibe-agent', '--', 'auth', 'status', '--json'],
                {
                    ...process.env,
                    VIBE_SERVER_URL: serverUrl,
                    VIBE_HOME_DIR: agentHome,
                }
            );
            const statusExit = await waitForExit(status, 'vibe-agent auth status');
            expect(statusExit).toBe(0);
            const statusPayload = JSON.parse(status.stdout.trim()) as { status: string; publicKey?: string };
            expect(statusPayload.status).toBe('authenticated');
            expect(statusPayload.publicKey).toBeTruthy();
        } finally {
            server.process.kill('SIGTERM');
            await rm(tempRoot, { recursive: true, force: true });
        }
    }, 120_000);

    it('restores app credentials through authQRStart/authQRWait using the real server flow', async () => {
        const port = await allocatePort();
        const serverUrl = `http://127.0.0.1:${port}`;
        const accountSecret = new Uint8Array(randomBytes(32));
        process.env.TEST_VIBE_SERVER_URL = serverUrl;

        const server = spawnProcess(
            'cargo',
            ['run', '-q', '-p', 'vibe-server', '--', '--host', '127.0.0.1', '--port', String(port)],
            {
                ...process.env,
                VIBE_MASTER_SECRET: 'wave6-integration-secret',
                VIBE_SERVER_HOST: '127.0.0.1',
                VIBE_SERVER_PORT: String(port),
                VIBE_WEBAPP_URL: 'https://app.vibe.engineering',
            }
        );

        try {
            await waitForServer(serverUrl);
            const appToken = await authGetToken(accountSecret);
            const keypair = generateAuthKeyPair();

            expect(await authQRStart(keypair)).toBe(true);

            const waitPromise = authQRWait(keypair);
            const responseBundle = encryptAccountSecret(accountSecret, keypair.publicKey);
            await authAccountApprove(appToken, keypair.publicKey, responseBundle);

            const credentials = await waitPromise;
            expect(credentials).not.toBeNull();
            expect(credentials?.token).toBeTruthy();
            expect(Array.from(credentials!.secret)).toEqual(Array.from(accountSecret));
        } finally {
            server.process.kill('SIGTERM');
        }
    }, 120_000);
});
