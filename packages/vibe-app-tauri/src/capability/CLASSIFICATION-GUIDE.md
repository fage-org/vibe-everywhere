# Capability Classification Guide

> **Version:** 1.0.0  
> **Scope:** Wave 10 - `packages/vibe-app-tauri`  
> **Purpose:** Define how to classify app surfaces using the Capability Classification System

---

## Quick Start

```typescript
import {
  CapabilityClass,
  validateEvidence,
  type SurfaceCapability,
  type CapabilityEvidence,
} from '@/capability';

// 1. Define your surface
const surface: SurfaceCapability = {
  id: 'settings-account',
  name: 'Account Settings',
  route: '/settings/account',
  capabilityClass: CapabilityClass.FullySupported,
  customerDescription: 'Manage your account settings and preferences',
};

// 2. Provide evidence
const evidence: CapabilityEvidence = {
  codePath: {
    files: ['src/settings/AccountSettings.tsx'],
    exports: ['AccountSettings'],
    linesOfCode: 150,
  },
  statePath: {
    store: 'settingsStore',
    schema: 'AccountSettingsSchema',
    persistence: 'localStorage',
  },
  tests: {
    unit: true,
    integration: true,
    e2e: true,
    coverage: 85,
  },
  platformScope: {
    desktop: CapabilityClass.FullySupported,
    android: CapabilityClass.Limited,
    browser: CapabilityClass.ReadOnly,
  },
};

// 3. Validate
const result = validateEvidence(evidence, surface.capabilityClass, surface);
console.log(result.valid); // true or false
console.log(result.errors);  // array of error messages
```

---

## Capability Classes

### Overview

| Class | Customer-Facing | Mutable | Description |
|-------|---------------|---------|-------------|
| `FullySupported` | ✅ | ✅ | Complete functionality, full support |
| `Limited` | ✅ | ✅ | Core functionality, some limitations |
| `HandoffOnly` | ❌ | ❌ | UI delegates to external tools |
| `ReadOnly` | ✅ | ❌ | View-only, no modifications |
| `Internal` | ❌ | ❌ | Developer/diagnostic tools only |
| `Unsupported` | ❌ | ❌ | Code exists but not maintained |

### Decision Tree

```
Is the surface customer-facing?
├── NO → Is it for developers/internal use?
│       ├── YES → Can users still access it accidentally?
│       │       ├── YES → Internal + clear warnings
│       │       └── NO → Internal
│       └── NO → Is the code maintained?
│               ├── YES → HandoffOnly (if no functionality)
│               └── NO → Unsupported
│
└── YES → Can users modify data through this surface?
        ├── NO → ReadOnly
        └── YES → Is all functionality implemented?
                ├── YES → FullySupported
                └── NO → Limited (document limitations)
```

### Examples by Class

#### FullySupported

**Examples:**
- Account Settings (all fields editable)
- Session composer (full functionality)
- Inbox (read, mark as read, delete)

**Requirements:**
- 80%+ test coverage
- Unit, integration, and E2E tests
- Complete code and state path documentation
- Customer-facing description
- Platform scope defined for all platforms

#### Limited

**Examples:**
- Android Settings (fewer options than desktop)
- Browser Export (view-only, no editing)
- Machine List (view only, no actions)

**Requirements:**
- 60%+ test coverage
- Unit and integration tests
- Code and state path documentation
- Customer-facing description with limitations noted
- Internal notes explaining limitations

#### HandoffOnly

**Examples:**
- Terminal Connect (opens external terminal)
- Vendor Setup (shows copy-paste commands)
- GitHub Integration (links to GitHub settings)

**Requirements:**
- 40%+ test coverage
- Unit tests
- Code path documentation
- Clear customer description explaining the handoff
- Internal notes explaining why handoff is used

#### ReadOnly

**Examples:**
- Usage Statistics (view only)
- Session History (view only, no editing)
- System Status (view only)

**Requirements:**
- 60%+ test coverage
- Unit and integration tests
- Code and state path documentation
- Customer-facing description

#### Internal

**Examples:**
- Debug Panel (developer tools)
- State Inspector (internal diagnostics)
- Test Harness (internal testing)

**Requirements:**
- 40%+ test coverage
- Unit tests
- Code path documentation
- Internal notes explaining purpose

#### Unsupported

**Examples:**
- Legacy Routes (kept for compatibility)
- Experimental Features (not maintained)
- Deprecated Surfaces (being removed)

**Requirements:**
- Internal notes explaining why unsupported
- No tests, documentation, or customer-facing content required

---

## Evidence Requirements by Class

### Test Requirements

| Class | Min Coverage | Unit | Integration | E2E |
|-------|-------------|------|-------------|-----|
| FullySupported | 80% | ✅ | ✅ | ✅ |
| Limited | 60% | ✅ | ✅ | ❌ |
| HandoffOnly | 40% | ✅ | ❌ | ❌ |
| ReadOnly | 60% | ✅ | ✅ | ❌ |
| Internal | 40% | ✅ | ❌ | ❌ |
| Unsupported | 0% | ❌ | ❌ | ❌ |

### Documentation Requirements

| Class | Code Path | State Path | Platform Scope | Customer Desc | Internal Notes |
|-------|-----------|------------|----------------|---------------|----------------|
| FullySupported | ✅ | ✅ | ✅ | ✅ | ❌ |
| Limited | ✅ | ✅ | ✅ | ✅ | ✅ |
| HandoffOnly | ✅ | ❌ | ✅ | ✅ | ✅ |
| ReadOnly | ✅ | ✅ | ✅ | ✅ | ❌ |
| Internal | ✅ | ❌ | ❌ | ❌ | ✅ |
| Unsupported | ❌ | ❌ | ❌ | ❌ | ✅ |

---

## Platform Scope

### Definition

Platform scope defines the capability class for each platform (desktop, Android, browser). This allows a surface to have different support levels on different platforms.

### Rules

1. **Base class is the ceiling**: Platform-specific classes cannot upgrade a non-customer-facing base class to customer-facing.
2. **Downgrades allowed**: Platform-specific classes can downgrade from customer-facing to non-customer-facing.
3. **Inheritance**: If a platform is not specified, the base class is used.

### Example

```typescript
const surface: SurfaceCapability = {
  id: 'settings-account',
  name: 'Account Settings',
  capabilityClass: CapabilityClass.FullySupported, // Desktop is fully supported
  platformCapabilities: {
    android: CapabilityClass.Limited,      // Android has limited features
    browser: CapabilityClass.ReadOnly,     // Browser is view-only
  },
};

// Desktop: FullySupported (from base class)
// Android: Limited (from platformCapabilities)
// Browser: ReadOnly (from platformCapabilities)
```

---

## Best Practices

### DO

- **Be honest about capability**: Don't overstate the capability of a surface.
- **Document limitations**: For Limited surfaces, clearly document what's not supported.
- **Use platform scope**: If a surface works differently on different platforms, use platformCapabilities.
- **Provide evidence**: Always provide evidence to support capability claims.
- **Review regularly**: Re-evaluate capability classes as the product evolves.

### DON'T

- **Don't claim FullSupport without evidence**: FullSupport requires 80% coverage and all test types.
- **Don't use Internal for customer-facing surfaces**: Internal is for developer tools only.
- **Don't leave platformCapabilities empty**: Define the capability for all platforms.
- **Don't skip customerDescription for customer-facing surfaces**: Always explain what the surface does.

---

## Migration Guide

### From Wave 9 to Wave 10

If you have existing surfaces without capability classification:

1. **Audit existing surfaces**: Review all existing routes and surfaces.
2. **Assign initial classes**: Use the decision tree to assign an initial capability class.
3. **Collect evidence**: Gather evidence for each surface.
4. **Validate**: Use `validateEvidence()` to check if the evidence supports the class.
5. **Iterate**: Adjust the capability class or collect more evidence as needed.

### Example Migration

```typescript
// Before (Wave 9)
const AccountSettings = () => { /* component */ };

// After (Wave 10)
import {
  CapabilityClass,
  validateEvidence,
  type SurfaceCapability,
  type CapabilityEvidence,
} from '@/capability';

const surface: SurfaceCapability = {
  id: 'settings-account',
  name: 'Account Settings',
  route: '/settings/account',
  capabilityClass: CapabilityClass.FullySupported,
  customerDescription: 'Manage your account settings and preferences',
};

const evidence: CapabilityEvidence = {
  codePath: {
    files: ['src/settings/AccountSettings.tsx'],
    exports: ['AccountSettings'],
    linesOfCode: 150,
  },
  statePath: {
    store: 'settingsStore',
    schema: 'AccountSettingsSchema',
    persistence: 'localStorage',
  },
  tests: {
    unit: true,
    integration: true,
    e2e: true,
    coverage: 85,
  },
  platformScope: {
    desktop: CapabilityClass.FullySupported,
    android: CapabilityClass.Limited,
    browser: CapabilityClass.ReadOnly,
  },
};

const result = validateEvidence(evidence, surface.capabilityClass, surface);
console.log('Valid:', result.valid);
```

---

## Appendix

### A. Capability Class Summary

| Class | Customer | Mutable | Test Coverage | Tests | Documentation |
|-------|----------|---------|---------------|-------|---------------|
| FullySupported | ✅ | ✅ | 80%+ | Unit, Integration, E2E | Full |
| Limited | ✅ | ✅ | 60%+ | Unit, Integration | Full + Limitations |
| HandoffOnly | ❌ | ❌ | 40%+ | Unit | Basic |
| ReadOnly | ✅ | ❌ | 60%+ | Unit, Integration | Full |
| Internal | ❌ | ❌ | 40%+ | Unit | Minimal + Internal Notes |
| Unsupported | ❌ | ❌ | 0% | None | Internal Notes Only |

### B. Evidence Checklist Template

```markdown
## Evidence for {SurfaceName}

### Testing
- [ ] Unit tests
- [ ] Integration tests
- [ ] E2E tests (if required)
- [ ] Coverage >= {minCoverage}%

### Documentation
- [ ] Code path documented
- [ ] State path documented (if required)
- [ ] Platform scope defined (if required)
- [ ] Customer description provided (if required)
- [ ] Internal notes added (if required)
```

### C. Common Patterns

**Pattern 1: Desktop-First with Platform Variants**
```typescript
capabilityClass: CapabilityClass.FullySupported, // Desktop
platformCapabilities: {
  android: CapabilityClass.Limited,
  browser: CapabilityClass.ReadOnly,
}
```

**Pattern 2: Handoff-Only with Platform Differences**
```typescript
capabilityClass: CapabilityClass.HandoffOnly,
platformCapabilities: {
  android: CapabilityClass.Unsupported, // Not available on Android
}
```

**Pattern 3: Internal Tool with No Platform Support**
```typescript
capabilityClass: CapabilityClass.Internal,
platformCapabilities: {
  desktop: CapabilityClass.Internal,
  android: CapabilityClass.Unsupported,
  browser: CapabilityClass.Unsupported,
}
```

---

## Changelog

### 1.0.0 (2026-04-11)
- Initial release of Capability Classification System
- Six capability classes defined
- Evidence validation system implemented
- Platform scope support added
- Complete test coverage

---

## License

This module is part of the `vibe-remote` project and follows the same license terms.
