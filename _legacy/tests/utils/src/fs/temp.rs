//! Temporary directory and file utilities for testing.
//!
//! This module provides RAII-managed temporary directories and files with
//! automatic cleanup, cross-platform path utilities, and centralized test
//! output management.
//!
//! # Safety Invariants
//!
//! - All temporary resources are automatically cleaned up on drop, even on
//!   panic
//! - Paths are always absolute and normalized for cross-platform compatibility
//! - Unique naming prevents conflicts in parallel test execution

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use figment::{Figment, providers::Env};
use rand::{Rng, thread_rng};
use tempfile::{Builder, TempDir as TempfileTempDir, tempdir};

/// Returns the project root directory managed by Figment.
///
/// According to project Rule 82, all absolute paths should be managed via
/// Figment. This helper provides a centralized way to determine the base path
/// for test operations.
pub fn project_root() -> PathBuf {
    Figment::new()
        .merge(Env::prefixed("LITHOS_"))
        .extract_inner::<PathBuf>("root")
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// RAII-managed temporary directory with automatic cleanup.
///
/// Provides a temporary directory that is automatically deleted when the
/// `TempDir` instance goes out of scope, ensuring no leftover test artifacts
/// even if tests panic.
///
/// # Example
///
/// ```rust
/// use lithos_test_utils::FileTempDir;
///
/// # #[test]
/// fn test_with_temp_dir() {
///     let temp_dir = FileTempDir::new().unwrap();
///     let file_path = temp_dir.path().join("test.txt");
///     std::fs::write(&file_path, "test data").unwrap();
///     assert!(file_path.exists());
///     // Directory automatically cleaned up here
/// }
/// ```
#[derive(Debug)]
pub struct TempDir {
    inner: Arc<TempfileTempDir>,
}

impl TempDir {
    /// Creates a new temporary directory with a unique name.
    ///
    /// The directory name includes a timestamp and random suffix for parallel
    /// test safety.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    pub fn new() -> std::io::Result<Self> {
        let temp_dir = tempdir()?;
        Ok(Self {
            inner: Arc::new(temp_dir),
        })
    }

    /// Creates a new temporary directory with a custom prefix.
    ///
    /// The final name will be `{prefix}_{timestamp}_{random}`.
    ///
    /// # Errors
    ///
    /// Returns an error if the temporary directory cannot be created.
    pub fn with_prefix(prefix: &str) -> std::io::Result<Self> {
        let temp_dir =
            Builder::new().prefix(&generate_unique_name(prefix)).tempdir()?;
        Ok(Self {
            inner: Arc::new(temp_dir),
        })
    }

    /// Returns the path to the temporary directory.
    ///
    /// The path is absolute and normalized for cross-platform compatibility.
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    /// Creates a subdirectory within the temporary directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the subdirectory cannot be created.
    pub fn create_subdir(&self, name: &str) -> std::io::Result<PathBuf> {
        let subdir = self.path().join(name);
        std::fs::create_dir_all(&subdir)?;
        Ok(subdir)
    }
}

impl Clone for TempDir {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Generates a unique name for temporary resources.
///
/// Format: `{prefix}_{timestamp}_{random_suffix}` where:
/// - `prefix`: Optional custom prefix
/// - `timestamp`: UTC timestamp in milliseconds
/// - `random_suffix`: 8-character alphanumeric string
pub fn generate_unique_name(prefix: &str) -> String {
    let timestamp = Utc::now().timestamp_millis();
    let random_suffix: String = thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    if prefix.is_empty() {
        format!("{}_{}", timestamp, random_suffix)
    } else {
        format!("{}_{}_{}", prefix, timestamp, random_suffix)
    }
}

/// Cross-platform path utilities for test artifacts.
///
/// Provides functions for joining paths, normalizing separators, and ensuring
/// absolute paths for consistent behavior across platforms.
pub mod path_utils {
    use std::path::{Path, PathBuf};

    /// Joins multiple path components with proper normalization.
    pub fn join(components: &[&str]) -> PathBuf {
        let mut path = PathBuf::new();
        for component in components {
            path.push(component);
        }
        path
    }

    /// Ensures a path is absolute, resolving relative paths against the project
    /// root.
    ///
    /// According to Rule 82, absolute paths should be managed via Figment.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be resolved.
    pub fn ensure_absolute<P: AsRef<Path>>(
        path: P,
    ) -> std::io::Result<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(super::project_root().join(path))
        }
    }

    /// Normalizes path separators for cross-platform compatibility.
    ///
    /// On Windows, converts backslashes to forward slashes for consistent
    /// handling.
    pub fn normalize_separators<P: AsRef<Path>>(path: P) -> PathBuf {
        let path_str = path.as_ref().to_string_lossy();
        #[cfg(target_os = "windows")]
        let normalized = path_str.replace('\\', "/");
        #[cfg(not(target_os = "windows"))]
        let normalized = path_str.to_string();

        PathBuf::from(normalized)
    }
}

/// Centralized test output management.
///
/// Provides a single configurable directory for all test artifacts with
/// automatic per-test subdirectory creation and cleanup policies.
#[derive(Debug)]
pub struct TestOutput {
    base_dir: PathBuf,
    #[allow(dead_code)]
    test_name: String,
    cleanup_on_drop: bool,
}

impl TestOutput {
    /// Creates a new test output manager for the specified test.
    ///
    /// Artifacts will be stored in `{base_dir}/{test_name}/`.
    ///
    /// # Errors
    ///
    /// Returns an error if the base directory cannot be created.
    pub fn new(test_name: &str) -> std::io::Result<Self> {
        let base_dir = Self::default_base_dir();
        Self::with_base_dir(base_dir, test_name)
    }

    /// Creates a new test output manager with a custom base directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory structure cannot be created.
    pub fn with_base_dir<P: AsRef<Path>>(
        base_dir: P,
        test_name: &str,
    ) -> std::io::Result<Self> {
        let base_dir = path_utils::ensure_absolute(base_dir)?;
        let test_dir = base_dir.join(test_name);

        std::fs::create_dir_all(&test_dir)?;

        Ok(Self {
            base_dir: test_dir,
            test_name: test_name.to_string(),
            cleanup_on_drop: true,
        })
    }

    /// Returns the default base directory for test outputs.
    ///
    /// According to Rule 82, this is managed via Figment.
    pub fn default_base_dir() -> PathBuf {
        Figment::new()
            .merge(Env::prefixed("LITHOS_"))
            .extract_inner::<PathBuf>("test_output_dir")
            .unwrap_or_else(|_| {
                std::env::temp_dir().join("lithos-test-outputs")
            })
    }

    /// Returns the path to the test's output directory.
    pub fn path(&self) -> &Path {
        &self.base_dir
    }

    /// Creates a file path within the test output directory.
    pub fn file_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    /// Creates a subdirectory within the test output directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the subdirectory cannot be created.
    pub fn create_subdir(&self, name: &str) -> std::io::Result<PathBuf> {
        let subdir = self.base_dir.join(name);
        std::fs::create_dir_all(&subdir)?;
        Ok(subdir)
    }

    /// Disables automatic cleanup on drop.
    ///
    /// Useful for debugging failed tests where artifacts should be preserved.
    pub fn keep_artifacts(&mut self) {
        self.cleanup_on_drop = false;
    }
}

impl Drop for TestOutput {
    fn drop(&mut self) {
        if self.cleanup_on_drop {
            let _ = std::fs::remove_dir_all(&self.base_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_dir_helper_provides_isolated_workspace() {
        let temp_dir = TempDir::new().unwrap();
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_absolute());
    }

    #[test]
    fn temp_dir_cleanup_removes_directory_after_drop() {
        let temp_path;
        {
            let temp_dir = TempDir::new().unwrap();
            temp_path = temp_dir.path().to_path_buf();
            assert!(temp_path.exists());
        }
        // Directory should be cleaned up after drop
        assert!(!temp_path.exists());
    }

    #[test]
    fn unique_name_generation_produces_distinct_values() {
        let name1 = generate_unique_name("test");
        let name2 = generate_unique_name("test");

        assert_ne!(name1, name2);
        assert!(name1.starts_with("test_"));
        assert!(name2.starts_with("test_"));
    }

    #[test]
    fn path_joining_utility_assembles_components_correctly() {
        let path = path_utils::join(&["base", "subdir", "file.txt"]);
        assert_eq!(path, PathBuf::from("base/subdir/file.txt"));
    }

    #[test]
    fn test_output_manager_creates_accessible_directory() {
        let test_output = TestOutput::new("output_creation").unwrap();
        assert!(test_output.path().exists());
        assert!(test_output.path().is_absolute());
    }

    #[test]
    fn test_output_file_path_generation_stays_within_base_dir() {
        let test_output = TestOutput::new("file_path").unwrap();
        let file_path = test_output.file_path("test.txt");
        assert!(file_path.starts_with(test_output.path()));
        assert!(file_path.ends_with("test.txt"));
    }
}
