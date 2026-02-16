//! File system abstraction for testable file I/O.
//!
//! This module provides the [`FileSource`] trait and its implementations for
//! abstracting file system operations, enabling dependency injection and
//! comprehensive testing without touching the real filesystem.
//!
//! # Architecture
//!
//! - **FileSource Trait**: Defines the contract for file system operations
//! - **FsFileSource**: Production implementation using `std::fs`
//! - **InMemoryFileSource**: Test double using in-memory storage
//!
//! # Usage
//!
//! ```rust
//! use std::path::Path;
//!
//! use lithos_core::fs::source::{FileSource, FsFileSource};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let root = Path::new("/vault");
//! let source = FsFileSource::new(root);
//!
//! if source.exists(Path::new("schemas/note.json")) {
//!     let content = source.read_to_string(Path::new("schemas/note.json"))?;
//!     println!("Schema content: {}", content);
//! }
//! # Ok(())
//! # }
//! ```

use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

/// Abstraction over file system operations for dependency injection.
///
/// This trait enables testable file I/O by allowing production code to use
/// real filesystem access ([`FsFileSource`]) while tests use in-memory
/// implementations ([`InMemoryFileSource`]).
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` to support concurrent access
/// patterns in ingestion services.
///
/// # Error Handling
///
/// Each implementation defines its own error type via the associated `Error`
/// type, which must implement `std::error::Error + Send + Sync + 'static` for
/// composability with application-level error handling.
pub trait FileSource: Send + Sync {
    /// Error type for file operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Checks if a file exists at the given path.
    ///
    /// Returns `true` if the file exists, `false` otherwise.
    /// Does not distinguish between "file not found" and other errors.
    #[must_use]
    fn exists(&self, path: &Path) -> bool;

    /// Lists all files matching a glob pattern.
    ///
    /// The pattern syntax follows standard glob conventions:
    /// - `*.json` - all JSON files in the root directory
    /// - `**/*.json` - all JSON files recursively
    /// - `schemas/*.{json,toml,yaml}` - schema files with multiple extensions
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The glob pattern is invalid
    /// - Directory traversal fails (permissions, I/O error)
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error>;

    /// Reads the entire contents of a file as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist
    /// - The file cannot be read (permissions, I/O error)
    /// - The file contents are not valid UTF-8
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error>;
}

/// Production file source using `std::fs` for real filesystem access.
///
/// This implementation provides scoped access to a root directory, ensuring
/// all file operations are confined to that subtree.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
///
/// use lithos_core::fs::source::{FileSource, FsFileSource};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let vault_root = Path::new("/Users/alice/vault");
/// let source = FsFileSource::new(vault_root);
///
/// // Read a schema file (relative to vault root)
/// let content = source.read_to_string(Path::new("schemas/note.json"))?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FsFileSource {
    /// Root directory for scoped file access.
    root: PathBuf,
}

impl FsFileSource {
    /// Creates a new filesystem source scoped to the given root directory.
    ///
    /// All subsequent file operations will be resolved relative to this root.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use lithos_core::fs::source::FsFileSource;
    ///
    /// let source = FsFileSource::new(Path::new("/vault"));
    /// ```
    #[inline]
    #[must_use]
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Returns the root directory for this file source.
    #[inline]
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a relative path against the root directory.
    #[inline]
    fn resolve_path(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }
}

impl FileSource for FsFileSource {
    type Error = io::Error;

    #[inline]
    fn exists(&self, path: &Path) -> bool {
        let full_path = self.resolve_path(path);
        full_path.exists()
    }

    #[inline]
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
        // Build the full glob pattern relative to root
        let full_pattern = self.root.join(pattern);
        let pattern_str = full_pattern.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Pattern contains invalid UTF-8",
            )
        })?;

        // Use walkdir to traverse directories and match pattern
        Ok(walkdir::WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                // Only include files (not directories or symlinks)
                if entry.file_type().is_dir() || entry.file_type().is_symlink()
                {
                    return None;
                }

                // Match against the pattern
                if glob::Pattern::new(pattern_str).ok()?.matches_path(path) {
                    // Return path relative to root
                    path.strip_prefix(&self.root).ok().map(Path::to_path_buf)
                } else {
                    None
                }
            })
            .collect())
    }

    #[inline]
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        let full_path = self.resolve_path(path);
        std::fs::read_to_string(full_path)
    }
}

/// In-memory file source for testing.
///
/// This implementation stores file contents in a `HashMap`, allowing tests to
/// set up fake filesystems without touching disk.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
///
/// use lithos_core::fs::source::{FileSource, InMemoryFileSource};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut source = InMemoryFileSource::new();
/// source.insert(Path::new("test.txt"), "Hello, world!".to_owned());
///
/// assert!(source.exists(Path::new("test.txt")));
/// assert_eq!(source.read_to_string(Path::new("test.txt"))?, "Hello, world!");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct InMemoryFileSource {
    /// In-memory file storage mapping paths to contents.
    files: HashMap<PathBuf, String>,
}

impl InMemoryFileSource {
    /// Creates a new empty in-memory file source.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Inserts a file into the in-memory filesystem.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    ///
    /// use lithos_core::fs::source::InMemoryFileSource;
    ///
    /// let mut source = InMemoryFileSource::new();
    /// source.insert(
    ///     Path::new("config.toml"),
    ///     "[settings]\nkey = \"value\"".to_owned(),
    /// );
    /// ```
    #[inline]
    pub fn insert(&mut self, path: &Path, content: String) {
        self.files.insert(path.to_path_buf(), content);
    }

    /// Returns the number of files in the in-memory filesystem.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` if the in-memory filesystem is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Clears all files from the in-memory filesystem.
    #[inline]
    pub fn clear(&mut self) {
        self.files.clear();
    }
}

impl FileSource for InMemoryFileSource {
    type Error = io::Error;

    #[inline]
    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    #[inline]
    fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>, Self::Error> {
        // Parse the glob pattern
        let glob_pattern = glob::Pattern::new(pattern).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid glob pattern: {e}"),
            )
        })?;

        // Filter files that match the pattern
        let mut matching_files: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|path| glob_pattern.matches_path(path))
            .cloned()
            .collect();

        // Sort for deterministic ordering in tests
        matching_files.sort();

        Ok(matching_files)
    }

    #[inline]
    fn read_to_string(&self, path: &Path) -> Result<String, Self::Error> {
        self.files.get(path).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_file_source_new_stores_root() {
        let root = Path::new("/vault");
        let source = FsFileSource::new(root);
        assert_eq!(source.root(), root);
    }

    #[test]
    fn fs_file_source_resolve_path_joins_root() {
        let source = FsFileSource::new(Path::new("/vault"));
        let resolved = source.resolve_path(Path::new("schemas/note.json"));
        assert_eq!(resolved, PathBuf::from("/vault/schemas/note.json"));
    }

    #[test]
    fn in_memory_file_source_new_creates_empty() {
        let source = InMemoryFileSource::new();
        assert!(source.is_empty());
        assert_eq!(source.len(), 0);
    }

    #[test]
    fn in_memory_file_source_insert_and_read() {
        let mut source = InMemoryFileSource::new();
        source.insert(Path::new("test.txt"), "Hello, world!".to_owned());

        assert_eq!(source.len(), 1);
        assert!(!source.is_empty());

        let content = source
            .read_to_string(Path::new("test.txt"))
            .expect("Failed to read file");
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn in_memory_file_source_exists_returns_true_for_inserted_file() {
        let mut source = InMemoryFileSource::new();
        source.insert(Path::new("test.txt"), "content".to_owned());

        assert!(source.exists(Path::new("test.txt")));
        assert!(!source.exists(Path::new("missing.txt")));
    }

    #[test]
    fn in_memory_file_source_read_returns_error_for_missing_file() {
        let source = InMemoryFileSource::new();
        let result = source.read_to_string(Path::new("missing.txt"));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn in_memory_file_source_clear_removes_all_files() {
        let mut source = InMemoryFileSource::new();
        source.insert(Path::new("file1.txt"), "content1".to_owned());
        source.insert(Path::new("file2.txt"), "content2".to_owned());

        assert_eq!(source.len(), 2);

        source.clear();

        assert_eq!(source.len(), 0);
        assert!(source.is_empty());
    }

    #[test]
    fn in_memory_file_source_list_files_matches_glob_pattern() {
        let mut source = InMemoryFileSource::new();
        source.insert(Path::new("schema1.json"), "{}".to_owned());
        source.insert(Path::new("schema2.json"), "{}".to_owned());
        source.insert(Path::new("template.md"), "# Template".to_owned());
        source.insert(Path::new("note.md"), "# Note".to_owned());

        let json_files =
            source.list_files("*.json").expect("Failed to list files");
        assert_eq!(json_files.len(), 2);
        assert!(json_files.contains(&PathBuf::from("schema1.json")));
        assert!(json_files.contains(&PathBuf::from("schema2.json")));

        let md_files = source.list_files("*.md").expect("Failed to list files");
        assert_eq!(md_files.len(), 2);
        assert!(md_files.contains(&PathBuf::from("template.md")));
        assert!(md_files.contains(&PathBuf::from("note.md")));
    }

    #[test]
    fn in_memory_file_source_list_files_returns_sorted_results() {
        let mut source = InMemoryFileSource::new();
        source.insert(Path::new("c.txt"), String::new());
        source.insert(Path::new("a.txt"), String::new());
        source.insert(Path::new("b.txt"), String::new());

        let files = source.list_files("*.txt").expect("Failed to list files");
        assert_eq!(files, vec![
            PathBuf::from("a.txt"),
            PathBuf::from("b.txt"),
            PathBuf::from("c.txt"),
        ]);
    }

    #[test]
    fn in_memory_file_source_list_files_returns_error_for_invalid_pattern() {
        let source = InMemoryFileSource::new();
        let result = source.list_files("[invalid");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }
}
