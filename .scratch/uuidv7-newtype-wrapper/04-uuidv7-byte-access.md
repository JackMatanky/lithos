---
title: 04-uuidv7-byte-access
category: enhancement
label: ready-for-human
status: completed
date_created: 2026-05-09
date_completed: 2026-05-09
---

# UUIDv7 Byte Access and Conversion

## Parent

UUIDv7 Hardening PRD

## What to build

Add byte-level access and conversion methods to `UuidV7` to enable zero-copy database storage.

### Location

- `lithos-core/src/support/uuid.rs` - UuidV7 methods and TryFrom impls
- `lithos-core/src/support/error.rs` - UuidV7Error enum (centralized per codebase convention)
- `lithos-core/src/support/mod.rs` - exports

### Implementation

**Methods added to `UuidV7`:**

```rust
impl UuidV7 {
    /// Returns the UUID as a 16-byte array (zero-copy).
    pub const fn as_bytes(&self) -> &[u8; 16]

    /// Consumes the UUID and returns the raw 16 bytes.
    pub fn into_bytes(self) -> [u8; 16]
}
```

**TryFrom impls added:**

```rust
impl TryFrom<[u8; 16]> for UuidV7 { ... }
impl TryFrom<&[u8; 16]> for UuidV7 { ... }  // Uses Uuid::from_bytes_ref
impl TryFrom<&[u8]> for UuidV7 { ... }      // Uses Uuid::from_slice
```

**Error variant added to UuidV7Error:**

```rust
pub enum UuidV7Error {
    InvalidBytes(#[source] uuid::Error),
}
```

## Test checklist

- [x] `as_bytes_returns_16_bytes` - verify length and content
- [x] `into_bytes_ownership_transfer` - verify owned conversion works
- [x] `try_from_bytes_accepts_valid_v7` - valid v7 bytes succeed
- [x] `try_from_bytes_rejects_non_v7` - non-v7 bytes fail with WrongVersion
- [x] `try_from_slice_accepts_valid_v7` - slice conversion works
- [x] `try_from_slice_rejects_wrong_length` - slice length validation

## Acceptance criteria

- [x] `as_bytes()` and `into_bytes()` methods compile and work correctly
- [x] `TryFrom<[u8; 16]>`, `TryFrom<&[u8; 16]>`, `TryFrom<&[u8]>` impls compile and work
- [x] All new unit tests pass
- [x] `mise run verify` passes (no regressions)

## Blocked by

None - can start immediately

## User stories covered

- #2: Enforce v7 invariant in DB APIs (byte conversion validates version)
- #3: Consistent conversion methods across ID wrappers

## Implementation Notes

- Error type centralized in `support/error.rs` per codebase convention
- Used `Uuid::from_bytes_ref` for borrowed array conversion (more explicit than dereferencing)
- Removed `Eq` derive from `UuidV7Error` since `uuid::Version` doesn't implement `Eq`
- 14 unit tests pass (6 new + 8 existing)
