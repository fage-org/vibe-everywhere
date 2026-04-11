# Capability Contract

> **Version**: 1.0  
> **Status**: Active (Wave 10)  
> **Applies to**: `packages/vibe-app-tauri`

This document defines the capability classification system used throughout Wave 10 to categorize app surfaces by their support level and evidence requirements.

---

## Overview

The Capability Contract establishes a clear, consistent framework for describing the completeness and support level of every visible surface in the app. It replaces vague "done" or "complete" labels with precise capability classes backed by evidence.

### Goals

1. **Eliminate ambiguity**: Every surface has a clear support classification
2. **Evidence-based claims**: Capabilities must be backed by code, tests, and documentation
3. **Platform transparency**: Support levels are defined per platform (desktop/Android/browser)
4. **Customer clarity**: External descriptions match internal reality

---

## Capability Classes

### Summary Table

| Class | Customer-Facing | Mutable | Description | Typical Evidence |
|-------|----------------|---------|-------------|------------------|
| **FullySupported** | ✅ | ✅ | Complete product feature | 80%+ coverage, all test types, full docs |
| **Limited** | ✅ | ✅ | Working core functionality | 60%+ coverage, unit+integration tests |
| **HandoffOnly** | ❌ | ❌ | UI delegates to external tools | 40%+ coverage, minimal docs |
| **ReadOnly** | ✅ | ❌ | View-only, no modifications | 60%+ coverage, read path tested |
| **Internal** | ❌ | ✅ | Developer/diagnostic only | 40%+ coverage, internal docs |
| **Unsupported** | ❌ | ❌ | Not maintained, may be removed | No requirements |

### Detailed Definitions

#### `FullySupported`

**Criteria:**
- Complete functionality matching product specifications
- 80%+ test coverage with unit, integration, and E2E tests
- Full customer-facing documentation
- Active maintenance and bug fix SLA
- Clear platform support matrix

**Evidence Required:**
- All test types present
- Coverage report showing 80%+
- Customer description
- Platform scope documented
- Code path documented
- State path documented

**Customer Message:**
> "This feature is fully supported across [platforms]. You can expect regular updates and prompt bug fixes."

---

#### `Limited`

**Criteria:**
- Core functionality works but edge cases may not be handled
- 60%+ test coverage with unit and integration tests
- Basic documentation with known limitations noted
- Best-effort maintenance

**Evidence Required:**
- Unit and integration tests
- Coverage report showing 60%+
- Customer description with limitations
- Platform scope documented
- Internal notes explaining limitations

**Customer Message:**
> "This feature provides core functionality with some limitations: [list]. Advanced use cases may require workarounds."

---

#### `HandoffOnly`

**Criteria:**
- UI exists but delegates to external tools or command-line
- No internal implementation of core logic
- Copy-to-clipboard or redirect only
- 40%+ test coverage for UI layer

**Evidence Required:**
- Unit tests for UI components
- Customer description explaining handoff
- Platform scope documented
- Internal notes explaining handoff rationale

**Customer Message:**
> "This feature provides a shortcut to [external tool]. Click to copy the command or open the external application."

---

#### `ReadOnly`

**Criteria:**
- Can view data but not create, update, or delete
- Read operations fully tested
- 60%+ test coverage
- No write operations exposed

**Evidence Required:**
- Unit and integration tests for read paths
- Coverage report showing 60%+
- Customer description
- Platform scope documented

**Customer Message:**
> "You can view [data type] but cannot make changes. Contact [role] if you need to modify this data."

---

#### `Internal`

**Criteria:**
- Not exposed to end users
- Developer or diagnostic tools only
- May lack polish or complete documentation
- 40%+ test coverage
- Access controlled

**Evidence Required:**
- Basic unit tests
- Internal notes explaining purpose
- Access control documentation

**Customer Message:**
> Not applicable (not customer-facing)

---

#### `Unsupported`

**Criteria:**
- Code may exist but is not maintained
- No testing or documentation
- May be removed in future releases
- No guarantees

**Evidence Required:**
- Internal notes explaining status

**Customer Message:**
> This feature is not available. Consider [alternative].

---

## Platform Scope

### Per-Platform Classification

Each surface can have platform-specific capability classes:

```typescript
interface PlatformScope {
  desktop: CapabilityClass;   // Windows/Mac/Linux
  android: CapabilityClass;   // Android app
  browser: CapabilityClass;   // Browser export
}
```

### Platform Downgrade Rules

1. **Platform downgrade allowed**: A platform-specific class can be lower than the base class
   - Example: `FullySupported` (base) → `Limited` (Android)

2. **Platform upgrade NOT allowed**: A platform-specific class cannot be higher than the base class
   - Example: `Internal` (base) cannot have `FullySupported` (desktop)

3. **Customer-facing constraint**: A non-customer-facing base class limits all platforms
   - Example: `Internal` (base) → all platforms are `Internal`

### Example Platform Matrix

| Surface | Base Class | Desktop | Android | Browser |
|---------|------------|---------|---------|---------|
| Session Composer | FullySupported | FullySupported | Limited | ReadOnly |
| Settings | FullySupported | FullySupported | FullySupported | HandoffOnly |
| Terminal | Limited | Limited | Unsupported | Unsupported |
| Dev Tools | Internal | Internal | N/A | N/A |

---

## Evidence Requirements by Class

### FullySupported Evidence Requirements

| Evidence Type | Requirement | Minimum Standard |
|---------------|-------------|------------------|
| **Test Coverage** | 80%+ | Coverage report |
| **Unit Tests** | Required | All core paths |
| **Integration Tests** | Required | API/state integration |
| **E2E Tests** | Required | Critical user flows |
| **Code Path** | Required | File paths, exports documented |
| **State Path** | Required | Store, schema, persistence |
| **Platform Scope** | Required | Per-platform classification |
| **Customer Description** | Required | Clear user-facing description |
| **Internal Notes** | Optional | Implementation notes |

### Limited Evidence Requirements

| Evidence Type | Requirement | Minimum Standard |
|---------------|-------------|------------------|
| **Test Coverage** | 60%+ | Coverage report |
| **Unit Tests** | Required | Core paths only |
| **Integration Tests** | Required | Basic integration |
| **E2E Tests** | Not required | - |
| **Code Path** | Required | File paths documented |
| **State Path** | Required | Store, schema |
| **Platform Scope** | Required | Per-platform classification |
| **Customer Description** | Required | With limitations noted |
| **Internal Notes** | Required | Limitations documented |

### Other Classes

See `EVIDENCE_REQUIREMENTS` in `src/capability/evidence.ts` for complete requirements for all classes.

---

## Usage Examples

### Defining a Surface Capability

```typescript
import {
  CapabilityClass,
  type SurfaceCapability,
} from './capability/types';

const sessionComposer: SurfaceCapability = {
  id: 'session-composer',
  name: 'Session Composer',
  route: '/session/:id/composer',
  capabilityClass: CapabilityClass.FullySupported,
  platformCapabilities: {
    desktop: CapabilityClass.FullySupported,
    android: CapabilityClass.Limited,
    browser: CapabilityClass.ReadOnly,
  },
  customerDescription: 'Compose and send messages to AI agents',
  internalNotes: 'Core product surface, high priority',
};
```

### Validating Evidence

```typescript
import { validateEvidence } from './capability/evidence';

const result = validateEvidence(evidence, surface.capabilityClass, surface);

if (!result.valid) {
  console.error('Validation errors:', result.errors);
}

if (result.warnings.length > 0) {
  console.warn('Validation warnings:', result.warnings);
}
```

### Generating Checklists

```typescript
import { generateEvidenceChecklist } from './capability/evidence';

const checklist = generateEvidenceChecklist(CapabilityClass.FullySupported);
console.log(checklist.join('\n'));
```

---

## Implementation Status

| Component | Status | Location |
|-----------|--------|----------|
| Core Types | ✅ Complete | `src/capability/types.ts` |
| Evidence System | ✅ Complete | `src/capability/evidence.ts` |
| Validation | ✅ Complete | `src/capability/evidence.ts` |
| Documentation | ✅ Complete | `docs/CAPABILITY-CONTRACT.md` |
| Surface Registry | 🔄 In Progress | `src/capability/*-registry.ts` |
| CI Integration | 🔄 Planned | Wave 10 implementation |

---

## Related Documents

- `docs/plans/rebuild/STATUS.md` - Current Wave 10 status
- `docs/plans/rebuild/wave10-master-plan.md` - Wave 10 master plan
- `src/capability/types.ts` - Core type definitions
- `src/capability/evidence.ts` - Evidence validation
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md` - Module plan

---

## Changelog

### 1.0 (2026-04-11)
- Initial capability contract definition
- Six capability classes defined
- Evidence requirements specified
- Platform scope rules established
- Usage examples provided
