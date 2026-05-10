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
/// Enforces type safety by requiring keys to implement the UUID marker trait.
///
/// Use with domain ID wrapper types (e.g., `SchemaId`, `NoteId`) that implement
/// `UuidV7DbType` via the `impl_redb_uuid!` macro.
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
/// Allows multiple values per key. Enforces type safety by requiring keys
/// to implement the UUID marker trait.
///
/// Note: Multimap values must implement `redb::Key` (not `Value`).
///
/// Use with domain ID wrapper types (e.g., `SchemaId`, `NoteId`) that implement
/// `UuidV7DbType` via the `impl_redb_uuid!` macro.
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

/// Table with static string keys (typically file paths).
///
/// Specialized for `&'static str` keys, commonly used for file path lookups.
///
/// # Examples
///
/// ```
/// use lithos_core::db::PathTable;
///
/// const PATHS: PathTable<&str> = PathTable::new("file_paths");
/// ```
pub struct PathTable<V: Value + 'static> {
    definition: TableDefinition<'static, &'static str, V>,
}

impl<V: Value + 'static> PathTable<V> {
    /// Create a new path-keyed table definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::db::PathTable;
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
    pub const fn definition(
        &self,
    ) -> TableDefinition<'static, &'static str, V> {
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
