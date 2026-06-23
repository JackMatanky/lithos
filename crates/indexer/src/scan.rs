//! Index scope and scan configuration types.
//!
//! Defines the boundaries and options used to control which filesystem nodes
//! are eligible for indexing and how an indexing run is executed.

use trace_fs::DirPath;

/// Filters applied during a filesystem scan to include or exclude nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanFilters {
    /// File extensions to include, without a leading dot. Empty means all
    /// files.
    included_extensions: Box<[Box<str>]>,
    /// Exact directory or file names to exclude (e.g., `[".git",
    /// "node_modules"]`).
    excluded_names: Box<[Box<str>]>,
}

impl ScanFilters {
    /// Creates a new `ScanFilters` with the given inclusion and exclusion
    /// lists.
    #[inline]
    #[must_use]
    pub(crate) fn new(
        included_extensions: Vec<Box<str>>,
        excluded_names: Vec<Box<str>>,
    ) -> Self {
        Self {
            included_extensions: included_extensions.into_boxed_slice(),
            excluded_names: excluded_names.into_boxed_slice(),
        }
    }

    /// Returns true when `ext` matches an included extension (or no extension
    /// filter is configured).
    pub(crate) fn is_included_extension(&self, ext: &str) -> bool {
        self.included_extensions.is_empty()
            || self.included_extensions.iter().any(|e| e.as_ref() == ext)
    }

    /// Returns true when `name` matches an excluded entry name.
    pub(crate) fn is_excluded_name(&self, name: &str) -> bool {
        self.excluded_names.iter().any(|n| n.as_ref() == name)
    }
}

/// The scope of an indexing operation.
///
/// `Full` covers the entire vault from its root. `Partial` restricts scanning
/// to a specific subtree identified by a vault-relative path key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexScope {
    /// Scan the full vault, applying the given filters.
    Full {
        /// The root directory of the vault.
        root: DirPath,
        /// Filters controlling node inclusion.
        filters: ScanFilters,
    },
    /// Scan a subtree rooted at `root`, applying the given filters.
    Partial {
        /// The root of the partial scan (concrete OS path).
        root: DirPath,
        /// Filters controlling node inclusion.
        filters: ScanFilters,
    },
}

/// Options that control the behaviour of a single indexing run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexOptions {
    /// Re-index all nodes even if they appear current.
    reindex: bool,
    /// Perform a dry run: discover nodes but do not persist index changes.
    dry_run: bool,
}

impl IndexOptions {
    /// Creates a new `IndexOptions`.
    #[inline]
    #[must_use]
    pub(crate) fn new(reindex: bool, dry_run: bool) -> Self {
        Self {
            reindex,
            dry_run,
        }
    }

    /// Whether to re-index all nodes even if they appear current.
    #[inline]
    #[must_use]
    pub(crate) const fn reindex(self) -> bool {
        self.reindex
    }

    /// Whether to perform a dry run.
    #[inline]
    #[must_use]
    pub(crate) const fn dry_run(self) -> bool {
        self.dry_run
    }
}

impl IndexScope {
    /// Returns the root directory for the scan.
    #[inline]
    #[must_use]
    pub(crate) fn root(&self) -> &DirPath {
        match self {
            Self::Full {
                root,
                ..
            }
            | Self::Partial {
                root,
                ..
            } => root,
        }
    }

    /// Returns the scan filters.
    #[inline]
    #[must_use]
    pub(crate) fn filters(&self) -> &ScanFilters {
        match self {
            Self::Full {
                filters,
                ..
            }
            | Self::Partial {
                filters,
                ..
            } => filters,
        }
    }
}

#[cfg(test)]
mod tests {
    mod scan_filters {
        mod constructor {
            use crate::scan::ScanFilters;

            #[test]
            fn excludes_files_when_extension_does_not_match() {
                let filters = ScanFilters::new(vec!["md".into()], vec![]);
                assert!(filters.is_included_extension("md"));
                assert!(!filters.is_included_extension("toml"));
            }

            #[test]
            fn includes_all_when_no_extension_filter() {
                let filters = ScanFilters::default();
                assert!(filters.is_included_extension("anything"));
            }

            #[test]
            fn excludes_entries_when_name_is_excluded() {
                let filters = ScanFilters::new(vec![], vec![
                    ".git".into(),
                    "node_modules".into(),
                ]);
                assert!(filters.is_excluded_name(".git"));
                assert!(filters.is_excluded_name("node_modules"));
                assert!(!filters.is_excluded_name("src"));
            }
        }
    }

    mod index_scope {
        mod constructor {
            use trace_fs::DirPath;

            use crate::scan::{IndexOptions, IndexScope, ScanFilters};

            #[test]
            fn full_scope_wraps_root_and_filters() {
                let tmp = tempfile::TempDir::new().unwrap();
                let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
                let filters = ScanFilters::default();
                let scope = IndexScope::Full {
                    root: root.clone(),
                    filters,
                };
                assert!(matches!(scope, IndexScope::Full { .. }));
            }

            #[test]
            fn partial_scope_stores_root_and_filters() {
                let tmp = tempfile::TempDir::new().unwrap();
                let root = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
                let filters = ScanFilters::default();
                let scope = IndexScope::Partial {
                    root: root.clone(),
                    filters,
                };
                assert!(matches!(scope, IndexScope::Partial { .. }));
            }

            #[test]
            fn options_defaults_are_false() {
                let opts = IndexOptions::default();
                assert!(!opts.reindex());
                assert!(!opts.dry_run());
            }

            #[test]
            fn options_can_be_set() {
                let opts = IndexOptions::new(true, true);
                assert!(opts.reindex());
                assert!(opts.dry_run());
            }
        }
    }
}
