//! Type-safe table definition wrappers.
//!
//! This module provides newtype wrappers around redb table definitions to
//! enforce type safety and prevent mixing different table patterns (UUID keys,
//! path keys, etc.).
//!
//! All wrappers are `const fn` constructible, allowing zero-runtime-cost
//! compile-time table definitions.

use redb::{Key, MultimapTableDefinition, TableDefinition, Value};

use super::UuidV7DbType;
use crate::{DbRkyvType, events::EventId, path::DbPathKey};

/// Table with UUID keys implementing [`UuidV7DbType`].
///
/// This wrapper specializes in UUID-keyed tables to eliminate the performance
/// bottleneck of string formatting.
///
/// # Why this exists
///
/// Many domain IDs are [`UuidV7`](traces_utils::UuidV7)s. Formatting a UUID as
/// a 36-byte string for every lookup is expensive in hot paths (e.g., LSP
/// indexing). This wrapper allows using the 16-byte raw UUID representation
/// directly as a key.
pub struct UuidTable<K: UuidV7DbType + 'static, V: Value + 'static> {
    definition: TableDefinition<'static, K, V>,
}

impl<K: UuidV7DbType + 'static, V: Value + 'static> UuidTable<K, V> {
    /// Create a new UUID-keyed table definition.
    ///
    /// Key type must implement `UuidV7DbType` (typically via `impl_redb_uuid!`
    /// macro).
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    ///
    /// Use this to open the table in a transaction.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, K, V> {
        self.definition
    }
}

/// Multimap table with UUID keys implementing [`UuidV7DbType`].
///
/// Specialized for 1:N relationships indexed by UUID.
///
/// Use this when a single domain ID (e.g., `SchemaId`) needs to look up
/// multiple related values (e.g., `NoteId`s) without performing multiple scans.
pub struct UuidMultimap<K: UuidV7DbType + 'static, V: Key + 'static> {
    definition: MultimapTableDefinition<'static, K, V>,
}

impl<K: UuidV7DbType + 'static, V: Key + 'static> UuidMultimap<K, V> {
    /// Create a new UUID-keyed multimap table definition.
    ///
    /// Key type must implement `UuidV7DbType` (typically via `impl_redb_uuid!`
    /// macro).
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: MultimapTableDefinition::new(name),
        }
    }

    /// Get the underlying redb multimap table definition.
    ///
    /// Use this to open the table in a transaction.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> MultimapTableDefinition<'static, K, V> {
        self.definition
    }
}

/// Table with `DbPathKey` keys, typically representing vault-relative file
/// paths.
///
/// Uses `DbPathKey` directly as the redb key type (requires `DbPathKey` to
/// implement `redb::Key` and `redb::Value`). This enforces type safety: only
/// normalized, validated paths can be stored.
///
/// # Design Note
///
/// Earlier versions used `String` keys, requiring manual `.to_owned()`
/// conversions. With `DbPathKey` implementing redb traits, we can store and
/// retrieve paths directly without string allocation.
pub struct PathTable<V: Value + 'static> {
    definition: TableDefinition<'static, DbPathKey, V>,
}

impl<V: Value + 'static> PathTable<V> {
    /// Create a new path-keyed table definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_db::PathTable;
    /// # use redb::TableDefinition;
    ///
    /// const FILES: PathTable<&str> = PathTable::new("file_metadata");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    ///
    /// Use this to open the table in a transaction.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, DbPathKey, V> {
        self.definition
    }
}

/// Table mapping `DbPathKey` → UUID (forward index).
///
/// Use this for path-based lookups where filesystem paths are the query key.
/// Typical use case: finding entity IDs by their vault-relative paths.
///
/// # Examples
///
/// ```ignore
/// use traces_db::PathUuidTable;
/// use traces_vault::model::FileId;
///
/// const FILE_ID_BY_PATH: PathUuidTable<FileId> =
///     PathUuidTable::new("file_id_by_path");
/// ```
pub struct PathUuidTable<V: UuidV7DbType + 'static> {
    definition: TableDefinition<'static, DbPathKey, V>,
}

impl<V: UuidV7DbType + 'static> PathUuidTable<V> {
    /// Create a new `DbPathKey` → UUID table definition.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, DbPathKey, V> {
        self.definition
    }
}

/// Table mapping UUID → `DbPathKey` (reverse index).
///
/// Use this for ID-to-path lookups, enabling O(1) path recovery during delete
/// operations and ID-based queries.
///
/// # Examples
///
/// ```ignore
/// use traces_db::UuidPathTable;
/// use traces_vault::model::FileId;
///
/// const PATH_BY_FILE_ID: UuidPathTable<FileId> =
///     UuidPathTable::new("path_by_file_id");
/// ```
pub struct UuidPathTable<K: UuidV7DbType + 'static> {
    definition: TableDefinition<'static, K, DbPathKey>,
}

impl<K: UuidV7DbType + 'static> UuidPathTable<K> {
    /// Create a new UUID → `DbPathKey` table definition.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, K, DbPathKey> {
        self.definition
    }
}

/// Generic table with any redb key/value types.
///
/// Fallback wrapper for table definitions that don't fit specialized patterns
/// (UUID keys, path keys, etc.).
///
/// # Examples
///
/// ```
/// use traces_db::Table;
/// # use redb::TableDefinition;
///
/// const COUNTERS: Table<&str, u64> = Table::new("counters");
/// ```
pub struct Table<K: Key + 'static, V: Value + 'static> {
    definition: TableDefinition<'static, K, V>,
}

impl<K: Key + 'static, V: Value + 'static> Table<K, V> {
    /// Create a new generic table definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_db::Table;
    /// # use redb::TableDefinition;
    ///
    /// const SETTINGS: Table<&str, &str> = Table::new("app_settings");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    ///
    /// Use this to open the table in a transaction.
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> TableDefinition<'static, K, V> {
        self.definition
    }
}

/// Table storing typed rkyv values.
pub struct RkyvTable<K: crate::RkyvDecode + Ord + 'static, V: 'static> {
    definition: TableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>>,
}

impl<K: crate::RkyvDecode + Ord + 'static, V: 'static> RkyvTable<K, V> {
    /// Create a new table definition with rkyv-encoded values.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(
        &self,
    ) -> TableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>> {
        self.definition
    }
}

/// Multimap storing typed rkyv values.
pub struct RkyvMultimap<
    K: crate::RkyvDecode + Ord + 'static,
    V: crate::RkyvDecode + Ord + 'static,
> {
    definition: MultimapTableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>>,
}

impl<K: crate::RkyvDecode + Ord + 'static, V: crate::RkyvDecode + Ord + 'static>
    RkyvMultimap<K, V>
{
    /// Create a new multimap definition with rkyv-encoded values.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: MultimapTableDefinition::new(name),
        }
    }

    /// Get the underlying redb multimap table definition.
    #[inline]
    #[must_use]
    pub const fn definition(
        &self,
    ) -> MultimapTableDefinition<'static, DbRkyvType<K>, DbRkyvType<V>> {
        self.definition
    }
}

/// Table with [`EventId`] keys.
///
/// Infrastructure primitive for event-log table definitions keyed by
/// monotonic event identifiers.
#[allow(
    dead_code,
    reason = "Infrastructure wrapper consumed in upcoming slices"
)]
pub struct EventTable<V: 'static> {
    definition: TableDefinition<'static, EventId, DbRkyvType<V>>,
}

#[allow(
    dead_code,
    reason = "Infrastructure wrapper API consumed in upcoming slices"
)]
impl<V: 'static> EventTable<V> {
    /// Create a new event-id keyed table definition.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            definition: TableDefinition::new(name),
        }
    }

    /// Get the underlying redb table definition.
    #[inline]
    #[must_use]
    pub const fn definition(
        &self,
    ) -> TableDefinition<'static, EventId, DbRkyvType<V>> {
        self.definition
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod path_table {
        use super::*;

        /// `PathTable::new()` creates const path-keyed table.
        ///
        /// Behavior: Compile-time table construction with zero runtime cost.
        /// Verification: Create const table, compile successfully.
        #[test]
        fn const_construction_works() {
            const TABLE: PathTable<&str> = PathTable::new("paths");
            let _def = TABLE.definition();
            // If this compiles, const construction works
        }

        /// `PathTable<V>` uses `DbPathKey` as key type, not `String`.
        ///
        /// Behavior: Type safety - only validated, normalized paths can be
        /// stored. Verification: `definition()` returns
        /// `TableDefinition<DbPathKey, V>`.
        #[test]
        fn accepts_pathkey_as_key_type() {
            use crate::path::DbPathKey;

            const TABLE: PathTable<u64> = PathTable::new("test");

            // This should compile with DbPathKey as key type
            let _def: TableDefinition<'static, DbPathKey, u64> =
                TABLE.definition();
        }
    }

    mod uuid_path_table {
        use traces_utils::UuidV7;

        use super::*;
        use crate::{impl_redb_uuid, path::DbPathKey};

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        struct TestId(UuidV7);
        impl_redb_uuid!(TestId);

        /// `UuidPathTable` inserts and retrieves UUID→`DbPathKey` mappings.
        ///
        /// Behavior: Store UUID keys with `DbPathKey` values, enabling reverse
        /// path lookups. Verification: Insert ID→path, retrieve by ID.
        #[test]
        fn inserts_and_retrieves_uuid_to_pathkey() {
            use redb::ReadableDatabase;

            const TABLE: UuidPathTable<TestId> = UuidPathTable::new("test");

            let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
            let db = redb::Database::create(tmp.path()).expect("db");
            let id = TestId(UuidV7::new());
            let key = DbPathKey::try_new("notes/test.md").expect("key");

            // Write
            let write_tx = db.begin_write().expect("tx");
            {
                let mut table =
                    write_tx.open_table(TABLE.definition()).expect("open");
                table.insert(&id, &key).expect("insert");
            }
            write_tx.commit().expect("commit");

            // Read
            let read_tx = db.begin_read().expect("tx");
            let table = read_tx.open_table(TABLE.definition()).expect("open");
            let retrieved = table.get(&id).expect("get").expect("value");

            assert_eq!(retrieved.value(), key);
        }

        /// `UuidPathTable` supports path recovery for delete operations.
        ///
        /// Behavior: Reverse index use case - given file ID, find its path for
        /// cleanup. Verification: Store ID→path, recover path by ID.
        #[test]
        fn supports_path_recovery_for_deletes() {
            use redb::ReadableDatabase;

            const TABLE: UuidPathTable<TestId> = UuidPathTable::new("test");

            // Demonstrates reverse index use case
            let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
            let db = redb::Database::create(tmp.path()).expect("db");
            let id = TestId(UuidV7::new());
            let key = DbPathKey::try_new("notes/daily.md").expect("key");

            // Store
            let write_tx = db.begin_write().expect("tx");
            {
                let mut table =
                    write_tx.open_table(TABLE.definition()).expect("open");
                table.insert(&id, &key).expect("insert");
            }
            write_tx.commit().expect("commit");

            // Recover path by ID (for delete operations)
            let read_tx = db.begin_read().expect("tx");
            let table = read_tx.open_table(TABLE.definition()).expect("open");
            let recovered_path =
                table.get(&id).expect("get").expect("value").value();

            assert_eq!(recovered_path, key);
        }
    }

    mod table {
        use super::*;

        /// `Table::new()` creates generic const table definition.
        ///
        /// Behavior: Compile-time table construction with zero runtime cost.
        /// Verification: Create const table, compile successfully.
        #[test]
        fn const_construction_works() {
            const TABLE: Table<&str, u64> = Table::new("counters");
            let _def = TABLE.definition();
            // If this compiles, const construction works
        }

        /// Table works with any redb Key and Value types.
        ///
        /// Behavior: Table wrapper is generic over redb Key and Value types.
        /// Verification: Multiple type combinations compile.
        #[test]
        fn supports_generic_key_value_types() {
            const _STR_TABLE: Table<&str, &str> = Table::new("strings");
            const _U64_TABLE: Table<u64, &str> = Table::new("by_id");
            // If these compile, generic types work correctly
        }
    }

    mod rkyv_table {
        use super::*;
        use crate::DbRkyvType;

        #[test]
        fn constructor_returns_db_rkyv_type_table_definition() {
            const TABLE: RkyvTable<u64, u32> = RkyvTable::new("entities");

            let _definition: TableDefinition<
                'static,
                DbRkyvType<u64>,
                DbRkyvType<u32>,
            > = TABLE.definition();
        }

        #[test]
        fn multimap_constructor_returns_db_rkyv_type_definition() {
            const TABLE: RkyvMultimap<u64, u32> =
                RkyvMultimap::new("entity_index");

            let _definition: MultimapTableDefinition<
                'static,
                DbRkyvType<u64>,
                DbRkyvType<u32>,
            > = TABLE.definition();
        }
    }

    mod event_table {
        use super::*;
        use crate::{DbRkyvType, EventId, RkyvBytes};

        #[test]
        fn constructor_returns_eventid_keyed_table_definition() {
            const EVENTS: EventTable<u64> = EventTable::new("events");

            let _definition: TableDefinition<
                'static,
                EventId,
                DbRkyvType<u64>,
            > = EVENTS.definition();
        }

        #[test]
        fn constructor_supports_const_construction() {
            const EVENTS: EventTable<u64> = EventTable::new("events");

            let _definition = EVENTS.definition();
        }

        #[test]
        fn definition_supports_eventid_keyed_integration_flow() {
            use redb::ReadableDatabase;

            const EVENTS: EventTable<u64> = EventTable::new("events");

            let temp = tempfile::NamedTempFile::new().expect("tmpfile");
            let db = redb::Database::create(temp.path()).expect("db");
            let event_id = EventId::try_from_raw(1).expect("event id");
            let event = RkyvBytes::<u64>::encode(&42).expect("encode");

            let write_tx = db.begin_write().expect("tx");
            {
                let mut table =
                    write_tx.open_table(EVENTS.definition()).expect("open");
                table.insert(&event_id, &event).expect("insert");
            }
            write_tx.commit().expect("commit");

            let read_tx = db.begin_read().expect("tx");
            let table = read_tx.open_table(EVENTS.definition()).expect("open");
            let stored = table.get(&event_id).expect("get").expect("value");

            assert_eq!(stored.value().decode().expect("decode"), 42);
        }
    }
}
