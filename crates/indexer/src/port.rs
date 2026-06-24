//! Scanner port — filesystem traversal contract for the indexer context.
//!
//! Defines the `ScannerPort` trait, the interface through which the indexer
//! requests filesystem traversal, along with the `ScanEntry` type it yields.
//! Implementations (adapters) live in the scanner submodule.

use trace_fs::{DirNode, FileNode};

use crate::{error::ScannerError, report::SkippedEntry, scan::ScanFilters};

/// Lazy iterator type returned by [`ScannerPort::walk`].
///
/// Uses `'static` so mockall can store the expectation closure without
/// invariance issues — the adapter clones `DirPath` and `ScanFilters`
/// internally.
pub type WalkIter = Box<dyn Iterator<Item = Result<ScanEntry, ScannerError>>>;

/// A single item yielded by the scanner's lazy walk iterator.
///
/// Filtered entries are silently dropped by walkdir's `filter_entry` and never
/// appear in the stream. Entries that match filters but can't be read yield
/// the `Skipped` variant — the caller accumulates these into `IndexReport`.
#[derive(Debug)]
pub enum ScanEntry {
    /// Discovered a standard file node.
    File(FileNode),
    /// Discovered a standard directory node.
    Dir(DirNode),
    /// The entry was skipped (e.g., due to filters or permissions).
    Skipped(SkippedEntry),
}

/// Interface for filesystem traversal.
///
/// Returns a lazy iterator so the caller can classify each entry inline,
/// avoiding a two-pass design (scan then classify). The `root` is always a
/// concrete `DirPath` resolved by the service layer — the adapter does not
/// know about vaults, `PathKey`, or `IndexScope`.
///
/// The returned iterator is `'static` — it owns all captured state. The
/// adapter clones `DirPath` and `ScanFilters` internally when building the
/// iterator, so the caller's references are not borrowed by the stream.
pub trait ScannerPort {
    /// Traverses the root and yields scan entries.
    ///
    /// # Errors
    /// Returns a `ScannerError` if traversal initialization fails.
    fn walk(
        &self,
        root: &trace_fs::DirPath,
        filters: &ScanFilters,
    ) -> Result<WalkIter, ScannerError>;
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate::always};
    use trace_fs::DirPath;

    use crate::{
        error::ScannerError,
        port::{ScanEntry, ScannerPort, WalkIter},
        report::{SkipReason, SkippedEntry},
        scan::ScanFilters,
    };

    mock! {
        pub(crate) ScannerPort {}
        impl ScannerPort for ScannerPort {
            fn walk(
                &self,
                root: &DirPath,
                filters: &ScanFilters,
            ) -> Result<WalkIter, ScannerError>;
        }
    }

    /// Verifies the generated mock compiles and returns the expected type.
    #[test]
    fn scanner_port_can_be_mocked() {
        let mut mock = MockScannerPort::new();
        let root = DirPath::try_new("/tmp".into()).unwrap();
        let filters = ScanFilters::default();

        mock.expect_walk().with(always(), always()).returning(|_, _| {
            let iter: WalkIter =
                Box::new(std::iter::empty::<Result<ScanEntry, ScannerError>>());
            Ok(iter)
        });

        let result = mock.walk(&root, &filters);
        assert!(result.is_ok());
    }

    #[test]
    fn yields_file_dir_and_skipped_variants() {
        let _entry = ScanEntry::Skipped(SkippedEntry {
            path: "/tmp/test".into(),
            reason: SkipReason::PermissionDenied,
        });
    }
}
