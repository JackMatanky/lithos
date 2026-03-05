# Config Version Race Condition - Fix Plan

## Problem Statement

**CRITICAL**: `rebuild_merged()` has a race condition where concurrent rebuilds can allocate the same version number, causing data loss.

```rust
// Current broken implementation:
pub fn rebuild_merged(&self, vault_id: VaultId, vault_root: &VaultRoot) -> Result<Version, ConfigCommandError> {
    let version = self.next_version(vault_id)?;  // ← Thread A: scan → max=5, return 6
                                                   // ← Thread B: scan → max=5, return 6
    let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
    let merged = Config::build(&raw_merged, vault_id, vault_root.clone(), version)?;

    self.command_port.record_config(vault_id, &merged)?;  // ← Thread A: write v6
                                                            // ← Thread B: write v6 (overwrites!)

    Ok(version)
}
```

## Root Cause

The operation has three separate steps:
1. **Read**: Scan for max version
2. **Compute**: Calculate next = max + 1
3. **Write**: Persist config with computed version

These are NOT atomic, so concurrent calls can interleave.

## Fix Options

### Option 1: Atomic Record-with-Version-Allocation (RECOMMENDED)

Combine version allocation and record into a single atomic operation.

**Changes Required**:

1. **Add new method to `CommandState` trait**:
```rust
pub(crate) trait CommandState {
    /// Atomically allocates the next version and records the config.
    ///
    /// This is a read-modify-write operation that must be atomic to prevent
    /// version collisions from concurrent rebuilds.
    ///
    /// # Errors
    /// Returns error if version overflow or write fails.
    fn allocate_and_record(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error>;
}
```

2. **Implement in `CommandAdapter` using transaction**:
```rust
impl CommandState for CommandAdapter<'_> {
    fn allocate_and_record(
        &self,
        vault_id: VaultId,
        config: &Config,
    ) -> Result<Version, Self::Error> {
        self.db.read_write_unit_of_work(|tx| {
            // Scan for max version
            let prefix = format!("{vault_id}:");
            let max_version = tx
                .scan_range::<Config>(CONFIG_VERSIONS, &prefix)?
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
                    DbError::Serialization(
                        "version overflow - vault has exceeded maximum rebuilds".into()
                    )
                })?,
                None => Version::initial(),
            };

            // Write config with computed version
            let key = format!("{}:{}", vault_id, next.value());

            // CRITICAL: We need to update the config's version field!
            // But Config has a private version field, so we need a new constructor
            // or builder method that allows setting version during construction.

            // For now, assume Config is already built with correct version
            tx.put(CONFIG_VERSIONS, &key, config)?;

            Ok(next)
        })
    }
}
```

3. **Update `rebuild_merged` to use atomic operation**:
```rust
pub fn rebuild_merged(
    &self,
    vault_id: VaultId,
    vault_root: &VaultRoot,
) -> Result<Version, ConfigCommandError> {
    // Build config with PLACEHOLDER version (will be replaced atomically)
    let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
    let temp_version = Version::initial(); // Placeholder
    let merged = Config::build(&raw_merged, vault_id, vault_root.clone(), temp_version)
        .map_err(ConfigCommandError::Domain)?;

    // Record vault path mapping
    self.command_port
        .record_vault_path_mapping(vault_id, vault_root)
        .map_err(|error| ConfigCommandError::Storage(error.into()))?;

    // Atomically allocate version and record config
    let actual_version = self.command_port
        .allocate_and_record(vault_id, &merged)
        .map_err(|error| ConfigCommandError::Storage(error.into()))?;

    Ok(actual_version)
}
```

**Problem with Option 1**: Config has a `version` field that's set at construction time. The stored config will have the wrong version (placeholder). We need to either:
- A) Allow updating version in Config (violates immutability)
- B) Rebuild Config with correct version inside transaction (expensive)
- C) Store version separately from Config payload

### Option 2: Optimistic Locking (ALTERNATIVE)

Use optimistic locking to detect and retry on version collision.

**Changes Required**:

1. **Modify `record_config` to return Result<(), ConflictError>**:
```rust
fn record_config(
    &self,
    vault_id: VaultId,
    config: &Config,
) -> Result<(), Self::Error> {
    let key = format!("{}:{}", vault_id, config.version().value());

    // Check if this version already exists
    if self.db.exists(CONFIG_VERSIONS, &key)? {
        return Err(DbError::Conflict(
            format!("Version {} already exists for vault {}", config.version().value(), vault_id)
        ));
    }

    self.db.put(CONFIG_VERSIONS, &key, config)
}
```

2. **Add retry logic to `rebuild_merged`**:
```rust
pub fn rebuild_merged(
    &self,
    vault_id: VaultId,
    vault_root: &VaultRoot,
) -> Result<Version, ConfigCommandError> {
    const MAX_RETRIES: usize = 10;

    for attempt in 0..MAX_RETRIES {
        // Allocate next version
        let version = self.next_version(vault_id)?;

        // Build merged config with version
        let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
        let merged = Config::build(&raw_merged, vault_id, vault_root.clone(), version)
            .map_err(ConfigCommandError::Domain)?;

        // Record vault path mapping
        self.command_port
            .record_vault_path_mapping(vault_id, vault_root)
            .map_err(|error| ConfigCommandError::Storage(error.into()))?;

        // Try to record config
        match self.command_port.record_config(vault_id, &merged) {
            Ok(()) => return Ok(version),
            Err(e) if is_conflict_error(&e) => {
                // Version collision, retry
                tracing::warn!("Version collision on attempt {}, retrying", attempt);
                continue;
            }
            Err(e) => return Err(ConfigCommandError::Storage(e.into())),
        }
    }

    Err(ConfigCommandError::Storage(
        DbError::Conflict("Failed to allocate version after max retries".into())
    ))
}
```

**Problem with Option 2**: Requires adding conflict detection to `record_config`, which complicates the simple write operation.

### Option 3: Mutex/Lock (SIMPLE BUT SLOW)

Use a mutex to serialize version allocation.

**Changes Required**:

1. **Add mutex to CommandAdapter**:
```rust
pub struct CommandAdapter<'db> {
    db: &'db Database,
    version_lock: std::sync::Mutex<()>,
}
```

2. **Wrap version allocation**:
```rust
pub fn rebuild_merged(
    &self,
    vault_id: VaultId,
    vault_root: &VaultRoot,
) -> Result<Version, ConfigCommandError> {
    let _guard = self.version_lock.lock().unwrap();

    // Rest of implementation (safe now that it's serialized)
    let version = self.next_version(vault_id)?;
    let raw_merged = ingest::build_merged_raw(vault_root.as_path())?;
    let merged = Config::build(&raw_merged, vault_id, vault_root.clone(), version)?;

    self.command_port.record_config(vault_id, &merged)?;

    Ok(version)
}
```

**Problem with Option 3**: Global mutex serializes ALL vault rebuilds, not just per-vault. Could use `DashMap<VaultId, Mutex<()>>` for per-vault locking.

## Recommendation

**Use Option 2 (Optimistic Locking)** because:
1. ✅ No schema changes required
2. ✅ Preserves existing Config immutability
3. ✅ Only adds retry logic to facade layer
4. ✅ Works with existing transaction model
5. ✅ Graceful degradation (retries then fails)

## Implementation Steps

1. Add `DbError::Conflict` variant
2. Update `record_config` to check for existing key
3. Add retry logic to `rebuild_merged`
4. Update concurrency test to verify fix
5. Document retry behavior in API docs

## Alternative: Accept the Limitation

Document that concurrent rebuilds on the same vault are unsupported and rely on application-level locking (e.g., LSP server only rebuilds from one thread at a time).

This is actually reasonable because:
- Config rebuilds are rare (only when files change)
- LSP server is single-threaded for file watching
- CLI commands are typically one-at-a-time

**If we accept the limitation**:
- Document in API: "Concurrent rebuilds on the same vault are undefined behavior"
- Add warning in logs if detected
- Consider adding application-level lock in ConfigService
