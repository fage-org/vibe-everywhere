# Module Plan: vibe-app-tauri/mobile-native-capabilities

## Status

- completed on 2026-04-09 after the Wave 9 capability inventory was turned into an explicit
  implementation/waiver register, desktop/browser parity-critical capability seams stayed wired in
  code, Android-only native blockers were deferred in writing, and the release migration plan now
  records the keep/defer decisions that unblock promotion planning

## Purpose

Implement or explicitly defer the Tauri-mobile-native and cross-platform capabilities required for
Wave 9 promotion.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
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

## Platform Scope Lock

- Android is the only active mobile platform in the current Wave 9 scope.
- iOS remains deferred and must not be treated as a promotion-blocking target.

## Target Location

- `packages/vibe-app-tauri` platform adapters and runtime capability seams

## Responsibilities

- deep links and callback ownership
- QR/camera flows where required
- push notifications and routing
- purchases and entitlement refresh
- voice/microphone permissions and flows
- file import/export/share where parity requires it
- explicit Happy mobile/native integration inventory and classification
- review prompts, haptics, and other platform-visible utility flows where still product-critical

## Non-Goals

- implementing every Happy mobile/native seam just because Happy used Expo to host it
- silently keeping unsupported capabilities out of the plan

## Dependencies

- `auth-and-identity-flows`
- `session-runtime-and-storage`
- `session-rendering-and-composer`

## Implementation Steps

1. Build and maintain an explicit inventory of Happy mobile/native seams and their pure-Tauri Wave 9
   replacements or deferrals.
2. Confirm every `C1 promotion-critical` capability in the matrix against actual Happy usage.
3. Implement the replacement path or explicit deferral note for each one.
4. Validate Android device permissions and runtime behavior on real devices.
5. Keep desktop-specific capability rules explicit where desktop parity still depends on them.
6. Record any release-impacting capability decisions in the migration plan.

## Edge Cases And Failure Modes

- notification routing drift after upgrade
- purchase entitlement refresh mismatches
- QR or camera permissions drifting across Android device variants
- microphone or voice flows working in local testing but not on Android hardware
- native-integration gaps getting dropped implicitly instead of being classified

## Tests

- Android real-device notification validation
- Android QR/camera validation
- Android purchase and entitlement smoke tests
- Android voice and microphone permission checks
- file/share flow checks where required
- mobile/native integration inventory review against the Wave 9 matrix

## Acceptance Criteria

- promotion-critical capabilities are implemented or explicitly waived in writing
- no Happy mobile/native integration seam remains unclassified in planning
- no hidden mobile-native blocker remains for Wave 9 promotion

## Locked Decisions

- capabilities must be explicit; silent omission is not allowed
- real-device validation is required for promotion-critical mobile capabilities
- capability-driven UI affordances must stay aligned with
  `docs/plans/rebuild/shared/ui-visual-parity.md` unless a narrower exception is recorded first

## Wave 9 Promotion Decision Register

| Capability | Wave 9 decision | Scope note | Validation or evidence |
| --- | --- | --- | --- |
| notification routing | deferred on Android; informational on desktop/browser | no notification-driven route restoration ships in the current candidate | package-local capability matrix plus migration-plan waiver |
| push registration | deferred on Android | the current candidate does not register device tokens | migration-plan waiver |
| purchases / entitlement refresh | deferred | no current Wave 9 route depends on purchase-gated state | migration-plan waiver |
| camera and QR scan | deferred | current auth/link flows use QR display plus fallback link/manual restore, not in-app scanning | migration-plan waiver |
| voice / microphone capture | deferred | settings continuity remains live, but capture/permission flows are not part of the promotion candidate | package-local capability matrix plus migration-plan waiver |
| file import/export/share on desktop/browser | implemented | restore, artifact export, and utility save flows stay available where the runtime supports them | `packages/vibe-app-tauri/src/native-capabilities.ts` plus route tests |
| file import/export/share on Android | deferred | Android keeps paste/copy fallback paths instead of native file/share ownership | package-local capability matrix plus route tests |
| desktop clipboard / dialog / notification semantics | implemented | desktop parity still depends on these non-mobile seams | existing Tauri commands plus route coverage |
| review prompts and haptics | waived for current promotion gate | non-blocking polish flows remain outside the current replacement slice | explicit waiver only |
