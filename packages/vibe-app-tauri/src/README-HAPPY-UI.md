# Happy-Aligned UI Refactor

Complete redesign of vibe-app-tauri to pixel-perfect replicate Happy's (happy.ai) UI using pure Tauri v2 + React.

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

## Architecture

```
src/
├── design-system/          # Design tokens and theme
│   ├── tokens.ts          # TypeScript design tokens
│   └── theme.css          # CSS custom properties
├── components/
│   ├── providers/         # React context providers
│   │   └── ThemeProvider.tsx
│   ├── ui/               # Primitive components
│   │   ├── Button.tsx
│   │   ├── Card.tsx
│   │   ├── Input.tsx
│   │   ├── TextArea.tsx
│   │   ├── Badge.tsx
│   │   └── Typography.tsx
│   ├── layout/           # Layout components
│   │   ├── Shell.tsx
│   │   ├── Sidebar.tsx
│   │   ├── Header.tsx
│   │   └── MobileShell.tsx
│   ├── surfaces/         # Happy-specific surfaces
│   │   ├── SessionList.tsx
│   │   ├── MessageView.tsx
│   │   ├── Composer.tsx
│   │   └── Timeline.tsx
│   ├── renderers/        # Content renderers
│   │   ├── DiffRenderer.tsx
│   │   ├── MarkdownRenderer.tsx
│   │   └── ToolRenderer.tsx
│   └── routes/           # Route surfaces
│       ├── HomeSurface.tsx
│       ├── SessionSurface.tsx
│       ├── SettingsSurface.tsx
│       └── InboxSurface.tsx
├── AppV2.tsx             # New App component
├── feature-flags.ts      # Feature flag system
├── styles.new.css        # Integrated styles
└── MIGRATION.md          # Migration guide
```

## Components

### UI Primitives

```tsx
import { Button, Card, Input, Badge } from "./components/ui";

// Button variants
<Button variant="primary" size="md">Primary</Button>
<Button variant="secondary" loading={isLoading}>Secondary</Button>
<Button variant="ghost" disabled={!canSubmit}>Ghost</Button>

// Card with subcomponents
<Card variant="elevated">
  <CardHeader>
    <CardTitle>Title</CardTitle>
    <CardDescription>Description</CardDescription>
  </CardHeader>
  <CardContent>Content</CardContent>
  <CardFooter>Footer</CardFooter>
</Card>

// Input with label and error
<Input
  label="Email"
  placeholder="Enter email"
  error={errors.email}
  helperText="We'll never share your email"
/>

// Badge variants
<Badge variant="primary">New</Badge>
<Badge variant="success">Active</Badge>
<Badge variant="danger">Error</Badge>
```

### Layout

```tsx
import { Shell, Sidebar, Header, MobileShell } from "./components/layout";

// Desktop layout
<Shell
  sidebar={<Sidebar brand="Vibe" primarySections={[...]} />}
  header={<Header title="Dashboard" eyebrow="Home" />}
>
  <HomeSurface />
</Shell>

// Mobile layout
<MobileShell
  header={<MobileNavBar title="Vibe" />}
  tabs={[...]}
  activeTab="home"
  onTabChange={...}
>
  <HomeSurface />
</MobileShell>
```

### Surfaces

```tsx
import { SessionList, Timeline, Composer } from "./components/surfaces";

// Session list
<SessionList
  sessions={sessions}
  selectedId={selectedId}
  onSelect={handleSelect}
  loading={isLoading}
/>

// Timeline with composer
<div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
  <Timeline messages={messages} />
  <Composer
    value={value}
    onChange={setValue}
    onSubmit={handleSend}
    suggestions={suggestions}
    isSending={isSending}
  />
</div>
```

### Route Surfaces

```tsx
import { HomeSurface, SessionSurface, SettingsSurface, InboxSurface } from "./components/routes";

// Home with quick actions and stats
<HomeSurface
  recentSessions={sessions}
  quickActions={actions}
  stats={stats}
  onNewSession={createSession}
/>

// Session detail
<SessionSurface
  session={currentSession}
  messages={messages}
  composerValue={composerValue}
  onComposerChange={setComposerValue}
  onSendMessage={sendMessage}
  models={availableModels}
/>

// Settings
<SettingsSurface
  sections={settingSections}
  onSave={saveSettings}
  hasChanges={hasChanges}
/>

// Inbox
<InboxSurface
  notifications={notifications}
  onMarkAsRead={markRead}
  onDismiss={dismiss}
/>
```

### Content Renderers

```tsx
import { DiffRenderer, MarkdownRenderer, ToolRenderer } from "./components/renderers";

// Code diff
<DiffRenderer
  files={diffFiles}
  showLineNumbers={true}
  collapsible={true}
/>

// Markdown
<MarkdownRenderer
  content={markdown}
  onLinkClick={openExternal}
/>

// Tool calls
<ToolRenderer
  toolCalls={toolCalls}
  tools={toolMetadata}
  defaultExpanded={false}
/>
```

## Design Tokens

Access design tokens programmatically:

```tsx
import { tokens } from "./components";

// Colors
const primary = tokens.colors.primary.dark;
const success = tokens.colors.success.dark;

// Spacing (4px base grid)
const padding = tokens.spacing[4]; // 16px
const gap = tokens.spacing[3];     // 12px

// Typography
const fontSize = tokens.typography.fontSize.lg;
const fontWeight = tokens.typography.fontWeight.semibold;

// Border radius
const radius = tokens.radii.lg;
```

## CSS Custom Properties

All components use CSS custom properties for theming:

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
var(--space-1)  /* 4px */
var(--space-2)  /* 8px */
var(--space-4)  /* 16px */
```

## Theme Support

Automatic dark/light mode support:

```tsx
import { useTheme, useIsDark } from "./components";

function MyComponent() {
  const { theme, setColorScheme } = useTheme();
  const isDark = useIsDark();

  return (
    <div>
      <p>Current theme: {theme}</p>
      <button onClick={() => setColorScheme("dark")}>Dark</button>
      <button onClick={() => setColorScheme("light")}>Light</button>
      <button onClick={() => setColorScheme("system")}>System</button>
    </div>
  );
}
```

## Migration from Legacy

### Gradual Migration

1. Enable feature flags to test new UI:
   ```
   ?happy-ui=true&new-components=true&new-theme=true
   ```

2. Import individual components:
   ```tsx
   import { Button, Card } from "./components/ui";
   ```

3. Replace legacy components incrementally

4. Update styles to use new CSS variables

### Full Migration

See [MIGRATION.md](./MIGRATION.md) for detailed migration steps.

## File Size Comparison

| File | Before | After |
|------|--------|-------|
| App.tsx | 282KB | ~400 lines (AppV2.tsx) |
| styles.css | 1500 lines | Integrated with theme.css |
| Component count | Monolithic | 25+ modular components |

## Browser Support

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+
- iOS Safari 14+
- Android Chrome 88+

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
- [ ] Phase 11: Documentation
- [ ] Phase 12: Remove legacy code

## License

Same as parent project.
