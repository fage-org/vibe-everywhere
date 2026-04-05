import { describe, expect, it } from 'vitest';

import { resolveExpoConfig } from '../../app.config.js';

describe('resolveExpoConfig', () => {
    it('defaults OTA-less config to development when no variant env is provided', () => {
        const config = resolveExpoConfig({} as unknown as NodeJS.ProcessEnv);

        expect(config.expo.name).toBe('Vibe (dev)');
        expect(config.expo.extra.app.consoleLoggingDefault).toBe(true);
        expect(config.expo.updates).toBeUndefined();
        expect(config.expo.owner).toBeUndefined();
    });

    it('loads preview env values into the exported Expo config', () => {
        const config = resolveExpoConfig({
            VIBE_APP_ENV: 'preview',
            VIBE_EAS_PROJECT_ID: 'project-123',
            VIBE_EAS_UPDATE_URL: 'https://example.invalid/update',
            VIBE_EAS_OWNER: 'vibe-owner',
            VIBE_GOOGLE_SERVICES_FILE: './google-services.preview.json',
            EXPO_PUBLIC_VIBE_SERVER_URL: 'https://api.vibe.example',
        } as unknown as NodeJS.ProcessEnv);

        expect(config.expo.name).toBe('Vibe (preview)');
        expect(config.expo.extra.eas?.projectId).toBe('project-123');
        expect(config.expo.updates?.url).toBe('https://example.invalid/update');
        expect(config.expo.updates?.requestHeaders?.['expo-channel-name']).toBe('preview');
        expect(config.expo.owner).toBe('vibe-owner');
        expect(config.expo.android?.googleServicesFile).toBe('./google-services.preview.json');
        expect(config.expo.extra.app.serverUrl).toBe('https://api.vibe.example');
    });
});
