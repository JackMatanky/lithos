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
//! // Note: Root must be absolute.
//! let root = PathBuf::from("/path/to/vault");
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
    /// Internal helper to enforce absolute path policy based on mode.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics allow borrowing PathBuf from &Mode without \
                  explicit ref patterns. This is idiomatic Rust 2021 and \
                  avoids verbose &Mode::Strict { root: ref r } syntax."
    )]
    fn check_absolute_path_policy(
        &self,
        path: &Path,
    ) -> Result<(), PathValidationError> {
        if !path.is_absolute() {
            return Ok(());
        }

        match &self.mode {
            Mode::Strict {
                root,
            } if path.starts_with(root) => {
                // Allowed if within strict root
                Ok(())
            }
            Mode::Flexible
            | Mode::Strict {
                ..
            } => Err(PathValidationError::AbsolutePathError(
                path.display().to_string(),
            )),
        }
    }

    /// Validates a single path component for security violations.
    #[inline]
    fn check_component_security(
        component: &Component<'_>,
    ) -> Result<(), PathValidationError> {
        match *component {
            Component::ParentDir => {
                Err(PathValidationError::PathTraversalError)
            }
            Component::Normal(os_str) => {
                if Self::is_hidden_os_str(os_str) {
                    Err(PathValidationError::RestrictedPathError(
                        os_str.to_string_lossy().into_owned(),
                    ))
                } else {
                    Ok(())
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {
                Ok(())
            }
        }
    }

    /// Internal helper to verify a resolved path stays within the strict root.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics allow borrowing PathBuf from &Mode without \
                  explicit ref patterns. This is idiomatic Rust 2021 and \
                  avoids verbose &Mode::Strict { root: ref r } syntax."
    )]
    fn check_strict_boundary(
        &self,
        resolved: &Path,
    ) -> Result<(), PathValidationError> {
        if let Mode::Strict {
            root,
        } = &self.mode
        {
            if !resolved.starts_with(root) {
                return Err(PathValidationError::SymlinkEscapeError);
            }

            // Hidden check on the relative portion only
            let relative = resolved.strip_prefix(root).unwrap_or(resolved);
            Self::validate_core(relative)?;
        }
        Ok(())
    }

    /// Internal helper to extract the path portion for security validation.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics allow borrowing PathBuf from &Mode without \
                  explicit ref patterns. This is idiomatic Rust 2021 and \
                  avoids verbose &Mode::Strict { root: ref r } syntax."
    )]
    fn get_relative_validation_path<'path>(
        &self,
        path: &'path Path,
    ) -> &'path Path {
        match &self.mode {
            Mode::Strict {
                root,
            } => path.strip_prefix(root).unwrap_or(path),
            Mode::Flexible => path,
        }
    }

    /// Robust check for hidden status of an `OsStr` across platforms.
    #[inline]
    fn is_hidden_os_str(os_str: &std::ffi::OsStr) -> bool {
        os_str.to_str().is_some_and(|s| s.starts_with('.'))
    }

    /// Creates a flexible validator that allows external symlinks.
    ///
    /// # Example
    ///
    /// ```
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let validator = Validator::new_flexible();
    /// assert!(validator.validate("config.toml").is_ok());
    /// ```
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
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let root = PathBuf::from("/path/to/vault");
    /// let validator = Validator::new_strict(root);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `root` is not an absolute path. The root should be
    /// canonicalized and absolute (provided by Figment/Config) before
    /// passing it here.
    #[inline]
    #[must_use]
    pub fn new_strict(root: PathBuf) -> Self {
        assert!(
            root.is_absolute(),
            "Validator root must be absolute: {}",
            root.display()
        );
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
    /// let root = std::env::current_dir().unwrap().join("/path/to/vault");
    /// let validator = Validator::new_strict(root);
    ///
    /// let symlink_path = PathBuf::from("link_to_file");
    /// let resolved = validator.resolve_safe_symlink(&symlink_path).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub async fn resolve_safe_symlink<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<PathBuf, PathValidationError> {
        let path_ref = path.as_ref();

        // 1. Validate the input path itself first
        let _validated: Cow<'_, Path> = self.validate(path_ref)?;

        // 2. Resolve symlinks asynchronously
        let resolved = tokio::fs::canonicalize(path_ref)
            .await
            .map_err(|e| PathValidationError::IoError(e.to_string()))?;

        // 3. Enforce boundary constraints based on mode
        self.check_strict_boundary(&resolved)?;

        Ok(resolved)
    }

    /// Validates a path for security issues.
    ///
    /// # Example
    ///
    /// ```
    /// use lithos_adapters::spi::fs::validator::Validator;
    ///
    /// let validator = Validator::new_flexible();
    /// assert!(validator.validate("safe/path.txt").is_ok());
    /// assert!(validator.validate("../../unsafe").is_err());
    /// ```
    ///
    /// # Checks Performed
    ///
    /// 1. **Traversal Check**: Rejects paths with `..` components
    /// 2. **Absolute Path Check**: Rejects absolute paths (unless within strict
    ///    root)
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

        // 0. UTF-8 Encoding Validation
        if path_ref.to_str().is_none() {
            return Err(PathValidationError::InvalidPathEncoding(
                path_ref.to_string_lossy().into_owned(),
            ));
        }

        // 1. Absolute Path Validation
        self.check_absolute_path_policy(path_ref)?;

        // 2. Core Security Validation (Traversal + Hidden)
        let check_path = self.get_relative_validation_path(path_ref);
        Self::validate_core(check_path)?;

        Ok(Cow::Borrowed(path_ref))
    }

    /// Internal core validation logic. Performs traversal and hidden checks in
    /// a single pass.
    #[inline]
    fn validate_core(path: &Path) -> Result<(), PathValidationError> {
        for component in path.components() {
            Self::check_component_security(&component)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use tempfile::TempDir;

        use super::*;

        pub const DEFAULT_CONTENT: &str = "test content";
        pub const OUTSIDE_NAME: &str = "outside.txt";
        pub const DOTFILE_NAME: &str = "dotfile.toml";

        pub struct Workspace {
            #[expect(
                dead_code,
                reason = "TempDir never read but must be stored for RAII \
                          lifecycle. Drop order ensures filesystem cleanup \
                          after test completion."
            )]
            pub temp_dir: TempDir,
            pub root: PathBuf,
        }

        impl Workspace {
            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture setup uses blocking std::fs for \
                          simplicity. These operations are synchronous and \
                          don't impact async test behavior."
            )]
            pub fn create_file<P: AsRef<Path>>(
                &self,
                path: P,
                content: Option<&str>,
            ) -> PathBuf {
                let full_path = self.root.join(path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent).expect(
                        "Test setup: Failed to create parent directories",
                    );
                }
                std::fs::write(&full_path, content.unwrap_or(DEFAULT_CONTENT))
                    .expect("Test setup: Failed to write file");
                full_path
            }

            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture setup uses blocking std::fs for \
                          simplicity. These operations are synchronous and \
                          don't impact async test behavior."
            )]
            pub fn create_symlink<P: AsRef<Path>, T: AsRef<Path>>(
                &self,
                link_path: P,
                target: T,
            ) -> PathBuf {
                let full_link_path = self.root.join(link_path);
                if let Some(parent) = full_link_path.parent() {
                    std::fs::create_dir_all(parent).expect(
                        "Test setup: Failed to create parent directories",
                    );
                }

                #[cfg(unix)]
                std::os::unix::fs::symlink(target, &full_link_path)
                    .expect("Test setup: Failed to create symlink");
                #[cfg(windows)]
                std::os::windows::fs::symlink_file(target, &full_link_path)
                    .expect("Test setup: Failed to create symlink");

                full_link_path
            }

            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture setup uses blocking std::fs for \
                          simplicity. These operations are synchronous and \
                          don't impact async test behavior."
            )]
            pub fn new() -> Self {
                let temp_dir =
                    TempDir::new().expect("failed to create temp dir");
                let root = std::fs::canonicalize(temp_dir.path())
                    .expect("failed to canonicalize temp dir");
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
            // GIVEN the need for a flexible validator that allows external
            // symlinks
            let validator = Validator::new_flexible();

            // WHEN checking the validator mode
            // THEN it should be configured with Flexible mode
            assert!(
                matches!(validator.mode, Mode::Flexible),
                "Expected Flexible mode, found {:?}",
                validator.mode
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            clippy::pattern_type_mismatch,
            reason = "Test setup requires std::env::current_dir() (disallowed \
                      in production; use Figment config instead). Pattern \
                      match on &Mode enum requires borrowing non-Copy PathBuf \
                      field without explicit `ref` pattern (idiomatic Rust \
                      2021)."
        )]
        fn creates_strict_validator_with_root() {
            // GIVEN an absolute root path
            let root = std::env::current_dir().expect("cwd").join("test_root");

            // WHEN creating a strict validator with the root
            let validator = Validator::new_strict(root);

            // THEN it should be configured with Strict mode and an absolute
            // root
            match &validator.mode {
                Mode::Strict {
                    root: validator_root,
                } => assert!(
                    validator_root.is_absolute(),
                    "Expected absolute root, found {}",
                    validator_root.display()
                ),
                #[expect(
                    clippy::panic,
                    reason = "panic!() used to fail test immediately if \
                              constructor created wrong mode. This is \
                              intentional test-only behavior for explicit \
                              invariant validation."
                )]
                Mode::Flexible => {
                    panic!("Test fixture guaranteed Strict mode");
                }
            }
        }
    }

    mod validate {
        use super::*;

        mod traversal {
            use rstest::rstest;

            use super::*;

            #[rstest]
            #[case::double_dot("../../etc/passwd")]
            #[case::single_parent("../config.toml")]
            #[case::mid_path("valid/../../etc/passwd")]
            fn rejects_traversal_attacks(#[case] input: &str) {
                let validator = Validator::new_flexible();
                let result = validator.validate(input);
                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::PathTraversalError)
                    ),
                    "Expected PathTraversalError for '{input}', found \
                     {result:?}"
                );
            }

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test assertions use Result::expect() for clear \
                          failure messages. See clippy.toml \
                          allow-expect-in-tests."
            )]
            fn accepts_encoded_path_as_literal() {
                let validator = Validator::new_flexible();
                let result = validator.validate("safe%2Ffile");
                result.expect("safe encoded characters should pass");
            }
        }

        mod absolute {
            use rstest::rstest;

            use super::*;

            #[rstest]
            #[case::unix("/etc/hosts")]
            #[cfg_attr(
                target_os = "windows",
                case::windows("C:\\Windows\\System32")
            )]
            fn rejects_absolute_paths(#[case] input: &str) {
                let validator = Validator::new_flexible();
                let result = validator.validate(input);
                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::AbsolutePathError(_))
                    ),
                    "Expected AbsolutePathError for '{input}', found \
                     {result:?}"
                );
            }

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test assertions use Result::expect() for clear \
                          failure messages. See clippy.toml \
                          allow-expect-in-tests."
            )]
            fn accepts_relative_paths() {
                let validator = Validator::new_flexible();
                let result = validator.validate("config/lithos.toml");
                result.expect("relative path should be valid");
            }
        }

        mod restricted {
            use rstest::rstest;

            use super::*;

            #[rstest]
            #[case::git(".git/config")]
            #[case::env(".env")]
            #[case::nested_env("config/.env")]
            #[case::ssh(".ssh/id_rsa")]
            fn rejects_hidden_files(#[case] input: &str) {
                let validator = Validator::new_flexible();
                let result = validator.validate(input);
                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::RestrictedPathError(_))
                    ),
                    "Expected RestrictedPathError for '{input}', found \
                     {result:?}"
                );
            }

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test assertions use Result::expect() for clear \
                          failure messages. See clippy.toml \
                          allow-expect-in-tests."
            )]
            fn accepts_normal_files() {
                let validator = Validator::new_flexible();
                let result = validator.validate("notes/daily.md");
                result.expect("normal file should be valid");
            }
        }
    }

    mod resolve_safe_symlink {
        mod strict {
            use super::super::{
                PathValidationError, Validator,
                fixtures::{self, Workspace},
            };

            #[tokio::test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture uses blocking std::fs for setup and \
                          expect() for assertions. See clippy.toml \
                          allow-expect-in-tests."
            )]
            async fn rejects_escaped_symlinks() {
                let ws = Workspace::new();
                let outside_target =
                    std::env::temp_dir().join(fixtures::OUTSIDE_NAME);
                std::fs::write(&outside_target, "outside")
                    .expect("Test setup: Failed to create outside target");

                let symlink_path =
                    ws.create_symlink("escaped_link", &outside_target);

                let validator = Validator::new_strict(ws.root.clone());
                let result =
                    validator.resolve_safe_symlink(&symlink_path).await;

                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::SymlinkEscapeError)
                    ),
                    "Expected SymlinkEscapeError, found {result:?}"
                );
                drop(std::fs::remove_file(&outside_target));
            }

            #[tokio::test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture uses blocking std::fs for setup. Async \
                          test validates production tokio::fs behavior."
            )]
            async fn detects_symlink_loops() {
                let ws = Workspace::new();
                let link_a = ws.root.join("link_a");
                let link_b = ws.root.join("link_b");

                super::create_symlink_loop(&link_a, &link_b);

                let validator = Validator::new_strict(ws.root.clone());
                let result = validator.resolve_safe_symlink(&link_a).await;

                assert!(
                    matches!(result, Err(PathValidationError::IoError(_))),
                    "Expected IoError for loop, found {result:?}"
                );
            }

            #[tokio::test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture uses blocking std::fs for setup. Async \
                          test validates production tokio::fs behavior."
            )]
            async fn rejects_internal_hidden_targets() {
                let ws = Workspace::new();
                let hidden_file = ws.create_file(".secret.txt", None);
                let symlink_path =
                    ws.create_symlink("link_to_secret", &hidden_file);
                let validator = Validator::new_strict(ws.root.clone());

                let result =
                    validator.resolve_safe_symlink(&symlink_path).await;

                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::RestrictedPathError(_))
                    ),
                    "Expected RestrictedPathError, found {result:?}"
                );
            }
        }

        mod flexible {
            use super::super::{fixtures::Workspace, *};

            #[tokio::test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test fixture uses blocking std::fs for setup and \
                          std::env::current_dir for CWD manipulation. Async \
                          test validates production tokio::fs behavior."
            )]
            async fn allows_external_symlinks() {
                let ws = Workspace::new();
                let outside_target =
                    std::env::temp_dir().join(fixtures::DOTFILE_NAME);
                std::fs::write(&outside_target, "dotfile")
                    .expect("Test setup: Failed to write outside target");

                let _link = ws.create_symlink("dotfile_link", &outside_target);

                let validator = Validator::new_flexible();
                let original_cwd = std::env::current_dir().expect("cwd");
                std::env::set_current_dir(&ws.root)
                    .expect("Test setup: Failed to change directory");

                let result =
                    validator.resolve_safe_symlink("dotfile_link").await;
                std::env::set_current_dir(original_cwd)
                    .expect("Test teardown: Failed to restore directory");

                assert!(
                    result.is_ok(),
                    "flexible mode should allow external symlinks, found \
                     {result:?}"
                );
                drop(std::fs::remove_file(&outside_target));
            }

            #[tokio::test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test validates error handling, no fixture setup \
                          needed. async test for consistency with module."
            )]
            async fn enforces_input_traversal_checks() {
                let validator = Validator::new_flexible();
                let result =
                    validator.resolve_safe_symlink("../../../dotfile").await;

                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::PathTraversalError)
                    ),
                    "Expected PathTraversalError on input, found {result:?}"
                );
            }
        }

        /// Helper to create circular symlinks (extracted to reduce nesting).
        #[cfg(unix)]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test helper uses platform-specific std::os::unix::fs \
                      for symlink creation. Blocking I/O acceptable for \
                      fixture setup."
        )]
        fn create_symlink_loop(
            link_a: &std::path::Path,
            link_b: &std::path::Path,
        ) {
            std::os::unix::fs::symlink(link_b, link_a)
                .expect("Test setup: Failed to create symlink");
            std::os::unix::fs::symlink(link_a, link_b)
                .expect("Test setup: Failed to create symlink");
        }

        #[cfg(windows)]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test helper uses platform-specific std::os::windows::fs \
                      for symlink creation. Blocking I/O acceptable for \
                      fixture setup."
        )]
        fn create_symlink_loop(
            link_a: &std::path::Path,
            link_b: &std::path::Path,
        ) {
            std::os::windows::fs::symlink_file(link_b, link_a)
                .expect("Test setup: Failed to create symlink");
            std::os::windows::fs::symlink_file(link_a, link_b)
                .expect("Test setup: Failed to create symlink");
        }
    }

    mod platform_specific {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test assertions use Result::expect() for clear failure \
                      messages. See clippy.toml allow-expect-in-tests."
        )]
        fn handles_platform_separators_consistently() {
            let validator = Validator::new_flexible();
            #[cfg(unix)]
            let path = "config/notes/file.md";
            #[cfg(windows)]
            let path = "config\\notes\\file.md";

            let result = validator.validate(path);
            result.expect("platform separators should be handled correctly");
        }

        #[test]
        #[cfg(unix)]
        fn rejects_non_utf8_hidden_files() {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

            let validator = Validator::new_flexible();
            let bytes = b".\xffinvalid";
            let os_str = OsStr::from_bytes(bytes);
            let path = Path::new(os_str);
            let result = validator.validate(path);

            assert!(
                result.is_err(),
                "Expected error for non-UTF8 hidden file, found success"
            );
        }

        #[test]
        #[cfg(unix)]
        fn rejects_non_utf8_paths() {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

            let validator = Validator::new_flexible();
            let bytes = b"invalid\xffutf8";
            let os_str = OsStr::from_bytes(bytes);
            let path = Path::new(os_str);
            let result = validator.validate(path);

            assert!(
                matches!(
                    result,
                    Err(PathValidationError::InvalidPathEncoding(_))
                ),
                "Expected InvalidPathEncoding error for non-UTF8 path, found \
                 {result:?}"
            );
        }
    }
}
