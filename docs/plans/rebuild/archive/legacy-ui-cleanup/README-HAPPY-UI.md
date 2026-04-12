# Happy-Aligned UI

> **Note:** This document has been superseded by the comprehensive [Design System Documentation](../docs/DESIGN_SYSTEM.md).
>
> Please refer to that document for the complete UI/UX specification.

## Quick Start

### Enable New UI

Add `?happy-ui=true` to the URL:
```
http://localhost:1420/?happy-ui=true
```

Or enable via localStorage:
```javascript
localStorage.setItem('vibe-feature-flags', JSON.stringify({ enableHappyUI: true }));
location.reload();
```

### Development Mode

In development, a feature flag panel appears in the bottom-right corner for easy toggling.

## Architecture Overview

```
src/
├── design-system/          # Design tokens and theme
│   ├── tokens.ts          # TypeScript design tokens
│   └── theme.css          # CSS custom properties
├── components/
│   ├── providers/         # React context providers
│   ├── ui/               # Primitive components
│   ├── layout/           # Layout components
│   ├── surfaces/         # Happy-specific surfaces
│   ├── renderers/        # Content renderers
│   └── routes/           # Route surfaces
└── AppV2.tsx             # New App component
```

## Documentation

- **[Design System](../docs/DESIGN_SYSTEM.md)** - Complete UI/UX specification
- **[Migration Guide](./MIGRATION.md)** - Migration from legacy architecture

## Development

```bash
# Install dependencies
yarn install

# Start dev server
yarn app

# Enable Happy UI
# Add ?happy-ui=true to URL

# Run tests
yarn workspace vibe-app-tauri test

# Type check
yarn workspace vibe-app-tauri typecheck

# Build
yarn workspace vibe-app-tauri build
```

## Roadmap

- [x] Phase 1: Design token system
- [x] Phase 2: UI primitive components
- [x] Phase 3: Layout components
- [x] Phase 4: Happy-specific surfaces
- [x] Phase 5: Content renderers
- [x] Phase 6: Route surfaces
- [x] Phase 7: App refactor
- [x] Phase 8: Feature flags and integration
- [ ] Phase 9: Performance optimization
- [ ] Phase 10: E2E testing
- [x] Phase 11: Documentation
- [ ] Phase 12: Remove legacy code
