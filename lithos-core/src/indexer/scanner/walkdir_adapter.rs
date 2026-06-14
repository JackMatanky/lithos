use std::path::Path;

use walkdir::{DirEntry, WalkDir};

use crate::indexer::{
    scan::ScanFilters,
    scanner::skipped::{ScanResult, SkipReason, SkippedEntry},
};

pub(crate) struct WalkdirAdapter {
    filters: ScanFilters,
}

impl WalkdirAdapter {
    pub(crate) fn new(filters: ScanFilters) -> Self {
        Self {
            filters,
        }
    }

    pub(crate) fn scan(&self, root: &Path) -> ScanResult {
        let mut result = ScanResult::default();
        let walker = WalkDir::new(root).into_iter();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if !entry.file_type().is_file()
                        && !entry.file_type().is_dir()
                        && !entry.file_type().is_symlink()
                    {
                        result.skipped.push(SkippedEntry {
                            path: entry.path().to_path_buf(),
                            reason: SkipReason::UnsupportedEntryType,
                        });
                    } else if self.filter_entry(&entry) {
                        result.entries.push(entry.path().to_path_buf());
                    } else {
                        // Entry filtered out
                    }
                }
                Err(e) => {
                    let path =
                        e.path().map(Path::to_path_buf).unwrap_or_default();
                    if e.io_error().is_some_and(|ioe| {
                        ioe.kind() == std::io::ErrorKind::PermissionDenied
                    }) {
                        result.skipped.push(SkippedEntry {
                            path,
                            reason: SkipReason::PermissionDenied,
                        });
                    } else {
                        result.skipped.push(SkippedEntry {
                            path,
                            reason: SkipReason::UnsupportedEntryType,
                        });
                    }
                }
            }
        }
        result
    }

    pub(crate) fn filter_entry(&self, entry: &DirEntry) -> bool {
        let name = entry.file_name().to_string_lossy();
        if self.filters.excluded_names.iter().any(|n| n.as_ref() == name) {
            return false;
        }

        if entry.file_type().is_dir() {
            return true;
        }

        if !self.filters.included_extensions.is_empty() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());

            if let Some(ext) = ext {
                if !self
                    .filters
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
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::indexer::scan::ScanFilters;

    #[test]
    fn excludes_files_when_extension_does_not_match() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let txt_path = root.join("test.txt");
        let md_path = root.join("test.md");
        std::fs::write(&txt_path, b"").unwrap();
        std::fs::write(&md_path, b"").unwrap();

        let filters = ScanFilters {
            included_extensions: vec!["md".into()],
            excluded_names: vec![],
        };

        let adapter = WalkdirAdapter::new(filters);

        // This is a bit manual, normally WalkDir does the filtering.
        // But for testing the logic in filter_entry:

        let walker = walkdir::WalkDir::new(root).into_iter();
        let mut results = Vec::new();

        for entry in walker {
            let entry = entry.unwrap();
            if adapter.filter_entry(&entry) {
                results.push(entry.path().to_path_buf());
            }
        }

        assert!(results.contains(&md_path));
        assert!(!results.contains(&txt_path));
    }

    #[test]
    fn excludes_entries_when_name_is_excluded() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let excluded_dir = root.join("excluded");
        let excluded_file = root.join("excluded_file.md");
        std::fs::create_dir(&excluded_dir).unwrap();
        std::fs::write(&excluded_file, b"").unwrap();

        let filters = ScanFilters {
            included_extensions: vec![],
            excluded_names: vec!["excluded".into(), "excluded_file.md".into()],
        };

        let adapter = WalkdirAdapter::new(filters);

        let walker = walkdir::WalkDir::new(root).into_iter();
        let mut results = Vec::new();

        for entry in walker {
            let entry = entry.unwrap();
            if adapter.filter_entry(&entry) {
                results.push(entry.path().to_path_buf());
            }
        }

        assert!(!results.contains(&excluded_dir));
        assert!(!results.contains(&excluded_file));
    }
}
