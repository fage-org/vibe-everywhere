# Wave 9 Route And Capability Matrix: `vibe-app-tauri`

## Purpose

This file records the Wave 9 route families and capability owners needed for `packages/vibe-app-tauri`
to replace `packages/vibe-app` across desktop, Tauri-mobile Android, and retained static browser
export behavior.

Plan directly from Happy modules. Use `packages/vibe-app` only when Happy cannot answer a
Vibe-specific continuity question; it is deprecated from active CI and release ownership.

## Ownership Model

Wave 9 keeps different UI shells for desktop and Android/browser surfaces while staying inside one
web-native app boundary.

Use the owner columns below as follows:

- `shared/runtime owner`: owns shared state, bootstrap semantics, adapters, or release/runtime rules
- `mobile/browser UI owner`: owns mobile/browser-facing web-native UI work rendered through Tauri
  mobile and the retained browser build
- `desktop UI owner`: owns active Wave 9 desktop shell, host-component, and desktop-specific adapter
  work

Historical Wave 8 desktop plans may still answer continuity questions, but they are not active
execution owners for Wave 9.

## Platform Scope Lock

- Android is the only active mobile platform in the current Wave 9 scope.
- iOS remains deferred and any Happy iOS-only behavior is continuity reference material only unless a
  later plan update explicitly reactivates it.
- `web export` in this file means retained static browser export, not a second fully interactive
  platform by default.
- A route marked with `web export` must say in notes whether the retained export is
  `static-export-safe` or `runtime-only`.

## Route Matrix

| Route family or surface | Priority | Platforms | Primary Happy source | Shared/runtime owner | Mobile/browser UI owner | Desktop UI owner | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| root provider chain, theme/bootstrap order, and top-level app init | `P0` | Android, web export | `/root/happy/packages/happy-app/sources/app/_layout.tsx`, `/root/happy/packages/happy-app/sources/theme.css`, `/root/happy/packages/happy-app/sources/theme.ts`, `/root/happy/packages/happy-app/sources/unistyles.ts` | `universal-bootstrap-and-runtime` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | preserve auth, modal, realtime, command-palette, status, theme, splash, and provider ordering |
| app route stack and top-level route naming | `P0` | Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | preserve route names and top-level entry semantics |
| retained static browser export bootstrap | `P1` | web export | `/root/happy/packages/happy-app/package.json`, `/root/happy/packages/happy-app/app.config.js`, `/root/happy/packages/happy-app/sources/app/_layout.tsx` | `web-export-and-browser-runtime` | `web-export-and-browser-runtime` | `n/a` | static-export-safe; retained static browser export capability must continue to exist explicitly |
| `/(app)/index` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/index.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | runtime-only for web export unless a later plan update promotes a static-export-safe variant explicitly |
| `/(app)/restore/index` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/restore/index.tsx` | `auth-and-identity-flows` | `auth-and-identity-flows` | `desktop-shell-and-platform-parity` | runtime-only for web export; QR/device-link semantics must remain compatible where browser handoff exists |
| `/(app)/restore/manual` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/restore/manual.tsx` | `auth-and-identity-flows` | `auth-and-identity-flows` | `desktop-shell-and-platform-parity` | static-export-safe only if the retained export keeps a usable manual-restore entry; otherwise runtime-only |
| `/(app)/inbox/index` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/inbox/index.tsx`, `/root/happy/packages/happy-app/sources/components/InboxView.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | preserve Android phone/tablet header rules and desktop information density |
| `/(app)/new/index` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/new/index.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | new-session creation and draft affordances |
| `/(app)/session/recent` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/recent.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | session inventory entry surface |
| `/(app)/session/[id]` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/[id].tsx`, `/root/happy/packages/happy-app/sources/-session/SessionView.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | core product route |
| `/(app)/settings/index` | `P0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/index.tsx`, `/root/happy/packages/happy-app/sources/components/SettingsView.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | static-export-safe only if the retained export keeps a usable settings landing surface; otherwise runtime-only |
| `/(app)/session/[id]/message/[messageId]` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/[id]/message/[messageId].tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | message permalink/detail route |
| `/(app)/session/[id]/info` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/[id]/info.tsx` | `session-runtime-and-storage` | `secondary-routes-and-social` | `secondary-routes-and-social` | session metadata and diagnostics |
| `/(app)/session/[id]/files` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/[id]/files.tsx` | `session-runtime-and-storage` | `secondary-routes-and-social` | `secondary-routes-and-social` | file list surface |
| `/(app)/session/[id]/file` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/session/[id]/file.tsx` | `session-runtime-and-storage` | `secondary-routes-and-social` | `secondary-routes-and-social` | single file viewer |
| `/(app)/settings/account` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/account.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | account detail surface |
| `/(app)/settings/appearance` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/appearance.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | theme and appearance state |
| `/(app)/settings/features` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/features.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | feature flags and experiments |
| `/(app)/settings/language` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/language.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | language settings surface |
| `/(app)/settings/voice` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/voice.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | voice settings hub |
| `/(app)/settings/voice/language` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/voice/language.tsx`, `/root/happy/packages/happy-app/sources/app/(app)/settings/voice.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | explicit child route reachable from the voice settings flow |
| `/(app)/settings/usage` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/usage.tsx`, `/root/happy/packages/happy-app/sources/components/usage/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | usage, plan, quota views |
| `/(app)/settings/connect/claude` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/settings/connect/claude.tsx`, `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | explicit vendor/connect route; do not leave it implied under settings |
| `/(app)/terminal/connect` and `/(app)/terminal/index` | `P1` | desktop, Android where applicable | `/root/happy/packages/happy-app/sources/app/(app)/terminal/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | keep device/platform applicability explicit |
| `/(app)/artifacts/**` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/artifacts/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | list, detail, create, edit flows |
| `/(app)/friends/**` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/friends/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | feed/social ownership stays explicit |
| `/(app)/user/[id]` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/user/[id].tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | user detail |
| `/(app)/machine/[id]` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/machine/[id].tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | machine detail |
| `/(app)/server` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/server.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | custom server configuration surface |
| `/(app)/changelog` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/changelog.tsx`, `/root/happy/packages/happy-app/sources/changelog/**` | `shared-core-from-happy` | `secondary-routes-and-social` | `secondary-routes-and-social` | release notes visibility |
| `/(app)/text-selection` | `P1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/text-selection.tsx` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | text utility surface |
| `dev/**` | `P2` | desktop, Android, web export where still useful | `/root/happy/packages/happy-app/sources/app/(app)/dev/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | do not block promotion on low-value dev pages unless explicitly promoted |

## Rendering And Component Matrix

| Surface | Priority | Primary Happy source | Shared/runtime owner | Mobile/browser UI owner | Desktop UI owner | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| main authenticated shell | `P0` | `/root/happy/packages/happy-app/sources/components/MainView.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | preserve phone/sidebar mode split |
| sidebar / drawer shell | `P0` | `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`, `/root/happy/packages/happy-app/sources/components/SidebarView.tsx` | `mobile-shell-and-navigation` | `mobile-shell-and-navigation` | `desktop-shell-and-platform-parity` | desktop and tablet navigation semantics |
| sessions list | `P0` | `/root/happy/packages/happy-app/sources/components/SessionsList.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | selected state, actions, status dots |
| session timeline | `P0` | `/root/happy/packages/happy-app/sources/components/MessageView.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | message kind dispatch must stay compatible |
| composer | `P0` | `/root/happy/packages/happy-app/sources/components/AgentInput.tsx` | `session-runtime-and-storage` | `session-rendering-and-composer` | `session-rendering-and-composer` | autocomplete, mode selectors, send/abort semantics |
| markdown | `P0` | `/root/happy/packages/happy-app/sources/components/markdown/**` | `shared-core-from-happy` | `session-rendering-and-composer` | `session-rendering-and-composer` | options, links, mermaid, parsing behavior |
| diff rendering | `P0` | `/root/happy/packages/happy-app/sources/components/diff/**` | `shared-core-from-happy` | `session-rendering-and-composer` | `session-rendering-and-composer` | diff parity |
| tool rendering | `P0` | `/root/happy/packages/happy-app/sources/components/tools/**` | `shared-core-from-happy` | `session-rendering-and-composer` | `session-rendering-and-composer` | tool-specific views and status affordances |
| usage views | `P1` | `/root/happy/packages/happy-app/sources/components/usage/**` | `secondary-routes-and-social` | `secondary-routes-and-social` | `secondary-routes-and-social` | settings usage route |
| browser-safe top-level providers and metadata affordances | `P1` | `/root/happy/packages/happy-app/sources/app/_layout.tsx`, `/root/happy/packages/happy-app/sources/components/web/FaviconPermissionIndicator.tsx` | `web-export-and-browser-runtime` | `web-export-and-browser-runtime` | `n/a` | static-export-safe; keep retained static browser export behavior from dropping web-specific affordances silently |

## Capability Matrix

| Capability | Priority | Platforms | Primary Happy source | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| auth credential storage | `C0` | desktop, Android | `/root/happy/packages/happy-app/sources/auth/tokenStorage.ts` | `auth-and-identity-flows` | storage backend differs by platform but semantics must match |
| create-account flow | `C0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`, `/root/happy/packages/happy-app/sources/auth/authGetToken.ts` | `auth-and-identity-flows` | Vibe naming only, same flow semantics |
| QR/device-link flow | `C0` | desktop, Android, web export where applicable | `/root/happy/packages/happy-app/sources/auth/authQRStart.ts`, `/root/happy/packages/happy-app/sources/auth/authQRWait.ts`, `/root/happy/packages/happy-app/sources/components/qr/**` | `auth-and-identity-flows` | desktop callback and Android deep-link ownership stay explicit |
| secret-key restore | `C0` | desktop, Android, web export where applicable | `/root/happy/packages/happy-app/sources/auth/secretKeyBackup.ts` | `auth-and-identity-flows` | backup import/export behavior must stay stable |
| sync bootstrap and storage | `C0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/sync/**` | `session-runtime-and-storage` | avoid protocol drift from `vibe-wire` |
| realtime subscriptions | `C0` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/realtime/**` | `session-runtime-and-storage` | reconnect and active-session behavior matter |
| retained static browser export generation | `C1` | web export | `/root/happy/packages/happy-app/package.json`, `/root/happy/packages/happy-app/app.config.js` | `web-export-and-browser-runtime` | retained static browser export capability must remain explicit |
| notification routing | `C1` | Android, desktop where applicable | `/root/happy/packages/happy-app/sources/utils/notificationRouting.ts` | `mobile-native-capabilities` | route restoration behavior |
| push registration | `C1` | Android | `/root/happy/packages/happy-app/sources/sync/pushRegistration.ts` | `mobile-native-capabilities` | Android-only in the active Wave 9 mobile scope |
| voice/microphone | `C1` | Android, desktop where required | `/root/happy/packages/happy-app/sources/realtime/RealtimeVoiceSession.tsx`, `/root/happy/packages/happy-app/sources/utils/microphonePermissions.ts` | `mobile-native-capabilities` | keep platform scope explicit |
| purchases / entitlement refresh | `C1` | Android, web where used | `/root/happy/packages/happy-app/sources/sync/purchases.ts`, `/root/happy/packages/happy-app/sources/sync/revenueCat/**` | `mobile-native-capabilities` | must preserve current product gating |
| camera and QR scan | `C1` | Android | `/root/happy/packages/happy-app/app.config.js`, `/root/happy/packages/happy-app/sources/components/qr/**` | `mobile-native-capabilities` | do not infer desktop parity unless needed |
| desktop clipboard and copy/export flows | `C1` | desktop, web export where applicable | `/root/happy/packages/happy-app/sources/utils/copySessionMetadataToClipboard.ts`, `/root/happy/packages/happy-app/sources/app/(app)/text-selection.tsx`, `/root/happy/packages/happy-app/sources/components/markdown/**` | `desktop-shell-and-platform-parity` | copy/export semantics must stay explicit for desktop promotion |
| file import/export/share | `C1` | desktop, Android, web export where applicable | Happy app routes and utilities that expose backup, artifacts, or exports | `mobile-native-capabilities` | platform-specific handoff allowed; semantics must match |
| desktop shell interaction semantics | `C1` | desktop | `/root/happy/packages/happy-app/sources/components/MainView.tsx`, `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`, `/root/happy/packages/happy-app/sources/modal/**` | `desktop-shell-and-platform-parity` | keyboard, focus, modal, and notification/file-dialog affordances must stay explicit before promotion |
| language and text ownership | `C1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/text/**`, `/root/happy/packages/happy-app/sources/constants/Languages.ts` | `shared-core-from-happy` | keep translation inventory explicit |
| changelog state | `C1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/changelog/**` | `shared-core-from-happy` | part of release continuity |
| analytics / tracking | `C1` | desktop, Android, web export | `/root/happy/packages/happy-app/sources/track/**`, `/root/happy/packages/happy-app/sources/app/_layout.tsx`, `/root/happy/packages/happy-app/sources/sync/sync.ts` | `release-ota-and-store-migration` | explicit keep/deferral decision required; include provider bootstrap, screen tracking, and analytics opt-in/out continuity |
| review prompts and store-review handoff | `C2` | Android where applicable | `/root/happy/packages/happy-app/sources/utils/requestReview.ts`, `/root/happy/packages/happy-app/sources/components/SessionsList.tsx` | `mobile-native-capabilities` | keep explicit even if deferred; do not leave prompt throttling or telemetry implications implicit |
| haptics and interaction feedback | `C2` | Android where applicable | `/root/happy/packages/happy-app/sources/components/haptics.ts` | `mobile-native-capabilities` | product-visible polish only; classify explicitly if omitted |
| app config, Android native project config, and release channels | `C1` | desktop, Android, web export | `/root/happy/packages/happy-app/app.config.js`, `/root/happy/packages/happy-app/eas.json` | `release-ota-and-store-migration` | promotion-blocking ownership; Happy Expo config is continuity input, not the required runtime/toolchain |

## Happy Native Integration Inventory

| Happy seam or capability signal | Priority | Platforms | Owner | Decision |
| --- | --- | --- | --- | --- |
| Expo Router route ownership in Happy | `C0` | Android, web export | `mobile-shell-and-navigation`, `web-export-and-browser-runtime` | replace with package-local web-native router ownership under Tauri/mobile/static-export builds |
| `withEinkCompatibility` | `C1` | Android | `mobile-native-capabilities`, `release-ota-and-store-migration` | preserve Android device-support policy through Tauri/native config unless a later Wave 9 plan update narrows support explicitly |
| `expo-updates` | `C1` | Android, web export | `release-ota-and-store-migration` | replace with explicit Tauri/browser update-channel policy; do not inherit Expo OTA as a requirement |
| localization setup in Happy | `C1` | Android, web export | `shared-core-from-happy`, `mobile-native-capabilities` | required as product capability, not as Expo tooling |
| `@more-tech/react-native-libsodium` | `C0` | Android, web export | `shared-core-from-happy` | required as crypto capability; implementation may differ from Happy's package seam |
| `expo-secure-store` | `C0` | Android | `auth-and-identity-flows` | replace with Tauri/mobile-native secure storage ownership |
| `expo-web-browser` | `C0` | Android, web export where applicable | `auth-and-identity-flows` | preserve browser handoff semantics where auth/connect flows require them |
| `expo-notifications` | `C1` | Android | `mobile-native-capabilities` | replace with Tauri/mobile-native notification ownership |
| `expo-camera` / `react-native-vision-camera` | `C1` | Android | `mobile-native-capabilities` | preserve QR/media capability through Tauri/mobile-native camera access where required |
| `expo-audio` / `react-native-audio-api` / `@livekit/react-native-expo-plugin` / `@config-plugins/react-native-webrtc` | `C1` | Android, desktop where required | `mobile-native-capabilities` | preserve voice capability without inheriting Expo as the runtime host |
| asset and splash bootstrap in Happy | `C0` | Android, web export | `universal-bootstrap-and-runtime` | required as bootstrap ownership; implementation should be Tauri/browser-native |
| `expo-local-authentication` | `C2` | Android | `mobile-native-capabilities` | deferred until a route-level need is confirmed |
| `expo-mail-composer` | `C2` | Android | `mobile-native-capabilities` | deferred until a route-level need is confirmed |
| `expo-location` | `C2` | Android | `mobile-native-capabilities` | deferred until a route-level need is confirmed |
| `expo-calendar` | `C2` | Android | `mobile-native-capabilities` | deferred until a route-level need is confirmed |

## Change Rule

If a route family, capability, or plugin classification changes priority, update this file and the
owning module plan before implementation continues.
