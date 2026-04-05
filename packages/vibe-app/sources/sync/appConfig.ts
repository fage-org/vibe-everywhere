import Constants from 'expo-constants';
import { requireOptionalNativeModule } from 'expo-modules-core';

export interface AppConfig {
    postHogKey?: string;
    revenueCatAppleKey?: string;
    revenueCatGoogleKey?: string;
    revenueCatStripeKey?: string;
    elevenLabsAgentId?: string;
    consoleLoggingDefault?: boolean;
    serverUrl?: string;
}

function readEnv(...keys: string[]): string | undefined {
    for (const key of keys) {
        const value = process.env[key];
        if (typeof value === 'string' && value.trim().length > 0) {
            return value;
        }
    }
    return undefined;
}

/**
 * Loads app configuration from various manifest sources.
 * Looks for the "app" field in expoConfig.extra across different manifests
 * and merges them into a single configuration object.
 * 
 * Priority (later overrides earlier):
 * 1. ExponentConstants native module manifest (fetches embedded manifest)
 * 2. Constants.expoConfig
 */
export function loadAppConfig(): AppConfig {
    const config: Partial<AppConfig> = {};

    try {
        // 1. Try ExponentConstants native module directly
        const ExponentConstants = requireOptionalNativeModule('ExponentConstants');
        if (ExponentConstants && ExponentConstants.manifest) {
            let exponentManifest = ExponentConstants.manifest;

            // On Android, manifest is passed as JSON string
            if (typeof exponentManifest === 'string') {
                try {
                    exponentManifest = JSON.parse(exponentManifest);
                } catch (e) {
                    console.warn('[loadAppConfig] Failed to parse ExponentConstants.manifest:', e);
                }
            }

            // Look for app config in various locations
            const appConfig = exponentManifest?.extra?.app;
            if (appConfig && typeof appConfig === 'object') {
                Object.assign(config, appConfig);
                console.log('[loadAppConfig] Loaded from ExponentConstants:', Object.keys(config));
            }
        }
    } catch (e) {
        console.warn('[loadAppConfig] Error accessing ExponentConstants:', e);
    }

    try {
        // 2. Try Constants.expoConfig
        if (Constants.expoConfig?.extra?.app) {
            const appConfig = Constants.expoConfig.extra.app;
            if (typeof appConfig === 'object') {
                Object.assign(config, appConfig);
                console.log('[loadAppConfig] Loaded from Constants.expoConfig:', Object.keys(config));
            }
        }
    } catch (e) {
        console.warn('[loadAppConfig] Error accessing Constants.expoConfig:', e);
    }

    console.log('[loadAppConfig] Final merged config:', JSON.stringify(config, null, 2));

    // Override with EXPO_PUBLIC_* env vars if present at runtime and different
    // Why: Native config is baked at prebuild time, but EXPO_PUBLIC_* vars
    // are available at runtime via process.env. This allows devs to change
    // keys without rebuilding native code.
    const revenueCatAppleKey = readEnv('EXPO_PUBLIC_VIBE_REVENUE_CAT_APPLE', 'EXPO_PUBLIC_REVENUE_CAT_APPLE');
    if (revenueCatAppleKey && config.revenueCatAppleKey !== revenueCatAppleKey) {
        console.log('[loadAppConfig] Override revenueCatAppleKey from EXPO_PUBLIC_VIBE_REVENUE_CAT_APPLE');
        config.revenueCatAppleKey = revenueCatAppleKey;
    }
    const revenueCatGoogleKey = readEnv('EXPO_PUBLIC_VIBE_REVENUE_CAT_GOOGLE', 'EXPO_PUBLIC_REVENUE_CAT_GOOGLE');
    if (revenueCatGoogleKey && config.revenueCatGoogleKey !== revenueCatGoogleKey) {
        console.log('[loadAppConfig] Override revenueCatGoogleKey from EXPO_PUBLIC_VIBE_REVENUE_CAT_GOOGLE');
        config.revenueCatGoogleKey = revenueCatGoogleKey;
    }
    const revenueCatStripeKey = readEnv('EXPO_PUBLIC_VIBE_REVENUE_CAT_STRIPE', 'EXPO_PUBLIC_REVENUE_CAT_STRIPE');
    if (revenueCatStripeKey && config.revenueCatStripeKey !== revenueCatStripeKey) {
        console.log('[loadAppConfig] Override revenueCatStripeKey from EXPO_PUBLIC_VIBE_REVENUE_CAT_STRIPE');
        config.revenueCatStripeKey = revenueCatStripeKey;
    }
    const postHogKey = readEnv(
        'EXPO_PUBLIC_VIBE_POSTHOG_KEY',
        'EXPO_PUBLIC_POSTHOG_KEY',
        'EXPO_PUBLIC_POSTHOG_API_KEY'
    );
    if (postHogKey && config.postHogKey !== postHogKey) {
        console.log('[loadAppConfig] Override postHogKey from EXPO_PUBLIC_VIBE_POSTHOG_KEY');
        config.postHogKey = postHogKey;
    }
    const serverUrl = readEnv(
        'EXPO_PUBLIC_VIBE_SERVER_URL',
        'EXPO_PUBLIC_SERVER_URL',
        'EXPO_PUBLIC_HAPPY_SERVER_URL'
    );
    if (serverUrl && config.serverUrl !== serverUrl) {
        console.log('[loadAppConfig] Override serverUrl from EXPO_PUBLIC_VIBE_SERVER_URL');
        config.serverUrl = serverUrl;
    }

    return config as AppConfig;
}
