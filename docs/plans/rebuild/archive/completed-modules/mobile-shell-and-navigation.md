# Module Plan: vibe-app-tauri/mobile-shell-and-navigation

## Purpose

Recreate the Happy mobile provider stack, route tree, and phone/tablet shell behavior inside
`packages/vibe-app-tauri` using the shared web-native shell rendered through Tauri mobile.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/inbox/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/index.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/sources/components/navigation/Header.tsx`

## Target Location

- `packages/vibe-app-tauri/sources/app/**`
- `packages/vibe-app-tauri/sources/mobile/**`

## Responsibilities

- root provider chain
- mobile route ownership through the package-local web-native router
- phone and tablet shell behavior
- headers, drawer/sidebar, and top-level navigation affordances
- major `P0` entry routes

## Non-Goals

- full session rendering parity
- release migration
- native capability implementation beyond what bootstrapping requires

## Dependencies

- `universal-bootstrap-and-runtime`
- `shared-core-from-happy`

## Implementation Steps

1. Recreate the Happy provider stack and init order in the new package.
2. Port the top-level route tree and route naming semantics.
3. Recreate phone/tablet navigation behavior and shell structure.
4. Port the main/home, inbox, and settings-hub entry surfaces.
5. Validate that mobile entry flows feel like Happy before deeper feature work starts.

## Current Refactor Direction

- confirmed gap on 2026-04-09:
  - `packages/vibe-app-tauri` Android currently boots through the same desktop-first `App.tsx`
    shell used for desktop/browser review, with runtime copy swaps instead of a Happy-style mobile
    host container
  - route inventory coverage is higher than interaction parity; many Android routes are `wired`
    through the desktop router and desktop state hook, but still do not follow Happy mobile shell
    behavior
- locked refactor direction:
  - Android must stop rendering through `DesktopShell` as the top-level host
  - the replacement must introduce a dedicated mobile host container that matches Happy's
    `MainView` / tab / header / route-stack interaction model even if some downstream surfaces still
    reuse shared data/state code temporarily
  - route inventory status must no longer be treated as proof of mobile interaction parity
- execution slices for the refactor:
  1. split a package-local `MobileShell` from the current desktop shell so Android no longer renders
     the desktop sidebar/topbar host
  2. restore Happy-style phone primary navigation semantics for sessions / inbox / settings before
     deeper route work
  3. replace desktop-only state and affordance language inside Android entry surfaces
  4. move session detail and settings detail routes onto mobile-first host composition instead of
     desktop review layout reuse

## Next Iteration Plan

This is the active next-iteration execution plan for restoring Android interaction parity. Execute it
in order; do not collapse the whole refactor into one change set.

### Iteration Goal

Turn the current Android runtime from a desktop-first review shell with mobile copy into a
Happy-shaped mobile host with:

- phone-first top-level navigation
- Happy-aligned home / inbox / settings interaction model
- mobile-owned session and settings route composition
- shared data/state reused only where it does not leak desktop interaction semantics into Android

### Slice 1 Status

- completed in the current iteration:
  - Android no longer depends on `DesktopShell` as its top-level host container
  - Android now renders through a dedicated package-local `MobileShell` host with a mobile header
    and bottom-tab primary navigation
- remaining limitation:
  - the mobile host still reuses multiple desktop-oriented route surfaces and desktop-oriented shared
    state semantics underneath the new shell

### Slice 2: Rebuild Happy Main View Semantics

- objective:
  - replace the current transitional mobile sessions/inbox/settings composition with a package-local
    equivalent of Happy `MainView variant="phone"`
- required source inputs:
  - `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
  - `/root/happy/packages/happy-app/sources/components/MainView.tsx`
  - `/root/happy/packages/happy-app/sources/components/TabBar.tsx`
  - `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
  - `/root/happy/packages/happy-app/sources/components/SettingsViewWrapper.tsx`
  - `/root/happy/packages/happy-app/sources/components/SessionsListWrapper.tsx`
- implementation tasks:
  1. introduce package-local mobile main-view components under `packages/vibe-app-tauri/sources/mobile/**`
  2. move Android tab ownership out of `HomeSurface` and into an explicit mobile main-view container
  3. preserve the Happy phone tab order: inbox / sessions / settings
  4. preserve Happy header action behavior for the mobile home and inbox entry surfaces
  5. remove desktop review framing from Android top-level entry surfaces
- exit criteria:
  - Android home no longer renders desktop hero or desktop review copy
  - Android inbox/settings routes render through the new mobile main-view container instead of
    desktop review surfaces
  - a targeted render test proves the Android root no longer contains desktop host copy or sidebar
    affordances

- status on 2026-04-09:
  - completed for the Android top-level host and primary entry routes
  - package-local mobile home / inbox / settings containers now own the top-level Android main-view
    semantics instead of reusing the desktop sidebar host
  - deeper route composition still needs follow-up in later slices

### Slice 3: Split Shared App State From Desktop Review State

- objective:
  - stop using `useWave8Desktop` as the de facto Android state contract
- required source inputs:
  - `packages/vibe-app-tauri/src/useWave8Desktop.ts`
  - Happy auth and sync state inputs under `/root/happy/packages/happy-app/sources/auth/**`,
    `/root/happy/packages/happy-app/sources/sync/**`, and `/root/happy/packages/happy-app/sources/realtime/**`
- implementation tasks:
  1. extract cross-platform auth/session/artifact/machine state into a package-local shared hook or
     service layer
  2. leave desktop-only affordances in a thin desktop adapter instead of the shared state hook
  3. remove desktop-only naming from Android-facing state APIs
  4. ensure mobile host components depend on shared state plus mobile composition helpers, not on
     `DesktopShell`
- exit criteria:
  - Android shell and top-level routes no longer import or depend on desktop-only state naming
  - desktop-only helpers such as keyboard shortcuts, desktop notifications, and desktop route review
    metadata remain outside the mobile state contract

- status on 2026-04-09:
  - completed for the top-level Android host and primary routes
  - package-local `useAppShellState` now acts as the cross-platform app-state contract used by the
    mobile host, while `useWave8Desktop` remains the desktop-oriented adapter entrypoint
  - Android top-level host composition no longer depends directly on `useWave8Desktop`

### Slice 4: Rebuild Mobile Session Host

- objective:
  - move Android session detail off the desktop review layout and onto a Happy-style mobile host
- required source inputs:
  - `/root/happy/packages/happy-app/sources/app/(app)/session/[id].tsx`
  - `/root/happy/packages/happy-app/sources/-session/SessionView.tsx`
- implementation tasks:
  1. introduce a dedicated mobile session host component
  2. remove desktop review copy, desktop layout assumptions, and desktop-specific diagnostics from
     Android session entry
  3. keep shared timeline/composer data semantics only where they do not force desktop interaction
     patterns into Android
  4. preserve Happy mobile route-stack behavior for session detail, message detail, file list, and
     file viewer
- exit criteria:
  - Android session entry no longer renders desktop review notes or desktop panel chrome
  - Android session child routes navigate from a mobile parent host rather than the desktop shell

- status on 2026-04-09:
  - completed for session detail and the core deep-linkable session children
  - Android session detail, message detail, session info, session files, and session file routes now
    compose under mobile-specific host surfaces instead of the desktop review layout
  - shared timeline/composer/file-loading data semantics are still reused underneath, but the mobile
    route host no longer renders desktop review framing or desktop sidebar shell chrome

### Slice 5: Rebuild Mobile Settings Host

- objective:
  - move Android settings detail routes under a mobile-first settings hub instead of desktop review
    cards
- required source inputs:
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/index.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/account.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/appearance.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/features.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/language.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/usage.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/voice.tsx`
- implementation tasks:
  1. rebuild Android settings hub structure around Happy mobile grouping and route push behavior
  2. remove desktop-only labels such as desktop configuration / desktop review / desktop route notes
  3. keep Android-native capability gaps explicit, but do not let deferred capabilities force desktop
     layout semantics into settings surfaces
- exit criteria:
  - Android settings routes behave like a mobile stack, not like a desktop review dashboard
  - Android settings copy is mobile-specific and no longer references desktop review semantics

- status on 2026-04-09:
  - completed for the active Android settings detail routes
  - Android account, appearance, features, language, usage, voice, voice-language, and
    connect-claude routes now compose through mobile-specific host surfaces instead of the desktop
    review dashboard
  - shared preference and account-setting mutations are still reused underneath, but the Android
    settings route host no longer depends on desktop review framing or desktop-only route copy

### Validation Gate For The Next Iteration

- required before the next iteration can be called complete:
  - `yarn workspace vibe-app-tauri typecheck`
  - `yarn workspace vibe-app-tauri test`
  - route-level render coverage for Android home / inbox / settings / session host
  - one Android smoke run proving the mobile shell boots without the desktop host container
- required narrative review:
  - compare Android home, inbox, settings, and session entry against Happy screenshots or live
    behavior and record the remaining interaction gaps in this file before starting the next slice

## Focused Parity Audit

### Audit Date

- 2026-04-09

### Scope Reviewed

- Happy mobile entry and tab host:
  - `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
  - `/root/happy/packages/happy-app/sources/components/MainView.tsx`
  - `/root/happy/packages/happy-app/sources/components/TabBar.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/inbox/index.tsx`
  - `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/settings/index.tsx`
  - `/root/happy/packages/happy-app/sources/components/SettingsView.tsx`
- Happy mobile restore and session flows:
  - `/root/happy/packages/happy-app/sources/app/(app)/restore/index.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/restore/manual.tsx`
  - `/root/happy/packages/happy-app/sources/app/(app)/session/[id].tsx`
  - `/root/happy/packages/happy-app/sources/-session/SessionView.tsx`
- Current Android host in `packages/vibe-app-tauri/src/App.tsx`

### Current Assessment

- top-level Android host parity:
  - improved materially
  - Android no longer boots through the desktop sidebar host
  - mobile-specific top-level host, session host, and settings host now exist
- remaining gap category:
  - interaction and capability parity is still behind Happy even after the host refactor
  - remaining gaps are now concentrated in route internals and mobile-native behavior rather than
    top-level shell architecture

### Priority 0 Gaps

- restore flow still behaves like a shared desktop/web fallback flow instead of Happy's mobile-first
  QR restore screen
  - current `packages/vibe-app-tauri` route still uses the generic restore surface and desktop link
    semantics
  - Happy mobile restore starts QR auth immediately, shows mobile-specific instructions, and keeps
    manual restore as a secondary escape hatch
  - impact: first-run account recovery still feels unlike the shipping app
- mobile inbox content model is still not Happy-aligned
  - current Android inbox shows a lightweight session/services summary
  - Happy inbox is a feed + friend-request + social surface with update banners and profile entry
    flows
  - impact: one of the three primary tabs still diverges at the product level, not just the UI level
- mobile main sessions entry still lacks Happy `MainView` behavior depth
  - current Android sessions tab is a simplified card list with create-session CTA
  - Happy `MainView` manages richer tab state, badge semantics, empty/loading behavior, and
    phone/tablet coordination
  - impact: home feels closer than before, but still not behaviorally equivalent
- mobile session detail still lacks Happy phone interaction model
  - current Android session host no longer uses desktop review framing, but it still renders a
    simplified timeline/composer stack
  - Happy mobile session view includes mobile header behavior, content/input composition, empty
    message states, session actions, and richer voice/resume overlays
  - impact: core daily-use route still has visible interaction drift

### Priority 1 Gaps

- mobile settings index/detail content remains structurally lighter than Happy
  - current Android settings host and detail routes are now mobile-specific, but still omit portions
    of Happy grouping such as support/social/machines and some richer account/settings affordances
  - impact: settings is directionally correct but not yet functionally complete
- mobile settings detail routes still reuse desktop-flavored preference semantics under the hood
  - appearance, voice, and usage mutations still ride desktop-oriented state and copy in several
    places
  - impact: labels improved, but some preference behavior still reflects desktop assumptions
- mobile file/message deep links still reuse desktop-oriented renderer semantics
  - session child routes now sit under a mobile host, but file/diff/message presentation still leans
    on desktop data and renderer assumptions
  - impact: deep links are serviceable but not yet mobile-native in feel

### Priority 2 Gaps

- social and friends surfaces remain deferred and therefore unlike Happy
- some about/support/developer affordances remain incomplete on Android
- mobile-native capability surfaces such as QR scanning, push restoration, file import/export, and
  voice capture remain explicitly deferred elsewhere in planning

### Execution Order After Audit

1. restore flow parity
   - rebuild Android restore/index and restore/manual around Happy mobile behavior
2. inbox parity
   - bring Android inbox closer to Happy feed/friends/update structure
3. session detail parity
   - enrich Android session host with Happy-style mobile interaction behavior
4. settings completeness
   - fill remaining Android settings grouping/detail gaps after the higher-frequency routes settle

### Success Condition For The Next Delivery Slice

- Android home, restore, inbox, and session entry all feel recognizably Happy-aligned in both route
  structure and interaction model, not just in palette and typography

## Focused Parity Regression Matrix

This matrix is the active regression view for Android route parity after the host refactor. Update it
when a route meaningfully changes behavior, not for cosmetic wording tweaks alone.

| Route family | Happy reference | Current Android state | Regression risk | Remaining gap | Priority |
| --- | --- | --- | --- | --- | --- |
| home / main tabs | `sources/app/(app)/index.tsx`, `components/MainView.tsx`, `components/TabBar.tsx` | mobile host, tabs, quick actions, status, and recent-session states now exist | low | still lighter than Happy `MainView` in loading/badge nuance and feed coupling | `P1` |
| restore / manual restore | `sources/app/(app)/restore/index.tsx`, `sources/app/(app)/restore/manual.tsx` | QR-first restore and manual secret-key restore now exist | medium | QR/device-link copy and approval choreography still need live-device comparison | `P1` |
| inbox | `components/InboxView.tsx`, `components/FeedItemCard.tsx`, `components/UserCard.tsx` | real feed/friends data now wired into Android inbox with richer section summaries and navigation CTAs | medium | feed card richness and user-card affordances are still lighter than Happy | `P0` |
| friends search | `sources/app/(app)/friends/search.tsx` | real search and friend mutation path now available on Android | low | result presentation is still lighter than Happy | `P1` |
| user detail | `sources/app/(app)/user/[id].tsx` | real user detail and friend action path now available on Android | medium | profile details and external profile affordances remain thinner than Happy | `P1` |
| settings hub | `components/SettingsView.tsx`, `sources/app/(app)/settings/index.tsx` | mobile-specific settings host and detail routes now exist, including explicit friends and support entry points | low | about/support richness is still lighter than Happy | `P1` |
| session detail | `sources/-session/SessionView.tsx` | mobile session host, empty state, resume hint, shared session context, and deep-link actions now exist | medium | header/action behavior is improved, but still not fully at Happy mobile depth | `P1` |
| session message / file deep links | `sources/app/(app)/session/[id]/**`, `sources/-session/SessionView.tsx` | mobile deep links now share session context, topbar actions, and denser content sections | low | renderer density and navigation polish still need refinement | `P1` |

### Selected Next Gap

- selected from the matrix on 2026-04-09:
  - `inbox` richer behavior
- reason:
  - inbox now has real feed/friends data, so the most visible remaining product gap is the
    presentation richness of update and friend cards rather than missing route ownership

## Closeout Summary

### Completed In This Refactor Pass

- Android no longer boots through the desktop sidebar host
- Android top-level tabs, session host, settings host, restore flows, and social entry routes all
  run through mobile-specific host surfaces
- session deep links now preserve shared session context and mobile topbar actions
- inbox now uses real feed and friend data instead of static placeholder content
- friends search, user detail, and friend mutation flows now exist on Android

### Remaining Highest-Signal Gaps

- inbox/feed cards still lack Happy-level richness
- session header/action polish still trails Happy mobile behavior
- some support/about/profile details remain lighter than Happy
- native capabilities remain partially deferred outside this host refactor scope

### Smallest Next Cut

- continue within `inbox` only:
  - tighten feed card hierarchy and social card affordances around the now-live feed/friends data
  - avoid reopening shell/state architecture while doing so

## Edge Cases And Failure Modes

- provider ordering drift that breaks auth or realtime state
- tablet and phone behavior collapsing into one incorrect layout
- route names drifting from Happy semantics
- font or safe-area boot timing differences causing broken first paint

## Tests

- route smoke tests for `P0` entry routes
- provider boot smoke checks
- phone and tablet layout validation

## Acceptance Criteria

- Android can boot into a recognizable Happy-aligned shell
- `P0` entry routes exist and are navigable
- route and shell semantics are stable enough for auth and session module work
- Android does not depend on `DesktopShell` as its top-level host container

## Locked Decisions

- mobile shell is reconstructed from Happy directly
- mobile shell stays on the shared web-native app boundary under Tauri mobile; do not reintroduce
  Expo or a React Native shell as the runtime host
- mobile shell visuals and hierarchy must remain governed by
  `docs/plans/rebuild/shared/ui-visual-parity.md` unless a narrower exception is recorded first
