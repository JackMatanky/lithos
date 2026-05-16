//! Directory scanning utilities for finding files matching criteria.
//!
//! This module provides [`DirScanner`] for efficient directory traversal and
//! filtering based on glob patterns, file extensions, and other criteria.
//!
//! ## Architecture
//!
//! `DirScanner` is a standalone utility that can be used independently or via
//! [`crate::fs::reader::Reader`]'s convenience methods (`filter_entries` and
//! `filter_dir_entries`). It uses [`walkdir`] for recursive traversal with
//! configurable depth, symlink handling, and filtering.
//!
//! ## Design Decisions
//!
//! - **Standalone type**: Not embedded in Reader to allow reuse across contexts
//! - **AND semantics**: When both `pattern` and `extensions` are specified,
//!   both must match
//! - **Root exclusion**: The root directory itself is excluded from results
//!   (walkdir includes it at depth 0)

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::{
    entry::FsEntry,
    error::{PathError, ScanError},
    path::{DirPath, FilePath, FsPath},
};

/// Standalone directory scanner for finding files matching criteria.
///
/// Stores the root directory path and provides methods to scan for paths
/// or file entries with metadata.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
///
/// use lithos_core::fs::scanner::{DirScanInput, DirScanner};
///
/// let scanner = DirScanner::new("/path/to/schemas");
///
/// // Find all TOML files
/// let paths =
///     scanner.paths(DirScanInput::new().with_extensions(&["toml"])).unwrap();
///
/// // Find files matching glob pattern with metadata
/// let entries =
///     scanner.entries(DirScanInput::new().with_pattern("**/*.toml")).unwrap();
/// ```
#[expect(
    clippy::module_name_repetitions,
    reason = "DirScanner in fs::scanner is intentional: it's the primary type \
              exported from this module and the Dir prefix clarifies its \
              purpose (directory scanning vs file scanning)."
)]
pub struct DirScanner {
    /// Root directory for scanning operations.
    path: PathBuf,
}

impl DirScanner {
    /// Creates a new scanner for the given directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanner;
    ///
    /// let scanner = DirScanner::new("/path/to/vault");
    /// ```
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
        }
    }

    /// Returns the root path of this scanner.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// use lithos_core::fs::scanner::DirScanner;
    ///
    /// let scanner = DirScanner::new("/vault");
    /// assert_eq!(scanner.path(), Path::new("/vault"));
    /// ```
    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Scans the directory and returns matching typed entries with metadata.
    ///
    /// Results are sorted by path alphabetically.
    ///
    /// # Errors
    ///
    /// Returns `ScanError` if directory traversal fails, pattern is
    /// invalid, or metadata cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lithos_core::fs::scanner::{DirScanInput, DirScanner};
    ///
    /// let scanner = DirScanner::new("/vault");
    /// let entries = scanner
    ///     .entries(DirScanInput::new().with_extensions(&["toml", "yaml"]))?;
    ///
    /// for entry in entries {
    ///     let Some(filename) = entry.filename() else {
    ///         continue;
    ///     };
    ///     let metadata = entry.metadata();
    ///     let Some(file_meta) = metadata.as_file() else {
    ///         continue;
    ///     };
    ///     println!("{}: {} bytes", filename.as_str(), file_meta.size());
    /// }
    /// # Ok::<(), lithos_core::fs::error::ScanError>(())
    /// ```
    /// Scans the directory and returns matching paths (File or Dir).
    ///
    /// Results are sorted by path alphabetically.
    ///
    /// # Errors
    ///
    /// Returns `ScanError` if directory traversal fails or pattern is
    /// invalid.
    #[inline]
    pub fn paths(&self, input: DirScanInput) -> Result<Vec<FsPath>, ScanError> {
        let walker = self.build_walker(&input);
        let mut results = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|e| ScanError::Traversal {
                path: e.path().map(Path::to_path_buf).unwrap_or_default(),
                source: e.into(),
            })?;

            if self.filter_entry(&entry, &input)?.is_some() {
                results.push(Self::to_fs_path(&entry)?);
            }
        }

        results.sort_by(|a, b| a.as_path().cmp(b.as_path()));
        Ok(results)
    }

    /// Scans the directory and returns matching typed entries with metadata.
    ///
    /// Results are sorted by path alphabetically.
    ///
    /// # Errors
    ///
    /// Returns `ScanError` if directory traversal fails, pattern is
    /// invalid, or metadata cannot be read.
    #[inline]
    pub fn entries(
        &self,
        input: DirScanInput,
    ) -> Result<Vec<FsEntry>, ScanError> {
        let walker = self.build_walker(&input);
        let mut results = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|e| ScanError::Traversal {
                path: e.path().map(Path::to_path_buf).unwrap_or_default(),
                source: e.into(),
            })?;

            if self.filter_entry(&entry, &input)?.is_some() {
                results.push(FsEntry::try_from(entry)?);
            }
        }

        results
            .sort_by(|a, b| a.path_ref().as_path().cmp(b.path_ref().as_path()));
        Ok(results)
    }

    // ─── Private Helper Methods ───────────────────────────────────────

    /// Converts a `walkdir::DirEntry` to a typed `FsPath`.
    fn to_fs_path(entry: &walkdir::DirEntry) -> Result<FsPath, ScanError> {
        // Borrow the path first; only clone when construction succeeds
        let path = entry.path();
        let metadata = entry.metadata().map_err(|e| ScanError::Traversal {
            path: path.to_path_buf(),
            source: e.into(),
        })?;

        if metadata.is_dir() {
            DirPath::new(path.to_path_buf()).map(FsPath::Dir).map_err(|_e| {
                ScanError::Path(PathError::InvalidUtf8(path.to_path_buf()))
            })
        } else {
            FilePath::new(path.to_path_buf()).map(FsPath::File).map_err(|_e| {
                ScanError::Path(PathError::InvalidUtf8(path.to_path_buf()))
            })
        }
    }

    /// Checks if a `walkdir::DirEntry` matches the scan input criteria.
    ///
    /// Returns the relative path if it matches, or `None` if it should be
    /// skipped.
    fn filter_entry(
        &self,
        entry: &walkdir::DirEntry,
        input: &DirScanInput,
    ) -> Result<Option<PathBuf>, ScanError> {
        let path = entry.path();

        // Filter by file type
        if !Self::matches_file_type(entry, input.include_dirs) {
            return Ok(None);
        }

        // Get relative path for filtering
        let Ok(relative) = path.strip_prefix(&self.path) else {
            return Ok(None); // Path outside root
        };

        // Skip the root directory itself
        if relative.as_os_str().is_empty() {
            return Ok(None);
        }

        // Filter by extensions (if specified)
        if !Self::matches_extensions(relative, input.extensions) {
            return Ok(None);
        }

        // Filter by pattern (if specified)
        if !Self::matches_pattern(relative, input.pattern)? {
            return Ok(None);
        }

        Ok(Some(relative.to_path_buf()))
    }

    /// Builds a `WalkDir` iterator based on input configuration.
    fn build_walker(&self, input: &DirScanInput) -> WalkDir {
        let mut walker = WalkDir::new(&self.path);

        if !input.recursive {
            walker = walker.max_depth(1);
        }

        walker = walker.follow_links(input.follow_symlinks);

        walker
    }

    /// Checks if entry matches the file type filter.
    fn matches_file_type(
        entry: &walkdir::DirEntry,
        include_dirs: bool,
    ) -> bool {
        let file_type = entry.file_type();
        // Include entry if: (1) it's not a directory, OR (2) it's a directory
        // and include_dirs is true
        !file_type.is_dir() || include_dirs
    }

    /// Checks if path matches the extensions filter (if specified).
    fn matches_extensions(path: &Path, extensions: Option<&[&str]>) -> bool {
        let Some(exts) = extensions else {
            return true; // No filter specified
        };

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return false; // No extension on file
        };

        exts.iter().any(|&allowed| ext.eq_ignore_ascii_case(allowed))
    }

    /// Checks if path matches the glob pattern (if specified).
    fn matches_pattern(
        path: &Path,
        pattern: Option<&str>,
    ) -> Result<bool, ScanError> {
        let Some(pattern_str) = pattern else {
            return Ok(true); // No filter specified
        };

        let glob_pattern = glob::Pattern::new(pattern_str).map_err(|e| {
            ScanError::InvalidPattern {
                pattern: pattern_str.into(),
                message: e.msg.into(),
            }
        })?;

        let path_str = path.to_str().ok_or_else(|| {
            ScanError::Path(PathError::InvalidUtf8(path.to_path_buf()))
        })?;

        Ok(glob_pattern.matches(path_str))
    }
}

/// Input parameters for directory scanning operations.
///
/// Uses builder pattern for flexible configuration. All filters use AND
/// semantics: if multiple filters are specified, all must match for a file to
/// be included.
///
/// # Examples
///
/// ```
/// use lithos_core::fs::scanner::DirScanInput;
///
/// // Simple glob pattern
/// let input = DirScanInput::new().with_pattern("schemas/**/*.toml");
///
/// // Extension filter only
/// let input = DirScanInput::new().with_extensions(&["toml", "yaml"]);
///
/// // Combined: pattern AND extensions (both must match)
/// let input = DirScanInput::new()
///     .with_pattern("schemas/**/*")
///     .with_extensions(&["toml"])
///     .recursive(true);
/// ```
#[expect(
    clippy::struct_excessive_bools,
    reason = "Three boolean flags with independent, clear semantics: \
              include_dirs (output filter), follow_symlinks (traversal \
              behavior), recursive (depth control). Converting to bitflags \
              would reduce clarity."
)]
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct DirScanInput<'params> {
    /// Optional glob pattern for matching paths (relative to scan directory).
    pub pattern: Option<&'params str>,

    /// Optional file extensions to filter (without dot, e.g., `["toml",
    /// "yaml"]`). When both pattern and extensions are specified, BOTH must
    /// match (AND semantics).
    pub extensions: Option<&'params [&'params str]>,

    /// Whether to include directories in results (default: false).
    pub include_dirs: bool,

    /// Whether to follow symlinks (default: false).
    pub follow_symlinks: bool,

    /// Whether to scan recursively (default: true).
    pub recursive: bool,
}

impl<'params> DirScanInput<'params> {
    /// Creates a new scan input with defaults.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the glob pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanInput;
    ///
    /// let input = DirScanInput::new().with_pattern("**/*.toml");
    /// ```
    #[inline]
    #[must_use]
    pub fn with_pattern(mut self, pattern: &'params str) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Sets the extensions filter (AND with pattern if both specified).
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanInput;
    ///
    /// let input = DirScanInput::new().with_extensions(&["toml", "yaml", "yml"]);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_extensions(
        mut self,
        extensions: &'params [&'params str],
    ) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Sets whether to include directories.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanInput;
    ///
    /// let input = DirScanInput::new().include_dirs(true);
    /// ```
    #[inline]
    #[must_use]
    pub fn include_dirs(mut self, include: bool) -> Self {
        self.include_dirs = include;
        self
    }

    /// Sets whether to follow symlinks.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanInput;
    ///
    /// let input = DirScanInput::new().follow_symlinks(true);
    /// ```
    #[inline]
    #[must_use]
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Sets whether to scan recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::scanner::DirScanInput;
    ///
    /// // Non-recursive (only immediate children)
    /// let input = DirScanInput::new().recursive(false);
    /// ```
    #[inline]
    #[must_use]
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
}

impl Default for DirScanInput<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            pattern: None,
            extensions: None,
            include_dirs: false,
            follow_symlinks: false,
            recursive: true,
        }
    }
}
#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test modules use conventional helper function before nested mod \
              ordering"
)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    // Helper to create test files
    fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, contents).expect("write test file");
        path
    }

    mod dir_scan_input {
        use super::*;

        #[test]
        fn default_has_expected_values() {
            let input = DirScanInput::default();
            assert!(input.pattern.is_none());
            assert!(input.extensions.is_none());
            assert!(!input.include_dirs);
            assert!(!input.follow_symlinks);
            assert!(input.recursive);
        }

        #[test]
        fn builder_pattern_works() {
            let input = DirScanInput::new()
                .with_pattern("**/*.toml")
                .with_extensions(&["toml", "yaml"])
                .include_dirs(true)
                .follow_symlinks(true)
                .recursive(false);

            assert_eq!(input.pattern, Some("**/*.toml"));
            assert_eq!(input.extensions, Some(&["toml", "yaml"][..]));
            assert!(input.include_dirs);
            assert!(input.follow_symlinks);
            assert!(!input.recursive);
        }
    }

    mod dir_scanner_basics {
        use super::*;

        #[test]
        fn new_stores_path() {
            let scanner = DirScanner::new("/test/path");
            assert_eq!(scanner.path(), Path::new("/test/path"));
        }

        #[test]
        fn paths_returns_empty_for_empty_dir() {
            let temp = TempDir::new().unwrap();
            let scanner = DirScanner::new(temp.path());
            let paths = scanner.paths(DirScanInput::new()).unwrap();
            assert!(paths.is_empty());
        }

        #[test]
        fn entries_returns_empty_for_empty_dir() {
            let temp = TempDir::new().unwrap();
            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            assert!(entries.is_empty());
        }
    }

    mod pattern_filtering {
        use super::*;

        #[test]
        fn paths_matches_glob_pattern() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"");
            write_file(temp.path(), "b.yaml", b"");
            write_file(temp.path(), "sub/c.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner
                .paths(DirScanInput::new().with_pattern("**/*.toml"))
                .unwrap();

            assert_eq!(paths.len(), 2);
            assert!(paths.iter().any(|p| p.as_path() == root.join("a.toml")));
            assert!(
                paths.iter().any(|p| p.as_path() == root.join("sub/c.toml"))
            );
        }

        #[test]
        fn paths_sorts_results() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "z.toml", b"");
            write_file(temp.path(), "a.toml", b"");
            write_file(temp.path(), "m.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(paths.len(), 3);
            let first = paths.first().expect("first path");
            let second = paths.get(1).expect("second path");
            let third = paths.get(2).expect("third path");
            assert_eq!(first.as_path(), root.join("a.toml"));
            assert_eq!(second.as_path(), root.join("m.toml"));
            assert_eq!(third.as_path(), root.join("z.toml"));
        }
    }

    mod extension_filtering {
        use super::*;

        #[test]
        fn paths_filters_by_extensions() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"");
            write_file(temp.path(), "b.yaml", b"");
            write_file(temp.path(), "c.json", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner
                .paths(DirScanInput::new().with_extensions(&["toml", "yaml"]))
                .unwrap();

            assert_eq!(paths.len(), 2);
            assert!(paths.iter().any(|p| p.as_path() == root.join("a.toml")));
            assert!(paths.iter().any(|p| p.as_path() == root.join("b.yaml")));
        }

        #[test]
        fn extensions_are_case_insensitive() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.TOML", b"");

            let scanner = DirScanner::new(temp.path());
            let paths = scanner
                .paths(DirScanInput::new().with_extensions(&["toml"]))
                .unwrap();

            assert_eq!(paths.len(), 1);
        }
    }

    mod combined_filters {
        use super::*;

        #[test]
        fn pattern_and_extensions_use_and_semantics() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "schemas/a.toml", b"");
            write_file(temp.path(), "schemas/b.yaml", b"");
            write_file(temp.path(), "other/c.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner
                .paths(
                    DirScanInput::new()
                        .with_pattern("schemas/**/*")
                        .with_extensions(&["toml"]),
                )
                .unwrap();

            // Only schemas/a.toml matches BOTH pattern AND extension
            assert_eq!(paths.len(), 1);
            assert_eq!(
                paths.first().unwrap().as_path(),
                root.join("schemas/a.toml")
            );
        }
    }

    mod recursive_control {
        use super::*;

        #[test]
        fn non_recursive_only_finds_immediate_children() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"");
            write_file(temp.path(), "sub/b.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths =
                scanner.paths(DirScanInput::new().recursive(false)).unwrap();

            assert_eq!(paths.len(), 1);
            assert_eq!(paths.first().unwrap().as_path(), root.join("a.toml"));
        }

        #[test]
        fn recursive_finds_nested_files() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"");
            write_file(temp.path(), "sub/b.toml", b"");
            write_file(temp.path(), "sub/deep/c.toml", b"");

            let scanner = DirScanner::new(temp.path());
            let paths =
                scanner.paths(DirScanInput::new().recursive(true)).unwrap();

            assert_eq!(paths.len(), 3);
        }
    }

    mod directory_handling {
        use super::*;

        #[test]
        fn excludes_directories_by_default() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "file.toml", b"");
            std::fs::create_dir_all(temp.path().join("subdir")).unwrap();
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(paths.len(), 1);
            assert_eq!(
                paths.first().unwrap().as_path(),
                root.join("file.toml")
            );
        }

        #[test]
        fn includes_directories_when_requested() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "file.toml", b"");
            std::fs::create_dir_all(temp.path().join("subdir")).unwrap();
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner
                .paths(DirScanInput::new().include_dirs(true).recursive(false))
                .unwrap();

            assert_eq!(paths.len(), 2);
            assert!(
                paths.iter().any(|p| p.as_path() == root.join("file.toml"))
            );
            assert!(paths.iter().any(|p| p.as_path() == root.join("subdir")));
        }
    }

    mod entries_method {
        use super::*;

        #[test]
        fn returns_file_entries_with_metadata() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"test content");

            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            let expected = temp.path().join("a.toml");

            assert_eq!(entries.len(), 1);
            assert_eq!(
                entries.first().map(|e| e.path_ref().as_path().to_path_buf()),
                Some(expected)
            );
            assert_eq!(
                entries
                    .first()
                    .and_then(crate::fs::entry::FsEntry::filename)
                    .map(|name| name.as_str().to_owned()),
                Some("a.toml".to_owned())
            );
            assert_eq!(
                entries.first().and_then(|e| e
                    .metadata()
                    .as_file()
                    .map(crate::fs::metadata::FileMetadata::size)),
                Some(12)
            );
        }

        #[test]
        fn sorts_entries_by_path() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "z.toml", b"");
            write_file(temp.path(), "a.toml", b"");

            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            let expected_a = temp.path().join("a.toml");
            let expected_z = temp.path().join("z.toml");

            assert_eq!(
                entries.first().map(|e| e.path_ref().as_path().to_path_buf()),
                Some(expected_a)
            );
            assert_eq!(
                entries.get(1).map(|e| e.path_ref().as_path().to_path_buf()),
                Some(expected_z)
            );
        }
    }

    mod typed_scanning {
        use super::*;

        #[test]
        fn paths_typed_returns_absolute_fs_paths() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(paths.len(), 1);
            let path = paths.first().unwrap();
            assert!(path.is_file());
            assert_eq!(path.as_path(), root.join("a.toml"));
        }

        #[test]
        fn entries_returns_fs_entries_with_absolute_paths() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "a.toml", b"test content");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let entries = scanner.entries(DirScanInput::new()).unwrap();

            assert_eq!(entries.len(), 1);
            let entry = entries.first().unwrap();
            assert!(entry.is_file());
            assert_eq!(entry.path().as_path(), root.join("a.toml"));
            assert_eq!(entry.as_file().unwrap().metadata().size(), 12);
        }

        #[test]
        fn typed_results_are_sorted() {
            let temp = TempDir::new().unwrap();
            write_file(temp.path(), "z.toml", b"");
            write_file(temp.path(), "a.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(
                paths.first().unwrap().as_path().file_name().unwrap(),
                "a.toml"
            );
            assert_eq!(
                paths.get(1).unwrap().as_path().file_name().unwrap(),
                "z.toml"
            );
        }
    }

    // Note: Error cases are minimal because:
    // - glob::Pattern::new() is very permissive (even malformed patterns like
    //   "[" are valid)
    // - Most I/O errors in walkdir are handled gracefully
    // - Path validation errors would occur at the filesystem level, not in
    //   DirScanner
}
