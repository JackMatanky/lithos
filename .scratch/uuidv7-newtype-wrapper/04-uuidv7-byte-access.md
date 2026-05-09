---
title: 04-uuidv7-byte-access
category: enhancement
label: needs-triage
status: pending
date_created: 2026-05-09
---

# UUIDv7 Byte Access and Conversion

## Parent

UUIDv7 Hardening PRD

## What to build

Add byte-level access and conversion methods to `UuidV7` to enable zero-copy database storage.

### Location

- File: `lithos-core/src/support/uuid.rs`
- No new exports needed (additions to existing `UuidV7`)

### Methods to add

```rust
impl UuidV7 {
    /// Returns the UUID as a 16-byte array (zero-copy).
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    /// Consumes the UUID and returns the raw 16 bytes.
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> [u8; 16] {
        self.0.into_bytes()
    }
}

impl TryFrom<[u8; 16]> for UuidV7 {
    type Error = UuidV7Error;

    fn try_from(bytes: [u8; 16]) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_bytes(bytes);
        Self::try_from(uuid)
    }
}

impl TryFrom<&[u8; 16]> for UuidV7 {
    type Error = UuidV7Error;

    fn try_from(bytes: &[u8; 16]) -> Result<Self, Self::Error> {
        Self::try_from(*bytes)
    }
}

impl TryFrom<&[u8]> for UuidV7 {
    type Error = UuidV7Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let uuid = Uuid::from_slice(bytes).map_err(UuidV7Error::InvalidBytes)?;
        Self::try_from(uuid)
    }
}
```

### Error variant to add

```rust
pub enum UuidV7Error {
    // ... existing variants ...
    /// Invalid byte slice (wrong length).
    #[error("invalid UUID bytes: expected 16 bytes, got {0}")]
    InvalidBytes(#[source] uuid::Error),
}
```

### Test checklist

- [ ] `as_bytes_returns_16_bytes` - verify length and content
- [ ] `into_bytes_ownership_transfer` - verify owned conversion works
- [ ] `try_from_bytes_accepts_valid_v7` - valid v7 bytes succeed
- [ ] `try_from_bytes_rejects_non_v7` - non-v7 bytes fail with WrongVersion
- [ ] `try_from_bytes_rejects_wrong_length` - wrong length fails with InvalidBytes
- [ ] `try_from_slice_accepts_valid_v7` - slice conversion works
- [ ] `try_from_slice_rejects_wrong_length` - slice length validation

## Acceptance criteria

- [ ] `as_bytes()` and `into_bytes()` methods compile and work correctly
- [ ] `TryFrom<[u8; 16]>`, `TryFrom<&[u8; 16]>`, `TryFrom<&[u8]>` impls compile and work
- [ ] All new unit tests pass
- [ ] `mise run verify` passes (no regressions)

## Blocked by

None - can start immediately

## User stories covered

- #2: Enforce v7 invariant in DB APIs (byte conversion validates version)
- #3: Consistent conversion methods across ID wrappers
