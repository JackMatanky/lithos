# rkyv Corruption Bug Investigation

## Error Details

**Error Message**:
```
Storage(Deserialization("subtree pointer overran range:
ptr 0x00000008ae80c09e size 4294967295 in range 0x00000008ae80c000..0x00000008ae80c09f
trace: while checking field index 0 of tuple struct 'ArchivedTuple2'"))
```

**When It Occurs**:
- When saving 2+ schemas in a batch
- When listing schemas after saving multiple schemas
- Error happens during rkyv deserialization (validation phase)

**Key Observations**:
- Size field corrupted to `4294967295` (u32::MAX)
- Happens in same session (not reopen-related)
- First schema corrupted by saving second schema
- Error in `ArchivedTuple2` - likely the HashMap entry type

## Analysis

### 1. HashMap Serialization Issue

The error mentions `ArchivedTuple2` which suggests this is happening inside a HashMap. Looking at Schema:

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
pub struct Schema {
    id: SchemaId,
    name: SchemaName,
    parent_id: Option<SchemaId>,
    children: Vec<SchemaId>,
    properties: HashMap<PropertyName, Property>,  // <-- HashMap here
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}
```

The `properties` field is a `HashMap<PropertyName, Property>`.

### 2. Potential Causes

#### Theory 1: Redb Table Key Collision

**Hypothesis**: Using `schema.id().to_string()` as key might cause issues if:
- UUID stringification is not consistent
- Table insert is overwriting data incorrectly

**Evidence**: Let me check if two schemas might somehow get the same key.

#### Theory 2: rkyv HashMap Serialization Bug

**Hypothesis**: ArchivedHashMap has known issues with:
- Hash seed determinism
- Pointer calculations
- Entry serialization

**Evidence**: The error is in HashMap tuple entries.

#### Theory 3: Serialization Context Not Reset

**Hypothesis**: rkyv serializer context might be reusing shared pointer positions between saves.

**Evidence**: The error shows pointer addresses that seem to reference previous data.

#### Theory 4: redb WriteBatch Not Isolating Writes

**Hypothesis**: Multiple writes in a batch might share memory or have ordering issues.

**Evidence**: Works fine with 1 schema, fails with 2+ in batch.

#### Theory 5: ArchivedHashMap FxHash Collision

**Hypothesis**: FxHash (used by ArchivedHashMap) might have collisions leading to corrupt entries.

**Evidence**: FxHash is deterministic but collision-prone on specific inputs.

## Investigation Plan

### Step 1: Minimal Reproduction ✅ DONE

Created test that reliably reproduces with 2 schemas.

### Step 2: Simplify Schema to Find Root Cause

Create a test with minimal Schema (no HashMap):

```rust
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
struct SimpleSchema {
    id: SchemaId,
    name: SchemaName,
}
```

**Test**: Does this fail with 2 schemas?
- If YES: Problem is in redb batch write or schema ID handling
- If NO: Problem is in HashMap serialization

### Step 3: Test HashMap Directly

Test rkyv HashMap serialization independently:

```rust
let mut map1 = HashMap::new();
map1.insert(PropertyName::try_new("prop1")?, prop1);
let bytes1 = rkyv::to_bytes(&map1)?;

let mut map2 = HashMap::new();
map2.insert(PropertyName::try_new("prop2")?, prop2);
let bytes2 = rkyv::to_bytes(&map2)?;

// Can we deserialize both?
let archived1 = rkyv::access(&bytes1)?;
let archived2 = rkyv::access(&bytes2)?;
```

**Test**: Does direct HashMap serialization work?
- If YES: Problem is in how we store to redb
- If NO: rkyv HashMap bug or our HashMap usage

### Step 4: Check redb Write Isolation

Test redb batch writes with simple types:

```rust
let mut tx = db.begin_write()?;
{
    let mut table = tx.open_table(TEST_TABLE)?;
    table.insert("key1", b"value1")?;
    table.insert("key2", b"value2")?;
}
tx.commit()?;
```

**Test**: Can we reliably write 2 entries?
- If YES: redb is fine, problem is in our serialization
- If NO: redb bug (unlikely)

### Step 5: Inspect Serialized Bytes

Add debugging to see what bytes are actually written:

```rust
let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&schema)?;
println!("Schema {} serialized to {} bytes", schema.id(), bytes.len());
println!("First 32 bytes: {:?}", &bytes[..32.min(bytes.len())]);
```

**Test**: Are the bytes consistent? Do they overlap in addresses?

### Step 6: Check for Shared References

Schema has these potential shared/referenced fields:
- `HashMap<PropertyName, Property>` - both key and value are complex types
- `Vec<SchemaId>` - simple Copy type
- `Option<SchemaId>` - simple Copy type

PropertyName and Property might have issues:

```rust
pub struct PropertyName(Box<str>);  // Box<str> - should be fine
```

```rust
pub struct Property {
    id: PropertyId,
    name: PropertyName,
    optionality: Optionality,
    multiplicity: Multiplicity,
    spec: PropertySpec,  // <-- This is an enum with data
}
```

PropertySpec is complex:
```rust
pub enum PropertySpec {
    Bool(BoolSpec),
    Number(NumberSpec),
    String(StringSpec),
    Date(DateSpec),
    File(FileSpec),
}
```

Each spec has nested data (e.g., StringSpec has Vec<String> for options).

**Test**: Does the problem occur with simpler properties (e.g., empty HashMap)?

## Hypothesis Ranking

1. **HIGH**: rkyv HashMap serialization has ordering/pointer issues
2. **MEDIUM**: PropertySpec enum causing serialization context confusion
3. **MEDIUM**: redb batch write not properly isolating serializations
4. **LOW**: Schema ID collision
5. **LOW**: FxHash collision in ArchivedHashMap

## BREAKTHROUGH FINDINGS ✅

### Tests Created (lithos-core/tests/rkyv_debug.rs)

1. ✅ **two_simple_schemas_no_properties** - PASSES
2. ✅ **two_schemas_with_one_property** - PASSES
3. ✅ **two_schemas_saved_separately** - PASSES
4. ✅ **direct_rkyv_serialization_test** - PASSES
5. ✅ **test_property_name_as_hashmap_key** - PASSES
6. ✅ **test_full_deserialization** - PASSES
7. ❌ **test_list_schemas_behavior** - FAILS (reproduces bug!)
8. ✅ **test_sequential_deserialization** - PASSES

### Root Cause Identified 🎯

**The bug is NOT in**:
- ✅ rkyv serialization
- ✅ rkyv deserialization (individual)
- ✅ HashMap serialization
- ✅ PropertyName as HashMap key
- ✅ Saving multiple schemas to redb
- ✅ Loading individual schemas from redb

**The bug IS in**:
- ❌ **`scan_table_tx()` function in db/reader.rs**
- ❌ **Iteration + deserialization loop**
- ❌ Specifically: `repository.list_schemas()`

### The Smoking Gun

Test `test_list_schemas_behavior` exactly replicates `schema_list` and FAILS with the same error:

```
Error: subtree pointer overran range: ptr 0x0000000ba8c640a1 size 4294967295
in range 0x0000000ba8c64000..0x0000000ba8c640a2
trace: while checking field index 0 of tuple struct 'ArchivedTuple2'
```

But test `test_sequential_deserialization` which deserializes the SAME schemas in a loop PASSES!

### The Critical Difference

**Working code** (test_sequential_deserialization):
```rust
let bytes1 = rkyv::to_bytes(&schema1)?;
let bytes2 = rkyv::to_bytes(&schema2)?;
let all_bytes = vec![bytes1, bytes2];

for bytes in all_bytes.iter() {
    let archived = access(&bytes)?;
    let deserialized = deserialize(archived)?;
    results.push(deserialized);
}
// ✅ WORKS
```

**Broken code** (scan_table_tx in db/reader.rs:753-770):
```rust
for result in table_ref.iter()? {
    let (_key, value): (_, redb::AccessGuard<&[u8]>) = result?;
    let bytes: &[u8] = value.value();

    let mut aligned = AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);

    let archived = rkyv::access(&aligned)?;
    let deserialized = rkyv::deserialize(archived)?;
    results.push(deserialized);
}
// ❌ FAILS on 2nd iteration
```

### Hypothesis: redb AccessGuard Lifetime Issue

**Theory**: The `value` AccessGuard might be getting dropped/invalidated during iteration, causing the bytes to become corrupted.

**Evidence**:
- Line 756: `let bytes: &[u8] = value.value();` borrows from AccessGuard
- Line 759: We copy bytes to `aligned` buffer
- But maybe there's some UB with the AccessGuard lifecycle?

### Alternative Hypothesis: AlignedVec Reuse

**Theory**: Creating a new `AlignedVec` in each iteration might be causing memory corruption.

**Evidence**: Each iteration creates fresh `AlignedVec`, but maybe there's some shared state?

## Next Steps

1. ✅ Test if copying to AlignedVec is the issue
2. ✅ Test if AccessGuard lifetime is the issue
3. Try fix: Extend AccessGuard lifetime
4. Try fix: Reuse AlignedVec across iterations
5. Try fix: Use different deserialization approach

## Code Locations to Check

**Schema definition**: `lithos-core/src/schema/aggregate.rs:73-92`
**Storage save**: `lithos-core/src/schema/storage.rs:634-655`
**Serialization**: `lithos-core/src/db/writer.rs:539`
**Deserialization**: `lithos-core/src/db/reader.rs:449-465`

## References

- rkyv docs: https://docs.rs/rkyv/0.8.14/rkyv/
- rkyv HashMap: https://docs.rs/rkyv/0.8.14/rkyv/collections/swiss_table/map/struct.ArchivedHashMap.html
- redb docs: https://docs.rs/redb/
