# Migration Guide: Happy-Aligned UI Architecture

This guide explains how to migrate from the legacy 282KB App.tsx to the new Happy-aligned component architecture.

## Overview

The new architecture provides:
- **Pixel-perfect Happy UI replication** - Matches happy.ai design system
- **Modular component structure** - Organized into logical layers
- **Type-safe design tokens** - Single source of truth for all visual values
- **Responsive layouts** - Desktop and mobile optimized
- **Improved maintainability** - ~500 lines vs 282KB monolith

## Architecture Layers

```
src/
├── design-system/
│   ├── tokens.ts          # Design tokens (colors, typography, spacing)
│   └── theme.css          # CSS custom properties
├── components/
│   ├── providers/         # Context providers (ThemeProvider)
│   ├── ui/               # Primitive components (Button, Card, Input, etc.)
│   ├── layout/           # Layout components (Shell, Sidebar, Header)
│   ├── surfaces/         # Happy-specific (SessionList, Composer, Timeline)
│   ├── renderers/        # Content renderers (Diff, Markdown, Tool)
│   └── routes/           # Route surfaces (Home, Session, Settings, Inbox)
└── App.new.tsx           # New streamlined App component
```

## Quick Start

### 1. Import Components

```tsx
import {
  // Providers
  ThemeProvider,
  
  // Layout
  Shell, Sidebar, Header, MobileShell,
  
  // UI
  Button, Card, Input, Badge,
  
  // Surfaces
  SessionList, Timeline, Composer,
  
  // Routes
  HomeSurface, SessionSurface, SettingsSurface, InboxSurface,
  
  // Renderers
  DiffRenderer, MarkdownRenderer, ToolRenderer,
} from "./components";
```

### 2. Wrap with ThemeProvider

```tsx
function App() {
  return (
    <ThemeProvider defaultScheme="system">
      <AppContent />
    </ThemeProvider>
  );
}
```

### 3. Use Layout Components

```tsx
// Desktop
<Shell
  sidebar={<Sidebar brand="Vibe" primarySections={[...]} />}
  header={<Header title="Dashboard" eyebrow="Home" />}
>
  <HomeSurface ... />
</Shell>

// Mobile
<MobileShell
  header={<MobileNavBar title="Vibe" />}
  tabs={[...]}
  activeTab="home"
  onTabChange={...}
>
  <HomeSurface ... />
</MobileShell>
```

## Component Usage Examples

### Button

```tsx
<Button variant="primary" size="md" onClick={handleClick}>
  Click Me
</Button>

<Button variant="secondary" loading={isLoading}>
  Save
</Button>

<Button variant="ghost" disabled={!canSubmit}>
  Cancel
</Button>
```

### Card

```tsx
<Card variant="elevated">
  <CardHeader>
    <CardTitle>Card Title</CardTitle>
    <CardDescription>Card description</CardDescription>
  </CardHeader>
  <CardContent>
    Content goes here
  </CardContent>
  <CardFooter>
    <Button variant="primary">Action</Button>
  </CardFooter>
</Card>
```

### SessionList

```tsx
<SessionList
  sessions={sessions}
  selectedId={selectedSessionId}
  onSelect={(session) => setSelectedSessionId(session.id)}
  loading={isLoading}
/>
```

### Timeline + Composer

```tsx
<div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
  <Timeline messages={messages} />
  <Composer
    value={composerValue}
    onChange={setComposerValue}
    onSubmit={handleSend}
    suggestions={suggestions}
    isSending={isSending}
  />
</div>
```

### SettingsSurface

```tsx
<SettingsSurface
  sections={[
    {
      id: "appearance",
      title: "Appearance",
      settings: [
        {
          id: "theme",
          label: "Theme",
          type: "select",
          value: theme,
          options: [
            { label: "Light", value: "light" },
            { label: "Dark", value: "dark" },
          ],
          onChange: setTheme,
        },
        {
          id: "sidebar",
          label: "Collapsed Sidebar",
          type: "toggle",
          value: sidebarCollapsed,
          onChange: setSidebarCollapsed,
        },
      ],
    },
  ]}
  onSave={handleSave}
  hasChanges={hasChanges}
/>
```

### DiffRenderer

```tsx
<DiffRenderer
  files={[
    {
      path: "src/App.tsx",
      status: "modified",
      additions: 10,
      deletions: 5,
      hunks: [...],
    },
  ]}
  showLineNumbers={true}
  collapsible={true}
/>
```

### MarkdownRenderer

```tsx
<MarkdownRenderer
  content={markdownText}
  onLinkClick={(url) => openExternal(url)}
/>
```

## Design Tokens

Access design tokens for custom styling:

```tsx
import { tokens } from "./components";

// Colors
const primaryColor = tokens.colors.primary.dark;

// Spacing
const padding = tokens.spacing[4]; // 1rem

// Typography
const fontSize = tokens.typography.fontSize.lg;
const fontWeight = tokens.typography.fontWeight.semibold;

// Border radius
const radius = tokens.radii.lg;
```

## CSS Custom Properties

The theme system exposes CSS custom properties:

```css
/* Backgrounds */
var(--bg-primary)
var(--bg-secondary)
var(--surface-primary)
var(--surface-secondary)

/* Text */
var(--text-primary)
var(--text-secondary)
var(--text-tertiary)

/* Colors */
var(--color-primary)
var(--color-success)
var(--color-warning)
var(--color-danger)

/* Spacing */
var(--space-4)
var(--radius-lg)
```

## Migration Checklist

### Phase 1: Foundation
- [x] Create design token system
- [x] Create ThemeProvider
- [x] Create base CSS (theme.css)

### Phase 2: UI Primitives
- [x] Button component
- [x] Card component
- [x] Input/TextArea components
- [x] Badge component
- [x] Typography components

### Phase 3: Layout
- [x] Shell component
- [x] Sidebar component
- [x] Header component
- [x] MobileShell component

### Phase 4: Surfaces
- [x] SessionList component
- [x] MessageView component
- [x] Composer component
- [x] Timeline component

### Phase 5: Renderers
- [x] DiffRenderer component
- [x] MarkdownRenderer component
- [x] ToolRenderer component

### Phase 6: Routes
- [x] HomeSurface component
- [x] SessionSurface component
- [x] SettingsSurface component
- [x] InboxSurface component

### Phase 7: App Refactor
- [x] Create new App.tsx architecture
- [ ] Migrate existing state management
- [ ] Migrate existing data fetching
- [ ] Replace old App.tsx with new version

### Phase 8: Polish
- [ ] Add brand assets
- [ ] Verify mobile responsiveness
- [ ] Test theme switching
- [ ] Performance optimization

## Breaking Changes

1. **CSS Classes**: Old BEM-style classes replaced with CSS-in-JS
2. **Component Props**: New prop interfaces aligned with Happy
3. **Theme System**: CSS variables instead of class-based theming
4. **Layout**: New Shell-based layout system

## Backwards Compatibility

To maintain backwards compatibility during migration:

1. Keep old App.tsx as `App.legacy.tsx`
2. Import new components incrementally
3. Use feature flags to toggle between old/new UI
4. Test thoroughly before full migration

## Next Steps

1. Review `App.new.tsx` for the new architecture pattern
2. Migrate state management hooks from old App.tsx
3. Connect to actual data sources (API calls)
4. Add error boundaries and loading states
5. Implement remaining route surfaces
6. Add E2E tests for critical user flows

## Resources

- `ExampleHappyApp.tsx` - Working example of new components
- `App.new.tsx` - New App component architecture
- `src/components/` - All new components with TypeScript types
