//! Raw template DTO.
//!
//! Provides [`RawTemplate`], a thin newtype around `String` for passing raw
//! template file content through the ingestion pipeline. The ingestion context
//! (processor) carries the path separately — this type holds only the content.

// ============================================================================
// RawTemplate
// ============================================================================

/// Thin raw-content newtype for template file content.
///
/// No validation beyond what `String` provides (valid UTF-8 guaranteed by
/// Rust). The ingestion context carries the associated path separately,
/// matching how `RawSchema` works.
///
/// # Examples
///
/// ```
/// use traces_template::RawTemplate;
///
/// let raw = RawTemplate::new("# Hello {{ name }}".to_owned());
/// assert_eq!(raw.as_ref(), "# Hello {{ name }}");
/// let s = raw.into_inner();
/// assert_eq!(s, "# Hello {{ name }}");
/// ```
#[derive(Debug)]
pub struct RawTemplate(String);

impl RawTemplate {
    /// Constructs a `RawTemplate` from the given content string.
    ///
    /// No validation is performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_template::RawTemplate;
    ///
    /// let raw = RawTemplate::new("content".to_owned());
    /// assert_eq!(raw.as_ref(), "content");
    /// ```
    #[inline]
    #[must_use]
    pub fn new(content: String) -> Self {
        Self(content)
    }

    /// Consumes the `RawTemplate` and returns the inner `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use traces_template::RawTemplate;
    ///
    /// let raw = RawTemplate::new("content".to_owned());
    /// assert_eq!(raw.into_inner(), "content");
    /// ```
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for RawTemplate {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod raw_template {
        use super::*;

        mod constructor {
            use pretty_assertions::assert_eq;

            use super::*;

            #[test]
            fn new_stores_content() {
                let raw = RawTemplate::new("hello world".to_owned());
                assert_eq!(raw.as_ref(), "hello world");
            }

            #[test]
            fn new_accepts_empty_string() {
                // RawTemplate has no validation — empty is allowed.
                let raw = RawTemplate::new(String::new());
                assert_eq!(raw.as_ref(), "");
            }

            #[test]
            fn as_ref_str_returns_content() {
                let raw = RawTemplate::new("template content".to_owned());
                let s: &str = raw.as_ref();
                assert_eq!(s, "template content");
            }

            #[test]
            fn into_inner_returns_owned_string() {
                let content = "owned content".to_owned();
                let raw = RawTemplate::new(content.clone());
                assert_eq!(raw.into_inner(), content);
            }

            #[test]
            fn into_inner_consumes_value() {
                // This test verifies the API works correctly by consuming the
                // value.
                let raw = RawTemplate::new("consumed".to_owned());
                let s = raw.into_inner();
                assert_eq!(s, "consumed");
                // raw is moved — this would not compile if we tried to use it
                // again.
            }
        }
    }
}
