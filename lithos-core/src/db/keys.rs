//! Type-safe database key newtypes.
//!
//! Prevents mixing up different key formats at compile time.
//! All key types format their underlying strings with appropriate namespacing.
//!
//! These types are internal (`pub(super)`) and are used within the db module
//! to ensure type safety when constructing database keys.
//!
//! # Key Types
//!
//! - [`NamespacedKey`] - For main data table (format: `"table:key"`)
//! - [`MultimapKey`] - For multimap tables (format: `"multimap:key"`)
//! - [`TablePrefix`] - For range scanning (format: `"table:"`)

use std::fmt::{self, Write as _};

use uuid::Uuid;

/// Namespaced key for main data table (format: `"table:key"`).
///
/// Used for single-value storage in the main `DATA_TABLE`.
/// Ensures all keys are properly namespaced to avoid collisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct NamespacedKey(String);

impl NamespacedKey {
    /// Create a namespaced key from table and string key.
    ///
    /// Formats as `"table:key"` with pre-allocated capacity.
    #[inline]
    #[must_use]
    pub(super) fn new(table: &str, key: &str) -> Self {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "String length arithmetic is safe and will not overflow"
        )]
        let mut s = String::with_capacity(table.len() + key.len() + 1);

        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut s, "{table}:{key}");

        Self(s)
    }

    /// Create a namespaced key from table and UUID.
    ///
    /// Formats as `"table:uuid"` with pre-allocated capacity.
    /// More efficient than converting UUID to string first.
    #[inline]
    #[must_use]
    pub(super) fn from_uuid(table: &str, id: Uuid) -> Self {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "String length arithmetic is safe and will not overflow"
        )]
        let mut s = String::with_capacity(table.len() + 37); // table + ":" + 36-char UUID

        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut s, "{table}:{id}");

        Self(s)
    }

    /// Get the key as a string slice.
    #[inline]
    #[must_use]
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for NamespacedKey {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NamespacedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Multimap key with namespace prefix (format: `"multimap:key"`).
///
/// Used for 1:N relationships in multimap tables.
/// The `"multimap:"` prefix distinguishes these from data table keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MultimapKey(String);

impl MultimapKey {
    /// Create a multimap key with namespace prefix.
    ///
    /// Formats as `"multimap:key"` with pre-allocated capacity.
    #[inline]
    #[must_use]
    pub(super) fn new(key: &str) -> Self {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "String length arithmetic is safe and will not overflow"
        )]
        let mut s = String::with_capacity(9 + key.len()); // "multimap:" + key

        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut s, "multimap:{key}");

        Self(s)
    }

    /// Get the key as a string slice.
    #[inline]
    #[must_use]
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for MultimapKey {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MultimapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Table prefix for range scanning (format: `"table:"`).
///
/// Used to scan all entries in a table via prefix matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct TablePrefix(String);

impl TablePrefix {
    /// Create a table prefix for scanning.
    ///
    /// Formats as `"table:"` with pre-allocated capacity.
    #[inline]
    #[must_use]
    pub(super) fn new(table: &str) -> Self {
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "String length arithmetic is safe and will not overflow"
        )]
        let mut s = String::with_capacity(table.len() + 1);

        #[expect(
            clippy::let_underscore_must_use,
            reason = "Writing to String is infallible"
        )]
        let _ = write!(&mut s, "{table}:");

        Self(s)
    }

    /// Get the prefix as a string slice.
    #[inline]
    #[must_use]
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TablePrefix {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TablePrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod namespaced_key {
        use super::*;

        mod construction {
            use super::*;

            #[test]
            fn new_formats_correctly() {
                let key = NamespacedKey::new("users", "alice");
                assert_eq!(key.as_str(), "users:alice");
            }

            #[test]
            fn new_handles_empty_key() {
                let key = NamespacedKey::new("table", "");
                assert_eq!(key.as_str(), "table:");
            }

            #[test]
            fn new_handles_empty_table() {
                let key = NamespacedKey::new("", "key");
                assert_eq!(key.as_str(), ":key");
            }

            #[test]
            fn new_handles_special_characters() {
                let key = NamespacedKey::new("my-table", "my:key/with:colons");
                assert_eq!(key.as_str(), "my-table:my:key/with:colons");
            }

            #[test]
            fn from_uuid_formats_correctly() {
                let id = Uuid::nil();
                let key = NamespacedKey::from_uuid("notes", id);
                assert_eq!(
                    key.as_str(),
                    "notes:00000000-0000-0000-0000-000000000000"
                );
            }

            #[test]
            fn from_uuid_handles_different_uuids() {
                let id =
                    Uuid::from_u128(0x12345678_9abc_def0_1234_567890abcdef);
                let key = NamespacedKey::from_uuid("items", id);
                assert_eq!(
                    key.as_str(),
                    "items:12345678-9abc-def0-1234-567890abcdef"
                );
            }
        }

        mod traits {
            use super::*;

            #[test]
            fn as_ref_str_works() {
                let key = NamespacedKey::new("table", "key");
                let s: &str = key.as_ref();
                assert_eq!(s, "table:key");
            }

            #[test]
            fn display_works() {
                let key = NamespacedKey::new("users", "alice");
                assert_eq!(format!("{key}"), "users:alice");
            }

            #[test]
            fn debug_works() {
                let key = NamespacedKey::new("table", "key");
                let debug_str = format!("{key:?}");
                assert!(debug_str.contains("NamespacedKey"));
                assert!(debug_str.contains("table:key"));
            }

            #[test]
            fn clone_works() {
                let key1 = NamespacedKey::new("table", "key");
                let key2 = key1.clone();
                assert_eq!(key1, key2);
                assert_eq!(key1.as_str(), key2.as_str());
            }

            #[test]
            fn equality_works() {
                let key1 = NamespacedKey::new("table", "key");
                let key2 = NamespacedKey::new("table", "key");
                let key3 = NamespacedKey::new("table", "other");

                assert_eq!(key1, key2);
                assert_ne!(key1, key3);
            }
        }
    }

    mod multimap_key {
        use super::*;

        mod construction {
            use super::*;

            #[test]
            fn new_formats_correctly() {
                let key = MultimapKey::new("tag:work");
                assert_eq!(key.as_str(), "multimap:tag:work");
            }

            #[test]
            fn new_handles_empty_key() {
                let key = MultimapKey::new("");
                assert_eq!(key.as_str(), "multimap:");
            }

            #[test]
            fn new_handles_special_characters() {
                let key = MultimapKey::new("user/alice:work");
                assert_eq!(key.as_str(), "multimap:user/alice:work");
            }

            #[test]
            fn new_adds_prefix_consistently() {
                let key1 = MultimapKey::new("a");
                let key2 = MultimapKey::new("ab");
                let key3 = MultimapKey::new("abc");

                assert!(key1.as_str().starts_with("multimap:"));
                assert!(key2.as_str().starts_with("multimap:"));
                assert!(key3.as_str().starts_with("multimap:"));
            }
        }

        mod traits {
            use super::*;

            #[test]
            fn as_ref_str_works() {
                let key = MultimapKey::new("tag");
                let s: &str = key.as_ref();
                assert_eq!(s, "multimap:tag");
            }

            #[test]
            fn display_works() {
                let key = MultimapKey::new("tag:work");
                assert_eq!(format!("{key}"), "multimap:tag:work");
            }

            #[test]
            fn debug_works() {
                let key = MultimapKey::new("tag");
                let debug_str = format!("{key:?}");
                assert!(debug_str.contains("MultimapKey"));
                assert!(debug_str.contains("multimap:tag"));
            }

            #[test]
            fn clone_works() {
                let key1 = MultimapKey::new("tag");
                let key2 = key1.clone();
                assert_eq!(key1, key2);
            }

            #[test]
            fn equality_works() {
                let key1 = MultimapKey::new("tag");
                let key2 = MultimapKey::new("tag");
                let key3 = MultimapKey::new("other");

                assert_eq!(key1, key2);
                assert_ne!(key1, key3);
            }
        }
    }

    mod table_prefix {
        use super::*;

        mod construction {
            use super::*;

            #[test]
            fn new_formats_correctly() {
                let prefix = TablePrefix::new("notes");
                assert_eq!(prefix.as_str(), "notes:");
            }

            #[test]
            fn new_handles_empty_table() {
                let prefix = TablePrefix::new("");
                assert_eq!(prefix.as_str(), ":");
            }

            #[test]
            fn new_handles_special_characters() {
                let prefix = TablePrefix::new("my-table_v2");
                assert_eq!(prefix.as_str(), "my-table_v2:");
            }
        }

        mod behavior {
            use super::*;

            #[test]
            fn can_match_keys_with_prefix() {
                let prefix = TablePrefix::new("users");

                assert!("users:alice".starts_with(prefix.as_str()));
                assert!("users:bob".starts_with(prefix.as_str()));
                assert!("users:".starts_with(prefix.as_str()));
                assert!(!"user:alice".starts_with(prefix.as_str()));
                assert!(!"products:widget".starts_with(prefix.as_str()));
            }
        }

        mod traits {
            use super::*;

            #[test]
            fn as_ref_str_works() {
                let prefix = TablePrefix::new("table");
                let s: &str = prefix.as_ref();
                assert_eq!(s, "table:");
            }

            #[test]
            fn display_works() {
                let prefix = TablePrefix::new("users");
                assert_eq!(format!("{prefix}"), "users:");
            }

            #[test]
            fn debug_works() {
                let prefix = TablePrefix::new("table");
                let debug_str = format!("{prefix:?}");
                assert!(debug_str.contains("TablePrefix"));
                assert!(debug_str.contains("table:"));
            }

            #[test]
            fn clone_works() {
                let prefix1 = TablePrefix::new("table");
                let prefix2 = prefix1.clone();
                assert_eq!(prefix1, prefix2);
            }

            #[test]
            fn equality_works() {
                let prefix1 = TablePrefix::new("table");
                let prefix2 = TablePrefix::new("table");
                let prefix3 = TablePrefix::new("other");

                assert_eq!(prefix1, prefix2);
                assert_ne!(prefix1, prefix3);
            }
        }
    }

    mod type_safety {
        use super::*;

        /// Compile-time test: Different key types cannot be mixed.
        #[test]
        fn different_types_are_distinct() {
            // Helper functions that only accept specific types
            fn accepts_namespaced(_key: &NamespacedKey) {}
            fn accepts_multimap(_key: &MultimapKey) {}
            fn accepts_prefix(_prefix: &TablePrefix) {}

            let ns_key = NamespacedKey::new("table", "key");
            let mm_key = MultimapKey::new("key");
            let prefix = TablePrefix::new("table");

            // These compile
            accepts_namespaced(&ns_key);
            accepts_multimap(&mm_key);
            accepts_prefix(&prefix);

            // These would NOT compile (demonstrates type safety):
            // accepts_namespaced(&mm_key);  // ✗ Type error
            // accepts_multimap(&ns_key);    // ✗ Type error
            // accepts_prefix(&ns_key);      // ✗ Type error
        }
    }
}
