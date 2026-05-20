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
            DirPath::new(path.to_path_buf())
                .map(FsPath::Dir)
                .map_err(ScanError::from)
        } else {
            FilePath::new(path.to_path_buf())
                .map(FsPath::File)
                .map_err(ScanError::from)
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

    mod fixtures {
        use super::*;

        /// Helper to create test files with content.
        pub(super) fn write_file(
            root: &Path,
            relative: &str,
            contents: &[u8],
        ) -> PathBuf {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create test dirs");
            }
            std::fs::write(&path, contents).expect("write test file");
            path
        }
    }

    mod defaults {
        use super::*;

        #[test]
        fn returns_no_pattern_by_default() {
            let input = DirScanInput::default();
            assert!(input.pattern.is_none());
        }

        #[test]
        fn returns_no_extensions_by_default() {
            let input = DirScanInput::default();
            assert!(input.extensions.is_none());
        }

        #[test]
        fn excludes_dirs_by_default() {
            let input = DirScanInput::default();
            assert!(!input.include_dirs);
        }

        #[test]
        fn ignores_symlinks_by_default() {
            let input = DirScanInput::default();
            assert!(!input.follow_symlinks);
        }

        #[test]
        fn enables_recursion_by_default() {
            let input = DirScanInput::default();
            assert!(input.recursive);
        }
    }

    mod builder {
        use super::*;

        #[test]
        fn sets_pattern_when_with_pattern_called() {
            let input = DirScanInput::new().with_pattern("**/*.toml");
            assert_eq!(input.pattern, Some("**/*.toml"));
        }

        #[test]
        fn sets_extensions_when_with_extensions_called() {
            let input = DirScanInput::new().with_extensions(&["toml", "yaml"]);
            assert_eq!(input.extensions, Some(&["toml", "yaml"][..]));
        }

        #[test]
        fn enables_dir_inclusion_when_include_dirs_called() {
            let input = DirScanInput::new().include_dirs(true);
            assert!(input.include_dirs);
        }

        #[test]
        fn enables_symlink_following_when_follow_symlinks_called() {
            let input = DirScanInput::new().follow_symlinks(true);
            assert!(input.follow_symlinks);
        }

        #[test]
        fn disables_recursion_when_recursive_false() {
            let input = DirScanInput::new().recursive(false);
            assert!(!input.recursive);
        }

        #[test]
        fn chains_multiple_builder_calls() {
            let input = DirScanInput::new()
                .with_pattern("**/*.toml")
                .with_extensions(&["toml"])
                .include_dirs(true);

            assert_eq!(input.pattern, Some("**/*.toml"));
            assert_eq!(input.extensions, Some(&["toml"][..]));
            assert!(input.include_dirs);
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn creates_scanner_with_provided_path() {
            let scanner = DirScanner::new("/test/path");
            assert_eq!(scanner.path(), Path::new("/test/path"));
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn path_returns_constructor_value() {
            let scanner = DirScanner::new("/vault");
            assert_eq!(scanner.path(), Path::new("/vault"));
        }
    }

    mod paths {
        use super::*;

        #[test]
        fn returns_empty_when_dir_is_empty() {
            let temp = TempDir::new().unwrap();
            let scanner = DirScanner::new(temp.path());
            let paths = scanner.paths(DirScanInput::new()).unwrap();
            assert!(paths.is_empty());
        }

        #[test]
        fn returns_sorted_paths_alphabetically() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "z.toml", b"");
            fixtures::write_file(temp.path(), "a.toml", b"");
            fixtures::write_file(temp.path(), "m.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(paths.len(), 3);
            assert_eq!(
                paths.first().expect("first path").as_path(),
                root.join("a.toml")
            );
            assert_eq!(
                paths.get(1).expect("second path").as_path(),
                root.join("m.toml")
            );
            assert_eq!(
                paths.get(2).expect("third path").as_path(),
                root.join("z.toml")
            );
        }

        #[test]
        fn returns_absolute_fs_paths() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "a.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths = scanner.paths(DirScanInput::new()).unwrap();

            assert_eq!(paths.len(), 1);
            let path = paths.first().expect("path");
            assert!(path.is_file());
            assert_eq!(path.as_path(), root.join("a.toml"));
        }
    }

    mod entries {
        use super::*;

        #[test]
        fn returns_empty_when_dir_is_empty() {
            let temp = TempDir::new().unwrap();
            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            assert!(entries.is_empty());
        }

        #[test]
        fn returns_entries_with_metadata() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "a.toml", b"test content");

            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            let expected = temp.path().join("a.toml");

            assert_eq!(entries.len(), 1);
            let entry = entries.first().expect("entry");
            assert_eq!(entry.path_ref().as_path(), expected);
            assert_eq!(
                entry.filename().map(|n| n.as_str().to_owned()),
                Some("a.toml".to_owned())
            );
            assert_eq!(
                entry
                    .metadata()
                    .as_file()
                    .map(crate::fs::metadata::FileMetadata::size),
                Some(12)
            );
        }

        #[test]
        fn returns_sorted_entries_alphabetically() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "z.toml", b"");
            fixtures::write_file(temp.path(), "a.toml", b"");

            let scanner = DirScanner::new(temp.path());
            let entries = scanner.entries(DirScanInput::new()).unwrap();
            let expected_a = temp.path().join("a.toml");
            let expected_z = temp.path().join("z.toml");

            assert_eq!(
                entries.first().expect("first entry").path_ref().as_path(),
                expected_a
            );
            assert_eq!(
                entries.get(1).expect("second entry").path_ref().as_path(),
                expected_z
            );
        }

        #[test]
        fn returns_entries_with_absolute_paths() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "a.toml", b"test content");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let entries = scanner.entries(DirScanInput::new()).unwrap();

            assert_eq!(entries.len(), 1);
            let entry = entries.first().expect("entry");
            assert!(entry.is_file());
            assert_eq!(entry.path().as_path(), root.join("a.toml"));
            assert_eq!(entry.as_file().unwrap().metadata().size(), 12);
        }
    }

    mod filter {
        use super::*;

        mod pattern {
            use super::*;

            #[test]
            fn returns_only_paths_matching_glob() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "a.toml", b"");
                fixtures::write_file(temp.path(), "b.yaml", b"");
                fixtures::write_file(temp.path(), "sub/c.toml", b"");
                let root = temp.path().to_path_buf();

                let scanner = DirScanner::new(&root);
                let paths = scanner
                    .paths(DirScanInput::new().with_pattern("**/*.toml"))
                    .unwrap();

                assert_eq!(paths.len(), 2);
                assert!(
                    paths.iter().any(|p| p.as_path() == root.join("a.toml"))
                );
                assert!(
                    paths
                        .iter()
                        .any(|p| p.as_path() == root.join("sub/c.toml"))
                );
            }
        }

        mod extension {
            use super::*;

            #[test]
            fn returns_only_files_with_specified_extensions() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "a.toml", b"");
                fixtures::write_file(temp.path(), "b.yaml", b"");
                fixtures::write_file(temp.path(), "c.json", b"");
                let root = temp.path().to_path_buf();

                let scanner = DirScanner::new(&root);
                let paths = scanner
                    .paths(
                        DirScanInput::new().with_extensions(&["toml", "yaml"]),
                    )
                    .unwrap();

                assert_eq!(paths.len(), 2);
                assert!(
                    paths.iter().any(|p| p.as_path() == root.join("a.toml"))
                );
                assert!(
                    paths.iter().any(|p| p.as_path() == root.join("b.yaml"))
                );
            }

            #[test]
            fn matches_extensions_case_insensitively() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "a.TOML", b"");

                let scanner = DirScanner::new(temp.path());
                let paths = scanner
                    .paths(DirScanInput::new().with_extensions(&["toml"]))
                    .unwrap();

                assert_eq!(paths.len(), 1);
            }
        }

        mod combined {
            use super::*;

            #[test]
            fn returns_only_matches_satisfying_pattern_and_extension() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "schemas/a.toml", b"");
                fixtures::write_file(temp.path(), "schemas/b.yaml", b"");
                fixtures::write_file(temp.path(), "other/c.toml", b"");
                let root = temp.path().to_path_buf();

                let scanner = DirScanner::new(&root);
                let paths = scanner
                    .paths(
                        DirScanInput::new()
                            .with_pattern("schemas/**/*")
                            .with_extensions(&["toml"]),
                    )
                    .unwrap();

                // Only schemas/a.toml matches BOTH
                assert_eq!(paths.len(), 1);
                assert_eq!(
                    paths.first().expect("path").as_path(),
                    root.join("schemas/a.toml")
                );
            }
        }

        mod directories {
            use super::*;

            #[test]
            fn excludes_directories_by_default() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "file.toml", b"");
                std::fs::create_dir_all(temp.path().join("subdir")).unwrap();
                let root = temp.path().to_path_buf();

                let scanner = DirScanner::new(&root);
                let paths = scanner.paths(DirScanInput::new()).unwrap();

                assert_eq!(paths.len(), 1);
                assert_eq!(
                    paths.first().expect("path").as_path(),
                    root.join("file.toml")
                );
            }

            #[test]
            fn includes_directories_when_include_dirs_enabled() {
                let temp = TempDir::new().unwrap();
                fixtures::write_file(temp.path(), "file.toml", b"");
                std::fs::create_dir_all(temp.path().join("subdir")).unwrap();
                let root = temp.path().to_path_buf();

                let scanner = DirScanner::new(&root);
                let paths = scanner
                    .paths(
                        DirScanInput::new().include_dirs(true).recursive(false),
                    )
                    .unwrap();

                assert_eq!(paths.len(), 2);
                assert!(
                    paths.iter().any(|p| p.as_path() == root.join("file.toml"))
                );
                assert!(
                    paths.iter().any(|p| p.as_path() == root.join("subdir"))
                );
            }
        }
    }

    mod traversal {
        use super::*;

        #[test]
        fn returns_only_immediate_children_when_non_recursive() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "a.toml", b"");
            fixtures::write_file(temp.path(), "sub/b.toml", b"");
            let root = temp.path().to_path_buf();

            let scanner = DirScanner::new(&root);
            let paths =
                scanner.paths(DirScanInput::new().recursive(false)).unwrap();

            assert_eq!(paths.len(), 1);
            assert_eq!(
                paths.first().expect("path").as_path(),
                root.join("a.toml")
            );
        }

        #[test]
        fn returns_nested_files_when_recursive() {
            let temp = TempDir::new().unwrap();
            fixtures::write_file(temp.path(), "a.toml", b"");
            fixtures::write_file(temp.path(), "sub/b.toml", b"");
            fixtures::write_file(temp.path(), "sub/deep/c.toml", b"");

            let scanner = DirScanner::new(temp.path());
            let paths =
                scanner.paths(DirScanInput::new().recursive(true)).unwrap();

            assert_eq!(paths.len(), 3);
        }
    }

    // Note: Error cases are minimal because:
    // - glob::Pattern::new() is very permissive (even malformed patterns like
    //   "[" are valid)
    // - Most I/O errors in walkdir are handled gracefully
    // - Path validation errors would occur at the filesystem level, not in
    //   DirScanner
}
