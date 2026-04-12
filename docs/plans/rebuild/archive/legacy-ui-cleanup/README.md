# Legacy UI Cleanup Archive

This directory contains documentation from the legacy UI migration that was completed on 2026-04-13.

## Archived Files

### README-HAPPY-UI.md
The original Happy-aligned UI documentation that described the migration from the legacy App.tsx to AppV2. This document is now historical as AppV2 is the sole UI shell.

### MIGRATION.md
The migration guide that explained how to migrate from the legacy 282KB App.tsx to the new Happy-aligned component architecture. The migration is now complete.

## Cleanup Summary

The following changes were made as part of the legacy UI cleanup:

1. **AppV2 became the sole UI shell** - All routes now use AppV2
2. **Legacy App.tsx removed** - The 283KB monolithic component (8,527 lines) was deleted
3. **packages/vibe-app deleted** - The deprecated legacy package was removed
4. **wave8 terminology cleaned** - Renamed to desktop-* naming:
   - `wave8-client.ts` → `desktop-client.ts`
   - `wave8-wire.ts` → `desktop-wire.ts`
   - `useWave8Desktop` → `useDesktopState`
   - `Wave8Client` → `DesktopClient`

## Current Architecture

See `packages/vibe-app-tauri/src/AppV2.tsx` for the current UI shell implementation.
