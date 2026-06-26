//! Rendered template output produced by the engine port.
//!
//! [`RenderedTemplate`] is the load-bearing newtype the [`TemplateEngine`]
//! port returns from `render`. Keeping it in the engine module (rather than in
//! the service) means the port speaks the domain newtype directly instead of a
//! bare `String`, so callers cannot confuse rendered output with arbitrary
//! text.
//!
//! [`TemplateEngine`]: super::TemplateEngine

/// Rendered template text produced by [`TemplateEngine::render`].
///
/// The [`TemplateService`] feeds the inner string into the artifact write
/// pipeline (via [`RenderedTemplate::into_inner`]) and surfaces a clone in
/// dry-run previews (via [`RenderedTemplate::as_str`]).
///
/// [`TemplateEngine::render`]: super::TemplateEngine::render
/// [`TemplateService`]: crate::TemplateService
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTemplate(String);

impl RenderedTemplate {
    /// Wraps rendered engine output in the load-bearing newtype.
    #[inline]
    #[must_use]
    pub fn new(content: String) -> Self {
        Self(content)
    }

    /// Returns the rendered template content as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the owned rendered string.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::RenderedTemplate;

    mod constructor {
        use pretty_assertions::assert_eq;

        use super::RenderedTemplate;

        #[test]
        fn new_stores_content() {
            let rendered = RenderedTemplate::new("Hello Alice".to_owned());

            assert_eq!(rendered.as_str(), "Hello Alice");
        }
    }

    mod into_inner {
        use pretty_assertions::assert_eq;

        use super::RenderedTemplate;

        #[test]
        fn returns_owned_content() {
            let rendered = RenderedTemplate::new("Hello Alice".to_owned());

            assert_eq!(rendered.into_inner(), "Hello Alice".to_owned());
        }
    }
}
