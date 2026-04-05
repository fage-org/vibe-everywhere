import { createRequire } from 'node:module';

import { describe, expect, it } from 'vitest';

const require = createRequire(import.meta.url);
const { findAction } = require('../../release.cjs') as {
    findAction: (input: string) => { id: string; description: string } | undefined;
};

describe('release action routing', () => {
    it('maps production to the store build action instead of OTA release', () => {
        const action = findAction('production');
        expect(action?.id).toBe('appstore-build');
    });

    it('keeps preview mapped to preview OTA', () => {
        const action = findAction('preview');
        expect(action?.id).toBe('ota-preview');
    });

    it('resolves the interactive production alias to the store build action', () => {
        const action = findAction('production');
        expect(action?.id).toBe('appstore-build');
    });
});
