# Shared UI And Visual Parity Rules

## Purpose

Define the durable UI and visual rules for the Happy-aligned rebuild.

For `packages/vibe-app-tauri`, the goal is not to invent a new design language. The goal is to
recreate `/root/happy/packages/happy-app` closely enough that side-by-side review reads as the same
product unless a plan file records a deliberate exception first.

## Scope

This shared rule file applies to any app-facing UI work that can affect what users see or how they
navigate, including:

- route shell and top-level navigation
- headers, sidebars, tab bars, grouped lists, cards, and modal chrome
- typography, spacing, corner radii, elevation, and surface treatment
- color tokens, theme behavior, icons, logos, and bundled visual assets
- session, inbox, settings, restore, changelog, and secondary route presentation
- web-native, Tauri, and mobile runtime adaptations of Happy app surfaces

This file does not authorize redesign work. It exists to constrain implementation toward parity.

## Source Of Truth

- `/root/happy/packages/happy-app/sources/theme.ts`
- `/root/happy/packages/happy-app/sources/constants/Typography.ts`
- `/root/happy/packages/happy-app/sources/theme.css`
- `/root/happy/packages/happy-app/sources/assets/**`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarView.tsx`
- `/root/happy/packages/happy-app/sources/components/navigation/Header.tsx`
- the Happy route/component file that owns the surface being migrated

If Happy and Vibe differ already, Happy remains the visual/product reference unless an active Vibe
plan explicitly records the deviation.

## Rules

1. Parity first, redesign later.
   - Do not introduce a new visual language, new chrome metaphor, or a new information hierarchy as
     part of the migration.
   - If the Happy surface is plain, the Vibe surface should stay plain.
   - If the Happy surface is dense, the Vibe surface must not be expanded into a spacious marketing-style layout.

2. Recreate Happy structure before polishing.
   - Match route shell hierarchy first: header, sidebar, main content, grouped sections, tabs, and
     action placement.
   - Do not substitute planning dashboards, debug panels, or migration-status cards for the primary
     user-facing layout.

3. Reuse Happy visual tokens by default.
   - Typography should follow Happy's font choices and weight usage unless an approved exception says otherwise.
   - Color, grouped-surface behavior, separators, button treatment, and dark/light theme behavior
     should track Happy rather than ad hoc web styling.
   - Rounded corners, borders, shadows, and spacing should stay within Happy's visual range unless
     the owning plan records a narrower exception first.

4. Reuse Happy brand assets by default.
   - Prefer Happy-aligned logos, logotypes, icons, and bundled image assets over newly invented desktop-only branding.
   - Do not swap to unrelated illustration systems, gradients, or decorative backgrounds without an
     approved plan update.

5. Preserve information density.
   - A Happy list should remain a list, not become a card gallery unless the owning Happy surface
     already behaves that way.
   - A Happy settings group should remain grouped and utilitarian, not become a feature-marketing grid.
   - A Happy session-first surface must keep sessions visually primary over migration metadata.

6. Preserve interaction semantics.
   - Primary actions, secondary actions, destructive actions, and navigation affordances should stay
     in the same relative places as Happy whenever the host runtime allows it.
   - Runtime adaptations may change implementation details, but should not change the mental
     model of how the surface is used.

7. Runtime adaptation must still look like Happy.
   - A Tauri/web/mobile adaptation may translate Happy's host primitives into different implementation
     primitives, but the result
     must still read as the Happy app rather than a separate admin tool or design experiment.
   - Runtime-specific affordances are allowed only when they are additive and do not overpower the Happy shell.

8. Divergence requires an explicit written exception.
   - If exact or close parity is impractical, update the owning module plan before implementation.
   - The exception must name:
     - the affected surface
     - why parity is impractical
     - what is preserved instead
     - how the deviation will be reviewed

## Review Checklist

Before calling a UI task complete, check all of the following:

- Would a reviewer recognize the surface as the same Happy feature at a glance?
- Are typography, spacing, button treatment, and surface layering still in the Happy family?
- Is the primary user task more visually prominent than migration/debug metadata?
- Did any new decorative treatment appear that Happy does not use?
- If parity was relaxed, is the exception documented in the owning plan?

## Failure Modes

- replacing the app shell with a migration dashboard
- introducing glossy/glassmorphism/marketing treatments that Happy does not use
- drifting to a web-admin visual language because HTML/CSS made it easy
- keeping route semantics while losing Happy's density and hierarchy
- treating placeholder/debug UI as acceptable long-term chrome

## Acceptance Criteria

- migrated Vibe app surfaces remain recognizably Happy-aligned in side-by-side review
- visual drift is treated as a planning violation, not just a polish issue
- any approved divergence is explicit, local, and documented before code lands
