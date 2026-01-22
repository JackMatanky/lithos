//! Path validation utilities for secure file system operations.
//!
//! This module provides validation utilities to prevent path traversal attacks,
//! restrict access to sensitive files, and safely handle symbolic links.
//!
//! # Security Guarantees
//!
//! - **Path Traversal Prevention**: Rejects paths containing `..` components
//! - **Absolute Path Rejection**: Ensures only relative paths are used
//! - **Restricted File Protection**: Blocks access to hidden/sensitive files
//! - **Symlink Escape Detection**: Validates symlinks stay within root
//!   boundaries
//!
//! # Modes
//!
//! - **Strict**: Enforces root boundary and rejects symlinks escaping the root
//! - **Flexible**: Allows external symlinks (e.g., dotfiles) while still
//!   checking input path for traversal
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//!
//! use lithos_adapters::spi::fs::validator::Validator;
//!
//! // Flexible validator for config files (allows dotfile symlinks)
//! let validator = Validator::new_flexible();
//! assert!(validator.validate("config/lithos.toml").is_ok());
//! assert!(validator.validate("../../etc/passwd").is_err());
//!
//! // Strict validator for vault files (enforces root boundary)
//! let root = PathBuf::from("/vault");
//! let validator = Validator::new_strict(root);
//! assert!(validator.validate("notes/daily.md").is_ok());
//! assert!(validator.validate(".git/config").is_err());
//! ```

use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

/// Path validation error types.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathValidationError {
    /// Path is absolute when only relative paths are allowed.
    #[error("Absolute path not allowed: {0}")]
    AbsolutePath(String),

    /// I/O error during symlink resolution.
    #[error("I/O error during symlink resolution: {0}")]
    IoError(String),

    /// Path contains `..` components attempting traversal outside allowed
    /// directory.
    #[error("Path traversal detected: path contains '..' components")]
    PathTraversal,

    /// Path accesses restricted or hidden files.
    #[error("Restricted path access denied: {0}")]
    RestrictedPath(String),

    /// Symlink target escapes the configured root directory.
    #[error("Symlink escape detected: target is outside root boundary")]
    SymlinkEscape,
}

/// Path validator with configurable security modes.
///
/// # Invariants
///
/// - **Traversal Safety**: Always rejects `..` components in input paths
/// - **Platform Agnostic**: Correctly handles Windows and Unix path separators
/// - **Async-Safe**: Uses `tokio::fs` for symlink resolution to avoid blocking
///
/// # Example
///
/// ```
/// use lithos_adapters::spi::fs::validator::Validator;
///
/// let validator = Validator::new_flexible();
/// match validator.validate("config.toml") {
///     Ok(safe_path) => println!("Safe path: {:?}", safe_path),
///     Err(e) => eprintln!("Validation error: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Validator {
    mode: ValidationMode,
}

#[derive(Debug, Clone)]
enum ValidationMode {
    /// Flexible mode: allows external symlinks (e.g., dotfiles), still checks
    /// input traversal.
    Flexible,
    /// Strict mode: enforces root boundary, rejects symlinks escaping root.
    Strict {
        root: PathBuf,
    },
}

impl Validator {
    /// Creates a flexible validator that allows external symlinks.
    ///
    /// # Use Cases
    ///
    /// - Configuration files that may be symlinked from dotfile repositories
    /// - Schema files in shared locations
    ///
    /// # Example
    ///
    /// ```
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let validator = Validator::new_flexible();
    /// assert!(validator.validate("config.toml").is_ok());
    /// ```
    #[inline]
    #[must_use]
    pub fn new_flexible() -> Self {
        Self {
            mode: ValidationMode::Flexible,
        }
    }

    /// Creates a strict validator with root boundary enforcement.
    ///
    /// # Use Cases
    ///
    /// - Vault note files that must remain within vault directory
    /// - Any file system jail/chroot scenario
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let root = PathBuf::from("/vault");
    /// let validator = Validator::new_strict(root);
    /// assert!(validator.validate("notes/daily.md").is_ok());
    /// ```
    #[inline]
    #[must_use]
    pub fn new_strict(root: PathBuf) -> Self {
        Self {
            mode: ValidationMode::Strict {
                root,
            },
        }
    }

    /// Safely resolves a symlink, ensuring it stays within bounds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let root = PathBuf::from("/vault");
    /// let validator = Validator::new_strict(root);
    ///
    /// let symlink_path = PathBuf::from("/vault/note_link");
    /// let resolved = validator.resolve_safe_symlink(&symlink_path).await?;
    /// println!("Resolved to: {:?}", resolved);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Behavior by Mode
    ///
    /// - **Flexible**: Follows symlink without boundary checks (allows
    ///   dotfiles)
    /// - **Strict**: Ensures resolved target stays within configured root
    ///
    /// # Async Safety
    ///
    /// Uses `tokio::fs::canonicalize` to avoid blocking the async runtime.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Resolved symlink target path
    /// - `Err(PathValidationError)`: Symlink escapes root or I/O error
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::SymlinkEscape`]: Symlink target outside root
    ///   (strict mode)
    /// - [`PathValidationError::IoError`]: I/O error during resolution
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: matching on &ValidationMode enum is more \
                  idiomatic than explicit dereferencing"
    )]
    pub async fn resolve_safe_symlink<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<PathBuf, PathValidationError> {
        let path_ref = path.as_ref();

        // Use tokio::fs::canonicalize for async symlink resolution
        let resolved = tokio::fs::canonicalize(path_ref)
            .await
            .map_err(|e| PathValidationError::IoError(e.to_string()))?;

        match &self.mode {
            ValidationMode::Flexible => {
                // Flexible mode: allow any resolved path
                Ok(resolved)
            }
            ValidationMode::Strict {
                root,
            } => {
                // Strict mode: ensure resolved path is within root
                let canonical_root = tokio::fs::canonicalize(root)
                    .await
                    .map_err(|e| PathValidationError::IoError(e.to_string()))?;

                if resolved.starts_with(&canonical_root) {
                    Ok(resolved)
                } else {
                    Err(PathValidationError::SymlinkEscape)
                }
            }
        }
    }

    /// Validates a path for security issues.
    ///
    /// # Checks Performed
    ///
    /// 1. **Traversal Check**: Rejects paths with `..` components
    /// 2. **Absolute Path Check**: Rejects absolute paths
    /// 3. **Restricted File Check**: Rejects hidden/sensitive files (default
    ///    mode)
    ///
    /// # Returns
    ///
    /// - `Ok(Cow<'_, Path>)`: Path is safe, returns borrowed or owned path
    /// - `Err(PathValidationError)`: Path is unsafe, returns specific error
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::PathTraversal`]: Path contains `..` components
    /// - [`PathValidationError::AbsolutePath`]: Path is absolute
    /// - [`PathValidationError::RestrictedPath`]: Path accesses
    ///   hidden/sensitive files
    ///
    /// # Example
    ///
    /// ```
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let validator = Validator::new_flexible();
    ///
    /// // Valid paths
    /// assert!(validator.validate("config.toml").is_ok());
    /// assert!(validator.validate("notes/daily.md").is_ok());
    ///
    /// // Invalid paths
    /// assert!(validator.validate("../../etc/passwd").is_err());
    /// assert!(validator.validate("/etc/hosts").is_err());
    /// assert!(validator.validate(".env").is_err());
    /// ```
    #[inline]
    pub fn validate<'path, PathType>(
        &self,
        path: &'path PathType,
    ) -> Result<Cow<'path, Path>, PathValidationError>
    where
        PathType: AsRef<Path> + ?Sized,
    {
        let path_ref = path.as_ref();

        // Check 1: Reject absolute paths
        if path_ref.is_absolute() {
            return Err(PathValidationError::AbsolutePath(
                path_ref.display().to_string(),
            ));
        }

        // Check 2: Reject path traversal (..)
        for component in path_ref.components() {
            if component == Component::ParentDir {
                return Err(PathValidationError::PathTraversal);
            }
        }

        // Check 3: Reject restricted/hidden files
        for component in path_ref.components() {
            if let Component::Normal(os_str) = component
                && let Some(name) = os_str.to_str()
                && name.starts_with('.')
            {
                return Err(PathValidationError::RestrictedPath(
                    path_ref.display().to_string(),
                ));
            }
        }

        // Path is valid - return as borrowed Cow to avoid allocation
        Ok(Cow::Borrowed(path_ref))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod constructor {
        use super::*;

        #[test]
        fn creates_flexible_validator() {
            let validator = Validator::new_flexible();
            assert!(matches!(validator.mode, ValidationMode::Flexible));
        }

        #[test]
        #[expect(
            clippy::unreachable,
            reason = "Validates exhaustive match in test"
        )]
        fn creates_strict_validator_with_root() {
            let root = PathBuf::from("/test");
            let validator = Validator::new_strict(root.clone());
            match validator.mode {
                ValidationMode::Strict {
                    root: r,
                } => assert_eq!(r, root),
                ValidationMode::Flexible => {
                    unreachable!("new_strict should create Strict mode")
                }
            }
        }
    }

    mod path_traversal {
        use super::*;

        #[test]
        fn rejects_double_dot_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("../../etc/passwd");

            assert!(result.is_err(), "Should reject path with .. components");
            assert!(matches!(result, Err(PathValidationError::PathTraversal)));
        }

        #[test]
        fn rejects_single_parent_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("../config.toml");

            assert!(result.is_err(), "Should reject single .. traversal");
            assert!(matches!(result, Err(PathValidationError::PathTraversal)));
        }

        #[test]
        fn rejects_mid_path_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("valid/../../etc/passwd");

            assert!(result.is_err(), "Should reject .. in middle of path");
            assert!(matches!(result, Err(PathValidationError::PathTraversal)));
        }

        #[test]
        fn handles_encoded_characters_as_literal() {
            let validator = Validator::new_flexible();
            // URL-encoded characters are treated as literal filename characters
            let result = validator.validate("safe%2Ffile");

            // This is a valid filename (% and chars are literal, not path
            // separators)
            assert!(
                result.is_ok(),
                "URL encoding creates literal filename chars"
            );
        }
    }

    mod absolute_paths {
        use super::*;

        #[test]
        fn rejects_unix_absolute_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("/etc/hosts");

            assert!(result.is_err(), "Should reject Unix absolute path");
            assert!(matches!(
                result,
                Err(PathValidationError::AbsolutePath(_))
            ));
        }

        #[test]
        #[cfg(target_os = "windows")]
        fn rejects_windows_absolute_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("C:\\Windows\\System32");

            assert!(result.is_err(), "Should reject Windows absolute path");
            assert!(matches!(
                result,
                Err(PathValidationError::AbsolutePath(_))
            ));
        }

        #[test]
        #[cfg(target_os = "windows")]
        fn rejects_windows_unc_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("\\\\server\\share\\file");

            assert!(result.is_err(), "Should reject UNC path");
            assert!(matches!(
                result,
                Err(PathValidationError::AbsolutePath(_))
            ));
        }

        #[test]
        fn accepts_relative_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config/lithos.toml");

            assert!(result.is_ok(), "Should accept valid relative path");
        }
    }

    mod restricted_files {
        use super::*;

        #[test]
        fn rejects_git_config() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".git/config");

            assert!(result.is_err(), "Should reject .git directory access");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPath(_))
            ));
        }

        #[test]
        fn rejects_env_file() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".env");

            assert!(result.is_err(), "Should reject .env file");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPath(_))
            ));
        }

        #[test]
        fn rejects_nested_hidden_file() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config/.env");

            assert!(result.is_err(), "Should reject nested hidden file");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPath(_))
            ));
        }

        #[test]
        fn rejects_ssh_keys() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".ssh/id_rsa");

            assert!(result.is_err(), "Should reject SSH key access");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPath(_))
            ));
        }

        #[test]
        fn accepts_normal_file() {
            let validator = Validator::new_flexible();
            let result = validator.validate("notes/daily.md");

            assert!(result.is_ok(), "Should accept normal file path");
        }
    }

    mod symlink_strict {
        use super::*;

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect for clarity"
        )]
        async fn rejects_escaped_symlink() {
            let temp_dir = tempfile::TempDir::new().expect("test setup failed");
            let root = temp_dir.path();

            // Create symlink pointing outside root
            let outside_target = std::env::temp_dir().join("outside.txt");
            std::fs::write(&outside_target, "outside content")
                .expect("test setup failed");

            let symlink_path = root.join("escaped_link");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&outside_target, &symlink_path)
                .expect("test setup failed");
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&outside_target, &symlink_path)
                .expect("test setup failed");

            let validator = Validator::new_strict(root.to_path_buf());
            let result = validator.resolve_safe_symlink(&symlink_path).await;

            assert!(result.is_err(), "Should reject symlink escaping root");
            assert!(matches!(result, Err(PathValidationError::SymlinkEscape)));

            // Cleanup - ignore errors as test is complete
            drop(std::fs::remove_file(&outside_target));
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect for clarity"
        )]
        async fn accepts_internal_symlink() {
            let temp_dir = tempfile::TempDir::new().expect("test setup failed");
            let root = temp_dir.path();

            let target = root.join("target.txt");
            std::fs::write(&target, "internal content")
                .expect("test setup failed");

            let symlink_path = root.join("internal_link");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &symlink_path)
                .expect("test setup failed");
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&target, &symlink_path)
                .expect("test setup failed");

            let validator = Validator::new_strict(root.to_path_buf());
            let result = validator.resolve_safe_symlink(&symlink_path).await;

            assert!(result.is_ok(), "Should accept symlink within root");
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect for clarity"
        )]
        async fn detects_symlink_loop() {
            let temp_dir = tempfile::TempDir::new().expect("test setup failed");
            let root = temp_dir.path();

            let link_a = root.join("link_a");
            let link_b = root.join("link_b");

            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&link_b, &link_a)
                    .expect("test setup failed");
                std::os::unix::fs::symlink(&link_a, &link_b)
                    .expect("test setup failed");
            };
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_file(&link_b, &link_a)
                    .expect("test setup failed");
                std::os::windows::fs::symlink_file(&link_a, &link_b)
                    .expect("test setup failed");
            }

            let validator = Validator::new_strict(root.to_path_buf());
            let result = validator.resolve_safe_symlink(&link_a).await;

            assert!(result.is_err(), "Should detect symlink loop");
            assert!(matches!(result, Err(PathValidationError::IoError(_))));
        }
    }

    mod symlink_flexible {
        use super::*;

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect for clarity"
        )]
        async fn allows_external_symlink() {
            let temp_dir = tempfile::TempDir::new().expect("test setup failed");
            let root = temp_dir.path();

            let outside_target = std::env::temp_dir().join("dotfile.toml");
            std::fs::write(&outside_target, "dotfile content")
                .expect("test setup failed");

            let symlink_path = root.join("dotfile_link");
            #[cfg(unix)]
            std::os::unix::fs::symlink(&outside_target, &symlink_path)
                .expect("test setup failed");
            #[cfg(windows)]
            std::os::windows::fs::symlink_file(&outside_target, &symlink_path)
                .expect("test setup failed");

            let validator = Validator::new_flexible();
            let result = validator.resolve_safe_symlink(&symlink_path).await;

            assert!(
                result.is_ok(),
                "Should allow external symlink in flexible mode"
            );

            // Cleanup - ignore errors as test is complete
            drop(std::fs::remove_file(&outside_target));
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "matches! macro uses expect internally"
        )]
        async fn still_checks_input_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("../../../dotfile");

            assert!(
                result.is_err(),
                "Flexible mode still rejects traversal in input"
            );
            assert!(matches!(result, Err(PathValidationError::PathTraversal)));
        }
    }

    mod valid_paths {
        use super::*;

        #[test]
        fn accepts_simple_filename() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config.toml");

            assert!(result.is_ok(), "Should accept simple filename");
        }

        #[test]
        fn accepts_nested_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("notes/2024/january/daily.md");

            assert!(result.is_ok(), "Should accept nested relative path");
        }

        #[test]
        fn returns_cow_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config.toml");

            assert!(
                result.is_ok(),
                "Validation should succeed: {:?}",
                result.err()
            );
            if let Ok(validated_path) = result {
                assert!(matches!(validated_path, Cow::Borrowed(_)));
                assert_eq!(validated_path.as_ref(), Path::new("config.toml"));
            }
        }

        #[test]
        fn normalization_preserves_valid_paths() {
            let validator = Validator::new_flexible();
            let result = validator.validate("./config.toml");

            assert!(result.is_ok(), "Should handle ./ prefix correctly");
        }
    }

    mod platform_specific {
        use super::*;

        #[test]
        fn handles_platform_separators() {
            let validator = Validator::new_flexible();

            #[cfg(unix)]
            let path = "config/notes/file.md";
            #[cfg(windows)]
            let path = "config\\notes\\file.md";

            let result = validator.validate(path);
            assert!(result.is_ok(), "Should handle platform separators");
        }

        #[test]
        fn mixed_separators() {
            let validator = Validator::new_flexible();

            #[cfg(windows)]
            {
                let result = validator.validate("config/notes\\file.md");
                assert!(
                    result.is_ok(),
                    "Windows should handle mixed separators"
                );
            }

            #[cfg(unix)]
            {
                // Backslash is valid filename character on Unix
                let result = validator.validate("config/notes\\file.md");
                assert!(
                    result.is_ok(),
                    "Unix treats backslash as filename char"
                );
            }
        }
    }
}
