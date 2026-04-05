const readEnv = (env, ...keys) => {
    for (const key of keys) {
        const value = env[key];
        if (typeof value === 'string' && value.trim().length > 0) {
            return value;
        }
    }
    return undefined;
};

export function resolveExpoConfig(env = process.env) {
    const variant = env.VIBE_APP_ENV || env.APP_ENV || 'development';
    const name = {
        development: "Vibe (dev)",
        preview: "Vibe (preview)",
        production: "Vibe"
    }[variant];
    const bundleId = {
        development: "engineering.vibe.app.dev",
        preview: "engineering.vibe.app.preview",
        production: "engineering.vibe.app"
    }[variant];
    const elevenLabsAgentId = {
        development: 'agent_7801k2c0r5hjfraa1kdbytpvs6yt',
        preview: 'agent_7801k2c0r5hjfraa1kdbytpvs6yt',
        production: 'agent_6701k211syvvegba4kt7m68nxjmw',
    }[variant];
    const consoleLoggingDefault = {
        development: true,
        preview: true,
        production: false,
    }[variant];
    const updateChannel = {
        development: 'development',
        preview: 'preview',
        production: 'production',
    }[variant];
    const serverUrl = readEnv(
        env,
        'EXPO_PUBLIC_VIBE_SERVER_URL',
        'EXPO_PUBLIC_SERVER_URL',
        'EXPO_PUBLIC_HAPPY_SERVER_URL'
    );
    const easProjectId = readEnv(env, 'VIBE_EAS_PROJECT_ID');
    const easUpdateUrl = readEnv(env, 'VIBE_EAS_UPDATE_URL');
    const easOwner = readEnv(env, 'VIBE_EAS_OWNER');
    const googleServicesFile = readEnv(env, 'VIBE_GOOGLE_SERVICES_FILE');

    const androidConfig = {
        adaptiveIcon: {
            foregroundImage: "./sources/assets/images/icon-adaptive.png",
            monochromeImage: "./sources/assets/images/icon-monochrome.png",
            backgroundColor: "#18171C"
        },
        permissions: [
            "android.permission.RECORD_AUDIO",
            "android.permission.MODIFY_AUDIO_SETTINGS",
            "android.permission.ACCESS_NETWORK_STATE",
            "android.permission.POST_NOTIFICATIONS",
        ],
        blockedPermissions: [
            "android.permission.ACTIVITY_RECOGNITION",
            // Not using external storage/media access for now — blocks Google Play photo/video permission declaration
            "android.permission.READ_EXTERNAL_STORAGE",
            "android.permission.WRITE_EXTERNAL_STORAGE",
            "android.permission.READ_MEDIA_IMAGES",
            "android.permission.READ_MEDIA_VIDEO",
        ],
        package: bundleId,
        ...(googleServicesFile ? { googleServicesFile } : {}),
        intentFilters: variant === 'production' ? [
            {
                "action": "VIEW",
                "autoVerify": true,
                "data": [
                    {
                        "scheme": "https",
                        "host": "app.vibe.engineering",
                        "pathPrefix": "/"
                    }
                ],
                "category": ["BROWSABLE", "DEFAULT"]
            }
        ] : []
    };

    const extra = {
        router: {
            root: "./sources/app"
        },
        ...(easProjectId ? {
            eas: {
                projectId: easProjectId
            }
        } : {}),
        app: {
            postHogKey: readEnv(
                env,
                'EXPO_PUBLIC_VIBE_POSTHOG_KEY',
                'EXPO_PUBLIC_POSTHOG_KEY',
                'EXPO_PUBLIC_POSTHOG_API_KEY'
            ),
            revenueCatAppleKey: readEnv(
                env,
                'EXPO_PUBLIC_VIBE_REVENUE_CAT_APPLE',
                'EXPO_PUBLIC_REVENUE_CAT_APPLE'
            ),
            revenueCatGoogleKey: readEnv(
                env,
                'EXPO_PUBLIC_VIBE_REVENUE_CAT_GOOGLE',
                'EXPO_PUBLIC_REVENUE_CAT_GOOGLE'
            ),
            revenueCatStripeKey: readEnv(
                env,
                'EXPO_PUBLIC_VIBE_REVENUE_CAT_STRIPE',
                'EXPO_PUBLIC_REVENUE_CAT_STRIPE'
            ),
            elevenLabsAgentId,
            consoleLoggingDefault,
            serverUrl,
        }
    };

    return {
        expo: {
            name,
            slug: "vibe",
            version: "1.7.0",
            runtimeVersion: "21",
            orientation: "default",
            icon: "./sources/assets/images/icon.png",
            scheme: "vibe",
            userInterfaceStyle: "automatic",
            ios: {
                supportsTablet: true,
                bundleIdentifier: bundleId,
                config: {
                    usesNonExemptEncryption: false
                },
                infoPlist: {
                    NSMicrophoneUsageDescription: "Allow $(PRODUCT_NAME) to access your microphone for voice conversations with AI.",
                    NSLocalNetworkUsageDescription: "Allow $(PRODUCT_NAME) to find and connect to local devices on your network.",
                    NSBonjourServices: ["_http._tcp", "_https._tcp"]
                },
                associatedDomains: variant === 'production' ? ["applinks:app.vibe.engineering"] : []
            },
            android: androidConfig,
            web: {
                bundler: "metro",
                output: "single",
                favicon: "./sources/assets/images/favicon.png"
            },
            plugins: [
                require("./plugins/withEinkCompatibility.js"),
                [
                    "expo-router",
                    {
                        root: "./sources/app"
                    }
                ],
                "expo-updates",
                "expo-asset",
                "expo-localization",
                "expo-mail-composer",
                "expo-secure-store",
                "expo-web-browser",
                "react-native-vision-camera",
                "@more-tech/react-native-libsodium",
                "react-native-audio-api",
                "@livekit/react-native-expo-plugin",
                "@config-plugins/react-native-webrtc",
                [
                    "expo-audio",
                    {
                        microphonePermission: "Allow $(PRODUCT_NAME) to access your microphone for voice conversations."
                    }
                ],
                [
                    "expo-location",
                    {
                        locationAlwaysAndWhenInUsePermission: "Allow $(PRODUCT_NAME) to improve AI quality by using your location.",
                        locationAlwaysPermission: "Allow $(PRODUCT_NAME) to improve AI quality by using your location.",
                        locationWhenInUsePermission: "Allow $(PRODUCT_NAME) to improve AI quality by using your location."
                    }
                ],
                [
                    "expo-calendar",
                    {
                        "calendarPermission": "Allow $(PRODUCT_NAME) to access your calendar to improve AI quality."
                    }
                ],
                [
                    "expo-camera",
                    {
                        cameraPermission: "Allow $(PRODUCT_NAME) to access your camera to scan QR codes and share photos with AI.",
                        microphonePermission: "Allow $(PRODUCT_NAME) to access your microphone for voice conversations.",
                        recordAudioAndroid: true
                    }
                ],
                [
                    "expo-notifications",
                    {
                        "enableBackgroundRemoteNotifications": true,
                        "icon": "./sources/assets/images/icon-notification.png"
                    }
                ],
                [
                    'expo-splash-screen',
                    {
                        ios: {
                            backgroundColor: "#F2F2F7",
                            dark: {
                                backgroundColor: "#1C1C1E",
                            }
                        },
                        android: {
                            image: "./sources/assets/images/splash-android-light.png",
                            backgroundColor: "#F5F5F5",
                            dark: {
                                image: "./sources/assets/images/splash-android-dark.png",
                                backgroundColor: "#1e1e1e",
                            }
                        }
                    }
                ]
            ],
            ...(easUpdateUrl ? {
                updates: {
                    url: easUpdateUrl,
                    requestHeaders: {
                        "expo-channel-name": updateChannel
                    }
                }
            } : {}),
            experiments: {
                typedRoutes: true
            },
            extra,
            ...(easOwner ? { owner: easOwner } : {})
        }
    };
}

export default resolveExpoConfig(process.env);
