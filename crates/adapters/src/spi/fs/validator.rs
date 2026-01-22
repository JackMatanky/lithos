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

    #[test]
    fn flexible_validator_creation() {
        let validator = Validator::new_flexible();
        assert!(matches!(validator.mode, ValidationMode::Flexible));
    }

    #[test]
    #[expect(
        clippy::unreachable,
        reason = "Validates exhaustive match in test"
    )]
    fn strict_validator_creation() {
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

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses unwrap for clarity"
    )]
    fn validate_accepts_simple_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("config.toml");
        result.unwrap();
    }

    #[test]
    fn validate_rejects_traversal() {
        let validator = Validator::new_flexible();
        let result = validator.validate("../../etc/passwd");
        assert!(matches!(result, Err(PathValidationError::PathTraversal)));
    }

    #[test]
    fn validate_rejects_absolute_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("/etc/hosts");
        assert!(matches!(result, Err(PathValidationError::AbsolutePath(_))));
    }

    #[test]
    fn validate_rejects_hidden_file() {
        let validator = Validator::new_flexible();
        let result = validator.validate(".env");
        assert!(matches!(result, Err(PathValidationError::RestrictedPath(_))));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses expect for clarity"
    )]
    fn validate_returns_borrowed_cow() {
        let validator = Validator::new_flexible();
        let result = validator.validate("config.toml");
        assert!(result.is_ok());
        let cow_path = result.expect("test setup failed");
        assert!(matches!(cow_path, Cow::Borrowed(_)));
    }
}
