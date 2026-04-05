import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it, vi } from 'vitest';

import { ApiUpdateContainerSchema, ApiMessageSchema } from './apiTypes';
import { normalizeRawMessage, RawRecordSchema } from './typesRaw';
import { VoiceTokenResponseSchema } from './vibeWireCompat';

type NamedFixture<T> = {
    name: string;
    value: T;
};

const fixtureDir = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    '../../../../crates/vibe-wire/fixtures'
);

function loadFixture<T>(fileName: string): NamedFixture<T>[] {
    return JSON.parse(fs.readFileSync(path.join(fixtureDir, fileName), 'utf8')) as NamedFixture<T>[];
}

describe('vibe-wire compatibility fixtures', () => {
    it('parses durable update containers exported by vibe-wire', () => {
        for (const fixture of loadFixture<unknown>('update-containers.json')) {
            const parsed = ApiUpdateContainerSchema.safeParse(fixture.value);
            expect(parsed.success, fixture.name).toBe(true);
        }
    });

    it('parses voice-token responses exported by vibe-wire', () => {
        for (const fixture of loadFixture<unknown>('voice-responses.json')) {
            const parsed = VoiceTokenResponseSchema.safeParse(fixture.value);
            expect(parsed.success, fixture.name).toBe(true);
        }
    });

    it('normalizes session envelopes exported by vibe-wire', () => {
        for (const fixture of loadFixture<any>('session-envelopes.json')) {
            const record = {
                role: 'session',
                content: fixture.value,
            };
            const parsed = RawRecordSchema.safeParse(record);
            expect(parsed.success, fixture.name).toBe(true);

            const normalized = normalizeRawMessage(
                fixture.value.id,
                null,
                fixture.value.time,
                record as any
            );
            const dropsByDesign = fixture.value.ev?.t === 'turn-start'
                || fixture.value.ev?.t === 'start'
                || fixture.value.ev?.t === 'stop';
            if (dropsByDesign) {
                expect(normalized, fixture.name).toBeNull();
            } else {
                expect(normalized, fixture.name).not.toBeNull();
            }
        }
    });

    it('rejects invalid session envelopes exported by vibe-wire', () => {
        const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});

        try {
            for (const fixture of loadFixture<any>('session-invalid-envelopes.json')) {
                const normalized = normalizeRawMessage(
                    fixture.value.id,
                    null,
                    fixture.value.time,
                    {
                        role: 'session',
                        content: fixture.value,
                    } as any
                );
                expect(normalized, fixture.name).toBeNull();
            }
        } finally {
            warn.mockRestore();
        }
    });

    it('parses legacy message fixtures exported by vibe-wire', () => {
        for (const fixture of loadFixture<any>('legacy-messages.json')) {
            const parsed = RawRecordSchema.safeParse(fixture.value);
            expect(parsed.success, fixture.name).toBe(true);

            const normalized = normalizeRawMessage(
                fixture.name,
                fixture.value.localKey ?? null,
                0,
                fixture.value
            );
            expect(normalized, fixture.name).not.toBeNull();
        }
    });

    it('parses session-message fixtures embedded in update containers', () => {
        for (const fixture of loadFixture<any>('update-containers.json')) {
            if (fixture.value?.body?.t !== 'new-message') {
                continue;
            }
            const parsed = ApiMessageSchema.safeParse(fixture.value.body.message);
            expect(parsed.success, fixture.name).toBe(true);
        }
    });
});
