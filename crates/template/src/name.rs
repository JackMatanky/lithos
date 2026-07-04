use std::fmt;

use crate::error::TemplateNameError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateName(String);

impl TemplateName {
    #[inline]
    pub fn new(name: impl Into<String>) -> Result<Self, TemplateNameError> {
        let name = name.into();
        if name.is_empty() {
            return Err(TemplateNameError::Empty);
        }
        Ok(Self(name))
    }

    #[inline]
    #[must_use]
    pub fn unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

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
mod tests {
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
