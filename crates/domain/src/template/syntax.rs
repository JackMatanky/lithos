/// Defines the syntax for variables placeholders within a template.
///
/// # Examples
/// ```
/// # use lithos_domain::PlaceholderSyntax;
/// let syntax = PlaceholderSyntax::new("{{", "}}");
/// assert_eq!(syntax.wrap("title"), "{{title}}");
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct PlaceholderSyntax {
    /// The opening delimiter (e.g., "{{").
    pub prefix: String,
    /// The closing delimiter (e.g., "}}").
    pub suffix: String,
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
    #[expect(clippy::arithmetic_side_effects, reason = "String capacity")]
    pub fn wrap(&self, var_name: &str) -> String {
        let mut placeholder = String::with_capacity(
            var_name.len() + self.prefix.len() + self.suffix.len(),
        );
        placeholder.push_str(&self.prefix);
        placeholder.push_str(var_name);
        placeholder.push_str(&self.suffix);
        placeholder
    }
}

impl Default for PlaceholderSyntax {
    #[inline]
    fn default() -> Self {
        Self::new("{{", "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_syntax_wraps_variables() {
        let syntax = PlaceholderSyntax::default();

        assert_eq!(syntax.wrap("name"), "{{name}}");
    }
}
