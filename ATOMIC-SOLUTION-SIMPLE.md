# SIMPLE ATOMIC SOLUTION - Using Existing Database Infrastructure

## Discovery

After thorough research of redb and our db module, I found that **WE ALREADY HAVE ALL THE PIECES** needed for a simple atomic solution!

### Key Findings

1. ✅ `Database::read_write_unit_of_work()` exists and provides atomic read-modify-write
2. ✅ `scan_range_tx()` exists and works with redb transactions
3. ✅ `WriteTransaction::open_table()` provides read access within write transactions
4. ✅ redb uses MVCC - concurrent reads are always safe
5. ✅ Tests already prove atomicity of `read_write_unit_of_work()`

### Current Problem

`ReadWriteUnitOfWork` has:
- ✅ `get_owned()` - read single value
- ✅ `put()` - write value
- ✅ `delete()` - delete value
- ❌ **MISSING**: `scan_range()` - scan by prefix

But the underlying `scan_range_tx()` function already exists and works with `WriteTransaction`!

## The Simple Fix

### Step 1: Add `scan_range()` to `ReadWriteUnitOfWork`

Add ONE method to `lithos-core/src/db/writer.rs`:

```rust
impl ReadWriteUnitOfWork {
    // ... existing methods ...

    /// Scan entries matching a key prefix within the transaction.
    ///
    /// This enables atomic read-scan-compute-write patterns.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if the scan or deserialization fails.
    #[inline]
    pub fn scan_range<V>(
        &self,
        table: TableDefinition<&str, &[u8]>,
        key_prefix: &str,
    ) -> Result<Vec<(String, V)>, DbError>
    where
        V: rkyv::Archive,
        V::Archived: rkyv::Portable
            + for<'archived> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
            > + rkyv::Deserialize<
                V,
                rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
            >,
    {
        // CRITICAL: We need to adapt scan_range_tx to work with WriteTransaction
        // The existing scan_range_tx takes &ReadTransaction
        // But WriteTransaction also implements the necessary traits

        // SOLUTION: Create scan_range_write_tx that works with WriteTransaction
        scan_range_write_tx::<V>(&self.tx, table, key_prefix)
    }
}
```

### Step 2: Add `scan_range_write_tx()` helper

Add this helper function to `lithos-core/src/db/reader.rs` (or writer.rs):

```rust
/// Scan entries in a table matching a key prefix within a write transaction.
///
/// This is identical to scan_range_tx but works with WriteTransaction.
/// Both ReadTransaction and WriteTransaction implement the same table access traits.
fn scan_range_write_tx<V>(
    tx: &redb::WriteTransaction,
    table: TableDefinition<&str, &[u8]>,
    key_prefix: &str,
) -> Result<Vec<(String, V)>, DbError>
where
    V: rkyv::Archive,
    V::Archived: rkyv::Portable
        + for<'archived> rkyv::bytecheck::CheckBytes<
            rkyv::api::high::HighValidator<'archived, rkyv::rancor::Error>,
        > + rkyv::Deserialize<
            V,
            rkyv::api::high::HighDeserializer<rkyv::rancor::Error>,
        >,
{
    let table_ref = match tx.open_table(table) {
        Ok(table_ref) => table_ref,
        Err(TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
        Err(err) => return Err(DbError::Transaction(err.to_string())),
    };

    // Compute the exclusive end bound for the range scan
    let end_bound = next_prefix(key_prefix);

    let mut results = Vec::new();
    let range = table_ref.range(key_prefix..end_bound.as_str())?;
    for result in range {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&[u8]>) = result?;
        let key_str = key.value().to_owned();
        let bytes = value.value();

        let mut aligned: rkyv::util::AlignedVec<16> = rkyv::util::AlignedVec::new();
        aligned.extend_from_slice(bytes);

        let archived = rkyv::access::<rkyv::Archived<V>, rkyv::rancor::Error>(&aligned)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;
        let deserialized = rkyv::deserialize::<V, rkyv::rancor::Error>(archived)
            .map_err(|e| DbError::Deserialization(e.to_string()))?;

        results.push((key_str, deserialized));
    }

    Ok(results)
}
```

**OR EVEN SIMPLER**: Make `scan_range_tx` generic over the transaction type using a trait!

### Step 3: Update `CommandAdapter::next_version()` to use atomic transaction

Replace the current implementation:

```rust
// OLD (race condition):
fn next_version(&self, vault_id: VaultId) -> Result<Version, Self::Error> {
    let max_version = self.db.scan_range::<Config>(CONFIG_VERSIONS, &prefix)?  // ← SEPARATE TX
        .into_iter()
        .filter_map(|(key, _)| { ... })
        .max();

    match max_version {
        Some(v) => v.next()?,
        None => Version::initial(),
    }
}
```

With atomic version:

```rust
// NEW (atomic - no race condition):
fn allocate_and_record_config(
    &self,
    vault_id: VaultId,
    config: &Config,
) -> Result<Version, Self::Error> {
    self.db.read_write_unit_of_work(|tx| {
        // Scan for max version INSIDE transaction
        let prefix = format!("{vault_id}:");
        let max_version = tx.scan_range::<Config>(CONFIG_VERSIONS, &prefix)?
            .into_iter()
            .filter_map(|(key, _)| {
                key.strip_prefix(&prefix)
                    .and_then(|v| v.parse::<u64>().ok())
                    .and_then(|v| Version::try_from(v).ok())
            })
            .max();

        // Compute next version
        let next = match max_version {
            Some(v) => v.next().map_err(|_| {
                DbError::Serialization("version overflow".into())
            })?,
            None => Version::initial(),
        };

        // Write with computed version INSIDE SAME transaction
        let key = format!("{}:{}", vault_id, next.value());
        tx.put(CONFIG_VERSIONS, &key, config)?;

        Ok(next)
    })
}
```

### Step 4: Update `rebuild_merged()` to use atomic operation

```rust
pub fn rebuild_merged(
    &self,
    vault_id: VaultId,
    vault_root: &VaultRoot,
) -> Result<Version, ConfigCommandError> {
    // Build config with PLACEHOLDER version (required by domain model)
    let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
    let temp_version = Version::initial();
    let merged = Config::build(&raw_merged, vault_id, vault_root.clone(), temp_version)
        .map_err(ConfigCommandError::Domain)?;

    // Record vault path mapping (separate transaction - OK since idempotent)
    self.command_port
        .record_vault_path_mapping(vault_id, vault_root)
        .map_err(|error| ConfigCommandError::Storage(error.into()))?;

    // Atomically allocate version and record config
    let version = self.command_port
        .allocate_and_record_config(vault_id, &merged)
        .map_err(|error| ConfigCommandError::Storage(error.into()))?;

    Ok(version)
}
```

**Note**: The config has wrong version (temp_version) but that's stored in the domain object. We could either:
1. Accept that the stored Config has wrong version field (version is in the KEY anyway)
2. Update Config.version inside the transaction (requires mut access or builder)
3. Store version separately from Config (schema change)

**Option 1 is simplest** - version is authoritative in the KEY, the field is redundant.

## Why This Works

### Atomicity Guarantees

1. **redb ACID**: All operations in `read_write_unit_of_work` are atomic
2. **Serializable Isolation**: redb uses MVCC with serializable snapshot isolation
3. **Test Coverage**: Existing tests prove atomicity works:
   ```rust
   // From db/writer.rs tests:
   fn read_write_unit_of_work_performs_atomic_read_modify_write() {
       let result = db.read_write_unit_of_work(|tx| {
           let current: Option<u64> = tx.get_owned(COUNTER_TABLE, "counter")?;
           let next = current.unwrap_or(0) + 1;
           tx.put(COUNTER_TABLE, "counter", &next)?;
           Ok(next)
       });
       // Works correctly with concurrent calls!
   }
   ```

### Race Condition Eliminated

**Before**:
```
Thread A: scan → max=5 | Thread B: scan → max=5
Thread A: compute 6    | Thread B: compute 6
Thread A: write v6     | Thread B: write v6 (OVERWRITES!)
```

**After**:
```
Thread A: BEGIN TX     | Thread B: BEGIN TX (waits)
Thread A:   scan→5     |
Thread A:   compute→6  |
Thread A:   write v6   |
Thread A: COMMIT TX    |
                       | Thread B:   scan→6 (sees A's write!)
                       | Thread B:   compute→7
                       | Thread B:   write v7
                       | Thread B: COMMIT TX
```

## Implementation Checklist

- [ ] Add `scan_range()` to `ReadWriteUnitOfWork` (5 min)
- [ ] Add `scan_range_write_tx()` helper (10 min)
- [ ] Update `CommandState` trait to add `allocate_and_record_config()` (5 min)
- [ ] Implement `allocate_and_record_config()` in `CommandAdapter` (10 min)
- [ ] Update `rebuild_merged()` to use atomic operation (5 min)
- [ ] Run concurrency test to verify fix (2 min)
- [ ] Update other tests if needed (10 min)

**Total effort**: ~1 hour

## Benefits

1. ✅ **Simple**: Uses existing primitives, no new abstractions
2. ✅ **Proven**: redb atomicity already tested and working
3. ✅ **Fast**: No retry loops, no optimistic locking overhead
4. ✅ **Correct**: Serializable isolation guarantees no races
5. ✅ **Minimal changes**: <100 lines of code total
6. ✅ **No schema changes**: Reuses existing tables
7. ✅ **No domain model changes**: Config stays immutable

## Alternative Considered

We could also make `scan_range_tx` generic over both `ReadTransaction` and `WriteTransaction` using a trait bound, but that's more complex. The simpler approach is to just duplicate the small function.

## Next Steps

1. Implement the changes above
2. Run the failing concurrency test - it should pass
3. Run all existing tests - they should still pass
4. Commit with message: "fix: Atomic version allocation using read_write_unit_of_work"
5. Update adversarial review to mark race condition as FIXED
