# rkyv Best Practices for Persistent Storage

**Version**: rkyv 0.8.16
**Last Updated**: 2026-05-10

This guide covers best practices for using rkyv as a serialization layer for persistent storage (databases, files, caches). It focuses on safety, performance, and maintainability when working with zero-copy deserialization in storage contexts.

---

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Alignment Requirements and Safety](#alignment-requirements-and-safety)
3. [Validation Boundaries](#validation-boundaries)
4. [Zero-Copy Reads vs Deserialization](#zero-copy-reads-vs-deserialization)
5. [Common Footguns and Gotchas](#common-footguns-and-gotchas)
6. [Best Practices for Persistent Storage](#best-practices-for-persistent-storage)
7. [Error Handling Strategies](#error-handling-strategies)
8. [Testing Archived Types](#testing-archived-types)

---

## Core Concepts

### What is Zero-Copy Deserialization?

rkyv serializes Rust types into a format that can be accessed **directly from bytes** without deserialization. Instead of parsing bytes into Rust structs, you get references to `Archived<T>` types that live in the byte buffer itself.

```rust
use rkyv::{Archive, Serialize, Deserialize};

#[derive(Archive, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    active: bool,
}

// Serialization produces bytes
let user = User { id: 42, name: "Alice".into(), active: true };
let bytes = rkyv::to_bytes::<rancor::Error>(&user).unwrap();

// Zero-copy access: no deserialization needed
let archived = rkyv::access::<ArchivedUser, rancor::Error>(&bytes).unwrap();
assert_eq!(archived.id, 42);
assert_eq!(archived.name, "Alice");
```

### When to Use rkyv for Persistent Storage

**✅ Good Use Cases:**
- **High-performance caching**: Minimal overhead for reads, especially for large objects
- **Read-heavy workloads**: Zero-copy access means near-instant reads
- **Memory-mapped files**: Direct access to mmap'd regions without parsing
- **Embedded databases**: Where allocations are expensive or forbidden
- **Cross-process data sharing**: When you need to share read-only data between processes

**❌ Not Ideal For:**
- **Schema evolution**: rkyv has limited schema evolution support (see [Format Stability](#format-stability))
- **Untrusted data sources**: Validation overhead can be significant
- **Frequent partial updates**: Zero-copy is optimized for reads, not in-place writes
- **Human-readable formats**: Binary format is not human-inspectable

---

## Alignment Requirements and Safety

### Why Alignment Matters

Modern CPUs require data to be aligned to word boundaries (4 bytes on 32-bit, 8 bytes on 64-bit) for efficient loads. Misaligned data can cause:

1. **Performance degradation** (2x slower due to multiple loads + correction logic)
2. **Crashes on some architectures** (e.g., older ARM chips, some RISC-V)
3. **Undefined behavior** when casting bytes to references

rkyv ensures safety by requiring aligned buffers for zero-copy access.

### Alignment Feature Flags

rkyv provides two alignment strategies controlled by feature flags:

| Feature | Behavior | Trade-offs |
|---------|----------|------------|
| `aligned` (default) | Primitives have natural alignment (e.g., `u64` is 8-aligned) | **Faster access**, but requires aligned buffers |
| `unaligned` | All primitives are 1-aligned | **Flexible storage**, but slower access (unaligned loads) |

⚠️ **Critical**: Changing alignment features is a **breaking format change**. Data serialized with `aligned` cannot be read with `unaligned` and vice versa.

### AlignedVec: The Safe Default

For most use cases, use `AlignedVec` instead of `Vec<u8>` to ensure proper alignment:

```rust
use rkyv::util::AlignedVec;

// ✅ GOOD: AlignedVec guarantees 16-byte alignment
let mut buffer = AlignedVec::<16>::new();
rkyv::api::high::to_bytes_in::<_, rancor::Error>(&my_data, &mut buffer).unwrap();

// Safe: AlignedVec's alignment matches rkyv's requirements
let archived = rkyv::access::<ArchivedMyData, rancor::Error>(&buffer).unwrap();
```

```rust
// ❌ BAD: Vec<u8> may not be properly aligned
let mut buffer = Vec::new();
rkyv::api::high::to_bytes_in::<_, rancor::Error>(&my_data, &mut buffer).unwrap();

// May panic or cause UB depending on data layout
let archived = rkyv::access::<ArchivedMyData, rancor::Error>(&buffer).unwrap();
```

### Alignment with redb and Memory-Mapped Storage

When using redb's `AccessGuard` or memory-mapped files, alignment is **not guaranteed**. Options:

1. **Copy to AlignedVec** (safe but allocates):
```rust
use redb::{Database, ReadableTable, TableDefinition};
use rkyv::util::AlignedVec;

const MY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("my_table");

fn read_from_redb(db: &Database, key: &str) -> Result<ArchivedMyData, Error> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(MY_TABLE)?;
    let guard = table.get(key)?.ok_or(Error::NotFound)?;

    // Copy to aligned buffer
    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(guard.value());

    // Safe: now properly aligned
    Ok(rkyv::access::<ArchivedMyData, rancor::Error>(&aligned)?.clone())
}
```

2. **Use `unaligned` feature** (zero-copy but slower):
```toml
[dependencies]
rkyv = { version = "0.8", default-features = false, features = ["unaligned", "bytecheck"] }
```

⚠️ **Trade-off**: `unaligned` removes the alignment requirement but uses slower unaligned loads on every access.

### Alignment Pitfalls

#### Pitfall 1: Prefixing Data

If you prefix archived data with metadata (e.g., length), ensure alignment is preserved:

```rust
// ❌ BAD: 4-byte length prefix misaligns 8-byte data
let mut buffer = AlignedVec::<16>::new();
buffer.extend_from_slice(&data_length.to_le_bytes()); // 4 bytes
buffer.extend_from_slice(&archived_data); // Now misaligned!

// ✅ GOOD: Pad to preserve alignment
let mut buffer = AlignedVec::<16>::new();
buffer.extend_from_slice(&data_length.to_le_bytes());
buffer.extend_from_slice(&[0; 4]); // Pad to 8-byte boundary
buffer.extend_from_slice(&archived_data);
```

#### Pitfall 2: Buffer Not Tight to End

`rkyv::access` assumes the root object is at the **end** of the buffer:

```rust
// ❌ BAD: Extra data after root object
let mut buffer = AlignedVec::<16>::new();
serialize_to(&my_data, &mut buffer);
buffer.push(0); // Breaks root calculation!

// ✅ GOOD: Root is at buffer end
let buffer = rkyv::to_bytes::<rancor::Error>(&my_data).unwrap();
```

---

## Validation Boundaries

### When to Validate

rkyv provides two access modes:

| Function | Validation | Use Case | Performance |
|----------|-----------|----------|-------------|
| `rkyv::access()` | ✅ Full validation | Untrusted data (files, network, user input) | Slower (O(n) scan) |
| `rkyv::access_unchecked()` | ❌ No validation | Trusted data (internally produced) | Fastest (O(1) cast) |

⚠️ **Rule of Thumb**: Always use `access()` when reading from **persistent storage** (files, databases). Use `access_unchecked()` only for in-memory buffers you've just serialized.

### Validation Example

```rust
use rkyv::{access, rancor::Error};

// ✅ SAFE: Validate data from disk
fn load_from_file(path: &Path) -> Result<ArchivedConfig, Error> {
    let bytes = std::fs::read(path)?;
    let archived = access::<ArchivedConfig, Error>(&bytes)?;
    Ok(archived.clone()) // Clone to owned memory
}

// ⚠️ UNSAFE: Only use for trusted in-memory data
fn serialize_and_access_immediate() {
    let data = MyData { value: 42 };
    let bytes = rkyv::to_bytes::<Error>(&data).unwrap();

    // Safe: we just serialized this, know it's valid
    let archived = unsafe { rkyv::access_unchecked::<ArchivedMyData>(&bytes) };
    assert_eq!(archived.value, 42);
}
```

### Validation Overhead

Validation performs these checks:

1. **Pointer bounds**: All pointers point inside the buffer
2. **Alignment**: All values are properly aligned
3. **Type validity**: Enums have valid discriminants, etc.
4. **Subtree ranges**: Memory ownership model is respected (no overlapping subobjects)

**Performance Impact**: O(n) in buffer size. For a 1MB buffer, validation typically adds 0.1-1ms.

**Mitigation Strategies**:
- Cache validation results if reading the same data multiple times
- Use signatures/checksums for untrusted data instead of full validation
- Consider `unaligned` feature if alignment is the main validation cost

### HighValidator and Custom Validators

```rust
use rkyv::{access, validation::validators::DefaultValidator};

// Default validator (used by `access()`)
let archived = access::<ArchivedMyData, rancor::Error>(&bytes)?;

// Custom validator for specialized use cases
use rkyv::validation::{ArchiveContext, validators::HighValidator};
let mut validator = HighValidator::default();
let archived = rkyv::api::high::access_with_context::<ArchivedMyData, _, _>(
    &bytes,
    &mut validator
)?;
```

---

## Zero-Copy Reads vs Deserialization

### When to Use Zero-Copy (`&Archived<T>`)

Zero-copy access is ideal when:

1. **Reading data multiple times** from the same buffer
2. **Accessing specific fields** without deserializing the whole object
3. **Working with large nested structures** where you only need part of the data

```rust
use rkyv::{Archive, Serialize};

#[derive(Archive, Serialize)]
struct LargeRecord {
    id: u64,
    metadata: Vec<String>, // Large field
    payload: Vec<u8>,       // Even larger
}

// ✅ GOOD: Zero-copy to access just the ID
fn get_id(bytes: &[u8]) -> Result<u64, Error> {
    let archived = access::<ArchivedLargeRecord, Error>(bytes)?;
    Ok(archived.id) // No allocation, just a read
}
```

### When to Deserialize (`rkyv::deserialize()`)

Deserialize back to native Rust types when:

1. **Mutating the data** (archived types are immutable)
2. **Returning owned data** that outlives the buffer
3. **Working with APIs that expect native types**

```rust
use rkyv::{Deserialize, Archived};

// ✅ GOOD: Deserialize when returning owned data
fn load_user(bytes: &[u8]) -> Result<User, Error> {
    let archived = access::<ArchivedUser, Error>(bytes)?;
    Ok(rkyv::deserialize::<User, Error>(archived)?)
}

// ❌ BAD: Can't return archived reference (lifetime issues)
fn load_user_ref(bytes: &[u8]) -> Result<&ArchivedUser, Error> {
    // Error: can't return reference tied to local bytes
    access::<ArchivedUser, Error>(bytes)
}
```

### Closure-Based Zero-Copy Pattern

For database storage where you can't return references (e.g., redb's `AccessGuard`):

```rust
use redb::AccessGuard;

// ✅ GOOD: Use closure to access data without cloning
fn with_archived<F, R>(guard: AccessGuard<'_, &[u8]>, f: F) -> Result<R, Error>
where
    F: FnOnce(&Archived<MyData>) -> R,
{
    let archived = access::<ArchivedMyData, Error>(guard.value())?;
    Ok(f(archived))
}

// Usage: zero-copy read
let result = with_archived(guard, |archived| {
    (archived.id, archived.name.as_str())
})?;
```

---

## Common Footguns and Gotchas

### 1. Mixing Archived Access with Other Lifetime-Dependent Types

**Problem**: `&Archived<T>` borrows from the byte buffer, which conflicts with types like redb's `AccessGuard`.

```rust
// ❌ COMPILE ERROR: Can't return archived ref that borrows from guard
fn get_data(guard: AccessGuard<'_, &[u8]>) -> Result<&ArchivedMyData, Error> {
    let archived = access::<ArchivedMyData, Error>(guard.value())?;
    Ok(archived) // Error: archived borrows from guard, which is dropped
}

// ✅ FIX 1: Use closure pattern
fn with_data<F, R>(guard: AccessGuard<'_, &[u8]>, f: F) -> Result<R, Error>
where F: FnOnce(&ArchivedMyData) -> R {
    let archived = access::<ArchivedMyData, Error>(guard.value())?;
    Ok(f(archived))
}

// ✅ FIX 2: Clone to owned data
fn get_data_owned(guard: AccessGuard<'_, &[u8]>) -> Result<MyData, Error> {
    let archived = access::<ArchivedMyData, Error>(guard.value())?;
    Ok(rkyv::deserialize::<MyData, Error>(archived)?)
}
```

### 2. Memory Safety with Loops

**Problem**: Reusing buffers in loops can lead to use-after-free if archived references outlive the buffer.

```rust
// ❌ DANGEROUS: Archived refs from previous iterations are invalidated
let mut buffer = AlignedVec::<16>::new();
let mut results = Vec::new();

for item in items {
    buffer.clear(); // Invalidates previous archived refs!
    serialize_to(&item, &mut buffer);
    let archived = unsafe { access_unchecked(&buffer) };
    results.push(archived); // Use-after-free on next iteration
}

// ✅ FIX: Clone to owned data or use separate buffers
for item in items {
    buffer.clear();
    serialize_to(&item, &mut buffer);
    let archived = access(&buffer)?;
    results.push(rkyv::deserialize(archived)?); // Owned data
}
```

### 3. Format Stability and Schema Evolution

**Problem**: Changing your Rust struct changes the serialized format, breaking old data.

```rust
// Version 1
#[derive(Archive, Serialize)]
struct Config {
    name: String,
    value: i32,
}

// Version 2 - BREAKING CHANGE
#[derive(Archive, Serialize)]
struct Config {
    name: String,
    value: i64, // Changed from i32 - old data is now invalid!
    new_field: bool, // Added field - old data can't be read
}
```

**Mitigation Strategies**:
1. **Version your data**: Include a version field and handle migrations
2. **Use `#[rkyv(with = ...)]` for custom serialization** to maintain compatibility
3. **Store schema hashes** to detect mismatches
4. **Document format changes** as breaking changes in your API

```rust
// ✅ Versioned format
#[derive(Archive, Serialize)]
struct ConfigV1 {
    version: u32, // Always 1
    name: String,
    value: i32,
}

#[derive(Archive, Serialize)]
struct ConfigV2 {
    version: u32, // Always 2
    name: String,
    value: i64,
    new_field: bool,
}

fn load_config(bytes: &[u8]) -> Result<ConfigV2, Error> {
    // Read version field first
    let version = u32::from_le_bytes(bytes[..4].try_into()?);
    match version {
        1 => {
            let v1 = access::<ArchivedConfigV1, Error>(bytes)?;
            Ok(migrate_v1_to_v2(v1))
        }
        2 => {
            let v2 = access::<ArchivedConfigV2, Error>(bytes)?;
            Ok(rkyv::deserialize(v2)?)
        }
        _ => Err(Error::UnsupportedVersion(version)),
    }
}
```

### 4. Feature Flag Mismatches

**Problem**: Enabling/disabling format control features creates incompatible formats.

```toml
# Crate A (producer)
[dependencies]
rkyv = { version = "0.8", features = ["little_endian"] }

# Crate B (consumer)
[dependencies]
rkyv = { version = "0.8", features = ["big_endian"] } # INCOMPATIBLE!
```

⚠️ **Critical**: All crates reading/writing the same data must use **identical feature flags** for:
- Endianness (`little_endian` / `big_endian`)
- Alignment (`aligned` / `unaligned`)
- Pointer width (`pointer_width_16` / `pointer_width_32` / `pointer_width_64`)

**Best Practice**: Document your format requirements explicitly:

```rust
// In your library's docs:
/// # Format Requirements
///
/// This crate uses rkyv with the following format:
/// - Endianness: little-endian
/// - Alignment: aligned (16-byte)
/// - Pointer width: 32-bit
///
/// Consumers MUST use these same settings:
/// ```toml
/// rkyv = { version = "0.8", features = ["little_endian", "aligned", "pointer_width_32"] }
/// ```
```

### 5. Shared Pointers and Validation

**Problem**: Validating shared pointers (`Rc`, `Arc`) requires checking that all pointers to the same object have the same type.

```rust
use std::sync::Arc;
use rkyv::{Archive, Serialize};

#[derive(Archive, Serialize)]
struct Data {
    shared: Arc<Vec<i32>>,
}

// Validation will fail if the same Arc appears with different types
// (e.g., Arc<Vec<i32>> vs Arc<dyn Any>)
```

**Workaround**: If you need heterogeneous shared pointers, consider alternative validation like checksums or signatures.

---

## Best Practices for Persistent Storage

### 1. Always Validate Data from Disk

```rust
// ✅ ALWAYS validate persistent data
fn read_from_db(db: &Database, key: &str) -> Result<MyData, Error> {
    let bytes = db.get(key)?;
    let archived = access::<ArchivedMyData, Error>(&bytes)?; // Validates
    Ok(rkyv::deserialize(archived)?)
}

// ❌ NEVER skip validation for persistent data
fn read_from_db_unchecked(db: &Database, key: &str) -> Result<&ArchivedMyData, Error> {
    let bytes = db.get(key)?;
    unsafe { Ok(access_unchecked(&bytes)) } // Disk corruption = UB
}
```

### 2. Use AlignedVec for Serialization Buffers

```rust
use rkyv::util::AlignedVec;

// ✅ GOOD: Guaranteed alignment
fn serialize_user(user: &User) -> Result<AlignedVec<16>, Error> {
    let mut buffer = AlignedVec::new();
    rkyv::api::high::to_bytes_in::<_, Error>(user, &mut buffer)?;
    Ok(buffer)
}

// Store AlignedVec directly if your storage supports it
db.put(key, buffer.as_slice())?;
```

### 3. Separate Hot and Cold Data

rkyv's zero-copy model works best for read-heavy workloads. For mixed read/write:

```rust
// Split hot (frequently updated) and cold (rarely updated) fields
#[derive(Archive, Serialize)]
struct UserProfile {
    cold: ColdData,    // Bio, avatar URL - rarely changes
    hot_ref: String,   // Reference to hot data (e.g., "user:{id}:stats")
}

#[derive(Archive, Serialize)]
struct UserStats {
    login_count: u64,  // Updated frequently
    last_seen: i64,
}

// Update hot data without reserializing cold data
fn increment_login_count(db: &Database, user_id: &str) -> Result<(), Error> {
    let key = format!("user:{}:stats", user_id);
    let bytes = db.get(&key)?;
    let mut stats = rkyv::deserialize::<UserStats, Error>(
        access(&bytes)?
    )?;

    stats.login_count += 1;

    let new_bytes = rkyv::to_bytes::<Error>(&stats)?;
    db.put(&key, &new_bytes)?;
    Ok(())
}
```

### 4. Consider Memory-Mapped Files for Large Datasets

For datasets that don't fit in memory, use `mmap` + zero-copy:

```rust
use memmap2::Mmap;

fn open_large_dataset(path: &Path) -> Result<Mmap, Error> {
    let file = File::open(path)?;
    unsafe { Mmap::map(&file) }
}

fn query_record(mmap: &Mmap, offset: usize) -> Result<&ArchivedRecord, Error> {
    let bytes = &mmap[offset..];
    access::<ArchivedRecord, Error>(bytes)
}
```

⚠️ **Alignment Note**: `mmap` does not guarantee alignment. Either:
- Use `unaligned` feature flag, or
- Copy to `AlignedVec` for the accessed region

### 5. Batch Validation for Performance

If validating many records, batch the work:

```rust
fn validate_all_records(records: &[&[u8]]) -> Result<(), Error> {
    // Parallel validation
    records.par_iter().try_for_each(|bytes| {
        access::<ArchivedRecord, Error>(bytes)?;
        Ok(())
    })
}
```

---

## Error Handling Strategies

### Structured Errors with `rancor`

rkyv 0.8 uses the `rancor` crate for error handling. Define domain-specific errors:

```rust
use rancor::{Error as RancorError, Fallible, Source};

#[derive(Debug)]
pub enum StorageError {
    Serialization(RancorError),
    Validation(RancorError),
    NotFound,
    Io(std::io::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl Source for StorageError {
    fn new() -> Self {
        StorageError::Serialization(RancorError::new())
    }
}

// Usage
fn load_record(path: &Path) -> Result<Record, StorageError> {
    let bytes = std::fs::read(path)?;
    let archived = access::<ArchivedRecord, StorageError>(&bytes)
        .map_err(StorageError::Validation)?;
    Ok(rkyv::deserialize(archived)
        .map_err(StorageError::Serialization)?)
}
```

### Fallback Strategies

Handle corrupted data gracefully:

```rust
fn load_with_fallback(path: &Path, default: &Config) -> Config {
    match std::fs::read(path) {
        Ok(bytes) => {
            match access::<ArchivedConfig, Error>(&bytes) {
                Ok(archived) => rkyv::deserialize(archived).unwrap_or_else(|_| default.clone()),
                Err(_) => {
                    eprintln!("Corrupted config, using default");
                    default.clone()
                }
            }
        }
        Err(_) => default.clone(),
    }
}
```

---

## Testing Archived Types

### 1. Roundtrip Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rkyv::{access, to_bytes, rancor::Error};

    #[test]
    fn test_roundtrip() {
        let original = MyData {
            id: 42,
            name: "test".into(),
            values: vec![1, 2, 3],
        };

        // Serialize
        let bytes = to_bytes::<Error>(&original).unwrap();

        // Access (zero-copy)
        let archived = access::<ArchivedMyData, Error>(&bytes).unwrap();
        assert_eq!(archived.id, original.id);
        assert_eq!(archived.name, original.name);

        // Deserialize
        let deserialized = rkyv::deserialize::<MyData, Error>(archived).unwrap();
        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.values, original.values);
    }
}
```

### 2. Format Stability Tests

```rust
#[test]
fn test_format_stability() {
    // Golden bytes from a known-good serialization
    const EXPECTED_BYTES: &[u8] = &[
        // ... hex dump of serialized data
    ];

    let data = MyData { id: 42, name: "test".into() };
    let bytes = to_bytes::<Error>(&data).unwrap();

    // Ensure format hasn't changed
    assert_eq!(&*bytes, EXPECTED_BYTES, "Format changed! Update migration code.");
}
```

### 3. Alignment Tests

```rust
#[test]
fn test_aligned_vec_guarantees() {
    let buffer = AlignedVec::<16>::with_capacity(100);

    // Verify alignment
    assert_eq!(buffer.as_ptr() as usize % 16, 0, "Buffer not 16-aligned");
}

#[test]
fn test_unaligned_data_fails() {
    let data = MyData { value: 42 };
    let mut misaligned = vec![0u8]; // 1-byte offset
    misaligned.extend_from_slice(&to_bytes::<Error>(&data).unwrap());

    // Should fail validation due to misalignment (if using aligned feature)
    let result = access::<ArchivedMyData, Error>(&misaligned[1..]);
    assert!(result.is_err());
}
```

### 4. Corruption Tests

```rust
#[test]
fn test_detects_corruption() {
    let data = MyData { id: 42, name: "test".into() };
    let mut bytes = to_bytes::<Error>(&data).unwrap();

    // Corrupt a byte
    bytes[10] ^= 0xFF;

    // Validation should catch this
    let result = access::<ArchivedMyData, Error>(&bytes);
    assert!(result.is_err(), "Validation failed to detect corruption");
}
```

---

## Summary Checklist

When using rkyv for persistent storage:

- [ ] **Always validate data from disk** using `rkyv::access()`
- [ ] **Use `AlignedVec<16>`** for serialization buffers
- [ ] **Document format requirements** (endianness, alignment, pointer width)
- [ ] **Handle format evolution** with versioning or migrations
- [ ] **Test roundtrips and format stability** in your test suite
- [ ] **Use closure-based patterns** for zero-copy access with lifetime constraints
- [ ] **Avoid `access_unchecked()` for persistent data** (corruption risk)
- [ ] **Consider `unaligned` feature** if working with memory-mapped or prefix'd data
- [ ] **Batch validation** for performance-critical paths
- [ ] **Separate hot and cold data** for mixed read/write workloads

---

## Additional Resources

- **rkyv Book**: https://rkyv.org/
- **API Documentation**: https://docs.rs/rkyv/latest/rkyv/
- **GitHub Repository**: https://github.com/rkyv/rkyv
- **Discord Community**: https://discord.gg/65F6MdnbQh (for questions and support)

---

**Disclaimer**: This document is based on rkyv 0.8.16 and reflects best practices as of 2026-05-10. Always consult the official documentation for the latest updates and changes.
