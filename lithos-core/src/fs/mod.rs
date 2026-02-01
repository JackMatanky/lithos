//! File system utilities for parsing and validation.
//!
//! This module provides generic file system operations with no domain
//! knowledge. Dependencies flow inward: domain contexts may use fs utilities,
//! but fs has no dependencies on domain logic.

/// File system error types.
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific error name is intentional"
)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// IO operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Checks if a path is a Windows-style absolute path (e.g., C:/, D:/).
#[inline]
#[must_use]
pub fn is_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    check_windows_path_bytes(bytes)
}

#[inline]
#[must_use]
fn check_windows_path_bytes(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes.get(1) == Some(&b':')
        && bytes.get(2).is_some_and(|&b| check_windows_separator(b))
}

#[inline]
#[must_use]
fn check_windows_separator(byte: u8) -> bool {
    byte == b'/' || byte == b'\\'
}

/// Validates a vault-relative path.
///
/// Bundles common path rules: non-empty, relative, no-traversal, optional
/// extension.
///
/// # Errors
///
/// Returns an error string if the path is empty, absolute, contains traversal
/// segments (`..`), or does not match the required extension.
#[inline]
pub fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_owned());
    }
    if path.starts_with('/') {
        return Err("Path must be relative".to_owned());
    }
    if is_windows_absolute_path(path) {
        return Err("Path must be relative (Windows absolute paths not \
                    allowed)"
            .to_owned());
    }
    if path.contains("..") {
        return Err("Path traversal not allowed".to_owned());
    }
    if let Some(required_ext) = require_extension
        && !std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(required_ext))
    {
        return Err(format!("Path must end with .{required_ext}"));
    }
    Ok(())
}
