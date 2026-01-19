//! Internal validation utilities shared across bounded contexts.
//!
//! This module provides reusable validation functions to eliminate redundancy
//! across Note, Schema, Template, and Config bounded contexts.
//!
//! # Visibility
//! All items are `pub(crate)` - internal to domain crate only. These utilities
//! are not part of the public API and should not leak to the application layer.
//!
//! # Design Principles
//! - **Pure Functions:** All validation is deterministic with no side effects
//! - **Error Consistency:** Uses existing `DomainError` variants
//! - **Composability:** Small, focused functions that can be combined
//! - **Zero I/O:** No external dependencies or file system access

#![allow(
    dead_code,
    reason = "Some functions will be used in future refactoring"
)]

// ============================================================================
// Error Message Constants
// ============================================================================

/// Static error messages to avoid repeated allocations.
///
/// These are used with `Cow::Borrowed` to avoid heap allocations for
/// common validation errors.
mod error_messages {
    pub(super) const PATH_RELATIVE: &str = "Path must be relative";
    pub(super) const PATH_WINDOWS_ABS: &str =
        "Path must be relative (Windows absolute paths not allowed)";
    pub(super) const PATH_TRAVERSAL: &str = "Path traversal not allowed";
}

use std::borrow::Cow;

use crate::errors::DomainError;

// ============================================================================
// Name Validation
// ============================================================================

/// Validates that a string is non-empty.
///
/// # Errors
/// Returns appropriate `DomainError` variant based on field type.
#[inline]
pub(crate) fn validate_non_empty(
    value: &str,
    field: &str,
) -> Result<(), DomainError> {
    if value.is_empty() {
        return Err(match field {
            "schema_name" => DomainError::EmptySchemaName,
            "property_name" => DomainError::EmptyPropertyName,
            "path" => DomainError::EmptyPath,
            _ => DomainError::ValidationFailed(format!(
                "{field} cannot be empty"
            )),
        });
    }
    Ok(())
}

/// Validates that a string length does not exceed the maximum.
///
/// # Errors
/// Returns appropriate `DomainError` variant based on field type.
#[inline]
pub(crate) fn validate_max_length(
    value: &str,
    max: usize,
    field: &str,
) -> Result<(), DomainError> {
    if value.len() > max {
        return Err(match field {
            "schema_name" => DomainError::SchemaNameTooLong(value.len()),
            "property_name" => DomainError::PropertyNameTooLong(value.len()),
            _ => DomainError::ValidationFailed(format!(
                "{field} too long: {} > {max}",
                value.len()
            )),
        });
    }
    Ok(())
}

// ============================================================================
// Path Validation
// ============================================================================

/// Validates a vault-relative path according to hexagonal architecture rules.
///
/// # Validation Rules
/// - Must not be empty
/// - Must be relative (not absolute Unix or Windows paths)
/// - Must not contain path traversal sequences (`..`)
/// - Optionally must have a specific extension
///
/// # Errors
/// - `DomainError::EmptyPath` if path is empty
/// - `DomainError::InvalidPath` if path fails validation
#[inline]
pub(crate) fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), DomainError> {
    validate_path_not_empty(path)?;
    validate_path_is_relative(path)?;
    validate_path_no_traversal(path)?;

    if let Some(ext) = require_extension {
        validate_path_has_extension(path, ext)?;
    }

    Ok(())
}

/// Validates that a path is not empty.
#[inline]
fn validate_path_not_empty(path: &str) -> Result<(), DomainError> {
    if path.is_empty() {
        return Err(DomainError::EmptyPath);
    }
    Ok(())
}

/// Validates that a path is relative (not absolute).
#[inline]
fn validate_path_is_relative(path: &str) -> Result<(), DomainError> {
    if path.starts_with('/') {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            error_messages::PATH_RELATIVE,
        )));
    }
    if is_windows_absolute_path(path) {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            error_messages::PATH_WINDOWS_ABS,
        )));
    }
    Ok(())
}

/// Validates that a path does not contain path traversal sequences.
#[inline]
fn validate_path_no_traversal(path: &str) -> Result<(), DomainError> {
    if path.contains("..") {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            error_messages::PATH_TRAVERSAL,
        )));
    }
    Ok(())
}

/// Validates that a path has the required extension.
#[inline]
fn validate_path_has_extension(
    path: &str,
    required_ext: &str,
) -> Result<(), DomainError> {
    if !std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(required_ext))
    {
        return Err(DomainError::InvalidPath(Cow::Owned(format!(
            "Path must end with .{required_ext}"
        ))));
    }
    Ok(())
}

/// Checks if a path is a Windows-style absolute path (e.g., C:/, D:/).
///
/// This function checks for the pattern `X:/` where X is an alphabetic character.
#[inline]
#[must_use]
#[expect(
    clippy::indexing_slicing,
    clippy::missing_asserts_for_indexing,
    reason = "Indices 0, 1, 2 are checked by `bytes.len() >= 3` guard"
)]
pub(crate) fn is_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

// ============================================================================
// Numeric Range Validation
// ============================================================================

/// Validates that a numeric value falls within optional min/max bounds.
///
/// # Errors
/// Returns `DomainError::ValidationFailed` if value is out of range.
#[inline]
pub(crate) fn validate_numeric_range(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    field: &str,
) -> Result<(), DomainError> {
    if let Some(min_val) = min
        && value < min_val
    {
        return Err(DomainError::ValidationFailed(format!(
            "{field} must be >= {min_val}, got {value}"
        )));
    }

    if let Some(max_val) = max
        && value > max_val
    {
        return Err(DomainError::ValidationFailed(format!(
            "{field} must be <= {max_val}, got {value}"
        )));
    }

    Ok(())
}

/// Validates that a numeric value aligns with a step increment from a base.
///
/// # Errors
/// Returns `DomainError::ValidationFailed` if value doesn't align with step.
#[inline]
#[expect(
    clippy::float_arithmetic,
    clippy::modulo_arithmetic,
    reason = "Safe f64 arithmetic for validation logic with epsilon comparison"
)]
pub(crate) fn validate_numeric_step(
    value: f64,
    base: f64,
    step: f64,
) -> Result<(), DomainError> {
    // Use epsilon for floating-point comparison
    const EPSILON: f64 = 1e-10;

    let offset = value - base;
    let remainder = offset % step;

    if remainder.abs() > EPSILON && (step - remainder).abs() > EPSILON {
        return Err(DomainError::ValidationFailed(format!(
            "Value {value} must align with step {step} from base {base}"
        )));
    }

    Ok(())
}

// ============================================================================
// String Length Validation
// ============================================================================

/// Validates that a string length falls within optional min/max bounds.
///
/// # Errors
/// Returns `DomainError::ValidationFailed` if length is out of range.
#[inline]
pub(crate) fn validate_string_length(
    value: &str,
    min: Option<usize>,
    max: Option<usize>,
    field: &str,
) -> Result<(), DomainError> {
    let len = value.len();

    if let Some(min_len) = min
        && len < min_len
    {
        return Err(DomainError::ValidationFailed(format!(
            "{field} must be at least {min_len} characters, got {len}"
        )));
    }

    if let Some(max_len) = max
        && len > max_len
    {
        return Err(DomainError::ValidationFailed(format!(
            "{field} must be at most {max_len} characters, got {len}"
        )));
    }

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::assertions_on_result_states,
    reason = "Unit tests use assert for simplicity"
)]
mod tests {
    use super::*;

    mod validate_non_empty {
        use super::*;

        #[test]
        fn returns_error_when_empty() {
            let result = validate_non_empty("", "test_field");
            assert!(result.is_err());
        }

        #[test]
        fn succeeds_when_non_empty() {
            let result = validate_non_empty("value", "test_field");
            assert!(result.is_ok());
        }

        #[test]
        fn returns_schema_name_specific_error() {
            let result = validate_non_empty("", "schema_name");
            assert!(matches!(result, Err(DomainError::EmptySchemaName)));
        }

        #[test]
        fn returns_property_name_specific_error() {
            let result = validate_non_empty("", "property_name");
            assert!(matches!(result, Err(DomainError::EmptyPropertyName)));
        }

        #[test]
        fn returns_path_specific_error() {
            let result = validate_non_empty("", "path");
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }
    }

    mod validate_max_length {
        use super::*;

        #[test]
        fn returns_error_when_too_long() {
            let result = validate_max_length("too_long", 5, "test_field");
            assert!(result.is_err());
        }

        #[test]
        fn succeeds_when_within_limit() {
            let result = validate_max_length("ok", 5, "test_field");
            assert!(result.is_ok());
        }

        #[test]
        fn succeeds_when_exactly_max() {
            let result = validate_max_length("exact", 5, "test_field");
            assert!(result.is_ok());
        }
    }

    mod validate_vault_path {
        use super::*;

        #[test]
        fn returns_error_when_empty() {
            let result = validate_vault_path("", None);
            assert!(matches!(result, Err(DomainError::EmptyPath)));
        }

        #[test]
        fn returns_error_when_absolute_unix() {
            let result = validate_vault_path("/absolute/path.md", None);
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn returns_error_when_absolute_windows() {
            let result = validate_vault_path("C:/absolute/path.md", None);
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn returns_error_when_traversal() {
            let result = validate_vault_path("../etc/passwd", None);
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn returns_error_when_missing_extension() {
            let result = validate_vault_path("path/to/file", Some("md"));
            assert!(matches!(result, Err(DomainError::InvalidPath(_))));
        }

        #[test]
        fn succeeds_for_valid_relative_path() {
            let result = validate_vault_path("projects/lithos.md", Some("md"));
            assert!(result.is_ok());
        }
    }

    mod is_windows_absolute_path {
        use super::*;

        #[test]
        fn detects_windows_absolute() {
            assert!(is_windows_absolute_path("C:/Users/file.txt"));
            assert!(is_windows_absolute_path("D:/Projects/"));
            assert!(is_windows_absolute_path("C:\\Users\\file.txt"));
            assert!(is_windows_absolute_path("D:\\Projects\\"));
        }

        #[test]
        fn rejects_relative_paths() {
            assert!(!is_windows_absolute_path("relative/path.txt"));
            assert!(!is_windows_absolute_path("/unix/absolute/path"));
            assert!(!is_windows_absolute_path("C:relative"));
            assert!(!is_windows_absolute_path("1:/invalid"));
        }
    }

    mod validate_numeric_range {
        use super::*;

        #[test]
        fn returns_error_when_below_min() {
            let result =
                validate_numeric_range(5.0f64, Some(10.0f64), None, "value");
            assert!(result.is_err());
        }

        #[test]
        fn returns_error_when_above_max() {
            let result =
                validate_numeric_range(15.0f64, None, Some(10.0f64), "value");
            assert!(result.is_err());
        }

        #[test]
        fn succeeds_when_within_range() {
            let result = validate_numeric_range(
                5.0f64,
                Some(0.0f64),
                Some(10.0f64),
                "value",
            );
            assert!(result.is_ok());
        }

        #[test]
        fn succeeds_when_no_constraints() {
            let result = validate_numeric_range(100.0f64, None, None, "value");
            assert!(result.is_ok());
        }
    }

    mod validate_string_length {
        use super::*;

        #[test]
        fn returns_error_when_too_short() {
            let result = validate_string_length("ab", Some(3), None, "field");
            assert!(result.is_err());
        }

        #[test]
        fn returns_error_when_too_long() {
            let result =
                validate_string_length("abcdef", None, Some(5), "field");
            assert!(result.is_err());
        }

        #[test]
        fn succeeds_when_within_range() {
            let result =
                validate_string_length("abc", Some(2), Some(5), "field");
            assert!(result.is_ok());
        }
    }
}
