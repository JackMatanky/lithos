#![allow(
    clippy::exhaustive_structs,
    reason = "rkyv Archive derive generates non-exhaustive archived type"
)]

/// Defines the syntax for variables placeholders within a template.
///
/// # Examples
/// ```
/// # use lithos_core::template::syntax::PlaceholderSyntax;
/// let syntax = PlaceholderSyntax::new("{{", "}}");
/// assert_eq!(syntax.wrap("title"), "{{title}}");
/// ```

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct PlaceholderSyntax {
    /// The opening delimiter (e.g., "{{").
    pub prefix: String,
    /// The closing delimiter (e.g., "}}").
    pub suffix: String,
}

impl Default for PlaceholderSyntax {
    #[inline]
    fn default() -> Self {
        Self::new("{{", "}}")
    }
}

impl PlaceholderSyntax {
    /// Creates a new placeholder syntax.
    #[inline]
    #[must_use]
    pub fn new<S: Into<String>>(prefix: S, suffix: S) -> Self {
        Self {
            prefix: prefix.into(),
            suffix: suffix.into(),
        }
    }

    /// Wraps a variable name with the defined delimiters.
    #[inline]
    #[must_use]
    pub fn wrap(&self, var_name: &str) -> String {
        let capacity = var_name
            .len()
            .saturating_add(self.prefix.len())
            .saturating_add(self.suffix.len());
        let mut placeholder = String::with_capacity(capacity);
        placeholder.push_str(&self.prefix);
        placeholder.push_str(var_name);
        placeholder.push_str(&self.suffix);
        placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_syntax_wraps_variables() {
        // GIVEN: the default placeholder syntax
        let syntax = PlaceholderSyntax::default();

        // WHEN: wrapping a variable name
        let wrapped = syntax.wrap("name");

        // THEN: it uses the standard double-curly braces
        assert_eq!(wrapped, "{{name}}");
    }
}
