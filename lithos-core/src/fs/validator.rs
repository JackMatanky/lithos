//! **Security-Critical Path Validation Utilities.**
//!
//! This module provides path validation to prevent **path traversal attacks**,
//! **arbitrary file access**, and **symlink escape vulnerabilities**.
//!
//! **SECURITY REQUIREMENT**: All file I/O operations in adapters MUST use these
//! validation utilities before accessing the filesystem. Bypassing these checks
//! creates critical security vulnerabilities.
//!
//! # Security Guarantees
//!
//! - **Path Traversal Prevention**: Rejects paths containing `..` components
//! - **Absolute Path Rejection**: Ensures only relative paths are used (unless
//!   within strict root)
//! - **Restricted File Protection**: Blocks access to hidden/sensitive files
//!   (`.git`, `.env`, `.ssh`)
//! - **Symlink Escape Detection**: Validates symlinks stay within root
//!   boundaries (strict mode)
//!
//! # Validation Modes
//!
//! - **Strict**: Enforces root boundary and rejects symlinks escaping the root
//!   - Use for: Vault files, user-controlled content
//!   - Example: `PathValidator::new_strict(vault_root)`
//!
//! - **Flexible**: Allows external symlinks (e.g., dotfiles) while still
//!   checking input traversal
//!   - Use for: Configuration files, schema files
//!   - Example: `PathValidator::new_flexible()`
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//! use lithos_core::fs::PathValidator; // Re-exported for ergonomics
//!
//! // Flexible validator for config files (allows dotfile symlinks)
//! let validator = PathValidator::new_flexible();
//! assert!(validator.validate("config/lithos.toml").is_ok());
//! assert!(validator.validate("../../etc/passwd").is_err()); // Traversal blocked
//!
//! // Strict validator for vault files (enforces root boundary)
//! // Note: Root must be absolute (provided by Figment config)
//! let root = PathBuf::from("/absolute/path/to/vault");
//! let validator = PathValidator::new_strict(root);
//! assert!(validator.validate("notes/daily.md").is_ok());
//! ```
//!
//! # Architecture Context
//!
//! This module represents the file loading strategy foundation (epic 4).
//! It provides the security foundation for all file-based adapters:
//! - `ConfigAdapter` (Flexible mode)
//! - `SchemaAdapter` (Flexible mode)
//! - `NoteAdapter` (Strict mode)

use std::{
    borrow::Cow,
    path::{Component, Path, PathBuf},
};

use tracing::{debug, warn};

use super::error::PathValidationError;

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

/// Public alias for validation mode configuration.
pub type ValidationMode = Mode;

/// Path validator with configurable security modes.
///
/// # Constraints
///
/// - **Security Focused**: Always rejects `..` components in input paths
/// - **Platform Agnostic**: Correctly handles Windows and Unix path separators
/// - **Resource Safe**: Validates paths before expensive I/O operations
#[derive(Debug, Clone)]
pub struct Validator {
    mode: Mode,
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
                warn!(
                    "Path traversal attempt blocked: contains '..' component"
                );
                Err(PathValidationError::PathTraversalError)
            }
            Component::Normal(os_str) => {
                if Self::is_hidden_os_str(os_str) {
                    warn!(
                        file = %os_str.to_string_lossy(),
                        "Restricted path access blocked: hidden/sensitive file"
                    );
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
    /// use lithos_core::fs::validator::Validator;
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
    /// use lithos_core::fs::validator::Validator;
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
    /// use lithos_core::fs::validator::Validator;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let root = std::env::current_dir()?.join("/path/to/vault");
    /// let validator = Validator::new_strict(root);
    ///
    /// let symlink_path = PathBuf::from("link_to_file");
    /// let resolved = validator.resolve_safe_symlink(&symlink_path)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn resolve_safe_symlink<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<PathBuf, PathValidationError> {
        let path_ref = path.as_ref();

        // 1. Validate the input path itself first
        let _validated: Cow<'_, Path> = self.validate(path_ref)?;

        // 2. Resolve symlinks
        #[expect(
            clippy::disallowed_methods,
            reason = "Security validation requires canonicalization"
        )]
        let resolved = std::fs::canonicalize(path_ref)
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
    /// use lithos_core::fs::validator::Validator;
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

        debug!(
            path = %path_ref.display(),
            mode = ?self.mode,
            "Path validation succeeded"
        );

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
            pub fn create_file<P: AsRef<Path>>(
                &self,
                path: P,
                content: Option<&str>,
            ) -> PathBuf {
                let full_path = self.root.join(path);
                if let Some(parent) = full_path.parent() {
                    let result = std::fs::create_dir_all(parent);
                    assert!(
                        result.is_ok(),
                        "Test setup: Failed to create parent directories: \
                         {result:?}"
                    );
                }
                let result = std::fs::write(
                    &full_path,
                    content.unwrap_or(DEFAULT_CONTENT),
                );
                assert!(
                    result.is_ok(),
                    "Test setup: Failed to write file: {result:?}"
                );
                full_path
            }

            pub fn create_symlink<P: AsRef<Path>, T: AsRef<Path>>(
                &self,
                link_path: P,
                target: T,
            ) -> PathBuf {
                let full_link_path = self.root.join(link_path);
                if let Some(parent) = full_link_path.parent() {
                    let result = std::fs::create_dir_all(parent);
                    assert!(
                        result.is_ok(),
                        "Test setup: Failed to create parent directories: \
                         {result:?}"
                    );
                }

                #[cfg(unix)]
                let result =
                    std::os::unix::fs::symlink(target, &full_link_path);
                #[cfg(unix)]
                assert!(
                    result.is_ok(),
                    "Test setup: Failed to create symlink: {result:?}"
                );
                #[cfg(windows)]
                {
                    let result = std::os::windows::fs::symlink_file(
                        target,
                        &full_link_path,
                    );
                    assert!(
                        result.is_ok(),
                        "Test setup: Failed to create symlink: {result:?}"
                    );
                }

                full_link_path
            }

            pub fn new() -> Result<Self, std::io::Error> {
                let temp_dir = TempDir::new()?;
                let root = temp_dir.path().canonicalize()?;
                Ok(Self {
                    temp_dir,
                    root,
                })
            }
        }
    }

    mod constructor {
        use tempfile::TempDir;

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
            reason = "Test uses expect for deterministic fixture setup. \
                      Failure indicates invalid test data. Expect is \
                      idiomatic in setup."
        )]
        fn creates_strict_validator_with_root() {
            // GIVEN an absolute root path
            let temp_dir = TempDir::new().expect("TempDir should be created");
            let root = temp_dir.path().join("test_root");

            // WHEN creating a strict validator with the root
            let validator = Validator::new_strict(root);

            // THEN it should be configured with Strict mode and an absolute
            // root
            assert!(
                matches!(
                    &validator.mode,
                    Mode::Strict { root: validator_root }
                        if validator_root.is_absolute()
                ),
                "Expected Strict mode with absolute root, found {:?}",
                validator.mode
            );
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
            fn accepts_encoded_path_as_literal() {
                let validator = Validator::new_flexible();
                let result = validator.validate("safe%2Ffile");
                assert!(
                    result.is_ok(),
                    "safe encoded characters should pass, got: {result:?}"
                );
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
            fn accepts_relative_paths() {
                let validator = Validator::new_flexible();
                let result = validator.validate("config/lithos.toml");
                assert!(
                    result.is_ok(),
                    "relative path should be valid, got: {result:?}"
                );
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
            fn accepts_normal_files() {
                let validator = Validator::new_flexible();
                let result = validator.validate("notes/daily.md");
                assert!(
                    result.is_ok(),
                    "normal file should be valid, got: {result:?}"
                );
            }
        }
    }

    mod resolve_safe_symlink {
        mod strict {
            use std::io::Write as _;

            use super::{
                super::{PathValidationError, Validator, fixtures::Workspace},
                NamedTempFile,
            };

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test uses expect for deterministic fixture setup \
                          and value extraction."
            )]
            fn rejects_escaped_symlinks() {
                let ws = Workspace::new().expect("Workspace should be created");
                let mut outside_file = NamedTempFile::new()
                    .expect("Outside file should be created");
                outside_file
                    .write_all(b"outside")
                    .expect("Test setup: Failed to write outside target");
                let outside_target = outside_file.path();

                let symlink_path =
                    ws.create_symlink("escaped_link", outside_target);

                let validator = Validator::new_strict(ws.root.clone());
                let resolve_result =
                    validator.resolve_safe_symlink(&symlink_path);

                assert!(
                    matches!(
                        resolve_result,
                        Err(PathValidationError::SymlinkEscapeError)
                    ),
                    "Expected SymlinkEscapeError, found {resolve_result:?}"
                );
            }

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test uses expect for deterministic fixture setup \
                          and value extraction."
            )]
            fn detects_symlink_loops() {
                let ws = Workspace::new().expect("Workspace should be created");
                let link_a = ws.root.join("link_a");
                let link_b = ws.root.join("link_b");

                super::create_symlink_loop(&link_a, &link_b);

                let validator = Validator::new_strict(ws.root.clone());
                let result = validator.resolve_safe_symlink(&link_a);

                assert!(
                    matches!(result, Err(PathValidationError::IoError(_))),
                    "Expected IoError for loop, found {result:?}"
                );
            }

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test uses expect for deterministic fixture setup \
                          and value extraction."
            )]
            fn rejects_internal_hidden_targets() {
                let ws = Workspace::new().expect("Workspace should be created");
                let hidden_file = ws.create_file(".secret.txt", None);
                let symlink_path =
                    ws.create_symlink("link_to_secret", &hidden_file);
                let validator = Validator::new_strict(ws.root.clone());

                let result = validator.resolve_safe_symlink(&symlink_path);

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
            use std::io::Write as _;

            use super::{
                super::{fixtures::Workspace, *},
                CwdGuard, NamedTempFile,
            };

            #[test]
            #[expect(
                clippy::disallowed_methods,
                reason = "Test uses expect for deterministic fixture setup \
                          and value extraction."
            )]
            fn allows_external_symlinks() {
                let ws = Workspace::new().expect("Workspace should be created");
                let mut outside_file = NamedTempFile::new()
                    .expect("Outside file should be created");
                outside_file
                    .write_all(b"dotfile")
                    .expect("Test setup: Failed to write outside target");
                let outside_target = outside_file.path();

                let _link_path =
                    ws.create_symlink("dotfile_link", outside_target);

                let validator = Validator::new_flexible();

                let _cwd_guard = CwdGuard::new(&ws.root)
                    .expect("Test setup: Failed to change directory");

                let resolve_result =
                    validator.resolve_safe_symlink("dotfile_link");

                assert!(
                    resolve_result.is_ok(),
                    "flexible mode should allow external symlinks, found \
                     {resolve_result:?}"
                );
            }

            #[test]
            fn enforces_input_traversal_checks() {
                let validator = Validator::new_flexible();
                let result = validator.resolve_safe_symlink("../../../dotfile");

                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::PathTraversalError)
                    ),
                    "Expected PathTraversalError on input, found {result:?}"
                );
            }
        }

        use std::sync::{Mutex, MutexGuard, OnceLock};

        use tempfile::NamedTempFile;

        fn cwd_lock() -> &'static Mutex<()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(()))
        }

        struct CwdGuard {
            _guard: MutexGuard<'static, ()>,
            previous: std::path::PathBuf,
        }

        impl CwdGuard {
            fn new(path: &std::path::Path) -> Result<Self, std::io::Error> {
                let guard = cwd_lock().lock().map_err(|err| {
                    std::io::Error::other(format!("CWD lock poisoned: {err}"))
                })?;
                #[expect(
                    clippy::disallowed_methods,
                    reason = "Test-only CWD guard needs current working \
                              directory."
                )]
                let previous = std::env::current_dir()?;
                std::env::set_current_dir(path)?;
                Ok(Self {
                    _guard: guard,
                    previous,
                })
            }
        }

        impl Drop for CwdGuard {
            fn drop(&mut self) {
                if std::env::set_current_dir(&self.previous).is_err() {}
            }
        }

        /// Helper to create circular symlinks (extracted to reduce nesting).
        #[cfg(unix)]
        fn create_symlink_loop(
            link_a: &std::path::Path,
            link_b: &std::path::Path,
        ) {
            let symlink_result = std::os::unix::fs::symlink(link_b, link_a);
            assert!(
                symlink_result.is_ok(),
                "Test setup: Failed to create symlink: {symlink_result:?}"
            );
            let symlink_result_second =
                std::os::unix::fs::symlink(link_a, link_b);
            assert!(
                symlink_result_second.is_ok(),
                "Test setup: Failed to create symlink: \
                 {symlink_result_second:?}"
            );
        }

        #[cfg(windows)]
        fn create_symlink_loop(
            link_a: &std::path::Path,
            link_b: &std::path::Path,
        ) {
            let symlink_result =
                std::os::windows::fs::symlink_file(link_b, link_a);
            assert!(
                symlink_result.is_ok(),
                "Test setup: Failed to create symlink: {symlink_result:?}"
            );
            let symlink_result =
                std::os::windows::fs::symlink_file(link_a, link_b);
            assert!(
                symlink_result.is_ok(),
                "Test setup: Failed to create symlink: {symlink_result:?}"
            );
        }
    }

    mod platform_specific {
        use super::*;

        #[test]
        fn handles_platform_separators_consistently() {
            let validator = Validator::new_flexible();
            #[cfg(unix)]
            let path = "config/notes/file.md";
            #[cfg(windows)]
            let path = "config\\notes\\file.md";

            let result = validator.validate(path);
            assert!(
                result.is_ok(),
                "platform separators should be handled correctly, got: \
                 {result:?}"
            );
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
