# rkyv - Reference Documentation

**Version:** 0.8.14
**Official Docs:** https://docs.rs/rkyv/0.8.14/rkyv/
**Guide:** https://rkyv.org/
**Repository:** https://github.com/rkyv/rkyv
**License:** MIT

## Overview

rkyv (archive) is a zero-copy deserialization framework for Rust. It enables accessing serialized data directly without deserialization, making it one of the fastest serialization frameworks available. rkyv scales from no-std to highly-capable environments and provides optional validation for safety.

Format-control features (endianness, alignment, pointer width) define compatibility boundaries for serialized data. Changing these after data is written can make it unreadable.
See https://rkyv.org/ for guide-level discussion of format control and compatibility.

## Core Zero-Copy Principles

### 1. Direct Memory Access

**Key Concept:** Archived data can be used directly from bytes without deserialization.

```rust
// Traditional approach
let data: Vec<MyStruct> = deserialize(bytes)?;  // COPY

// rkyv approach
let archived: &ArchivedVec<MyStruct> = access(bytes)?;  // NO COPY
archived[0].field  // Direct access
```

**Performance Benefits:**

- Zero memory allocations for access
- Constant-time "deserialization"
- No parsing overhead
- Direct memory mapping possible

### 2. [Archive](https://docs.rs/rkyv/0.8.14/rkyv/trait.Archive.html) Trait - Core Abstraction

```rust
pub trait Archive {
    type Archived: Portable;
    type Resolver;

    const COPY_OPTIMIZATION: CopyOptimization<Self>;

    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>);
}
```

**Components:**

**Archived Type:**

- Zero-copy representation
- Can be used directly from bytes
- Must impl `Portable` (stable layout)
- Often similar to original type

**Important:** `Archived<T>` is a distinct type from `T`. Zero-copy access works with `Archived<T>`; deserialization is required to get `T`.
See https://rkyv.org/ for conceptual guidance on working with archived types.

**Resolver:**

- Contains offset information
- Needed to create archived form
- Computed during serialization
- Not stored in final output

**resolve() Method:**

- Creates archived value from resolver
- Writes to provided memory location
- Called during serialization

### 3. [Portable](https://docs.rs/rkyv/0.8.14/rkyv/trait.Portable.html) Trait - Layout Stability

```rust
pub trait Portable {
    // Type has stable, well-defined layout
    // Same on all targets (with format features)
}
```

**Guarantees:**

- Consistent byte representation
- No padding variations
- Cross-platform compatibility
- Version stability

**Guidance:** Use `Portable` when archived values must be stable across targets or persisted formats.

**Derive Support:**

```rust
#[derive(Archive, Portable)]
#[repr(C)]  // Often needed for Portable
struct MyStruct {
    field: u32,
}
```

### 4. Serialization APIs

#### High-Level API (Recommended)

**to_bytes - Simple Serialization:**

```rust
use rkyv::{to_bytes, Archive, Serialize};

#[derive(Archive, Serialize)]
struct Data {
    value: u32,
    text: String,
}

let value = Data { value: 42, text: "hello".to_string() };
let bytes = to_bytes::<rkyv::rancor::Error>(&value)?;
```

**access - Safe Zero-Copy Access:**

```rust
use rkyv::access;

let archived = access::<ArchivedData, rkyv::rancor::Error>(&bytes)?;
println!("{}", archived.value);  // Direct access, no copy
```

**access_unchecked - Unsafe Zero-Copy Access:**

```rust
use rkyv::access_unchecked;

// UNSAFE: Must validate bytes are correct format
let archived = unsafe { access_unchecked::<ArchivedData>(&bytes) };
println!("{}", archived.value);  // Fastest possible access
```

**deserialize - Back to Original:**

```rust
use rkyv::deserialize;

let original: Data = deserialize::<Data, rkyv::rancor::Error>(archived)?;
// Now we have native type (with allocation)
```

**Note:** `deserialize` allocates and should be treated as an escape hatch for cold paths or interoperability.

#### Low-Level API (Advanced)

```rust
use rkyv::{
    api::high::to_bytes_with_alloc,
    ser::allocator::Arena,
};

// Custom allocator for better performance
let mut arena = Arena::new();
let bytes = to_bytes_with_alloc::<_, Error>(&value, arena.acquire())?;
```

### 5. [Format Control](https://docs.rs/rkyv/0.8.14/rkyv/#format-control) Features
See https://rkyv.org/ for guide-level guidance on compatibility boundaries.

#### Endianness

```toml
# Cargo.toml
[dependencies]
rkyv = { version = "0.8", features = ["little_endian"] }
# or
rkyv = { version = "0.8", features = ["big_endian"] }
```

**Default:** Little-endian

**Impact:**

- Controls byte order of primitives
- Affects cross-platform compatibility
- Must match on serialize and deserialize
- Choose based on target platform

**Compatibility Note:** Changing endianness features after data is written is a breaking change for on-disk formats.

#### Alignment

```toml
[dependencies]
rkyv = { version = "0.8", features = ["aligned"] }
# or
rkyv = { version = "0.8", features = ["unaligned"] }
```

**Default:** Aligned

**Aligned:**

- Faster access on most platforms
- Requires aligned memory
- Can't work with unaligned buffers

**Unaligned:**

- Works with any byte buffer
- Slight performance penalty
- Needed for memory-mapped files
- Better for network protocols

**Guidance:** Use `unaligned` for memory-mapped or unaligned buffers (e.g., mmap-backed stores). Use `aligned` for in-memory buffers where alignment can be guaranteed.

#### Pointer Width

```toml
[dependencies]
rkyv = { version = "0.8", features = ["pointer_width_32"] }
# or "pointer_width_16", "pointer_width_64"
```

**Default:** 32-bit

**Trade-offs:**

- 16-bit: Smallest, limited to small data
- 32-bit: Good balance, handles most data
- 64-bit: Largest, for huge datasets

**Compatibility Note:** Pointer width impacts both serialized size and the maximum addressable data size.

### 6. Zero-Copy Collections

#### ArchivedVec

```rust
use rkyv::vec::{ArchivedVec, ArchivedVecResolver};

// Original
let vec = vec![1, 2, 3, 4, 5];

// Serialized form
let bytes = to_bytes(&vec)?;

// Zero-copy access
let archived: &ArchivedVec<i32> = access(&bytes)?;

// Direct indexing (no copy)
let value = archived[0];  // Returns i32 by value (small types)

// Iteration (no allocation)
for &item in archived.iter() {
    println!("{}", item);
}

// Length (stored in archived form)
let len = archived.len();
```

**Performance Characteristics:**

- O(1) indexing
- O(1) length
- Iterator is zero-copy
- No heap allocation for access

#### ArchivedString

```rust
use rkyv::string::{ArchivedString, ArchivedStringResolver};

let s = String::from("hello world");
let bytes = to_bytes(&s)?;

let archived: &ArchivedString = access(&bytes)?;

// Direct access as str (zero-copy)
let s: &str = archived.as_str();

// All str methods available
let upper = s.to_uppercase();  // Now allocated
```
See https://docs.rs/rkyv/0.8.14/rkyv/string/struct.ArchivedString.html for ArchivedString APIs.

**Features:**

- Stored as UTF-8 bytes
- Direct conversion to `&str`
- No validation needed (validated once)
- All `str` methods work

#### ArchivedHashMap

```rust
use rkyv::collections::swiss_table::map::ArchivedHashMap;
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key1", 100);
map.insert("key2", 200);

let bytes = to_bytes(&map)?;
let archived: &ArchivedHashMap<&str, i32> = access(&bytes)?;

// Zero-copy lookup
let value = archived.get("key1");  // Option<&i32>

// Iteration
for (key, value) in archived.iter() {
    println!("{} => {}", key, value);
}
```

**Implementation:**

- Based on Swiss Tables
- FxHash for deterministic hashing
- Same O(1) lookup as native HashMap
- Zero allocation for access
- Target-independent hash function

**Note:** Archived collections use rkyv's hashing/ordering semantics, which may differ from the default `std` hasher.

#### ArchivedBTreeMap

```rust
use rkyv::collections::btree_map::ArchivedBTreeMap;
use std::collections::BTreeMap;

let mut map = BTreeMap::new();
map.insert("apple", 1);
map.insert("banana", 2);

let bytes = to_bytes(&map)?;
let archived: &ArchivedBTreeMap<&str, i32> = access(&bytes)?;

// Zero-copy lookup
let value = archived.get("apple");

// Range iteration
for (key, value) in archived.range("a".."c") {
    println!("{} => {}", key, value);
}
```

**Advantages:**

- Compact representation
- Good locality of reference
- Efficient range queries
- Sorted iteration

### 7. Shared Pointers (Arc/Rc)

```rust
use rkyv::{Archive, Serialize, Deserialize};
use std::sync::Arc;

#[derive(Archive, Serialize, Deserialize)]
struct Node {
    value: i32,
    shared: Arc<String>,
}

let shared_data = Arc::new("shared".to_string());

let node1 = Node { value: 1, shared: shared_data.clone() };
let node2 = Node { value: 2, shared: shared_data.clone() };

let data = vec![node1, node2];
let bytes = to_bytes(&data)?;

// Archived form preserves sharing
let archived: &ArchivedVec<Node> = access(&bytes)?;
// archived[0].shared and archived[1].shared point to same data
```

**Features:**

- Deduplication during serialization
- Shared data stored once
- Archived pointers maintain sharing
- Supports non-cyclic structures

**Guidance:** Use shared pointers to deduplicate repeated large values (e.g., repeated strings in metadata).
See https://docs.rs/rkyv/0.8.14/rkyv/rc/index.html for shared pointer support.

**Limitations:**

- Cyclic structures need special support
- Use serializer/deserializer bounds for cycles

### 8. Validation (bytecheck)

```toml
[dependencies]
rkyv = { version = "0.8", features = ["bytecheck"] }
```

**Safe Access:**

```rust
use rkyv::access;

// Validates before returning reference
let archived = access::<ArchivedData, Error>(&bytes)?;
```
See https://docs.rs/rkyv/0.8.14/rkyv/fn.access.html and https://docs.rs/rkyv/0.8.14/rkyv/fn.access_unchecked.html.

**What Gets Validated:**

- Pointer alignment
- Pointer bounds
- UTF-8 validity (strings)
- Enum variant validity
- Structure invariants

**Performance:**

- Overhead on first access
- Amortized over data lifetime
- Much cheaper than deserialization
- Can be skipped with `access_unchecked`

**Cost Note:** Validation cost scales with structure size. Validate at boundaries and cache validated references when possible.

**When to Use:**

- Untrusted data sources
- Network protocols
- File formats
- User-provided data

**When to Skip:**

- Trusted internal data
- Performance-critical paths
- Pre-validated data
- In-memory only

### 9. Relative Pointers

```rust
use rkyv::rel_ptr::RelPtr;

// Archived types use relative pointers
pub struct ArchivedString {
    ptr: RelPtr<str>,  // Offset-based, not absolute address
}
```

**Benefits:**

- Position-independent
- Can relocate archived data
- No address fixup needed
- Supports memory mapping

**Use Case:** Relative pointers enable mmap and shared-memory use without pointer fixups.
See https://docs.rs/rkyv/0.8.14/rkyv/rel_ptr/index.html for relative pointer types.

**Types:**

- `RelPtr<T>`: Relative pointer to T
- `RawRelPtr`: Raw relative pointer
- `RelPtrMetadata`: Metadata for unsized types

### 10. Copy Optimization

```rust
impl Archive for MyType {
    const COPY_OPTIMIZATION: CopyOptimization<Self> =
        CopyOptimization::enable();  // UNSAFE attestation

    // ... rest of implementation
}
```

**When Safe:**

- Type is `Copy`
- Type is `Portable`
- Archived form == Native form
- No pointer fields
- No padding variations

**Benefits:**

- Direct memory copy
- Skip serialize() call
- Maximum performance
- Minimal code

**Safety Note:** Only enable `CopyOptimization` when the archived and native layouts are guaranteed identical.

**Examples:**

- Primitives (u8, i32, etc.)
- `#[repr(C)]` structs of primitives
- Arrays of copy-optimized types

### 11. With - Wrapper Types

```rust
use rkyv::with::{Inline, Skip, Raw};

#[derive(Archive, Serialize)]
struct Data {
    #[with(Inline)]
    inline_string: String,  // Stored inline, not via pointer

    #[with(Skip)]
    computed: u64,  // Not serialized

    #[with(Raw)]
    raw_data: Vec<u8>,  // Stored without length prefix
}
```

**Common Wrappers:**

**Inline:**

- Store data inline instead of via pointer
- Reduces indirection
- Better cache locality
- Larger serialized size

**Guidance:** `Inline` is best for small, frequently accessed fields; keep large fields pointer-based to avoid bloat.

**Skip:**

- Don't serialize this field
- Use `Default::default()` on deserialize
- Useful for computed fields
- Reduces serialized size

**Raw:**

- Raw byte storage
- No length prefix
- Fixed-size or externally sized
- Maximum compactness

**Custom:**

- Implement `ArchiveWith` trait
- Full control over serialization
- Reusable transformation logic

### 12. Derive Macros

```rust
use rkyv::{Archive, Serialize, Deserialize};

#[derive(Archive, Serialize, Deserialize)]
#[rkyv(
    // Comparison traits between archived and original
    compare(PartialEq),
    compare(PartialOrd),

    // Derive traits on archived type
    derive(Debug, Clone),

    // Attributes for archived type
    attr(repr(C), derive(Copy)),
)]
struct MyStruct {
    field: u32,

    // Per-field attributes
    #[with(Inline)]
    text: String,
}
```

**Derive Options:**

**compare:**

- Generate comparison impls
- Between `T` and `Archived<T>`
- Useful for testing
- Available: `PartialEq`, `PartialOrd`

**derive:**

- Derive traits on archived type
- Available: `Debug`, `Clone`, `Copy`, etc.
- Passes through to generated type

**attr:**

- Apply attributes to archived type
- Common: `repr(C)`, `derive(Copy)`

### 13. Advanced Serialization

#### Custom Allocators

```rust
use rkyv::ser::allocator::{Arena, ArenaHandle};

struct MySerializer {
    arena: Arena,
    // ... other fields
}

impl MySerializer {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    fn serialize<T: Serialize<Self>>(&mut self, value: &T)
        -> Result<Vec<u8>, Error>
    {
        let mut handle = self.arena.acquire();
        to_bytes_with_alloc(value, &mut handle)
    }
}
```

**Benefits:**

- Reuse allocations
- Reduce allocation overhead
- Better memory locality
- Custom allocation strategies

#### Streaming Serialization

```rust
use rkyv::ser::Writer;

struct FileWriter {
    file: File,
}

impl Writer for FileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.file.write_all(bytes)?;
        Ok(())
    }
}
```

**Use Cases:**

- Large data sets
- Network streaming
- File I/O
- Memory-constrained environments

### 14. Place - Memory Safety

```rust
use rkyv::Place;

fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    // out is an uninitialized memory location
    // Must initialize all fields
    munge::munge!(let MyArchivedStruct { field1, field2 } = out);
    field1.write(value1);
    field2.write(value2);
}
```

**Safety:**

- Prevents uninitialized memory reads
- Enforces field-by-field initialization
- Uses `munge` crate for safety
- Compile-time guarantees

## Integration with Lithos System

### Recommended Use Cases

1. **Persistent State Snapshots**
   - Zero-copy access to historical state
   - Memory-map large ledgers
   - Instant "deserialization"
   - Cross-version compatibility

2. **Network Protocol**
   - Fast message encoding
   - Zero-copy decoding
   - Validation for untrusted data
   - Compact wire format

3. **Cache Serialization**
   - Serialize cache to disk
   - Memory-map on startup
   - Zero-copy access
   - Fast cold-start

4. **IPC (Inter-Process Communication)**
   - Shared memory segments
   - Zero-copy data sharing
   - Process-local pointers work
   - Minimal overhead

### Performance Optimization Strategies

1. **Choose Right API Level**
   - `access` for safety (validated)
   - `access_unchecked` for speed (trusted)
   - Custom allocator for control

2. **Format Features**
   - Use `unaligned` for mmap
   - Use `aligned` for in-memory
   - Match endianness to platform
   - Right pointer width for data size

3. **Minimize Indirection**
   - Use `#[with(Inline)]` for small data
   - Flatten structures when possible
   - Avoid deep nesting
   - Consider data layout

4. **Validation Strategy**
   - Validate at system boundaries
   - Skip validation for internal data
   - Batch validation when possible
   - Cache validated references

5. **Memory Mapping**
   - Use for large datasets
   - Combine with `unaligned` feature
   - Zero-copy across restarts
   - OS manages paging

### Benchmarking Notes

**Strengths:**

- Fastest "deserialization" (zero-copy)
- Excellent read performance
- Low memory overhead
- Good compression potential

**Considerations:**

- Serialization overhead
- Larger than some formats (depends)
- Validation cost (if used)
- Format stability requirements

## Code Examples

### Basic Zero-Copy Example

```rust
use rkyv::{
    access_unchecked, to_bytes, Archive, Deserialize, Serialize,
};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
struct Transaction {
    id: u64,
    amount: i64,
    description: String,
}

// Serialize
let tx = Transaction {
    id: 42,
    amount: 1000,
    description: "Payment".to_string(),
};

let bytes = to_bytes::<rkyv::rancor::Error>(&tx).unwrap();

// Zero-copy access (UNSAFE - we trust our own data)
let archived = unsafe {
    access_unchecked::<ArchivedTransaction>(&bytes)
};

// Direct field access, no deserialization
assert_eq!(archived.id, 42);
assert_eq!(archived.amount, 1000);
assert_eq!(archived.description.as_str(), "Payment");

// Deserialize back if needed
let deserialized: Transaction =
    rkyv::deserialize(archived).unwrap();
assert_eq!(deserialized, tx);
```

### Safe Access with Validation

```rust
use rkyv::{access, rancor::Error};

// Serialize
let bytes = to_bytes::<Error>(&tx)?;

// Safe access with validation
let archived = access::<ArchivedTransaction, Error>(&bytes)?;

// Now safe to use
println!("{}", archived.description.as_str());
```

### Memory-Mapped File

```rust
use memmap2::MmapOptions;
use std::fs::File;

// Write to file
let file = File::create("data.rkyv")?;
let bytes = to_bytes::<Error>(&large_dataset)?;
file.write_all(&bytes)?;

// Memory-map for zero-copy access
let file = File::open("data.rkyv")?;
let mmap = unsafe { MmapOptions::new().map(&file)? };

// Access without loading into memory
let archived = unsafe {
    access_unchecked::<ArchivedLargeDataset>(&mmap)
};

// Use directly from disk via OS page cache
for item in archived.items.iter() {
    process(item);  // No deserialization
}
```

### Custom With Wrapper

```rust
use rkyv::{with::ArchiveWith, Archive, Fallible, Serialize};
use std::collections::HashMap;

// Custom wrapper for HashMapcompression
struct Compressed;

impl<K, V> ArchiveWith<HashMap<K, V>> for Compressed
where
    K: Archive,
    V: Archive,
{
    type Archived = ArchivedCompressedMap<K::Archived, V::Archived>;
    type Resolver = CompressedMapResolver;

    fn resolve_with(
        field: &HashMap<K, V>,
        resolver: Self::Resolver,
        out: Place<Self::Archived>,
    ) {
        // Custom compression logic
    }
}

#[derive(Archive, Serialize)]
struct Data {
    #[with(Compressed)]
    map: HashMap<String, Vec<u8>>,
}
```

## Summary for Lithos

rkyv provides unmatched zero-copy performance through:

- Direct memory access to archived data
- No deserialization overhead
- Memory-mapping support
- Optional validation for safety
- Flexible format control
- Efficient collections
- Shared pointer deduplication
- Position-independent data

**Best suited for:** Scenarios requiring fastest possible read access, memory-mapped data, IPC, or where deserialization overhead is unacceptable.

**Key Decision Factors:**

- **Use rkyv when:** Read performance is critical, data is accessed frequently without modification, or zero-copy is essential
- **Consider alternatives when:** Data changes frequently, human-readability matters, or schema evolution is complex
- **Combine with redb:** Use rkyv for serialization format in redb for zero-copy database
- **Combine with moka:** Use rkyv to serialize cache entries for persistent caching

**Compatibility Reminder:** Explicitly choose format-control features early and treat changes as breaking for persisted data.
