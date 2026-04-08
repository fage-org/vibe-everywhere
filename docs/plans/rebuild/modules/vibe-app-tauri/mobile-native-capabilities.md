# Module Plan: vibe-app-tauri/mobile-native-capabilities

## Purpose

Implement or explicitly defer the mobile-native and cross-platform capabilities required for Wave 9
promotion.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/components/qr/**`
- `/root/happy/packages/happy-app/sources/realtime/RealtimeVoiceSession.tsx`
- `/root/happy/packages/happy-app/sources/sync/pushRegistration.ts`
- `/root/happy/packages/happy-app/sources/sync/purchases.ts`
- `/root/happy/packages/happy-app/sources/sync/revenueCat/**`
- `/root/happy/packages/happy-app/sources/utils/notificationRouting.ts`
- `/root/happy/packages/happy-app/sources/utils/requestReview.ts`
- `/root/happy/packages/happy-app/sources/utils/microphonePermissions.ts`

## Target Location

- `packages/vibe-app-tauri` platform adapters and runtime capability seams

## Responsibilities

- deep links and callback ownership
- QR/camera flows where required
- push notifications and routing
- purchases and entitlement refresh
- voice/microphone permissions and flows
- file import/export/share where parity requires it
- explicit Happy `app.config.js` plugin inventory and classification
- review prompts, haptics, and other platform-visible utility flows where still product-critical

## Non-Goals

- implementing every Expo API just because Happy imported it
- silently keeping unsupported capabilities out of the plan

## Dependencies

- `auth-and-identity-flows`
- `session-runtime-and-storage`
- `session-rendering-and-composer`

## Implementation Steps

1. Build and maintain an explicit inventory of Happy `app.config.js` plugins and capability seams.
2. Confirm every `C1 promotion-critical` capability in the matrix against actual Happy usage.
3. Implement the replacement path or explicit deferral note for each one.
4. Validate mobile device permissions and runtime behavior on real devices.
5. Keep desktop-specific capability rules explicit where desktop parity still depends on them.
6. Record any release-impacting capability decisions in the migration plan.

## Edge Cases And Failure Modes

- notification routing drift after upgrade
- purchase entitlement refresh mismatches
- QR or camera permissions differing between iOS and Android
- microphone or voice flows working in simulator but not on device
- plugin-level capability gaps getting dropped implicitly instead of being classified

## Tests

- real-device notification validation
- QR/camera validation
- purchase and entitlement smoke tests
- voice and microphone permission checks
- file/share flow checks where required
- `app.config.js` plugin inventory review against the Wave 9 matrix

## Acceptance Criteria

- promotion-critical capabilities are implemented or explicitly waived in writing
- no Happy `app.config.js` plugin remains unclassified in planning
- no hidden mobile-native blocker remains for Wave 9 promotion

## Locked Decisions

- capabilities must be explicit; silent omission is not allowed
- real-device validation is required for promotion-critical mobile capabilities
