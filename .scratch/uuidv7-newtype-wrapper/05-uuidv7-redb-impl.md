---
title: 05-uuidv7-redb-impl
category: enhancement
label: ready-for-agent
status: pending
date_created: 2026-05-09
---

# UUIDv7 Redb Value and Key Implementation

## Parent

UUIDv7 Hardening PRD

## What to build

Implement `redb::Value` and `redb::Key` traits for `UuidV7` to enable zero-copy binary storage as database keys.

### Location

- File: `lithos-core/src/db/uuid.rs` (new file)
- Module: `lithos-core/src/db/mod.rs` - add `mod uuid;` and `pub use uuid::UuidV7Redb;`
- Note: The impl must live in `db/` to keep redb imports centralized (per project convention)

### Implementation

#### 1. Direct impls for UuidV7

```rust
use redb::{Key, TypeName, Value};
use std::cmp::Ordering;

use crate::support::UuidV7;

/// redb Value and Key implementation for UuidV7.
///
/// Enables zero-copy binary storage of UUIDs as keys (16 bytes vs ~37 byte string).
impl Value for UuidV7 {
    type SelfType<'a> = &'a UuidV7;
    type AsBytes<'a> = &'a [u8; 16];

    #[inline]
    fn fixed_width() -> Option<usize> {
        Some(16)
    }

    #[inline]
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        // Convert bytes to Uuid, validate v7, return reference
        // Note: Need to handle the fact that we can't return a reference to
        // a newly constructed value. May need to store in thread-local or
        // use a different SelfType pattern.
        // See: redb impl for &[u8; N] for reference pattern
        todo!("Implement with proper zero-copy pattern")
    }

    #[inline]
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.as_bytes()
    }

    fn type_name() -> TypeName {
        TypeName::new("lithos::UuidV7")
    }
}

impl Key for UuidV7 {
    #[inline]
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        data1.cmp(data2)
    }
}
```

#### 2. Macro for wrappers (SchemaId, NoteId, PropertyId, TemplateId, VaultId, etc.)

````rust
/// Derive-like macro to implement redb::Value and redb::Key for UuidV7 wrappers.
///
/// Usage:
/// ```
/// impl_redb_uuid!(SchemaId);
/// impl_redb_uuid!(NoteId);
/// impl_redb_uuid!(PropertyId);
/// ```
///
/// Assumes the wrapper is a tuple struct with UuidV7 as the first field:
/// `pub struct SchemaId(UuidV7);`
macro_rules! impl_redb_uuid {
    ($wrapper:ty) => {
        impl redb::Value for $wrapper {
            type SelfType<'a> = &'a $wrapper;
            type AsBytes<'a> = &'a [u8; 16];

            #[inline]
            fn fixed_width() -> Option<usize> {
                Some(16)
            }

            #[inline]
            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                // TODO: Implement - convert bytes to wrapper type
                todo!("Implement from_bytes for {}", stringify!($wrapper))
            }

            #[inline]
            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'b,
            {
                // Access .0 directly since wrapper is a tuple struct with UuidV7 as first field
                value.0.as_bytes()
            }

            fn type_name() -> TypeName {
                TypeName::new(concat!("lithos::", stringify!($wrapper)))
            }
        }

        impl redb::Key for $wrapper {
            #[inline]
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                data1.cmp(data2)
            }
        }
    };
}
````

Apply to existing wrappers:
```rust
impl_redb_uuid!(crate::schema::identifier::SchemaId);
impl_redb_uuid!(crate::note::identifier::NoteId);
impl_redb_uuid!(crate::schema::property_spec::identifier::PropertyId);
// etc.
```

### Design considerations

1. **SelfType<'a> pattern**: The `uuid` crate's `Uuid` doesn't implement `AsRef<[u8; 16]>`. Need to check if we can use a borrowed pattern or need to use an owned pattern.

2. **Reference stability**: For true zero-copy, we need to ensure the bytes remain valid for the lifetime 'a. The `UuidV7` struct owns the bytes, so this should work.

3. **Fixed width**: UUID is always 16 bytes, so `fixed_width()` returns `Some(16)`.

### Test checklist

- [ ] `redb_value_impl_compiles` - Value trait impl compiles
- [ ] `redb_key_impl_compiles` - Key trait impl compiles
- [ ] `roundtrip_via_redb` - Store and retrieve UuidV7 via redb table
- [ ] `ordering_preserved` - Keys sort correctly (lexicographic byte order)
- [ ] `macro_expands_for_schema_id` - macro generates impl for SchemaId
- [ ] `macro_expands_for_note_id` - macro generates impl for NoteId
- [ ] `macro_expands_for_property_id` - macro generates impl for PropertyId
- [ ] `wrapper_roundtrip_via_redb` - SchemaId (or other wrapper) roundtrips correctly

## Acceptance criteria

- [ ] `db/uuid.rs` file created with impls
- [ ] `redb::Value for UuidV7` compiles and works
- [ ] `redb::Key for UuidV7` compiles and works
- [ ] Basic roundtrip test: put UuidV7, get UuidV7 back
- [ ] `mise run verify` passes (no regressions)

## Blocked by

- 04-uuidv7-byte-access (byte access methods needed for impl)

## User stories covered

- #2: DB APIs accept only valid UUID v7 (zero-copy binary keys, no string allocation)
- #3: Consistent conversion methods (as_bytes/into_bytes enable impl)

## Notes

- Reference: `impl Value for &[u8; N]` in redb source (fixed-width byte array pattern)
- The `type_name()` uses `TypeName::new` (not `internal`) since this is a user-defined type
- Lexicographic byte comparison is appropriate for UUIDs (matches byte order of stored value)
