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
//! - **Single Responsibility:** Each function performs a specific validation
//!   check
//! - **Standardized Structure:** High-level validators decompose into
//!   specialized private helpers
//! - **Simple Parameters:** Functions take simple types for easy usage

use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;

use crate::{errors::DomainError, patterns};

// ----------------------------------------------------------- //
//                        Logic Helpers                        //
// ----------------------------------------------------------- //

/// Checks if a string matches the alphanumeric name pattern.
#[inline]
#[must_use]
pub(crate) fn is_alphanumeric_name(name: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        #[expect(
            clippy::expect_used,
            reason = "Static regex literal is safe and efficient"
        )]
        Regex::new(patterns::ALPHANUMERIC_NAME).expect("Static regex literal")
    });
    RE.is_match(name)
}

/// Checks if a string matches the identifier name pattern.
#[inline]
#[must_use]
pub(crate) fn is_identifier_name(name: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        #[expect(
            clippy::expect_used,
            reason = "Static regex literal is safe and efficient"
        )]
        Regex::new(patterns::IDENTIFIER_NAME).expect("Static regex literal")
    });
    RE.is_match(name)
}

/// Checks if a path is a Windows-style absolute path (e.g., C:/, D:/).
#[inline]
#[must_use]
pub(crate) fn is_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    check_windows_path_bytes(bytes)
}

#[inline]
#[must_use]
#[expect(
    clippy::indexing_slicing,
    clippy::missing_asserts_for_indexing,
    reason = "Indices 0, 1, 2 are checked by `bytes.len() >= 3` guard"
)]
fn check_windows_path_bytes(bytes: &[u8]) -> bool {
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && check_windows_separator(bytes[2])
}

#[inline]
#[must_use]
fn check_windows_separator(byte: u8) -> bool {
    byte == b'/' || byte == b'\\'
}

// ----------------------------------------------------------- //
//                      Domain Validators                      //
// ----------------------------------------------------------- //

/// Validates a vault-relative path according to hexagonal architecture rules.
///
/// Bundles common path rules: non-empty, relative, no-traversal, optional
/// extension.
#[inline]
pub(crate) fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), DomainError> {
    ensure_path_not_empty(path)?;
    ensure_path_is_relative(path)?;
    ensure_path_no_traversal(path)?;
    ensure_path_extension_if_required(path, require_extension)?;
    Ok(())
}

fn ensure_path_not_empty(path: &str) -> Result<(), DomainError> {
    if path.is_empty() {
        return Err(DomainError::EmptyPath);
    }
    Ok(())
}

fn ensure_path_is_relative(path: &str) -> Result<(), DomainError> {
    if path.starts_with('/') {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path must be relative",
        )));
    }
    check_windows_absolute(path)
}

fn check_windows_absolute(path: &str) -> Result<(), DomainError> {
    if is_windows_absolute_path(path) {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path must be relative (Windows absolute paths not allowed)",
        )));
    }
    Ok(())
}

fn ensure_path_no_traversal(path: &str) -> Result<(), DomainError> {
    if path.contains("..") {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path traversal not allowed",
        )));
    }
    Ok(())
}

fn ensure_path_extension_if_required(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), DomainError> {
    if let Some(required_ext) = require_extension {
        return check_path_extension(path, required_ext);
    }
    Ok(())
}

fn check_path_extension(
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

/// Validates that a numeric value falls within optional min/max bounds.
#[inline]
pub(crate) fn validate_numeric_range(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<(), DomainError> {
    ensure_numeric_at_least_min(value, min, max)?;
    ensure_numeric_at_most_max(value, min, max)?;
    Ok(())
}

fn ensure_numeric_at_least_min(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<(), DomainError> {
    if let Some(min_val) = min
        && value < min_val
    {
        return Err(DomainError::NumberOutOfRange {
            value,
            min,
            max,
        });
    }
    Ok(())
}

fn ensure_numeric_at_most_max(
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<(), DomainError> {
    if let Some(max_val) = max
        && value > max_val
    {
        return Err(DomainError::NumberOutOfRange {
            value,
            min,
            max,
        });
    }
    Ok(())
}

/// Validates that a numeric value aligns with a step increment from a base.
#[inline]
#[expect(
    clippy::float_arithmetic,
    clippy::modulo_arithmetic,
    reason = "Core numeric validation logic with epsilon comparison"
)]
pub(crate) fn validate_numeric_step(
    value: f64,
    base: f64,
    step: f64,
) -> Result<(), DomainError> {
    const EPSILON: f64 = 1e-10;
    let offset = (value - base).abs();
    let remainder = offset % step;
    check_numeric_remainder(value, step, remainder, EPSILON)
}

#[expect(
    clippy::float_arithmetic,
    reason = "Precision check for step validation"
)]
fn check_numeric_remainder(
    value: f64,
    step: f64,
    remainder: f64,
    epsilon: f64,
) -> Result<(), DomainError> {
    if remainder > epsilon && (step - remainder) > epsilon {
        return Err(DomainError::InvalidStepValue {
            value,
            step,
        });
    }
    Ok(())
}

/// Validates that a string length falls within optional min/max bounds.
#[inline]
pub(crate) fn validate_string_length(
    value: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), DomainError> {
    let len = value.len();
    ensure_string_at_least_min(len, min)?;
    ensure_string_at_most_max(len, max)?;
    Ok(())
}

fn ensure_string_at_least_min(
    len: usize,
    min: Option<usize>,
) -> Result<(), DomainError> {
    if let Some(min_len) = min
        && len < min_len
    {
        return Err(DomainError::StringTooShort {
            min: min_len,
            actual: len,
        });
    }
    Ok(())
}

fn ensure_string_at_most_max(
    len: usize,
    max: Option<usize>,
) -> Result<(), DomainError> {
    if let Some(max_len) = max
        && len > max_len
    {
        return Err(DomainError::StringTooLong {
            max: max_len,
            actual: len,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_vault_path_works() {
        // GIVEN: various path strings and requirements
        // WHEN: validating the paths
        // THEN: they succeed or fail as expected
        validate_vault_path("valid/path.md", Some("md")).unwrap();
        assert!(matches!(
            validate_vault_path("", None),
            Err(DomainError::EmptyPath)
        ));
        assert!(matches!(
            validate_vault_path("/abs", None),
            Err(DomainError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_vault_path("C:/abs.md", None),
            Err(DomainError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_vault_path("traversal/../path.md", None),
            Err(DomainError::InvalidPath(_))
        ));
        assert!(matches!(
            validate_vault_path("missing_ext.txt", Some("md")),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    #[expect(
        clippy::default_numeric_fallback,
        reason = "Float literals (5.0, 10.0) default to f64 which is correct \
                  type for numeric validation."
    )]
    fn validate_numeric_range_works() {
        // GIVEN: a value and range constraints
        // WHEN: validating the range
        // THEN: range boundaries are enforced
        validate_numeric_range(5.0, Some(0.0), Some(10.0)).unwrap();
        assert!(matches!(
            validate_numeric_range(-1.0, Some(0.0), None),
            Err(DomainError::NumberOutOfRange { .. })
        ));
        assert!(matches!(
            validate_numeric_range(11.0, None, Some(10.0)),
            Err(DomainError::NumberOutOfRange { .. })
        ));
    }

    #[test]
    fn validate_numeric_step_works() {
        // GIVEN: a value, base, and step increment
        // WHEN: validating the step alignment
        // THEN: only valid multiples are accepted
        validate_numeric_step(10.0, 0.0, 2.0).unwrap();
        validate_numeric_step(10.1, 0.0, 0.1).unwrap();
        assert!(matches!(
            validate_numeric_step(10.5, 0.0, 2.0),
            Err(DomainError::InvalidStepValue { .. })
        ));
    }

    #[test]
    fn validate_string_length_works() {
        // GIVEN: a string and length constraints
        // WHEN: validating the length
        // THEN: min/max limits are enforced
        validate_string_length("abc", Some(2), Some(5)).unwrap();
        assert!(matches!(
            validate_string_length("a", Some(2), None),
            Err(DomainError::StringTooShort { .. })
        ));
        assert!(matches!(
            validate_string_length("abcdef", None, Some(5)),
            Err(DomainError::StringTooLong { .. })
        ));
    }

    #[test]
    fn is_alphanumeric_name_works() {
        // GIVEN: various alphanumeric name candidates
        // THEN: format rules are enforced correctly
        assert!(is_alphanumeric_name("valid-name_123"));
        assert!(!is_alphanumeric_name("invalid name"));
        assert!(!is_alphanumeric_name("invalid!"));
    }

    #[test]
    fn is_identifier_name_works() {
        // GIVEN: various identifier name candidates
        // THEN: format rules for identifiers (snake_case) are enforced
        assert!(is_identifier_name("valid_name_123"));
        assert!(is_identifier_name("_private"));
        assert!(!is_identifier_name("123invalid"));
        assert!(!is_identifier_name("invalid-name"));
    }

    #[test]
    fn is_windows_absolute_path_works() {
        // GIVEN: various path strings
        // THEN: Windows absolute path formats are identified
        assert!(is_windows_absolute_path("C:/path"));
        assert!(is_windows_absolute_path("D:\\path"));
        assert!(!is_windows_absolute_path("/unix/path"));
        assert!(!is_windows_absolute_path("relative/path"));
    }
}
