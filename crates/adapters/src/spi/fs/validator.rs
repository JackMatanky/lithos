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
//! // Note: Root is canonicalized during creation for performance.
//! let root = PathBuf::from(".");
//! let validator = Validator::new_strict(root);
//! assert!(validator.validate("Cargo.toml").is_ok());
//! ```

use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

/// Public alias for validation mode configuration.
pub use Mode as ValidationMode;

use crate::spi::errors::PathValidationError;

/// Path validator with configurable security modes.
///
/// # Invariants
///
/// - **Traversal Safety**: Always rejects `..` components in input paths
/// - **Platform Agnostic**: Correctly handles Windows and Unix path separators
/// - **Async-Safe**: Uses `tokio::fs` for symlink resolution to avoid blocking
#[derive(Debug, Clone)]
pub struct Validator {
    mode: Mode,
}

/// Internal validation mode representation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Mode {
    /// Flexible mode: allows external symlinks (e.g., dotfiles), still checks
    /// input traversal.
    Flexible,
    /// Strict mode: enforces root boundary, rejects symlinks escaping root.
    Strict {
        /// The canonicalized root directory.
        root: PathBuf,
    },
}

impl Validator {
    /// Internal helper for traversal checks.
    #[inline]
    fn check_traversal(path: &Path) -> Result<(), PathValidationError> {
        for component in path.components() {
            if component == Component::ParentDir {
                return Err(PathValidationError::PathTraversalError);
            }
        }
        Ok(())
    }

    /// Creates a flexible validator that allows external symlinks.
    ///
    /// # Use Cases
    ///
    /// - Configuration files that may be symlinked from dotfile repositories
    /// - Schema files in shared locations
    #[inline]
    #[must_use]
    pub fn new_flexible() -> Self {
        Self {
            mode: Mode::Flexible,
        }
    }

    /// Creates a strict validator with root boundary enforcement.
    ///
    /// The `root` path is canonicalized during construction to optimize
    /// subsequent validation performance.
    ///
    /// # Use Cases
    ///
    /// - Vault note files that must remain within vault directory
    /// - Any file system jail/chroot scenario
    #[inline]
    #[must_use]
    #[expect(
        clippy::disallowed_methods,
        reason = "std::fs::canonicalize is used once during initialization to \
                  optimize subsequent performance. Construction typically \
                  happens during adapter setup, making sync I/O acceptable."
    )]
    pub fn new_strict(root: PathBuf) -> Self {
        // Canonicalize root once at construction to improve resolution
        // performance.
        let root = std::fs::canonicalize(&root).unwrap_or(root);
        Self {
            mode: Mode::Strict {
                root,
            },
        }
    }

    /// Safely resolves a symlink, ensuring it stays within bounds.
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
    /// - `Err(PathValidationError)`: Validation or I/O failure
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::SymlinkEscapeError`]: Target is outside root
    ///   (strict)
    /// - [`PathValidationError::IoError`]: File system error
    /// - [`PathValidationError::PathTraversalError`]: Input path contains `..`
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let root = PathBuf::from(".");
    /// let validator = Validator::new_strict(root);
    ///
    /// let symlink_path = PathBuf::from("link_to_file");
    /// let resolved = validator.resolve_safe_symlink(&symlink_path).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics: matching on &Mode enum is more idiomatic \
                  than explicit dereferencing"
    )]
    pub async fn resolve_safe_symlink<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<PathBuf, PathValidationError> {
        let path_ref = path.as_ref();

        // Enforce traversal checks on the input path
        Self::check_traversal(path_ref)?;

        // Use tokio::fs::canonicalize for async symlink resolution
        let resolved = tokio::fs::canonicalize(path_ref)
            .await
            .map_err(|e| PathValidationError::IoError(e.to_string()))?;

        if let Mode::Strict {
            root,
        } = &self.mode
            && !resolved.starts_with(root)
        {
            return Err(PathValidationError::SymlinkEscapeError);
        }

        Ok(resolved)
    }

    /// Validates a path for security issues.
    ///
    /// # Checks Performed
    ///
    /// 1. **Traversal Check**: Rejects paths with `..` components
    /// 2. **Absolute Path Check**: Rejects absolute paths
    /// 3. **Restricted File Check**: Rejects hidden/sensitive files (those
    ///    starting with `.`)
    ///
    /// # Returns
    ///
    /// - `Ok(Cow<'_, Path>)`: Path is safe, returns borrowed path
    /// - `Err(PathValidationError)`: Path is unsafe, returns specific error
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::PathTraversalError`]: Path contains `..`
    /// - [`PathValidationError::AbsolutePathError`]: Path is absolute
    /// - [`PathValidationError::RestrictedPathError`]: Path accesses hidden
    ///   files
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
            return Err(PathValidationError::AbsolutePathError(
                path_ref.display().to_string(),
            ));
        }

        // Check 2 & 3: Single-pass traversal and hidden file check
        for component in path_ref.components() {
            match component {
                Component::ParentDir => {
                    return Err(PathValidationError::PathTraversalError);
                }
                Component::Normal(os_str) => {
                    if let Some(name) = os_str.to_str()
                        && name.starts_with('.')
                    {
                        return Err(PathValidationError::RestrictedPathError(
                            path_ref.display().to_string(),
                        ));
                    }
                }
                Component::Prefix(_)
                | Component::RootDir
                | Component::CurDir => {}
            }
        }

        Ok(Cow::Borrowed(path_ref))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use tempfile::TempDir;

        use super::*;

        pub struct Workspace {
            /// Keep `temp_dir` to ensure it is not deleted until Workspace is
            /// dropped.
            #[expect(dead_code, reason = "Field used for directory lifecycle")]
            pub temp_dir: TempDir,
            pub root: PathBuf,
        }

        impl Workspace {
            #[expect(
                clippy::disallowed_methods,
                reason = "Setup logic uses expect"
            )]
            pub fn create_file<P: AsRef<Path>>(
                &self,
                path: P,
                content: &str,
            ) -> PathBuf {
                let full_path = self.root.join(path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)
                        .expect("failed to create dirs");
                }
                std::fs::write(&full_path, content)
                    .expect("failed to write file");
                full_path
            }

            #[expect(
                clippy::disallowed_methods,
                reason = "Setup logic uses expect"
            )]
            pub fn create_symlink<P: AsRef<Path>, T: AsRef<Path>>(
                &self,
                link_path: P,
                target: T,
            ) -> PathBuf {
                let full_link_path = self.root.join(link_path);
                if let Some(parent) = full_link_path.parent() {
                    std::fs::create_dir_all(parent)
                        .expect("failed to create dirs");
                }

                #[cfg(unix)]
                std::os::unix::fs::symlink(target, &full_link_path)
                    .expect("failed to create symlink");
                #[cfg(windows)]
                std::os::windows::fs::symlink_file(target, &full_link_path)
                    .expect("failed to create symlink");

                full_link_path
            }

            #[expect(
                clippy::disallowed_methods,
                reason = "Setup logic uses expect"
            )]
            pub fn new() -> Self {
                let temp_dir =
                    TempDir::new().expect("failed to create temp dir");
                let root = temp_dir.path().to_path_buf();
                Self {
                    temp_dir,
                    root,
                }
            }
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn creates_flexible_validator() {
            let validator = Validator::new_flexible();
            assert!(matches!(validator.mode, Mode::Flexible));
        }

        #[test]
        #[expect(clippy::unreachable, reason = "Explicit check for test mode")]
        fn creates_strict_validator_with_root() {
            let root = PathBuf::from(".");
            let validator = Validator::new_strict(root);
            match validator.mode {
                Mode::Strict {
                    root: r,
                } => assert!(r.is_absolute()),
                Mode::Flexible => {
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
            assert!(matches!(
                result,
                Err(PathValidationError::PathTraversalError)
            ));
        }

        #[test]
        fn rejects_single_parent_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("../config.toml");
            assert!(matches!(
                result,
                Err(PathValidationError::PathTraversalError)
            ));
        }

        #[test]
        fn rejects_mid_path_traversal() {
            let validator = Validator::new_flexible();
            let result = validator.validate("valid/../../etc/passwd");
            assert!(matches!(
                result,
                Err(PathValidationError::PathTraversalError)
            ));
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn handles_encoded_characters_as_literal() {
            let validator = Validator::new_flexible();
            validator.validate("safe%2Ffile").expect("should be valid");
        }
    }

    mod absolute_paths {
        use super::*;

        #[test]
        fn rejects_unix_absolute_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("/etc/hosts");
            assert!(matches!(
                result,
                Err(PathValidationError::AbsolutePathError(_))
            ));
        }

        #[test]
        #[cfg(target_os = "windows")]
        fn rejects_windows_absolute_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("C:\\Windows\\System32");
            assert!(matches!(
                result,
                Err(PathValidationError::AbsolutePathError(_))
            ));
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn accepts_relative_path() {
            let validator = Validator::new_flexible();
            validator.validate("config/lithos.toml").expect("should be valid");
        }
    }

    mod restricted_files {
        use super::*;

        #[test]
        fn rejects_git_config() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".git/config");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPathError(_))
            ));
        }

        #[test]
        fn rejects_env_file() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".env");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPathError(_))
            ));
        }

        #[test]
        fn rejects_nested_hidden_file() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config/.env");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPathError(_))
            ));
        }

        #[test]
        fn rejects_ssh_keys() {
            let validator = Validator::new_flexible();
            let result = validator.validate(".ssh/id_rsa");
            assert!(matches!(
                result,
                Err(PathValidationError::RestrictedPathError(_))
            ));
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn accepts_normal_file() {
            let validator = Validator::new_flexible();
            validator.validate("notes/daily.md").expect("should be valid");
        }
    }

    mod symlink_strict {
        use super::{fixtures::Workspace, *};

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect and std::fs for setup"
        )]
        async fn rejects_escaped_symlink() {
            let ws = Workspace::new();

            // Create symlink pointing outside root
            let outside_target = std::env::temp_dir().join("outside.txt");
            std::fs::write(&outside_target, "outside content")
                .expect("test setup failed");

            let symlink_path =
                ws.create_symlink("escaped_link", &outside_target);

            let validator = Validator::new_strict(ws.root.clone());
            let result = validator.resolve_safe_symlink(&symlink_path).await;

            assert!(matches!(
                result,
                Err(PathValidationError::SymlinkEscapeError)
            ));
            drop(std::fs::remove_file(&outside_target));
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses Result validation"
        )]
        async fn accepts_internal_symlink() {
            let ws = Workspace::new();
            let target = ws.create_file("target.txt", "internal content");
            let symlink_path = ws.create_symlink("internal_link", &target);

            let validator = Validator::new_strict(ws.root.clone());
            validator
                .resolve_safe_symlink(&symlink_path)
                .await
                .expect("should be valid");
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect and std::fs for setup"
        )]
        async fn detects_symlink_loop() {
            let ws = Workspace::new();
            let link_a = ws.root.join("link_a");
            let link_b = ws.root.join("link_b");

            #[cfg(unix)]
            {
                drop(std::os::unix::fs::symlink(&link_b, &link_a));
                drop(std::os::unix::fs::symlink(&link_a, &link_b));
            };
            #[cfg(windows)]
            {
                drop(std::os::windows::fs::symlink_file(&link_b, &link_a));
                drop(std::os::windows::fs::symlink_file(&link_a, &link_b));
            }

            let validator = Validator::new_strict(ws.root.clone());
            let result = validator.resolve_safe_symlink(&link_a).await;
            assert!(matches!(result, Err(PathValidationError::IoError(_))));
        }
    }

    mod symlink_flexible {
        use super::{fixtures::Workspace, *};

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test setup uses expect and std::fs for setup"
        )]
        async fn allows_external_symlink() {
            let ws = Workspace::new();

            let outside_target = std::env::temp_dir().join("dotfile.toml");
            std::fs::write(&outside_target, "dotfile content")
                .expect("test setup failed");

            let symlink_path =
                ws.create_symlink("dotfile_link", &outside_target);

            let validator = Validator::new_flexible();
            validator
                .resolve_safe_symlink(&symlink_path)
                .await
                .expect("should be valid");
            drop(std::fs::remove_file(&outside_target));
        }

        #[tokio::test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses Result validation"
        )]
        async fn still_checks_input_traversal() {
            let validator = Validator::new_flexible();
            let result =
                validator.resolve_safe_symlink("../../../dotfile").await;
            assert!(matches!(
                result,
                Err(PathValidationError::PathTraversalError)
            ));
        }
    }

    mod valid_paths {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn accepts_simple_filename() {
            let validator = Validator::new_flexible();
            validator.validate("config.toml").expect("should be valid");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn accepts_nested_path() {
            let validator = Validator::new_flexible();
            validator
                .validate("notes/2024/january/daily.md")
                .expect("should be valid");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn returns_cow_path() {
            let validator = Validator::new_flexible();
            let result = validator.validate("config.toml");
            let validated_path = result.expect("should be valid");
            assert!(matches!(validated_path, Cow::Borrowed(_)));
            assert_eq!(validated_path.as_ref(), Path::new("config.toml"));
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn normalization_preserves_valid_paths() {
            let validator = Validator::new_flexible();
            validator.validate("./config.toml").expect("should be valid");
        }
    }

    mod platform_specific {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for Result validation"
        )]
        fn handles_platform_separators() {
            let validator = Validator::new_flexible();
            #[cfg(unix)]
            let path = "config/notes/file.md";
            #[cfg(windows)]
            let path = "config\\notes\\file.md";
            validator.validate(path).expect("should be valid");
        }
    }
}
