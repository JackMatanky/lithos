---
parent: UUIDv7 Hardening PRD
labels: needs-triage
status: completed
date_created: 2026-05-06
date_completed: 2026-05-06
---

## Parent

UUIDv7 Hardening PRD

## What to build

Implement the `UuidV7` support type in `lithos-core/src/support/uuid.rs` as a newtype wrapper over `uuid::Uuid` that enforces UUID v7 invariant at construction and parsing time.

### Location

- File: `lithos-core/src/support/uuid.rs`
- Export: `lithos-core/src/support/mod.rs` with `pub use uuid::{UuidV7, UuidV7Error};`
- If `support/mod.rs` does not exist yet, create it and wire from crate root

### Final API Surface (Post-Step-7 Hardening)

```rust
pub struct UuidV7(uuid::Uuid);

impl UuidV7 {
    pub fn new() -> Self;                              // generates now_v7
    pub fn parse(input: &str) -> Result<Self, UuidV7Error>;
    pub fn try_from_uuid(uuid: uuid::Uuid) -> Result<Self, UuidV7Error>;
    pub const fn as_uuid(&self) -> &uuid::Uuid;
    pub const fn into_uuid(self) -> uuid::Uuid;
}
```

**Note**: `from_uuid_unchecked` was removed during Step 7 API hardening. The API now prefers validated construction via `try_from_uuid` and `TryFrom<Uuid>`.

### Error Type

```rust
pub enum UuidV7Error {
    Parse(uuid::Error),
    WrongVersion { got: Option<uuid::Version> },
}
```

### Required Trait Impls

- `Default` (delegates to `new`)
- `Display`
- `TryFrom<uuid::Uuid> for UuidV7`
- `From<UuidV7> for uuid::Uuid`
- `FromStr for UuidV7` (enables `str.parse::<UuidV7>()`)

### Required Derives

- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`
- `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize`

### Test Checklist

Unit tests for `lithos-core/src/support/uuid.rs`:

1. **`new_creates_v7_uuid`**: assert `as_uuid().get_version() == Some(Version::SortRand)`
2. **`parse_accepts_valid_v7`**: parse a known/generated v7 string and assert success
3. **`parse_rejects_non_v7`**: parse v4 string, assert `WrongVersion`
4. **`parse_rejects_invalid_string`**: assert `Parse` variant
5. **`try_from_uuid_rejects_non_v7`**: pass v4 UUID, assert `WrongVersion`
6. **`roundtrip_into_from_uuid`**: `UuidV7 -> Uuid -> TryFrom<Uuid> -> UuidV7` roundtrip succeeds
7. **`display_matches_inner_uuid`**: `format!("{}", id)` equals inner UUID display
8. **`default_is_v7`**: `UuidV7::default()` produces v7

## Acceptance criteria

- [x] `UuidV7` and `UuidV7Error` compile and are exported from `support` module
- [x] All 8 unit tests pass (new_creates_v7_uuid, parse_accepts_valid_v7, parse_rejects_non_v7, parse_rejects_invalid_string, try_from_uuid_rejects_non_v7, roundtrip_into_from_uuid, display_matches_inner_uuid, default_is_v7)
- [x] No behavioral changes outside `support` module
- [x] API hardened: `from_uuid_unchecked` removed, only validated construction available

## Blocked by

None - can start immediately

## Notes

- Step 1 of the implementation plan
- Naming: `UuidV7` (not `Uuid7`) - explicit, aligns with Rust ecosystem versioned naming
- File name: `uuid.rs` (not `uuid_v7.rs`)
- Methods aligned with existing ID wrappers: `as_uuid`, `into_uuid`
- Ran `cargo test -p lithos-core support::uuid::tests` (all green) and `cargo fmt --all`
- Impact analysis before edits: `SchemaId` and `PropertyId` reported LOW risk
- GitNexus impact check was unavailable (`Not connected`) - proceeded with compile/test verification
- Verified: `mise run verify` passes (985 unit tests, 36 integration tests, doctests)
