//! Diagnostics for skipped filesystem nodes.

use std::path::PathBuf;

/// A record of a node that could not be indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkippedEntry {
    pub(crate) path: PathBuf,
    pub(crate) reason: SkipReason,
}

/// The reason a node was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkipReason {
    PermissionDenied,
    UnsupportedEntryType,
    Unknown(String),
}

/// The result of a scan operation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ScanResult {
    pub(crate) entries: Vec<PathBuf>,
    pub(crate) skipped: Vec<SkippedEntry>,
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::indexer::{
        scan::ScanFilters, scanner::walkdir_adapter::WalkdirAdapter,
    };

    #[test]
    fn records_permission_denied_entry_as_skipped() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let protected_dir = root.join("protected");
        std::fs::create_dir(&protected_dir).unwrap();

        // Remove read permissions (this is platform dependent, may not work on
        // all)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &protected_dir,
                std::fs::Permissions::from_mode(0o000),
            )
            .unwrap();
        }

        let filters = ScanFilters::default();
        let adapter = WalkdirAdapter::new(filters);

        // Act
        let result = adapter.scan(root);

        // Assert
        assert!(result.skipped.iter().any(|s| s.path == protected_dir
            && s.reason == SkipReason::PermissionDenied));

        // Restore permissions so cleanup works
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                &protected_dir,
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
    }

    #[test]
    fn records_unsupported_entry_type_as_skipped() {
        // Need to create a socket or similar. This is very platform dependent.
        // On Linux, we can create a unix socket.
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::net::UnixListener;
            let temp = TempDir::new().unwrap();
            let socket_path = temp.path().join("test.sock");
            let _listener = UnixListener::bind(&socket_path).unwrap();

            let filters = ScanFilters::default();
            let adapter = WalkdirAdapter::new(filters);

            // Act
            let result = adapter.scan(temp.path());

            // Assert - if the adapter considers this unsupported
            // Current WalkdirAdapter doesn't explicitly check entry types
            // besides dir/file. If it's a socket, walkdir includes
            // it, but it might not be a file. The task implies I
            // should *make* it handle it. For now, this test might
            // fail or pass depending on current implementation.
            assert!(result.skipped.iter().any(|s| s.path == socket_path
                && s.reason == SkipReason::UnsupportedEntryType));
        }
    }
}
