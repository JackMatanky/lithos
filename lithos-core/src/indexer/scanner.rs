use std::path::PathBuf;

use crate::fs::entry::{DirNode, FileNode};

/// Raw filesystem nodes discovered during a scan, before ID assignment or
/// index-status comparison. Uses `fs::entry::FileNode` / `fs::entry::DirNode`
/// directly — no redundant wrapper types.
pub(crate) struct ScanResult {
    pub(crate) files: Vec<FileNode>,
    pub(crate) dirs: Vec<DirNode>,
    pub(crate) skipped: Vec<SkippedEntry>,
}

/// An entry encountered during scanning that could not be indexed.
/// Has no `FsRecordId` because it was never persisted.
pub(crate) struct SkippedEntry {
    pub(crate) path: PathBuf,
    pub(crate) reason: SkipReason,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SkipReason {
    PermissionDenied,
    UnsupportedEntryType,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ScanResult, SkipReason, SkippedEntry};
    use crate::fs::{
        entry::{DirNode, FileNode},
        metadata::{DirMetadata, FileMetadata, FsTimes},
        path::{DirPath, FilePath},
    };

    mod scan_result {
        use super::*;

        #[test]
        fn stores_files_dirs_and_skipped_entries() {
            let temp_dir = tempfile::TempDir::new().unwrap();

            // Construct a FileNode
            let file_pb = temp_dir.path().join("a.txt");
            std::fs::write(&file_pb, "test").unwrap();
            let file_path = FilePath::try_new(file_pb).unwrap();
            let file_metadata =
                FileMetadata::new(FsTimes::new(None, None), 4, false);
            let file_node = FileNode::new(file_path, file_metadata);

            // Construct a DirNode
            let dir_pb = temp_dir.path().join("dir");
            std::fs::create_dir(&dir_pb).unwrap();
            let dir_path = DirPath::try_new(dir_pb).unwrap();
            let dir_metadata =
                DirMetadata::new(FsTimes::new(None, None), false);
            let dir_node = DirNode::new(dir_path, dir_metadata);

            let files = vec![file_node];
            let dirs = vec![dir_node];
            let skipped = vec![SkippedEntry {
                path: PathBuf::from("hidden"),
                reason: SkipReason::PermissionDenied,
            }];

            let result = ScanResult {
                files,
                dirs,
                skipped,
            };

            assert_eq!(result.files.len(), 1);
            assert_eq!(result.dirs.len(), 1);
            assert_eq!(result.skipped.len(), 1);
            assert_eq!(
                result.skipped.first().unwrap().reason,
                SkipReason::PermissionDenied
            );
        }
    }
}
