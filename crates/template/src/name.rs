use std::fmt;

use crate::error::TemplateNameError;

/// A validated template name.
///
/// Wraps a [`String`] and guarantees it is non-empty when constructed via
/// [`TemplateName::new`]. Use [`TemplateName::unchecked`] when the caller has
/// already validated the input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateName(String);

impl TemplateName {
    /// Validates and wraps a template name.
    ///
    /// # Errors
    ///
    /// Returns [`TemplateNameError::Empty`] when `name` is the empty string.
    #[inline]
    pub fn new(name: &str) -> Result<Self, TemplateNameError> {
        if name.is_empty() {
            return Err(TemplateNameError::Empty);
        }
        Ok(Self(name.to_owned()))
    }

    /// Wraps a name without validation.
    ///
    /// Use only when the name is guaranteed valid (e.g., already validated by
    /// the caller).
    #[inline]
    #[must_use]
    pub fn unchecked(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// Returns the template name as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TemplateName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TemplateName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod constructor {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn new_creates_name_from_string() {
        let name = TemplateName::new("greeting").unwrap();
        assert_eq!(name.as_str(), "greeting");
    }

    #[test]
    fn new_rejects_empty_string() {
        let err = TemplateName::new("").unwrap_err();
        assert!(matches!(err, TemplateNameError::Empty));
    }

    /// Verifies that `unchecked("")` creates a `TemplateName` wrapping the
    /// empty string rather than panicking or rejecting it.
    #[test]
    fn unchecked_accepts_empty_string() {
        let name = TemplateName::unchecked("");
        assert_eq!(name.as_str(), "");
    }
}

#[cfg(test)]
mod formatting {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn display_returns_name() {
        let name = TemplateName::unchecked("daily/standup");
        assert_eq!(name.to_string(), "daily/standup");
    }

    #[test]
    fn as_ref_returns_str() {
        let name = TemplateName::unchecked("test");
        assert_eq!(name.as_ref(), "test");
    }
}
