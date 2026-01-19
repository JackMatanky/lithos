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
//! - **Single Responsibility:** Each function performs a specific validation check
//! - **Error Consistency:** Uses existing `DomainError` variants
//! - **Simple Parameters:** Functions take simple types for easy usage

use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;

use crate::{errors::DomainError, patterns};

// ============================================================================
// Logic Helpers (Predicates)
// ============================================================================

/// Checks if a string matches the alphanumeric name pattern.
#[inline]
#[must_use]
pub(crate) fn is_alphanumeric_name(name: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        #[expect(
            clippy::expect_used,
            clippy::disallowed_methods,
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
            clippy::disallowed_methods,
            reason = "Static regex literal is safe and efficient"
        )]
        Regex::new(patterns::IDENTIFIER_NAME).expect("Static regex literal")
    });
    RE.is_match(name)
}

/// Checks if a path is a Windows-style absolute path (e.g., C:/, D:/).
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
// Domain Validators (Reusable Complex Logic)
// ============================================================================

/// Validates a vault-relative path according to hexagonal architecture rules.
///
/// Bundles common path rules: non-empty, relative, no-traversal, optional extension.
#[inline]
pub(crate) fn validate_vault_path(
    path: &str,
    require_extension: Option<&str>,
) -> Result<(), DomainError> {
    if path.is_empty() {
        return Err(DomainError::EmptyPath);
    }
    if path.starts_with('/') {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path must be relative",
        )));
    }
    if is_windows_absolute_path(path) {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path must be relative (Windows absolute paths not allowed)",
        )));
    }
    if path.contains("..") {
        return Err(DomainError::InvalidPath(Cow::Borrowed(
            "Path traversal not allowed",
        )));
    }

    if let Some(required_ext) = require_extension
        && !std::path::Path::new(path)
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
    if let Some(min_val) = min
        && value < min_val
    {
        return Err(DomainError::NumberOutOfRange {
            value,
            min,
            max,
        });
    }

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
    // Use epsilon for floating-point comparison
    const EPSILON: f64 = 1e-10;

    let offset = (value - base).abs();
    let remainder = offset % step;

    if remainder > EPSILON && (step - remainder) > EPSILON {
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

    if let Some(min_len) = min
        && len < min_len
    {
        return Err(DomainError::StringTooShort {
            min: min_len,
            actual: len,
        });
    }

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
    #[expect(
        clippy::disallowed_methods,
        reason = "Test expectations use unwrap"
    )]
    fn validate_vault_path_works() {
        validate_vault_path("valid/path.md", Some("md")).unwrap();
        assert!(matches!(
            validate_vault_path("", None),
            Err(DomainError::EmptyPath)
        ));
        assert!(matches!(
            validate_vault_path("/abs", None),
            Err(DomainError::InvalidPath(_))
        ));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test expectations use unwrap"
    )]
    #[expect(
        clippy::default_numeric_fallback,
        reason = "Literal f64 in tests is standard"
    )]
    fn validate_numeric_range_works() {
        validate_numeric_range(5.0, Some(0.0), Some(10.0)).unwrap();
        assert!(matches!(
            validate_numeric_range(-1.0, Some(0.0), None),
            Err(DomainError::NumberOutOfRange { .. })
        ));
    }

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test expectations use unwrap"
    )]
    fn validate_string_length_works() {
        validate_string_length("abc", Some(2), Some(5)).unwrap();
        assert!(matches!(
            validate_string_length("a", Some(2), None),
            Err(DomainError::StringTooShort { .. })
        ));
    }
}
