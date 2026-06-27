use std::path::PathBuf;

use traces_fs::{DirPath, FsNode, error::ScanError};
use walkdir::WalkDir;

use crate::{
    error::ScannerError,
    port::{ScanEntry, ScannerPort, WalkIter},
    report::{SkipReason, SkippedEntry},
    scan::ScanFilters,
};

/// Walkdir-based filesystem traversal adapter.
///
/// Stateless: receives the resolved `DirPath` per call via the port contract.
/// No vault knowledge lives here.
pub struct WalkdirAdapter;

/// Classify a [`ScanError`] raised while converting a walkdir entry into a
/// [`SkippedEntry`].
///
/// `Traversal` errors carry permission/IO failures, so they map to
/// [`SkipReason::PermissionDenied`]. Every other variant — including the
/// `#[non_exhaustive]` fallthrough — is a structural problem with the entry
/// (e.g. a non-UTF-8 `Path` error or an unsupported entry type) and maps to
/// [`SkipReason::UnsupportedEntryType`] so its context is not mislabeled as a
/// permission failure.
fn classify_scan_error(e: ScanError) -> SkippedEntry {
    let (path, reason) = match e {
        ScanError::UnsupportedEntryType(p) => {
            (p, SkipReason::UnsupportedEntryType)
        }
        ScanError::Traversal {
            path,
            ..
        } => (path, SkipReason::PermissionDenied),
        _ => (PathBuf::new(), SkipReason::UnsupportedEntryType),
    };
    SkippedEntry {
        path,
        reason,
    }
}

impl WalkdirAdapter {
    fn filter_entry(entry: &walkdir::DirEntry, filters: &ScanFilters) -> bool {
        let name = entry.file_name().to_string_lossy();
        if filters.is_excluded_name(&name) {
            return false;
        }

        if entry.file_type().is_dir() {
            return true;
        }

        let ext =
            entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
        filters.is_included_extension(ext)
    }

    fn map_error(e: walkdir::Error) -> Result<ScanEntry, ScannerError> {
        let path =
            e.path().map(std::path::Path::to_path_buf).unwrap_or_default();
        if e.io_error().is_some_and(|ioe| {
            ioe.kind() == std::io::ErrorKind::PermissionDenied
        }) {
            Ok(ScanEntry::Skipped(SkippedEntry {
                path,
                reason: SkipReason::PermissionDenied,
            }))
        } else {
            let source = e.into_io_error().unwrap_or_else(|| {
                std::io::Error::other("Unknown traversal error")
            });
            Err(ScannerError::Traversal {
                path,
                source,
            })
        }
    }
}

impl TryFrom<walkdir::DirEntry> for ScanEntry {
    type Error = ScannerError;

    #[inline]
    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        match FsNode::try_from(entry) {
            Ok(FsNode::File(n)) => Ok(ScanEntry::File(n)),
            Ok(FsNode::Dir(n)) => Ok(ScanEntry::Dir(n)),
            // `FsNode` is `#[non_exhaustive]`; this arm is required by the
            // compiler for forward compatibility. The originating
            // `walkdir::DirEntry` is consumed by `FsNode::try_from`, so the
            // path cannot be recovered here — graceful degradation only.
            Ok(_) => Ok(ScanEntry::Skipped(SkippedEntry {
                path: PathBuf::new(),
                reason: SkipReason::UnsupportedEntryType,
            })),
            Err(e) => Ok(ScanEntry::Skipped(classify_scan_error(e))),
        }
    }
}

impl ScannerPort for WalkdirAdapter {
    #[inline]
    fn walk(
        &self,
        root: &DirPath,
        filters: &ScanFilters,
    ) -> Result<WalkIter, ScannerError> {
        let filters = filters.clone();
        let walker = WalkDir::new(root.as_path()).min_depth(1).into_iter();
        let filtered =
            walker.filter_entry(move |e| Self::filter_entry(e, &filters));

        Ok(Box::new(filtered.map(move |result| match result {
            Ok(entry) => ScanEntry::try_from(entry),
            Err(e) => Self::map_error(e),
        })))
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use traces_fs::error::PathError;

    use super::*;

    #[test]
    fn classify_scan_error_maps_path_error_to_unsupported() {
        let skipped = classify_scan_error(ScanError::Path(PathError::Empty));
        assert_eq!(skipped.reason, SkipReason::UnsupportedEntryType);
    }

    #[test]
    fn classify_scan_error_maps_traversal_to_permission_denied() {
        let skipped = classify_scan_error(ScanError::Traversal {
            path: PathBuf::from("/blocked"),
            source: std::io::Error::other("denied"),
        });
        assert_eq!(skipped.path, PathBuf::from("/blocked"));
        assert_eq!(skipped.reason, SkipReason::PermissionDenied);
    }

    #[test]
    fn classify_scan_error_maps_unsupported_entry_type() {
        let skipped = classify_scan_error(ScanError::UnsupportedEntryType(
            PathBuf::from("/socket"),
        ));
        assert_eq!(skipped.path, PathBuf::from("/socket"));
        assert_eq!(skipped.reason, SkipReason::UnsupportedEntryType);
    }

    #[test]
    fn returns_nodes_when_scope_is_full() {
        let temp_dir = TempDir::new().unwrap();
        let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();

        std::fs::write(temp_dir.path().join("a.md"), "").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/b.md"), "").unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::default();

        let results: Vec<_> = adapter
            .walk(&root, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        assert!(results.iter().any(|e| matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with("a.md"))));
        assert!(results.iter().any(|e| matches!(e, ScanEntry::Dir(n) if n.path().as_path().ends_with("subdir"))));
        assert!(results.iter().any(|e| matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with("subdir/b.md"))));
    }

    #[test]
    fn scans_different_root_on_each_call() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::create_dir_all(temp_dir.path().join("a")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("b")).unwrap();

        let root_a =
            DirPath::try_new(temp_dir.path().join("a").clone()).unwrap();
        let _root_b =
            DirPath::try_new(temp_dir.path().join("b").clone()).unwrap();
        std::fs::write(temp_dir.path().join("a/file.md"), "").unwrap();
        std::fs::write(temp_dir.path().join("b/other.md"), "").unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::default();

        let results_a: Vec<_> = adapter
            .walk(&root_a, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        assert!(results_a.iter().any(|e| matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with("a/file.md"))));
        assert!(!results_a.iter().any(|e| matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with("b/other.md"))));
    }

    #[test]
    fn rejects_file_when_extension_mismatches() {
        let temp_dir = TempDir::new().unwrap();
        let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();

        std::fs::write(temp_dir.path().join("a.md"), "").unwrap();
        std::fs::write(temp_dir.path().join("b.txt"), "").unwrap();
        std::fs::write(temp_dir.path().join("c.md"), "").unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::new(vec!["md".into()], vec![]);

        let results: Vec<_> = adapter
            .walk(&root, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        let files: Vec<_> = results
            .into_iter()
            .filter_map(|e| match e {
                ScanEntry::File(n) => Some(n.path().as_path().to_path_buf()),
                _ => None,
            })
            .collect();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("a.md")));
        assert!(files.iter().any(|p| p.ends_with("c.md")));
    }

    #[test]
    fn rejects_entry_when_name_is_excluded() {
        let temp_dir = TempDir::new().unwrap();
        let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();

        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
        std::fs::write(temp_dir.path().join(".git/head"), "").unwrap();
        std::fs::write(temp_dir.path().join("readme.md"), "").unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::new(vec![], vec![".git".into()]);

        let results: Vec<_> = adapter
            .walk(&root, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        assert!(!results.iter().any(|e| {
            matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with(".git/head"))
        }));
        assert!(results.iter().any(|e| {
            matches!(e, ScanEntry::File(n) if n.path().as_path().ends_with("readme.md"))
        }));
    }

    #[test]
    #[cfg(unix)]
    fn returns_skipped_entry_when_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let root = DirPath::try_new(temp.path().to_path_buf()).unwrap();
        let protected_dir = temp.path().join("protected");
        std::fs::create_dir(&protected_dir).unwrap();

        std::fs::set_permissions(
            &protected_dir,
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::default();

        let results: Vec<_> = adapter
            .walk(&root, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        assert!(results.iter().any(|e| matches!(e, ScanEntry::Skipped(s) if s.path == protected_dir && s.reason == SkipReason::PermissionDenied)));

        std::fs::set_permissions(
            &protected_dir,
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn returns_skipped_entry_when_unsupported_type() {
        use std::os::unix::net::UnixListener;

        let temp = TempDir::new().unwrap();
        let root = DirPath::try_new(temp.path().to_path_buf()).unwrap();
        let socket_path = temp.path().join("socket");
        UnixListener::bind(&socket_path).unwrap();

        let adapter = WalkdirAdapter;
        let filters = ScanFilters::default();

        let results: Vec<_> = adapter
            .walk(&root, &filters)
            .expect("walk should succeed")
            .collect::<Result<Vec<_>, _>>()
            .expect("no fatal errors");

        assert!(results.iter().any(|e| {
            matches!(
                e,
                ScanEntry::Skipped(s)
                    if s.path == socket_path
                    && s.reason == SkipReason::UnsupportedEntryType
            )
        }));
    }
}
