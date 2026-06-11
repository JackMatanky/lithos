//! Index scope and scan configuration types.
//!
//! Defines the boundaries and options used to control which filesystem nodes
//! are eligible for indexing and how an indexing run is executed.

use crate::fs::path::PathKey;

/// Filters applied during a filesystem scan to include or exclude nodes.
///
/// Currently holds no fields; reserved for extension with glob patterns,
/// extension filters, and depth limits.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanFilters {}

/// The scope of an indexing operation.
///
/// `Full` covers the entire vault from its root. `Partial` restricts scanning
/// to a specific subtree identified by a vault-relative path key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexScope {
    /// Scan the full vault, applying the given filters.
    Full {
        /// Filters controlling node inclusion.
        filters: ScanFilters,
    },
    /// Scan a subtree rooted at `root`, applying the given filters.
    Partial {
        /// The vault-relative root of the partial scan.
        root: PathKey,
        /// Filters controlling node inclusion.
        filters: ScanFilters,
    },
}

/// Options that control the behaviour of a single indexing run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexOptions {
    /// Re-index all nodes even if they appear current.
    pub reindex: bool,
    /// Perform a dry run: discover nodes but do not persist index changes.
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    mod scan_scope {
        mod constructor {
            use crate::indexer::scan::{IndexOptions, IndexScope, ScanFilters};

            #[test]
            fn full_scope_wraps_filters() {
                let filters = ScanFilters::default();
                let scope = IndexScope::Full {
                    filters,
                };
                assert!(matches!(scope, IndexScope::Full { .. }));
            }

            #[test]
            fn partial_scope_stores_root_and_filters() {
                use crate::fs::path::PathKey;
                let root = PathKey::try_new("notes").unwrap();
                let filters = ScanFilters::default();
                let scope = IndexScope::Partial {
                    root: root.clone(),
                    filters,
                };
                assert!(matches!(scope, IndexScope::Partial { .. }));
                if let IndexScope::Partial {
                    root: r,
                    ..
                } = &scope
                {
                    assert_eq!(r.as_str(), "notes");
                }
            }

            #[test]
            fn options_defaults_are_false() {
                let opts = IndexOptions::default();
                assert!(!opts.reindex);
                assert!(!opts.dry_run);
            }

            #[test]
            fn options_can_be_set() {
                let opts = IndexOptions {
                    reindex: true,
                    dry_run: true,
                };
                assert!(opts.reindex);
                assert!(opts.dry_run);
            }
        }
    }
}
