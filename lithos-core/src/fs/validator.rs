//! **Security-critical path validation utilities.**
//!
//! This module provides path validation to prevent **path traversal attacks**,
//! **arbitrary file access**, and **symlink escape vulnerabilities**. All
//! file I/O operations in adapters must pass paths through a [`Validator`]
//! before touching the filesystem; bypassing these checks creates critical
//! security vulnerabilities.
//!
//! # Security guarantees
//!
//! - **Path traversal prevention** — rejects paths containing `..` components.
//! - **Absolute path rejection** — ensures only relative paths are used (unless
//!   within a strict root).
//! - **Restricted file protection** — blocks access to hidden or sensitive
//!   files (components starting with `.`, e.g. `.git`, `.env`, `.ssh`).
//! - **Symlink escape detection** — validates symlinks stay within root
//!   boundaries in strict mode.
//!
//! # Validation modes
//!
//! - **Flexible** — allows external symlinks (e.g. dotfiles) while still
//!   checking input traversal. Use for configuration files and schema files.
//! - **Strict** — enforces a root boundary and rejects symlinks that escape it.
//!   Use for vault files and user-controlled content.
//!
//! # Examples
//!
//! ```
//! use std::path::PathBuf;
//!
//! use lithos_core::fs::PathValidator;
//!
//! // Flexible validator for config files (allows dotfile symlinks).
//! let validator = PathValidator::new_flexible();
//! assert!(validator.validate("config/lithos.toml").is_ok());
//! assert!(validator.validate("../../etc/passwd").is_err());
//!
//! // Strict validator for vault files (enforces root boundary).
//! // Note: root must be absolute and canonicalized.
//! let root = PathBuf::from("/absolute/path/to/vault");
//! let validator = PathValidator::try_new_strict(root)?;
//! assert!(validator.validate("notes/daily.md").is_ok());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Architecture context
//!
//! This module is the security foundation for all file-based adapters:
//!
//! - `ConfigAdapter` — flexible mode
//! - `SchemaAdapter` — flexible mode
//! - `NoteAdapter` — strict mode

use std::path::{Component, Path, PathBuf};

use super::error::PathValidationError;

/// Internal validation mode representation.
#[derive(Debug, Clone)]
pub(crate) enum Mode {
    /// Flexible mode: allows external symlinks (e.g. dotfiles) while still
    /// checking input traversal and hidden-file access.
    Flexible,
    /// Strict mode: enforces root boundary and rejects symlinks that escape it.
    Strict {
        /// The canonicalized root directory.
        root: PathBuf,
    },
}

/// Path validator with configurable security modes.
///
/// Construct with [`Validator::new_flexible`] or [`Validator::try_new_strict`]
/// and then call [`Validator::validate`] on each path before passing it to
/// filesystem operations. For symlink-following operations use
/// [`Validator::resolve_safe_symlink`].
///
/// # Security properties
///
/// - Always rejects `..` components in input paths.
/// - Always rejects hidden path components (those starting with `.`).
/// - Always rejects non-UTF-8 paths.
/// - Strict mode additionally rejects symlinks whose targets escape the root.
#[derive(Debug, Clone)]
pub struct Validator {
    mode: Mode,
}

impl Validator {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Creates a flexible validator that allows external symlinks.
    ///
    /// Flexible mode is appropriate for configuration and schema files that
    /// may legitimately be symlinked from a dotfile repository.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::validator::Validator;
    ///
    /// let validator = Validator::new_flexible();
    /// assert!(validator.validate("config.toml").is_ok());
    /// assert!(validator.validate("../../escape").is_err());
    /// ```
    #[inline]
    #[must_use]
    pub fn new_flexible() -> Self {
        Self {
            mode: Mode::Flexible,
        }
    }

    /// Creates a strict validator with root boundary enforcement.
    ///
    /// Strict mode is appropriate for vault files and user-controlled content
    /// where symlinks must not escape the root directory.
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError::RelativeRoot`] if `root` is not an
    /// absolute path. Callers must canonicalize the root with
    /// [`std::fs::canonicalize`] before calling this constructor; a
    /// non-canonicalized root containing symlinks may cause
    /// [`resolve_safe_symlink`] to reject valid paths.
    ///
    /// [`resolve_safe_symlink`]: Validator::resolve_safe_symlink
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::fs::validator::Validator;
    ///
    /// let root = PathBuf::from("/path/to/vault");
    /// let validator = Validator::try_new_strict(root)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn try_new_strict(root: PathBuf) -> Result<Self, PathValidationError> {
        if !root.is_absolute() {
            return Err(PathValidationError::RelativeRoot(root));
        }
        Ok(Self {
            mode: Mode::Strict {
                root,
            },
        })
    }

    // -----------------------------------------------------------------------
    // Public validation API
    // -----------------------------------------------------------------------

    /// Validates a vault-relative path string against common security
    /// constraints.
    ///
    /// This is the preferred entry point for validating paths that arrive as
    /// strings (e.g. from user input or configuration). It checks, in order:
    ///
    /// 1. The path must not be empty.
    /// 2. The path must not be a Windows drive-absolute or drive-relative path
    ///    (e.g. `C:\foo` or `C:relative`). This check runs before the standard
    ///    validator because on Unix `Path::is_absolute("C:\\foo")` returns
    ///    `false`, so the standard check alone is insufficient on
    ///    cross-platform code.
    /// 3. Standard validation via [`Validator::new_flexible`]: no traversal
    ///    (`..`), no hidden components, no absolute paths, valid UTF-8.
    /// 4. If `require_extension` is `Some`, the path must end with that
    ///    extension (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns [`PathValidationError`] if any constraint is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::PathValidator;
    ///
    /// assert!(
    ///     PathValidator::validate_vault_path("notes/daily.md", Some("md"))
    ///         .is_ok()
    /// );
    /// assert!(PathValidator::validate_vault_path("", None).is_err());
    /// assert!(
    ///     PathValidator::validate_vault_path("../../etc/passwd", None).is_err()
    /// );
    /// assert!(
    ///     PathValidator::validate_vault_path("note.txt", Some("md")).is_err()
    /// );
    /// ```
    #[inline]
    pub fn validate_vault_path(
        path: &str,
        require_extension: Option<&str>,
    ) -> Result<(), PathValidationError> {
        // 1. Empty check.
        if path.is_empty() {
            return Err(PathValidationError::EmptyPath);
        }

        // 2. Windows drive-absolute / drive-relative check. Must run before the
        //    standard validator because on Unix, Path::is_absolute("C:\\foo")
        //    returns false, so check_absolute_path_policy would pass it through
        //    unchanged.
        if Self::is_windows_absolute(path) {
            return Err(PathValidationError::AbsolutePathError(PathBuf::from(
                path,
            )));
        }

        // 3. Standard validation (traversal, hidden files, absolute paths,
        //    UTF-8 encoding).
        Self::new_flexible().validate(path)?;

        // 4. Extension check.
        if let Some(required_ext) = require_extension {
            let has_ext = Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(required_ext));
            if !has_ext {
                return Err(PathValidationError::InvalidExtension {
                    path: PathBuf::from(path),
                    required: required_ext.into(),
                });
            }
        }

        Ok(())
    }

    /// Validates a path for security issues.
    ///
    /// Performs the following checks in order:
    ///
    /// 1. **UTF-8 encoding** — the path must be representable as a UTF-8 string
    ///    (required for cross-platform consistency).
    /// 2. **Absolute path** — rejected unless in strict mode and the path
    ///    starts with the configured root.
    /// 3. **Traversal and hidden files** — each component is checked for `..`
    ///    (traversal) and a leading `.` (hidden/restricted file).
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::InvalidPathEncoding`] — path is not valid
    ///   UTF-8.
    /// - [`PathValidationError::AbsolutePathError`] — path is absolute and not
    ///   permitted by the current mode.
    /// - [`PathValidationError::PathTraversalError`] — path contains `..`.
    /// - [`PathValidationError::RestrictedPathError`] — path accesses a hidden
    ///   file or directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use lithos_core::fs::validator::Validator;
    ///
    /// let v = Validator::new_flexible();
    /// assert!(v.validate("safe/path.txt").is_ok());
    /// assert!(v.validate("../../unsafe").is_err());
    /// assert!(v.validate(".hidden/file").is_err());
    /// ```
    #[inline]
    pub fn validate<PathType>(
        &self,
        path: &PathType,
    ) -> Result<(), PathValidationError>
    where
        PathType: AsRef<Path> + ?Sized,
    {
        let path_ref = path.as_ref();

        // 1. UTF-8 encoding check.
        if path_ref.to_str().is_none() {
            return Err(PathValidationError::InvalidPathEncoding(
                path_ref.to_path_buf(),
            ));
        }

        // 2. Absolute path check.
        self.check_absolute_path_policy(path_ref)?;

        // 3. Traversal and hidden-file check on the relevant path portion. In
        //    strict mode, strip the known-good root prefix before checking so
        //    that root components (which are absolute) are not flagged.
        let check_path = match self.mode.clone() {
            Mode::Strict {
                root,
            } => path_ref.strip_prefix(&root).unwrap_or(path_ref),
            Mode::Flexible => path_ref,
        };
        Self::validate_core(check_path)?;

        Ok(())
    }

    /// Safely resolves a symlink target, validating the resolved path against
    /// the current mode's security policy.
    ///
    /// # Behaviour by mode
    ///
    /// - **Flexible** — follows the symlink without boundary checks, permitting
    ///   external targets (e.g. dotfile symlinks). Input traversal checks still
    ///   apply.
    /// - **Strict** — resolves the full canonical target and rejects it if it
    ///   escapes the configured root or resolves to a hidden path within the
    ///   vault.
    ///
    /// # Errors
    ///
    /// - [`PathValidationError::PathTraversalError`] — input path contains
    ///   `..`.
    /// - [`PathValidationError::IoError`] — OS error during canonicalization
    ///   (e.g. a symlink loop or permission denial).
    /// - [`PathValidationError::SymlinkEscapeError`] — resolved target is
    ///   outside the strict root.
    /// - [`PathValidationError::RestrictedPathError`] — resolved target is a
    ///   hidden path within the strict root.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use lithos_core::fs::validator::Validator;
    ///
    /// let root = PathBuf::from("/path/to/vault");
    /// let validator = Validator::try_new_strict(root)?;
    ///
    /// let resolved = validator.resolve_safe_symlink("link_to_file")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn resolve_safe_symlink<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<PathBuf, PathValidationError> {
        let path_ref = path.as_ref();

        // Validate the input path before touching the filesystem.
        self.validate(path_ref)?;

        // Resolve all symlink components to a canonical absolute path.
        // canonicalize() is required here: only the fully-resolved path can
        // be compared against the root boundary.
        #[expect(
            clippy::disallowed_methods,
            reason = "Security validation requires canonicalization to \
                      resolve all symlink components for boundary checking. \
                      There is no alternative API that provides the same \
                      guarantee."
        )]
        let resolved = std::fs::canonicalize(path_ref)
            .map_err(PathValidationError::IoError)?;

        // Enforce boundary (symlink escape) and vault hidden-file policy on
        // the resolved target. These are distinct checks: boundary prevents
        // escape, hidden-file prevents access to restricted vault contents.
        self.check_strict_boundary_and_hidden(&resolved)?;

        Ok(resolved)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Returns `true` if `path` looks like a Windows drive-absolute or
    /// drive-relative path (e.g. `C:\foo`, `C:/foo`, or `C:relative`).
    ///
    /// On Unix, `Path::is_absolute("C:\\foo")` returns `false`, so this check
    /// is necessary to catch Windows-style absolute paths on all platforms.
    #[inline]
    #[must_use]
    fn is_windows_absolute(path: &str) -> bool {
        let b = path.as_bytes();
        b.first().is_some_and(u8::is_ascii_alphabetic)
            && b.get(1) == Some(&b':')
    }

    /// Enforces the absolute-path policy for the current mode.
    ///
    /// - **Flexible** — all absolute paths are rejected.
    /// - **Strict** — absolute paths that start with the root are allowed; all
    ///   other absolute paths are rejected.
    #[inline]
    fn check_absolute_path_policy(
        &self,
        path: &Path,
    ) -> Result<(), PathValidationError> {
        if !path.is_absolute() {
            return Ok(());
        }

        // Strict mode allows absolute paths that are within the configured
        // root; all other absolute paths (flexible mode, or strict mode but
        // outside the root) are rejected.
        let within_strict_root = matches!(
            &self.mode,
            Mode::Strict { root } if path.starts_with(root)
        );
        if within_strict_root {
            Ok(())
        } else {
            Err(PathValidationError::AbsolutePathError(path.to_path_buf()))
        }
    }

    /// Checks a single path component for traversal (`..`) and hidden-file
    /// (leading `.`) violations.
    #[inline]
    fn check_component_security(
        component: &Component<'_>,
    ) -> Result<(), PathValidationError> {
        match *component {
            Component::ParentDir => {
                Err(PathValidationError::PathTraversalError)
            }
            Component::Normal(os_str) => {
                if os_str.to_str().is_some_and(|s| s.starts_with('.')) {
                    Err(PathValidationError::RestrictedPathError(
                        PathBuf::from(os_str),
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

    /// Verifies that a canonicalized symlink target stays within the strict
    /// root and does not resolve to a hidden path within the vault.
    ///
    /// This function enforces two distinct security properties:
    /// 1. **Boundary check** — the resolved path must start with the root.
    /// 2. **Hidden-file check** — the relative portion of the resolved path
    ///    must not contain hidden components.
    ///
    /// Only operates in strict mode; returns `Ok(())` in flexible mode.
    #[inline]
    fn check_strict_boundary_and_hidden(
        &self,
        resolved: &Path,
    ) -> Result<(), PathValidationError> {
        match self.mode.clone() {
            Mode::Strict {
                root,
            } => {
                if !resolved.starts_with(&root) {
                    return Err(PathValidationError::SymlinkEscapeError);
                }

                // Run traversal and hidden checks on the relative portion only,
                // so the known-good root components are not flagged.
                let relative = resolved.strip_prefix(&root).unwrap_or(resolved);
                Self::validate_core(relative)?;
            }
            Mode::Flexible => {}
        }
        Ok(())
    }

    /// Core validation: iterates components and rejects traversal or hidden
    /// file violations. Used by both [`validate`] and
    /// [`check_strict_boundary_and_hidden`].
    ///
    /// [`validate`]: Validator::validate
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
            // Prefixed with `_` to suppress dead_code lint. TempDir must be
            // stored here so the directory stays alive for the test's
            // duration; Drop cleans it up afterward.
            pub _temp_dir: TempDir,
            pub root: PathBuf,
        }

        impl Workspace {
            pub fn new() -> Result<Self, std::io::Error> {
                let temp_dir = TempDir::new()?;
                let root = temp_dir.path().canonicalize()?;
                Ok(Self {
                    _temp_dir: temp_dir,
                    root,
                })
            }

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
                        "test setup: failed to create parent directories: \
                         {result:?}"
                    );
                }
                let result = std::fs::write(
                    &full_path,
                    content.unwrap_or(DEFAULT_CONTENT),
                );
                assert!(
                    result.is_ok(),
                    "test setup: failed to write file: {result:?}"
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
                        "test setup: failed to create parent directories: \
                         {result:?}"
                    );
                }

                #[cfg(unix)]
                {
                    let result =
                        std::os::unix::fs::symlink(target, &full_link_path);
                    assert!(
                        result.is_ok(),
                        "test setup: failed to create symlink: {result:?}"
                    );
                };

                #[cfg(windows)]
                {
                    let result = std::os::windows::fs::symlink_file(
                        target,
                        &full_link_path,
                    );
                    assert!(
                        result.is_ok(),
                        "test setup: failed to create symlink: {result:?}"
                    );
                };

                full_link_path
            }
        }
    }

    mod constructor {
        use tempfile::TempDir;

        use super::*;

        #[test]
        fn creates_flexible_validator() {
            let validator = Validator::new_flexible();
            assert!(
                matches!(validator.mode, Mode::Flexible),
                "expected Flexible mode, found {:?}",
                validator.mode
            );
        }

        #[test]
        fn creates_strict_validator_with_absolute_root() {
            let temp_dir = TempDir::new().expect("tempdir");
            let root_path = temp_dir.path().to_path_buf();
            let validator =
                Validator::try_new_strict(root_path).expect("strict validator");
            assert!(
                matches!(&validator.mode, Mode::Strict { root } if root.is_absolute()),
                "expected Strict mode with absolute root, found {:?}",
                validator.mode
            );
        }

        #[test]
        fn try_new_strict_rejects_relative_root() {
            let result =
                Validator::try_new_strict(PathBuf::from("relative/path"));
            assert!(
                matches!(result, Err(PathValidationError::RelativeRoot(_))),
                "expected RelativeRoot error, got {result:?}"
            );
        }
    }

    mod validate_vault_path {
        use super::*;

        #[test]
        fn rejects_empty_path() {
            let result = Validator::validate_vault_path("", None);
            assert!(
                matches!(result, Err(PathValidationError::EmptyPath)),
                "expected EmptyPath, got {result:?}"
            );
        }

        #[test]
        fn rejects_windows_absolute_path_before_validator() {
            // C:\foo is not absolute on Unix (Path::is_absolute returns false),
            // so the Windows check must run first.
            let result = Validator::validate_vault_path("C:\\Users\\foo", None);
            assert!(
                matches!(
                    result,
                    Err(PathValidationError::AbsolutePathError(_))
                ),
                "expected AbsolutePathError for Windows drive path, got \
                 {result:?}"
            );
        }

        #[test]
        fn rejects_windows_drive_relative_path() {
            let result = Validator::validate_vault_path("C:relative", None);
            assert!(
                matches!(
                    result,
                    Err(PathValidationError::AbsolutePathError(_))
                ),
                "expected AbsolutePathError for Windows drive-relative path, \
                 got {result:?}"
            );
        }

        #[test]
        fn rejects_traversal() {
            let result =
                Validator::validate_vault_path("../../etc/passwd", None);
            assert!(
                matches!(result, Err(PathValidationError::PathTraversalError)),
                "expected PathTraversalError, got {result:?}"
            );
        }

        #[test]
        fn rejects_wrong_extension() {
            let result = Validator::validate_vault_path("note.txt", Some("md"));
            assert!(
                matches!(
                    result,
                    Err(PathValidationError::InvalidExtension { .. })
                ),
                "expected InvalidExtension, got {result:?}"
            );
        }

        #[test]
        fn accepts_valid_path_with_extension() {
            Validator::validate_vault_path("notes/daily.md", Some("md"))
                .expect("valid vault path with extension should be accepted");
        }

        #[test]
        fn accepts_valid_path_without_extension_requirement() {
            Validator::validate_vault_path("schemas/note.json", None).expect(
                "valid vault path without extension check should be accepted",
            );
        }
    }

    mod validate {
        mod traversal {
            use rstest::rstest;

            use super::super::*;

            #[rstest]
            #[case::double_dot("../../etc/passwd")]
            #[case::single_parent("../config.toml")]
            #[case::mid_path("valid/../../etc/passwd")]
            fn rejects_traversal_attacks(#[case] input: &str) {
                let v = Validator::new_flexible();
                assert!(
                    matches!(
                        v.validate(input),
                        Err(PathValidationError::PathTraversalError)
                    ),
                    "expected PathTraversalError for '{input}'"
                );
            }

            #[test]
            fn accepts_encoded_path_as_literal() {
                let v = Validator::new_flexible();
                v.validate("safe%2Ffile").expect(
                    "percent-encoded path should be treated as a literal",
                );
            }
        }

        mod absolute {
            use rstest::rstest;

            use super::super::*;

            #[rstest]
            #[case::unix("/etc/hosts")]
            #[cfg_attr(windows, case::windows("C:\\Windows\\System32"))]
            fn rejects_absolute_paths(#[case] input: &str) {
                let v = Validator::new_flexible();
                assert!(
                    matches!(
                        v.validate(input),
                        Err(PathValidationError::AbsolutePathError(_))
                    ),
                    "expected AbsolutePathError for '{input}'"
                );
            }

            #[test]
            fn accepts_relative_paths() {
                let v = Validator::new_flexible();
                v.validate("config/lithos.toml")
                    .expect("relative path should be valid");
            }
        }

        mod restricted {
            use rstest::rstest;

            use super::super::*;

            #[rstest]
            #[case::git(".git/config")]
            #[case::env(".env")]
            #[case::nested_env("config/.env")]
            #[case::ssh(".ssh/id_rsa")]
            fn rejects_hidden_files(#[case] input: &str) {
                let v = Validator::new_flexible();
                assert!(
                    matches!(
                        v.validate(input),
                        Err(PathValidationError::RestrictedPathError(_))
                    ),
                    "expected RestrictedPathError for '{input}'"
                );
            }

            #[test]
            fn accepts_normal_files() {
                let v = Validator::new_flexible();
                v.validate("notes/daily.md")
                    .expect("normal file should be valid");
            }
        }
    }

    mod resolve_safe_symlink {
        mod strict {
            use std::io::Write as _;

            use super::*;

            #[test]
            fn rejects_escaped_symlinks() {
                let ws = Workspace::new().expect("workspace");
                let mut outside = NamedTempFile::new().expect("outside file");
                outside.write_all(b"outside").expect("write");
                let symlink_path =
                    ws.create_symlink("escaped_link", outside.path());

                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                assert!(
                    matches!(
                        v.resolve_safe_symlink(&symlink_path),
                        Err(PathValidationError::SymlinkEscapeError)
                    ),
                    "expected SymlinkEscapeError for symlink escaping root"
                );
            }

            #[test]
            fn detects_symlink_loops() {
                let ws = Workspace::new().expect("workspace");
                let link_a = ws.root.join("link_a");
                let link_b = ws.root.join("link_b");
                create_symlink_loop(&link_a, &link_b);

                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                assert!(
                    matches!(
                        v.resolve_safe_symlink(&link_a),
                        Err(PathValidationError::IoError(_))
                    ),
                    "expected IoError for symlink loop"
                );
            }

            #[test]
            fn rejects_internal_hidden_targets() {
                let ws = Workspace::new().expect("workspace");
                let hidden = ws.create_file(".secret.txt", None);
                let symlink_path = ws.create_symlink("link_to_secret", &hidden);
                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                assert!(
                    matches!(
                        v.resolve_safe_symlink(&symlink_path),
                        Err(PathValidationError::RestrictedPathError(_))
                    ),
                    "expected RestrictedPathError for symlink to hidden file"
                );
            }
        }

        mod flexible {
            use std::io::Write as _;

            use super::*;

            #[test]
            fn allows_external_symlinks() {
                let ws = Workspace::new().expect("workspace");
                let mut outside = NamedTempFile::new().expect("outside file");
                outside.write_all(b"dotfile").expect("write");
                ws.create_symlink("dotfile_link", outside.path());

                let v = Validator::new_flexible();
                let _guard = CwdGuard::new(&ws.root)
                    .expect("failed to change directory");
                assert!(
                    v.resolve_safe_symlink("dotfile_link").is_ok(),
                    "flexible mode should allow external symlinks"
                );
            }

            #[test]
            fn enforces_input_traversal_checks() {
                let v = Validator::new_flexible();
                assert!(
                    matches!(
                        v.resolve_safe_symlink("../../../dotfile"),
                        Err(PathValidationError::PathTraversalError)
                    ),
                    "expected PathTraversalError on traversal input"
                );
            }
        }

        mod security_edge_cases {
            use super::*;

            #[cfg(unix)]
            #[test]
            fn rejects_symlink_chain_escaping_root() {
                // Given: A chain of symlinks that eventually escapes root
                let ws = Workspace::new().expect("workspace");
                ws.create_file("target.txt", Some("inside"));
                // First symlink points to second symlink
                let link1 = ws.root.join("link1.txt");
                let link2 = ws.root.join("link2.txt");
                // Second symlink points outside
                std::os::unix::fs::symlink(
                    ws.root.parent().unwrap_or(&ws.root),
                    &link2,
                )
                .expect("symlink");
                std::os::unix::fs::symlink(&link2, &link1).expect("symlink");

                // When: Resolving the chain in strict mode
                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                let result = v.resolve_safe_symlink(&link1);

                // Then: Rejects chain that escapes
                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::SymlinkEscapeError)
                    ),
                    "Expected SymlinkEscapeError for chain escaping root, \
                     got: {result:?}"
                );
            }

            #[cfg(unix)]
            #[test]
            fn resolves_valid_internal_symlink() {
                // Given: A valid symlink within the root
                let ws = Workspace::new().expect("workspace");
                let target = ws.create_file("target.txt", Some("content"));
                let symlink_path = ws.create_symlink("link.txt", &target);

                // When: Resolving in strict mode
                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                let result = v.resolve_safe_symlink(&symlink_path);

                // Then: Resolves to canonical target
                assert!(
                    result.is_ok(),
                    "Expected successful resolution for valid internal \
                     symlink, got: {result:?}"
                );
                let resolved = result.expect("resolved path");
                assert!(
                    resolved.starts_with(&ws.root),
                    "Resolved path should be within root"
                );
            }

            #[test]
            fn returns_error_for_nonexistent_symlink_target() {
                // Given: A symlink pointing to non-existent target
                let v = Validator::new_flexible();

                // When: Resolving broken symlink (relative to CWD)
                let result = v.resolve_safe_symlink("nonexistent_link");

                // Then: Returns IoError (canonicalize fails)
                assert!(
                    matches!(result, Err(PathValidationError::IoError(_))),
                    "Expected IoError for non-existent target, got: {result:?}"
                );
            }

            #[cfg(unix)]
            #[test]
            fn rejects_symlink_to_symlink_escaping() {
                // Given: A symlink pointing to another symlink that escapes
                let ws = Workspace::new().expect("workspace");
                let outside_dir = ws.root.parent().unwrap_or(&ws.root);
                let link1 = ws.root.join("inner_link.txt");
                let link2 = ws.root.join("outer_link.txt");

                // link2 points outside
                std::os::unix::fs::symlink(outside_dir, &link2)
                    .expect("symlink");
                // link1 points to link2
                std::os::unix::fs::symlink(&link2, &link1).expect("symlink");

                // When: Resolving link1
                let v = Validator::try_new_strict(ws.root.clone())
                    .expect("strict validator");
                let result = v.resolve_safe_symlink(&link1);

                // Then: Rejects because chain escapes
                assert!(
                    matches!(
                        result,
                        Err(PathValidationError::SymlinkEscapeError)
                    ),
                    "Expected SymlinkEscapeError for symlink chain, got: \
                     {result:?}"
                );
            }
        }

        use std::sync::{Mutex, MutexGuard, OnceLock};

        use tempfile::NamedTempFile;

        use super::{fixtures::Workspace, *};

        fn cwd_lock() -> &'static Mutex<()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(()))
        }

        struct CwdGuard {
            _guard: MutexGuard<'static, ()>,
            previous: PathBuf,
        }

        impl CwdGuard {
            fn new(path: &Path) -> Result<Self, std::io::Error> {
                let guard = cwd_lock().lock().map_err(|err| {
                    std::io::Error::other(format!("CWD lock poisoned: {err}"))
                })?;
                #[expect(
                    clippy::disallowed_methods,
                    reason = "Test-only CWD guard needs current_dir to \
                              restore state after the test. No alternative \
                              API exists for saving and restoring the working \
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
                if let Err(err) = std::env::set_current_dir(&self.previous) {
                    eprintln!("CwdGuard: failed to restore CWD: {err}");
                }
            }
        }

        #[cfg(unix)]
        fn create_symlink_loop(link_a: &Path, link_b: &Path) {
            let r1 = std::os::unix::fs::symlink(link_b, link_a);
            assert!(r1.is_ok(), "test setup: failed to create symlink: {r1:?}");
            let r2 = std::os::unix::fs::symlink(link_a, link_b);
            assert!(r2.is_ok(), "test setup: failed to create symlink: {r2:?}");
        }

        #[cfg(windows)]
        fn create_symlink_loop(link_a: &Path, link_b: &Path) {
            let r1 = std::os::windows::fs::symlink_file(link_b, link_a);
            assert!(r1.is_ok(), "test setup: failed to create symlink: {r1:?}");
            let r2 = std::os::windows::fs::symlink_file(link_a, link_b);
            assert!(r2.is_ok(), "test setup: failed to create symlink: {r2:?}");
        }
    }

    mod platform_specific {
        use super::*;

        #[test]
        fn handles_platform_separators_consistently() {
            let v = Validator::new_flexible();
            #[cfg(unix)]
            let path = "config/notes/file.md";
            #[cfg(windows)]
            let path = "config\\notes\\file.md";
            assert!(
                v.validate(path).is_ok(),
                "platform separators should be handled correctly"
            );
        }

        #[test]
        #[cfg(unix)]
        fn rejects_non_utf8_paths() {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

            let v = Validator::new_flexible();
            let os_str = OsStr::from_bytes(b"invalid\xffutf8");
            assert!(
                matches!(
                    v.validate(std::path::Path::new(os_str)),
                    Err(PathValidationError::InvalidPathEncoding(_))
                ),
                "expected InvalidPathEncoding for non-UTF-8 path"
            );
        }

        #[test]
        #[cfg(unix)]
        fn rejects_non_utf8_hidden_files() {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

            let v = Validator::new_flexible();
            let os_str = OsStr::from_bytes(b".\xffinvalid");
            assert!(
                v.validate(std::path::Path::new(os_str)).is_err(),
                "expected error for non-UTF-8 hidden file"
            );
        }
    }
}
