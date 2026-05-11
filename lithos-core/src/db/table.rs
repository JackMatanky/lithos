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

/// Table with UUID keys implementing [`UuidV7DbType`].
///
/// This wrapper specializes in UUID-keyed tables to eliminate the performance
/// bottleneck of string formatting.
///
/// # Why this exists
///
/// Many domain IDs are [`UuidV7`](crate::utils::UuidV7)s. Formatting a UUID as
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

/// Table with string keys, typically representing file paths.
///
/// Uses `String` keys to support runtime path lookups (as opposed to
/// compile-time static strings). This is the correct choice for path-based
/// indices where keys come from filesystem discovery.
///
/// # Design Note
///
/// Earlier versions used `&'static str` keys, which prevented inserting runtime
/// paths. The "parse, don't validate" principle here means: validate paths are
/// proper at the call site, then store them as strings in the database.
pub struct PathTable<V: Value + 'static> {
    definition: TableDefinition<'static, String, V>,
}

impl<V: Value + 'static> PathTable<V> {
    /// Create a new path-keyed table definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::db::PathTable;
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
    pub const fn definition(&self) -> TableDefinition<'static, String, V> {
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
/// use lithos_core::db::Table;
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
    /// use lithos_core::db::Table;
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
}
