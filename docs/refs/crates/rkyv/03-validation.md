# Validation

Accessing `rkyv` archives is inherently unsafe if the data is untrusted, because you are casting arbitrary bytes into memory structures. To do this safely, `rkyv` uses the `bytecheck` crate.

## The Validation Boundary

### 1. Trusted Data (`access_unchecked`)
If the byte buffer is generated internally by your application and stored in a secure location (e.g., an internal cache, an embedded database where you control all writes), you can skip validation for maximum performance.

```rust
use rkyv::access_unchecked;

// SAFETY: We wrote this buffer ourselves in the previous step.
let archived = unsafe { access_unchecked::<ArchivedExample>(&buffer) };
```

### 2. Untrusted Data (`access`)
If the data comes from the network, a user-provided file, or any external source, you **MUST** validate it. Enable the `bytecheck` feature in your `Cargo.toml`.

```rust
use rkyv::{access, rancor::Failure};

// Checks bounds, alignment, and invariants before returning a reference.
let archived = access::<ArchivedExample, Failure>(&buffer).expect("Validation failed");
```

## How Validation Works

When `bytecheck` is enabled, deriving `Archive` also derives `CheckBytes` for the `Archived` type.

The validation context checks:
1.  **Bounds**: All pointers must point inside the archive and have enough space to hold the object.
2.  **Alignment**: Objects must be properly aligned for their type.
3.  **Subtree Ranges**: `rkyv` maintains a memory model where sub-objects must reside in contiguous memory, preventing recursion attacks and overlapping memory violations.

## Shared Pointer Restrictions

While `rkyv` can validate shared pointers, there are limitations to prevent malicious data from passing checks. For example, if two shared pointers point to the exact same bytes but expect different types (e.g., `[u8; 4]` vs `[u32; 1]`), validation will fail. In scenarios involving highly polymorphic shared pointers, alternative security models like cryptographic signatures might be required.
