import fs from 'node:fs';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

const appRoot = path.resolve(__dirname, '../..');

function readWorkflow(fileName: string): string {
    return fs.readFileSync(path.join(appRoot, '.eas', 'workflows', fileName), 'utf8');
}

describe('EAS workflow invariants', () => {
    it('keeps preview OTA manual and variant-pinned', () => {
        const workflow = readWorkflow('preview.yaml');
        expect(workflow).not.toMatch(/\bon:\s*\n\s*push:/);
        expect(workflow).toContain('environment: preview');
        expect(workflow).toContain('VIBE_APP_ENV: preview');
        expect(workflow).toContain('APP_ENV: preview');
    });

    it('pins production OTA to the production variant', () => {
        const workflow = readWorkflow('ota.yaml');
        expect(workflow).toContain('environment: production');
        expect(workflow).toContain('VIBE_APP_ENV: production');
        expect(workflow).toContain('APP_ENV: production');
    });
});
