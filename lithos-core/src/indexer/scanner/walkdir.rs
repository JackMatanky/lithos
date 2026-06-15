use std::path::Path;

use walkdir::WalkDir;

use crate::{
    fs::{DirPath, FsNode},
    indexer::{
        error::ScannerError,
        port::{ScanResult, ScannerPort},
        report::{SkipReason, SkippedEntry},
        scan::{IndexScope, ScanFilters},
    },
};

/// Adapter for walkdir-based filesystem traversal.
pub struct WalkdirAdapter {
    vault_root: DirPath,
}

impl WalkdirAdapter {
    /// Create a new `WalkdirAdapter`.
    #[must_use]
    #[inline]
    pub fn new(vault_root: DirPath) -> Self {
        Self {
            vault_root,
        }
    }

    fn filter_entry(entry: &walkdir::DirEntry, filters: &ScanFilters) -> bool {
        let name = entry.file_name().to_string_lossy();
        if filters.excluded_names.iter().any(|n| n.as_ref() == name.as_ref()) {
            return false;
        }

        if entry.file_type().is_dir() {
            return true;
        }

        if !filters.included_extensions.is_empty() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());

            if let Some(ext) = ext {
                if !filters
                    .included_extensions
                    .iter()
                    .any(|allowed| allowed.as_ref() == ext)
                {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn handle_entry(
        entry: walkdir::DirEntry,
        result: &mut ScanResult,
        scan_root: &Path,
        filters: &ScanFilters,
    ) {
        let file_type = entry.file_type();
        if !file_type.is_file()
            && !file_type.is_dir()
            && !file_type.is_symlink()
        {
            result.skipped.push(SkippedEntry {
                path: entry.path().to_path_buf(),
                reason: SkipReason::UnsupportedEntryType,
            });
            return;
        }

        if !Self::filter_entry(&entry, filters) {
            return;
        }

        let entry_path = entry.path().to_path_buf();
        match FsNode::try_from(entry) {
            Ok(FsNode::File(node)) => result.files.push(node),
            Ok(FsNode::Dir(node)) => {
                if node.path().as_path() != scan_root {
                    result.dirs.push(node);
                }
            }
            Err(_) => {
                result.skipped.push(SkippedEntry {
                    path: entry_path,
                    reason: SkipReason::PermissionDenied,
                });
            }
        }
    }

    fn handle_entry_error(
        e: walkdir::Error,
        result: &mut ScanResult,
    ) -> Result<(), ScannerError> {
        let path = e.path().map(Path::to_path_buf).unwrap_or_default();
        if e.io_error().is_some_and(|ioe| {
            ioe.kind() == std::io::ErrorKind::PermissionDenied
        }) {
            result.skipped.push(SkippedEntry {
                path,
                reason: SkipReason::PermissionDenied,
            });
            Ok(())
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

impl ScannerPort for WalkdirAdapter {
    fn scan(&self, scope: &IndexScope) -> Result<ScanResult, ScannerError> {
        let mut result = ScanResult::default();

        let scan_root_buf;
        let (scan_root, filters) = match scope {
            IndexScope::Full {
                filters,
            } => (self.vault_root.as_path(), filters),
            IndexScope::Partial {
                root,
                filters,
            } => {
                scan_root_buf = self.vault_root.join(root.as_str());
                (scan_root_buf.as_path(), filters)
            }
        };

        let walker = WalkDir::new(scan_root).into_iter();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    Self::handle_entry(entry, &mut result, scan_root, filters);
                }
                Err(e) => {
                    Self::handle_entry_error(e, &mut result)?;
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::fs::path::PathKey;

    #[test]
    fn returns_file_and_dir_nodes_for_full_scope() {
        let temp_dir = TempDir::new().unwrap();
        let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();

        std::fs::write(temp_dir.path().join("a.md"), "").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/b.md"), "").unwrap();

        let adapter = WalkdirAdapter::new(root);
        let scope = IndexScope::Full {
            filters: ScanFilters::default(),
        };

        let result = adapter.scan(&scope).expect("scan should succeed");

        assert!(result.files.iter().any(|n: &crate::fs::FileNode| {
            n.path().as_path().ends_with("a.md")
        }));
        assert!(result.dirs.iter().any(|n: &crate::fs::DirNode| {
            n.path().as_path().ends_with("subdir")
        }));
        assert!(result.files.iter().any(|n: &crate::fs::FileNode| {
            n.path().as_path().ends_with("subdir/b.md")
        }));
    }

    #[test]
    fn scans_only_partial_scope_root() {
        let temp_dir = TempDir::new().unwrap();
        let root = DirPath::try_new(temp_dir.path().to_path_buf()).unwrap();

        std::fs::write(temp_dir.path().join("a.md"), "").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        std::fs::write(temp_dir.path().join("subdir/b.md"), "").unwrap();

        let adapter = WalkdirAdapter::new(root);

        let partial_root = PathKey::try_new("subdir").unwrap();
        let scope = IndexScope::Partial {
            root: partial_root,
            filters: ScanFilters::default(),
        };

        let result = adapter.scan(&scope).expect("scan should succeed");

        assert!(result.files.iter().any(|n: &crate::fs::FileNode| {
            n.path().as_path().ends_with("subdir/b.md")
        }));
        assert!(!result.files.iter().any(|n: &crate::fs::FileNode| {
            n.path().as_path().ends_with("a.md")
        }));
    }

    #[test]
    #[cfg(unix)]
    fn records_permission_denied_entry_as_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let vault_root = DirPath::try_new(root.to_path_buf()).unwrap();
        let protected_dir = root.join("protected");
        std::fs::create_dir(&protected_dir).unwrap();

        std::fs::set_permissions(
            &protected_dir,
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let adapter = WalkdirAdapter::new(vault_root);
        let scope = IndexScope::Full {
            filters: ScanFilters::default(),
        };

        let result = adapter.scan(&scope).unwrap();

        assert!(result.skipped.iter().any(|s| s.path == protected_dir
            && s.reason == SkipReason::PermissionDenied));

        std::fs::set_permissions(
            &protected_dir,
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
}
