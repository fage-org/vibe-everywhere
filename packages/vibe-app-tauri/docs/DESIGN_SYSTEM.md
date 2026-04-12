# Design System Documentation

## vibe-app-tauri Happy-Aligned UI

**Version:** 1.0  
**Last Updated:** 2026-04-10  
**Reference:** [Happy UI](https://happy.ai) - Pixel-perfect replication

---

## Documentation Index

| Document | Purpose |
|----------|---------|
| **DESIGN_SYSTEM.md** (this file) | Complete UI/UX design system specification |
| [MIGRATION.md](../src/MIGRATION.md) | Migration guide from legacy architecture |

---

## Table of Contents

1. [Design Principles & Visual Language](#1-design-principles--visual-language)
2. [Component Usage Guidelines](#2-component-usage-guidelines)
3. [Layout Patterns](#3-layout-patterns)
4. [Interaction Patterns](#4-interaction-patterns)
5. [Animation & Transitions](#5-animation--transitions)
6. [Content Presentation](#6-content-presentation)
7. [Accessibility Standards](#7-accessibility-standards)
8. [AI Agent Development Guidelines](#8-ai-agent-development-guidelines)
9. [Appendices](#appendix-a-token-reference-table)

---

## Quick Reference

- **Design Tokens:** `src/design-system/tokens.ts`
- **Theme CSS:** `src/design-system/theme.css`
- **UI Components:** `src/components/ui/`
- **Layout Components:** `src/components/layout/`
- **Surface Components:** `src/components/surfaces/`
- **Route Surfaces:** `src/components/routes/`
- **Content Renderers:** `src/components/renderers/`

---

## 1. Design Principles & Visual Language

### Core Design Philosophy

The vibe-app-tauri UI follows **Happy's design philosophy**:

- **Clarity over density**: Information is presented clearly with ample breathing room
- **Progressive disclosure**: Show essential information first, details on demand
- **Consistent patterns**: Reuse established patterns to minimize cognitive load
- **Responsive by default**: All components work across desktop, tablet, and mobile
- **Native feel**: Matches platform conventions (iOS-style on all platforms)

### Color System

Located in: `/packages/vibe-app-tauri/src/design-system/tokens.ts`

#### System Colors

| Color | Light Mode | Dark Mode | Hex Values |
|-------|-----------|-----------|------------|
| Red | `#ff3b30` | `#ff453a` | Danger, errors, destructive actions |
| Orange | `#ff9500` | `#ff9f0a` | Warnings, caution |
| Yellow | `#ffcc00` | `#ffd60a` | Cautions, highlights |
| Green | `#34c759` | `#30d158` | Success, positive states |
| Teal | `#5ac8fa` | `#64d2ff` | Info, neutral emphasis |
| Blue | `#007aff` | `#0a84ff` | Primary actions, links |
| Indigo | `#5856d6` | `#5e5ce6` | Secondary brand |
| Purple | `#af52de` | `#bf5af2` | Tertiary accent |
| Pink | `#ff2d55` | `#ff375f` | Special highlights |
| Brown | `#a2845e` | `#ac8e68` | Neutral emphasis |

#### Semantic Colors

```typescript
// Primary - Main action color
tokens.colors.primary.light  // "#007aff"
tokens.colors.primary.dark   // "#0a84ff"

// Success - Positive outcomes
tokens.colors.success.light  // "#34c759"
tokens.colors.success.dark   // "#30d158"

// Warning - Cautionary states
tokens.colors.warning.light  // "#ff9500"
tokens.colors.warning.dark   // "#ff9f0a"

// Danger - Errors and destructive actions
tokens.colors.danger.light   // "#ff3b30"
tokens.colors.danger.dark    // "#ff453a"

// Info - Neutral information
tokens.colors.info.light     // "#5ac8fa"
tokens.colors.info.dark      // "#64d2ff"
```

#### Gray Scale (iOS-Style)

```typescript
tokens.colors.gray[50]   // "#f2f2f7"  - Lightest
tokens.colors.gray[100]  // "#e5e5ea"
tokens.colors.gray[200]  // "#d1d1d6"
tokens.colors.gray[300]  // "#c7c7cc"
tokens.colors.gray[400]  // "#aeaeb2"
tokens.colors.gray[500]  // "#8e8e93"  - Mid gray
tokens.colors.gray[600]  // "#636366"
tokens.colors.gray[700]  // "#48484a"
tokens.colors.gray[800]  // "#3a3a3c"
tokens.colors.gray[900]  // "#2c2c2e"
tokens.colors.gray[950]  // "#1c1c1e"  - Darkest
```

#### Theme Tokens (CSS Variables)

**Dark Mode (Default):**
```css
--bg-primary: #000000
--bg-secondary: #1c1c1e
--bg-tertiary: #2c2c2e
--bg-elevated: #1c1c1e

--surface-primary: #1c1c1e
--surface-secondary: #2c2c2e
--surface-tertiary: #3a3a3c
--surface-elevated: #2c2c2e

--text-primary: #ffffff
--text-secondary: #ebebf5
--text-tertiary: #8e8e93
--text-quaternary: #636366
--text-placeholder: #8e8e93
--text-disabled: #636366

--border-primary: #38383a
--border-secondary: #48484a
--border-tertiary: #636366

--separator-primary: #38383a
--separator-secondary: #2c2c2e
```

### Typography System

Font Stack: SF Pro Display, -apple-system, BlinkMacSystemFont, Segoe UI, Roboto

| Style | Size | Weight | Line Height | Use Case |
|-------|------|--------|-------------|----------|
| LargeTitle | 34px | 700 | 1.2 | Main page titles |
| Title1 | 28px | 700 | 1.25 | Section headers |
| Title2 | 22px | 700 | 1.3 | Card titles |
| Title3 | 20px | 600 | 1.3 | Subsection headers |
| Headline | 17px | 600 | 1.3 | Navigation labels |
| Body | 17px | 400 | 1.4 | Primary content |
| Body Bold | 17px | 600 | 1.4 | Emphasized content |
| Callout | 16px | 400 | 1.4 | Secondary content |
| Subheadline | 15px | 400 | 1.4 | Metadata labels |
| Footnote | 13px | 400 | 1.4 | Helper text |
| Caption1 | 12px | 400 | 1.3 | Timestamps |
| Caption2 | 11px | 400 | 1.3 | Fine details |

```typescript
// Usage with components
import { LargeTitle, Title1, Body, Caption1 } from "./components/ui";

<LargeTitle>Page Title</LargeTitle>
<Body color="secondary">Secondary text</Body>
<Caption1 color="tertiary">Metadata</Caption1>
```

### Spacing System

Base grid: 4px (0.25rem)

```typescript
tokens.spacing[0.5]   // 2px   - Micro adjustments
tokens.spacing[1]     // 4px   - Tight packing
tokens.spacing[2]     // 8px   - Element spacing
tokens.spacing[3]     // 12px  - Component padding
tokens.spacing[4]     // 16px  - Standard padding
tokens.spacing[6]     // 24px  - Section padding
tokens.spacing[8]     // 32px  - Large gaps
tokens.spacing[12]    // 48px  - Major sections
```

### Border Radius

```typescript
tokens.radii.xs    // 4px   - Small elements
tokens.radii.sm    // 6px   - Buttons, inputs
tokens.radii.md    // 8px   - Cards, panels
tokens.radii.lg    // 10px  - Larger cards
tokens.radii.xl    // 12px  - Modals, sheets
tokens.radii["2xl"] // 16px - Large containers
tokens.radii.full  // 9999px - Pills, avatars
```

### Shadows

**iOS-Style Shadows:**
```typescript
tokens.shadows.ios.small   // "0 2px 8px rgba(0, 0, 0, 0.12)"
tokens.shadows.ios.medium  // "0 4px 16px rgba(0, 0, 0, 0.16)"
tokens.shadows.ios.large   // "0 8px 32px rgba(0, 0, 0, 0.2)"
```

---

## 2. Component Usage Guidelines

### Button

**File:** `/packages/vibe-app-tauri/src/components/ui/Button.tsx`

**Variants:**
- `primary` - Main actions (CTAs)
- `secondary` - Secondary actions
- `ghost` - Tertiary actions, icon buttons
- `danger` - Destructive actions
- `success` - Confirmatory actions

**Sizes:**
- `sm` - 28px height (compact UIs)
- `md` - 36px height (default)
- `lg` - 44px height (emphasized actions)

```tsx
import { Button } from "./components/ui";

// Primary CTA
<Button variant="primary" size="lg">Create Session</Button>

// Secondary action
<Button variant="secondary" loading={isLoading}>
  Save Changes
</Button>

// Icon button
<Button variant="ghost" size="sm" leadingIcon={<Icon />}>
  Settings
</Button>

// Destructive
<Button variant="danger" disabled={!canDelete}>
  Delete
</Button>
```

**When to Use:**
- Use **primary** for the most important action on a screen (limit to 1 per view)
- Use **secondary** for alternative actions
- Use **ghost** for icon buttons or subtle actions
- Use **danger** for delete, remove, or irreversible actions
- Use **success** for completion or confirmation actions

### Card

**File:** `/packages/vibe-app-tauri/src/components/ui/Card.tsx`

**Variants:**
- `default` - Standard bordered card
- `elevated` - Card with shadow
- `outlined` - Border only, transparent background
- `interactive` - Hover state for clickable cards

```tsx
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "./components/ui";

<Card variant="elevated" padding="lg">
  <CardHeader>
    <CardTitle>Session Overview</CardTitle>
    <CardDescription>Manage your active sessions</CardDescription>
  </CardHeader>
  <CardContent>{/* Content */}</CardContent>
  <CardFooter>
    <Button variant="secondary">Cancel</Button>
    <Button variant="primary">Save</Button>
  </CardFooter>
</Card>
```

**When to Use:**
- Use **elevated** for modal-like surfaces, floating content
- Use **default** for grouped content within a page
- Use **outlined** for subtle grouping, form sections
- Use **interactive** for clickable list items, navigation cards

### Input

**File:** `/packages/vibe-app-tauri/src/components/ui/Input.tsx`

```tsx
import { Input } from "./components/ui";

<Input
  label="Email Address"
  placeholder="Enter your email"
  helperText="We'll never share your email"
  error={errors.email}
  leadingElement={<EmailIcon />}
  trailingElement={<CheckIcon />}
  fullWidth
/>
```

**States:**
- Default: `--surface-secondary` background, `--border-primary` border
- Focus: Border changes to `--color-primary`
- Error: Border is `--color-danger`, error message displayed
- Disabled: Reduced opacity, `not-allowed` cursor

**Sizes:**
- `sm` - 32px height
- `md` - 44px height (default, matches iOS minimum touch target)
- `lg` - 52px height

### TextArea

**File:** `/packages/vibe-app-tauri/src/components/ui/TextArea.tsx`

```tsx
import { TextArea } from "./components/ui";

<TextArea
  label="Description"
  placeholder="Enter description..."
  minRows={3}
  maxRows={10}
  autoResize
  error={errors.description}
/>
```

**Features:**
- Auto-resize option grows with content
- Min/max row constraints
- Same styling as Input component

### Badge

**File:** `/packages/vibe-app-tauri/src/components/ui/Badge.tsx`

**Variants:**
- `default` - Neutral status
- `primary` - Highlighted status
- `success`, `warning`, `danger`, `info` - Semantic statuses
- `outline` - Subtle status
- `ghost` - Minimal status

```tsx
import { Badge } from "./components/ui";

<Badge variant="primary">New</Badge>
<Badge variant="success">Active</Badge>
<Badge variant="danger">Error</Badge>
<Badge variant="outline">Draft</Badge>
```

**When to Use:**
- Status indicators (Active, Pending, Completed)
- Count badges (notifications, unread)
- Tags and categories
- State labels (Draft, Published, Archived)

### Typography Components

**File:** `/packages/vibe-app-tauri/src/components/ui/Typography.tsx`

| Component | Size | Purpose |
|-----------|------|---------|
| `LargeTitle` | 34px | Hero text, welcome screens |
| `Title1` | 28px | Page titles |
| `Title2` | 22px | Section headers |
| `Title3` | 20px | Card titles, subsections |
| `Headline` | 17px | Navigation, emphasized |
| `Body` | 17px | Primary content text |
| `Callout` | 16px | Secondary descriptions |
| `Subheadline` | 15px | Labels, metadata |
| `Footnote` | 13px | Helper text, legal |
| `Caption1` | 12px | Timestamps, small labels |
| `Caption2` | 11px | Fine print |
| `Eyebrow` | 12px | Section labels (uppercase) |

**Color Props:**
- `color="primary"` - Main text
- `color="secondary"` - Secondary text
- `color="tertiary"` - Tertiary/muted
- `color="quaternary"` - Very subtle

**Utility Props:**
- `truncate` - Ellipsis overflow
- `lineClamp={3}` - Multi-line clamp
- `bold` - Bold weight (Body, Callout, Subheadline, Footnote only)

---

## 3. Layout Patterns

### Shell

**File:** `/packages/vibe-app-tauri/src/components/layout/Shell.tsx`

The Shell provides the top-level application structure with sidebar and main content areas.

```tsx
import { Shell, Sidebar, Header } from "./components/layout";

<Shell
  sidebar={<Sidebar brand="Vibe" primarySections={navSections} />}
  header={<Header title="Dashboard" eyebrow="Home" />}
  sidebarCollapsed={false}
>
  <HomeSurface />
</Shell>
```

**Structure:**
```
┌─────────────────────────────────────┐
│  Sidebar   │  Header               │
│  (260px)   ├───────────────────────┤
│            │                       │
│  Navigation│     Main Content      │
│            │    (scrollable)       │
│            │                       │
└─────────────────────────────────────┘
```

### Sidebar

**File:** `/packages/vibe-app-tauri/src/components/layout/Sidebar.tsx`

Navigation sidebar with brand, primary sections, secondary sections, and connection status.

```tsx
import { Sidebar } from "./components/layout";

<Sidebar
  brand={<Logo />}
  primarySections={[
    {
      title: "Navigation",
      items: [
        { id: "home", label: "Home", icon: <HomeIcon />, state: "active" },
        { id: "sessions", label: "Sessions", icon: <ChatIcon />, badge: 3 },
      ],
    },
  ]}
  secondarySections={[/* Bottom navigation */]}
  connectionStatus={<ConnectionBadge />}
/>
```

### Header

**File:** `/packages/vibe-app-tauri/src/components/layout/Header.tsx`

Route header with eyebrow, title, subtitle, and actions.

```tsx
import { Header, HeaderAction } from "./components/layout";

<Header
  eyebrow="Settings"
  title="Account Preferences"
  subtitle="Manage your account settings"
  size="default" // "large" | "default" | "compact"
  leading={<BackButton />}
  actions={
    <>
      <HeaderAction><SearchIcon /></HeaderAction>
      <HeaderAction><MenuIcon /></HeaderAction>
    </>
  }
/>
```

### MobileShell

**File:** `/packages/vibe-app-tauri/src/components/layout/MobileShell.tsx`

Mobile-specific layout with bottom tab bar.

```tsx
import { MobileShell, MobileNavBar } from "./components/layout";

<MobileShell
  header={<MobileNavBar title="Vibe" leading={<Back />} trailing={<Menu />} />}
  tabs={[
    { id: "home", icon: <HomeIcon />, activeIcon: <HomeFilledIcon />, label: "Home" },
    { id: "chat", icon: <ChatIcon />, label: "Chat", badge: 2 },
  ]}
  activeTab={currentTab}
  onTabChange={setTab}
>
  <Content />
</MobileShell>
```

### Responsive Behavior

**Breakpoints:**
- Mobile: < 768px
- Tablet: 768px - 1024px
- Desktop: > 1024px

**Mobile Adaptations:**
- Sidebar becomes drawer or hamburger menu
- Header title centers
- Bottom tab bar appears
- Touch targets minimum 44px
- Increased padding for thumb reach

---

## 4. Interaction Patterns

### Hover States

**Buttons:**
- Primary: Slight brightness increase (10%)
- Secondary: Background darkens to `--surface-tertiary`
- Ghost: Background becomes `--surface-secondary`

**Cards (Interactive):**
- Background: `--surface-primary` → `--surface-secondary`
- Border: `--border-primary` → `--border-secondary`

**Navigation Items:**
- Background: transparent → `--surface-secondary`
- Text: `--text-secondary` → `--text-primary`

### Focus States

All interactive elements use consistent focus indicators:

```css
:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
```

**Implementation:**
```tsx
<button style={{ outline: "none" }}> {/* Remove default */} </button>

// Or rely on global CSS
```

### Active/Selected States

**Navigation:**
- Background: `--surface-secondary`
- Text: `--text-primary`
- Badge: Background changes to `--color-primary`

**Tabs:**
- Text/icon color: `--color-primary`
- Optional underline or background pill

**List Items:**
- Similar to navigation, plus left border accent (optional)

### Disabled States

```typescript
opacity: 0.5;
cursor: not-allowed;
// Prevent all interaction events
```

**Buttons:**
- Reduced opacity (50%)
- `not-allowed` cursor
- No hover effects

**Inputs:**
- Reduced opacity
- No focus border color change

### Loading States

**Buttons:**
- Spinner replaces icon (or appears)
- Text remains
- Disabled interaction

**Skeletons:**
- Pulsing background animation
- Structured to match content shape
- Duration: 2s infinite

```typescript
const skeletonStyles = {
  backgroundColor: "var(--surface-secondary)",
  animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
};
```

### Error States

All user-triggered actions must surface failures in a visible, durable way.

**Rules:**
- Never let a tap or click fail silently.
- If an action can fail because of backend, permissions, network, clipboard, file system, or native bridge availability, the UI must show a visible error state.
- Error feedback must be rendered inline (`ErrorBanner`, helper text, field error, or persistent feedback card) unless the action is purely informational.
- Toasts/notifications may supplement errors, but they do **not** replace an on-screen error state.
- Async actions triggered from buttons must either:
  - catch and render errors locally, or
  - update a shared/global error state that is guaranteed to be rendered in the current shell.

**Preferred pattern:**
```tsx
const [error, setError] = useState<string | null>(null);
const [submitting, setSubmitting] = useState(false);

const handleSubmit = async () => {
  setSubmitting(true);
  setError(null);

  try {
    await action();
  } catch (submitError) {
    setError(
      submitError instanceof Error ? submitError.message : "Something went wrong",
    );
  } finally {
    setSubmitting(false);
  }
};

return (
  <>
    {error ? <ErrorBanner message={error} /> : null}
    <Button onClick={() => void handleSubmit()} disabled={submitting}>
      {submitting ? "Saving..." : "Save"}
    </Button>
  </>
);
```

**Avoid:**
```tsx
<Button onClick={() => void doAsyncThing()}>
  Save
</Button>
```

That pattern is only acceptable when `doAsyncThing()` is fully self-contained and guaranteed to write to a rendered error surface on failure.

---

## 5. Animation & Transitions

### Duration Guidelines

```typescript
tokens.animation.duration.instant  // 0ms   - Immediate response
tokens.animation.duration.fast     // 100ms - Micro-interactions
tokens.animation.duration.normal   // 200ms - Standard transitions
tokens.animation.duration.slow     // 300ms - Larger movements
tokens.animation.duration.slower   // 400ms - Complex animations
```

### Easing Functions

```typescript
tokens.animation.easing.default    // cubic-bezier(0.4, 0, 0.2, 1)  - Standard
tokens.animation.easing.in         // cubic-bezier(0.4, 0, 1, 1)    - Accelerate
tokens.animation.easing.out        // cubic-bezier(0, 0, 0.2, 1)    - Decelerate
tokens.animation.easing.inOut      // cubic-bezier(0.4, 0, 0.2, 1)  - Smooth
tokens.animation.easing.bounce     // cubic-bezier(0.68, -0.55, 0.265, 1.55) - Playful
tokens.animation.easing.ios        // cubic-bezier(0.32, 0.72, 0, 1) - iOS-style
```

### Common Transition Patterns

**Button Hover:**
```typescript
transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`
```

**Card Hover:**
```typescript
transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`
```

**Expand/Collapse:**
```typescript
transition: `transform ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`
// Transform: rotate(90deg) for chevrons
```

**Modal/Sidebar:**
```typescript
transition: `transform ${tokens.animation.duration.normal} ${tokens.animation.easing.ios},
            opacity ${tokens.animation.duration.normal} ${tokens.animation.easing.ios}`
```

**Input Focus:**
```typescript
transition: `border-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`
```

### Performance Considerations

**Always Animate:**
- `transform` (translate, scale, rotate)
- `opacity`

**Avoid Animating:**
- `width`, `height` (causes reflow)
- `top`, `left`, `right`, `bottom`
- `margin`, `padding`

**Use `will-change` Sparingly:**
```typescript
willChange: "transform, opacity"; // Only on animating elements
```

---

## 6. Content Presentation

### MessageView

**File:** `/packages/vibe-app-tauri/src/components/surfaces/MessageView.tsx`

Renders individual chat messages with proper styling for different roles.

```tsx
import { MessageView } from "./components/surfaces";

<MessageView
  message={{
    id: "msg-1",
    role: "assistant", // "user" | "assistant" | "system" | "tool"
    content: "Here's your summary...",
    timestamp: new Date(),
    senderName: "Assistant",
    isStreaming: false,
  }}
  showAvatar={true}
  isGrouped={false}
/>
```

**Message Roles:**
- **User**: Right-aligned, primary color bubble
- **Assistant**: Left-aligned, neutral surface
- **System**: Centered, subtle styling
- **Tool**: Collapsible tool call view

### Timeline

**File:** `/packages/vibe-app-tauri/src/components/surfaces/Timeline.tsx`

Scrollable message list with auto-scroll and date grouping.

```tsx
import { Timeline } from "./components/surfaces";

<Timeline
  messages={messages}
  loading={false}
  loadingMore={false}
  onLoadMore={fetchMore}
  emptyState={<EmptyState />}
  autoScroll={true}
/>
```

**Features:**
- Auto-scroll to bottom on new messages
- Message grouping (consecutive from same sender)
- Date separators ("Today", "Yesterday", "Monday, Jan 15")
- Infinite scroll support
- Skeleton loading state

### Composer

**File:** `/packages/vibe-app-tauri/src/components/surfaces/Composer.tsx`

Message input with auto-resize and suggestions.

```tsx
import { Composer } from "./components/surfaces";

<Composer
  value={inputValue}
  onChange={setInputValue}
  onSubmit={handleSend}
  placeholder="Message..."
  suggestions={[
    { id: "1", label: "Summarize", insertText: "/summarize" },
  ]}
  onSuggestionSelect={handleSuggestion}
  isSending={false}
  toolbar={<AttachmentButton />}
  footer={<ModelSelector />}
/>
```

**Keyboard Shortcuts:**
- `Cmd/Ctrl + Enter` - Send message
- `Arrow Up/Down` - Navigate suggestions
- `Enter` - Select suggestion
- `Escape` - Close suggestions

### SessionList

**File:** `/packages/vibe-app-tauri/src/components/surfaces/SessionList.tsx`

List of sessions with status and metadata.

```tsx
import { SessionList } from "./components/surfaces";

<SessionList
  sessions={sessions}
  selectedId={currentSessionId}
  onSelect={handleSelect}
  onArchive={handleArchive}
  loading={false}
  emptyState={<NoSessions />}
/>
```

### Content Renderers

**MarkdownRenderer** (`/packages/vibe-app-tauri/src/components/renderers/MarkdownRenderer.tsx`):
- Headings (H1-H6)
- Paragraphs with proper spacing
- Code blocks with language labels
- Inline code
- Lists (ordered/unordered)
- Blockquotes
- Links (external handling)
- Horizontal rules

**DiffRenderer** (`/packages/vibe-app-tauri/src/components/renderers/DiffRenderer.tsx`):
- Unified diff view
- Line numbers (optional)
- Syntax highlighting
- File status badges
- Collapsible sections
- Add/remove coloring

**ToolRenderer** (`/packages/vibe-app-tauri/src/components/renderers/ToolRenderer.tsx`):
- Tool call visualization
- Arguments display
- Result/error presentation
- Execution timing
- Status indicators
- Copy-to-clipboard

---

## 7. Accessibility Standards

### Color Contrast

All text must meet WCAG 2.1 AA standards:
- Normal text: 4.5:1 minimum
- Large text (18px+ bold or 24px+): 3:1 minimum
- UI components: 3:1 minimum

**Current System Compliance:**
- `--text-primary` on `--bg-primary`: 21:1 (exceeds)
- `--text-secondary` on `--bg-primary`: 12:1 (exceeds)
- `--text-tertiary` on `--bg-primary`: 4.6:1 (passes AA)

### Focus Management

**Implementation:**
```tsx
// Always visible focus indicator
:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

// Skip link for keyboard users
<a href="#main-content" className="skip-link">Skip to main content</a>
```

**Focus Order:**
1. Logical DOM order
2. Top to bottom, left to right
3. Modal traps focus within
4. Escape returns focus to trigger

### ARIA Patterns

**Buttons:**
```tsx
<button
  aria-label="Close dialog"
  aria-pressed={isActive}
  aria-disabled={isDisabled}
  aria-busy={isLoading}
>
```

**Navigation:**
```tsx
<nav aria-label="Main navigation">
  <ul role="menubar">
    <li role="none">
      <a role="menuitem" aria-current="page">Home</a>
    </li>
  </ul>
</nav>
```

**Dialogs:**
```tsx
<div
  role="dialog"
  aria-modal="true"
  aria-labelledby="dialog-title"
  aria-describedby="dialog-description"
>
```

**Live Regions:**
```tsx
<div aria-live="polite" aria-atomic="true">{statusMessage}</div>
<div aria-live="assertive">{errorMessage}</div>
```

### Keyboard Navigation

**Global Shortcuts:**
- `Tab` / `Shift+Tab` - Navigate between focusable elements
- `Enter` / `Space` - Activate buttons
- `Escape` - Close modals, menus, escape current context
- `Cmd/Ctrl + K` - Command palette (if implemented)

**List Navigation:**
- `Arrow Up/Down` - Navigate list items
- `Enter` - Select item
- `Home` / `End` - First/last item

**Composer Shortcuts:**
- `Cmd/Ctrl + Enter` - Send
- `Shift + Enter` - New line

### Touch Targets

Minimum touch target: **44x44px** (iOS HIG)

**Implementation:**
```typescript
minWidth: "44px",
minHeight: "44px",
// Or use component tokens
tokens.layout.touchTarget.min  // 44px
tokens.layout.touchTarget.comfortable  // 48px
```

### Screen Reader Considerations

**Images:**
```tsx
<img src={src} alt="Descriptive text" />
// Or for decorative:
<img src={src} alt="" />
```

**Icons:**
```tsx
// Icon has accessible label
<svg aria-label="Settings" role="img">...</svg>

// Or hide from screen readers
<svg aria-hidden="true">...</svg>
```

**Loading States:**
```tsx
<div aria-busy="true" aria-live="polite">
  <span className="visually-hidden">Loading sessions...</span>
  <LoadingSpinner aria-hidden="true" />
</div>
```

---

## 8. AI Agent Development Guidelines

### How to Choose the Right Component

**Decision Tree:**

1. **Is it the primary action on the screen?**
   - Yes → Button `variant="primary"` `size="lg"`
   - No → Continue

2. **Is it a secondary action?**
   - Yes → Button `variant="secondary"`
   - No → Continue

3. **Is it an icon-only button or subtle action?**
   - Yes → Button `variant="ghost"`
   - No → Continue

4. **Does it navigate to a new page?**
   - Yes → Use React Router `<Link>` styled as Button
   - No → Continue

5. **Is it destructive (delete, remove)?**
   - Yes → Button `variant="danger"`

### Common UI Patterns for New Features

#### Creating a New Page

```tsx
// 1. Create route surface
// File: src/components/routes/MyFeatureSurface.tsx

import { Shell, Header, Sidebar } from "../layout";
import { Title3, Body } from "../ui";

export function MyFeatureSurface() {
  return (
    <div style={{ padding: tokens.spacing[6] }}>
      <Title3>My Feature</Title3>
      <Body color="secondary">Description here</Body>
      
      <Card variant="elevated" style={{ marginTop: tokens.spacing[4] }}>
        {/* Content */}
      </Card>
    </div>
  );
}

// 2. Add to router
// In AppV2.tsx or routing config
```

#### Creating a Form

```tsx
import { Card, CardHeader, CardTitle, CardContent } from "../ui/Card";
import { Input, TextArea } from "../ui";
import { Button } from "../ui/Button";

<Card>
  <CardHeader>
    <CardTitle>Create New</CardTitle>
  </CardHeader>
  <CardContent style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
    <Input
      label="Name"
      value={name}
      onChange={(e) => setName(e.target.value)}
      error={errors.name}
    />
    <TextArea
      label="Description"
      value={description}
      onChange={(e) => setDescription(e.target.value)}
      minRows={3}
    />
    <div style={{ display: "flex", gap: tokens.spacing[2], justifyContent: "flex-end" }}>
      <Button variant="ghost" onClick={onCancel}>Cancel</Button>
      <Button variant="primary" onClick={onSubmit}>Create</Button>
    </div>
  </CardContent>
</Card>
```

#### Creating a List

```tsx
import { Card } from "../ui";

<div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[2] }}>
  {items.map((item) => (
    <Card
      key={item.id}
      variant="interactive"
      padding="md"
      onClick={() => handleSelect(item)}
    >
      <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
        <Body bold>{item.title}</Body>
        <Caption1 color="tertiary">{item.timestamp}</Caption1>
      </div>
    </Card>
  ))}
</div>
```

#### Creating Modal/Dialog

```tsx
// Use elevated card with overlay
<div
  style={{
    position: "fixed",
    inset: 0,
    backgroundColor: "var(--overlay-thick)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    zIndex: tokens.zIndex.modal,
  }}
>
  <Card variant="elevated" padding="lg" style={{ width: "100%", maxWidth: "500px" }}>
    <CardHeader>
      <CardTitle>Confirm Action</CardTitle>
    </CardHeader>
    <CardContent>
      <Body>Are you sure you want to proceed?</Body>
    </CardContent>
    <CardFooter>
      <Button variant="ghost" onClick={onCancel}>Cancel</Button>
      <Button variant="danger" onClick={onConfirm}>Delete</Button>
    </CardFooter>
  </Card>
</div>
```

### Do's and Don'ts

#### DO

✅ **Use design tokens for all values:**
```tsx
// Good
<button style={{ padding: tokens.spacing[3] }}>

// Avoid
<button style={{ padding: "12px" }}>
```

✅ **Use semantic color variables:**
```tsx
// Good
color: "var(--text-primary)"
backgroundColor: "var(--surface-secondary)"

// Avoid
color: "#ffffff"
backgroundColor: "#1c1c1e"
```

✅ **Handle loading and error states:**
```tsx
{loading && <LoadingSkeleton />}
{error && <ErrorMessage error={error} />}
{data && <Content data={data} />}
```

✅ **Wrap async button actions with visible failure handling:**
```tsx
const [error, setError] = useState<string | null>(null);

const handleRefresh = async () => {
  try {
    await refreshSessions();
  } catch (refreshError) {
    setError(
      refreshError instanceof Error ? refreshError.message : "Failed to refresh sessions",
    );
  }
};

{error ? <ErrorBanner message={error} /> : null}
<Button variant="secondary" onClick={() => void handleRefresh()}>
  Refresh sessions
</Button>
```

✅ **Ensure global async errors are actually rendered in every shell:**
```tsx
{appState.globalError ? <ErrorBanner message={appState.globalError} /> : null}
```

If a mobile shell and desktop shell share the same async state source, both shells must render the same global error surface.

✅ **Use Typography components for text:**
```tsx
// Good
<Body color="secondary">Content</Body>

// Avoid
<p style={{ fontSize: "17px", color: "#ebebf5" }}>Content</p>
```

✅ **Provide accessible labels:**
```tsx
// Good
<button aria-label="Close dialog">×</button>

// Avoid
<button>×</button>
```

#### DON'T

❌ **Hardcode colors:**
```tsx
// NEVER DO THIS
<button style={{ backgroundColor: "#007aff" }}>
```

❌ **Ignore touch targets:**
```tsx
// Too small
<button style={{ width: "24px", height: "24px" }}>

// Minimum 44px
<button style={{ minWidth: "44px", minHeight: "44px" }}>
```

❌ **Mix spacing values:**
```tsx
// Inconsistent
<div style={{ padding: "10px 16px 12px" }}>

// Consistent (use tokens)
<div style={{ padding: `${tokens.spacing[2.5]} ${tokens.spacing[4]}` }}>
```

❌ **Use px for font sizes:**
```tsx
// Avoid
fontSize: "16px"

// Use rem-based tokens
fontSize: tokens.typography.fontSize.base  // 0.9375rem = 15px
```

❌ **Forget focus states:**
```tsx
// Always ensure focus is visible
// Don't use:
<button style={{ outline: "none" }}>

// Instead rely on global :focus-visible or add explicit focus styles
```

❌ **Fire-and-forget async actions from interactive controls without error UI:**
```tsx
// Avoid unless the callee guarantees rendered feedback
<button onClick={() => void refreshSessions()}>
  Refresh
</button>
```

If the action can fail, route the error to a visible local or global surface.

### Code Examples

#### Complete Component Example

```tsx
// packages/vibe-app-tauri/src/components/surfaces/DashboardSurface.tsx

import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "../ui/Card";
import { Button } from "../ui/Button";
import { Title2, Body, Caption1 } from "../ui/Typography";

interface StatCardProps {
  title: string;
  value: string | number;
  change?: string;
  icon?: ReactNode;
}

function StatCard({ title, value, change, icon }: StatCardProps) {
  return (
    <Card variant="default" padding="lg">
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between" }}>
        <div>
          <Caption1 color="tertiary">{title}</Caption1>
          <Title2 style={{ marginTop: tokens.spacing[1] }}>{value}</Title2>
          {change && (
            <Caption1 color="success" style={{ marginTop: tokens.spacing[1] }}>
              {change}
            </Caption1>
          )}
        </div>
        {icon && (
          <div style={{ 
            padding: tokens.spacing[2],
            backgroundColor: "var(--surface-secondary)",
            borderRadius: tokens.radii.md,
          }}>
            {icon}
          </div>
        )}
      </div>
    </Card>
  );
}

export function DashboardSurface() {
  return (
    <div style={{ padding: tokens.spacing[6] }}>
      <div style={{ marginBottom: tokens.spacing[6] }}>
        <Title2>Dashboard</Title2>
        <Body color="secondary">Overview of your activity</Body>
      </div>

      <div style={{ 
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))",
        gap: tokens.spacing[4],
      }}>
        <StatCard 
          title="Active Sessions" 
          value="12" 
          change="+3 this week"
          icon={<SessionsIcon />}
        />
        <StatCard 
          title="Messages" 
          value="1,234" 
          change="+12% from last month"
          icon={<MessagesIcon />}
        />
        <StatCard 
          title="Agents" 
          value="5" 
          icon={<AgentsIcon />}
        />
      </div>

      <Card variant="elevated" style={{ marginTop: tokens.spacing[6] }}>
        <CardHeader>
          <div>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>Your latest interactions</CardDescription>
          </div>
          <Button variant="secondary" size="sm">View All</Button>
        </CardHeader>
        <CardContent>
          {/* Activity list */}
        </CardContent>
      </Card>
    </div>
  );
}
```

#### Custom Hook Example

```tsx
// packages/vibe-app-tauri/src/hooks/useConfirm.ts

import { useState, useCallback } from "react";

interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "primary";
}

export function useConfirm() {
  const [isOpen, setIsOpen] = useState(false);
  const [options, setOptions] = useState<ConfirmOptions | null>(null);
  const [resolveRef, setResolveRef] = useState<((value: boolean) => void) | null>(null);

  const confirm = useCallback((opts: ConfirmOptions): Promise<boolean> => {
    setOptions(opts);
    setIsOpen(true);
    return new Promise((resolve) => {
      setResolveRef(() => resolve);
    });
  }, []);

  const handleConfirm = useCallback(() => {
    setIsOpen(false);
    resolveRef?.(true);
    setResolveRef(null);
  }, [resolveRef]);

  const handleCancel = useCallback(() => {
    setIsOpen(false);
    resolveRef?.(false);
    setResolveRef(null);
  }, [resolveRef]);

  return {
    confirm,
    isOpen,
    options,
    handleConfirm,
    handleCancel,
  };
}

// Usage
function DeleteButton({ itemId }: { itemId: string }) {
  const { confirm, isOpen, options, handleConfirm, handleCancel } = useConfirm();

  const handleDelete = async () => {
    const confirmed = await confirm({
      title: "Delete Item",
      message: "Are you sure you want to delete this item? This action cannot be undone.",
      confirmLabel: "Delete",
      variant: "danger",
    });

    if (confirmed) {
      await deleteItem(itemId);
    }
  };

  return (
    <>
      <Button variant="ghost" onClick={handleDelete}>
        Delete
      </Button>

      {isOpen && options && (
        <ConfirmDialog
          {...options}
          onConfirm={handleConfirm}
          onCancel={handleCancel}
        />
      )}
    </>
  );
}
```

### File Organization

When adding new features, follow this structure:

```
src/
├── components/
│   ├── ui/                    # Primitive components (buttons, inputs)
│   ├── layout/                # Layout components (shell, sidebar)
│   ├── surfaces/              # Feature surfaces (composer, timeline)
│   ├── routes/                # Route-level surfaces
│   └── renderers/             # Content renderers (markdown, code)
├── hooks/                     # Custom React hooks
├── utils/                     # Utility functions
└── types/                     # TypeScript types
```

### Quick Reference Card

| Pattern | Component | Token |
|---------|-----------|-------|
| Primary button | `<Button variant="primary">` | `tokens.colors.primary` |
| Secondary text | `<Body color="secondary">` | `var(--text-secondary)` |
| Card container | `<Card variant="elevated">` | `tokens.radii.lg` |
| Section spacing | - | `tokens.spacing[6]` |
| Element spacing | - | `tokens.spacing[3]` |
| Page title | `<Title2>` | `tokens.typography.textStyle.title2` |
| Standard transition | - | `tokens.animation.duration.fast` |
| iOS easing | - | `tokens.animation.easing.ios` |

---

## Appendix A: Token Reference Table

### Colors

| Token | Dark | Light | Usage |
|-------|------|-------|-------|
| `--bg-primary` | `#000000` | `#f2f2f7` | Page background |
| `--bg-secondary` | `#1c1c1e` | `#ffffff` | Cards, elevated surfaces |
| `--surface-primary` | `#1c1c1e` | `#ffffff` | Component backgrounds |
| `--surface-secondary` | `#2c2c2e` | `#f2f2f7` | Input backgrounds |
| `--text-primary` | `#ffffff` | `#000000` | Primary text |
| `--text-secondary` | `#ebebf5` | `#3c3c43` | Secondary text |
| `--text-tertiary` | `#8e8e93` | `#8e8e93` | Muted text |
| `--border-primary` | `#38383a` | `#c7c7cc` | Default borders |
| `--color-primary` | `#0a84ff` | `#007aff` | Primary accent |

### Spacing

| Token | Value | Usage |
|-------|-------|-------|
| `--space-1` | 4px | Tight packing |
| `--space-2` | 8px | Element spacing |
| `--space-3` | 12px | Component padding |
| `--space-4` | 16px | Standard padding |
| `--space-6` | 24px | Section padding |
| `--space-8` | 32px | Large gaps |

### Typography

| Token | Value | CSS Variable |
|-------|-------|--------------|
| `fontSize.base` | 0.9375rem | `--font-size-base` |
| `fontSize.lg` | 1.0625rem | `--font-size-lg` |
| `fontWeight.medium` | 500 | `--font-weight-medium` |
| `fontWeight.semibold` | 600 | `--font-weight-semibold` |
| `lineHeight.normal` | 1.4 | `--line-height-normal` |

---

## Appendix B: Migration from Legacy

When migrating existing components:

1. **Replace hardcoded colors** with CSS variables or tokens
2. **Replace hardcoded spacing** with token values
3. **Replace custom buttons** with `<Button>` component
4. **Replace inline styles** with Typography components
5. **Wrap screens** with Shell/Header/Sidebar components
6. **Test both themes** - always check dark and light mode

---

## Appendix C: Browser Support

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+
- iOS Safari 14+
- Android Chrome 88+

All components use modern CSS features with fallbacks where needed.

---

*End of Design System Documentation*
