//! Extracted reference link definitions.
//!
//! This module provides the [`ReferenceDefinitions`] type, which extracts and
//! normalizes link references (e.g., `[label]: /url`) from a markdown document
//! so they can be resolved during the event stream phase.
//!
//! # Pipeline Flow
//!
//! The reference resolution process happens in an eager extraction phase:
//! 1. The pulldown-cmark parser scans the document for reference definitions.
//! 2. [`ReferenceDefinitions::new`] extracts these, normalizing the labels
//!    according to CommonMark whitespace and case-folding rules.
//! 3. Later pipeline stages query [`ReferenceDefinitions::resolve`] to connect
//!    link events with their target URLs.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "Reference store is consumed incrementally")
)]

use std::collections::HashMap;

/// A collection of normalized reference link definitions.
///
/// This extracts references from the parser and provides O(1) resolution
/// for reference links during the event stream iteration.
#[derive(Debug, Clone)]
pub(crate) struct ReferenceDefinitions(
    HashMap<ReferenceLabel, ReferenceTarget>,
);

impl ReferenceDefinitions {
    /// Creates a new set of reference definitions from raw pulldown-cmark refs.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Note: Cannot run doctest for pub(crate) types from external test crate
    /// use std::collections::HashMap;
    /// use lithos_core::note::parser::references::ReferenceDefinitions;
    ///
    /// let mut raw = HashMap::new();
    /// raw.insert("  Foo  BAR  ".to_string(), "/url".to_string());
    /// let refs = ReferenceDefinitions::new(raw);
    /// assert_eq!(refs.resolve("foo bar"), Some("/url"));
    /// ```
    #[must_use]
    #[inline]
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Order is not important for normalization map"
    )]
    pub(crate) fn new(raw: HashMap<String, String>) -> Self {
        let mut normalized = HashMap::new();
        for (label, dest) in raw {
            let key = ReferenceLabel::new(&label);
            normalized
                .entry(key)
                .or_insert_with(|| ReferenceTarget(dest.into_boxed_str()));
        }
        Self(normalized)
    }

    /// Resolves a reference label to its destination URL.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Note: Cannot run doctest for pub(crate) types from external test crate
    /// use std::collections::HashMap;
    /// use lithos_core::note::parser::references::ReferenceDefinitions;
    ///
    /// let mut raw = HashMap::new();
    /// raw.insert("label".to_string(), "/dest".to_string());
    /// let refs = ReferenceDefinitions::new(raw);
    ///
    /// assert_eq!(refs.resolve("label"), Some("/dest"));
    /// assert_eq!(refs.resolve("missing"), None);
    /// ```
    #[must_use]
    #[inline]
    pub(crate) fn resolve(&self, label: &str) -> Option<&str> {
        let normalized = ReferenceLabel::new(label);
        self.0.get(&normalized).map(ReferenceTarget::as_str)
    }
}

/// A normalized reference link label.
///
/// Labels are case-insensitive and have internal whitespace collapsed according
/// to the `CommonMark` specification. This type guarantees normalization
/// through its constructor, implementing "parse, don't validate".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ReferenceLabel(Box<str>);

impl ReferenceLabel {
    /// Creates a normalized reference label from a raw string.
    ///
    /// Applies `CommonMark` normalization rules:
    /// - Case folding (lowercase ASCII)
    /// - Whitespace collapsing (trim + collapse internal)
    /// - Backslash escape removal
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::note::parser::references::ReferenceLabel;
    ///
    /// let label = ReferenceLabel::new("  Foo  BAR  ");
    /// assert_eq!(label.as_str(), "foo bar");
    /// ```
    #[must_use]
    pub(crate) fn new(raw_label: &str) -> Self {
        if Self::is_normalized(raw_label) {
            return Self(raw_label.to_owned().into_boxed_str());
        }

        let unescaped = Self::unescape_label(raw_label);
        let folded = Self::fold_case(&unescaped);
        let collapsed = Self::collapse_whitespace(&folded);

        Self(collapsed.into_boxed_str())
    }

    /// Returns the string slice of the normalized label.
    #[must_use]
    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Checks if a label is already normalized.
    fn is_normalized(label: &str) -> bool {
        !label.chars().any(|c| c.is_ascii_uppercase())
            && !label.starts_with([' ', '\t'])
            && !label.ends_with([' ', '\t'])
            && !label.contains("  ")
            && !label.chars().any(|c| c.is_whitespace() && c != ' ')
            && !label.contains('\\')
    }

    /// Removes backslash escapes from the label.
    fn unescape_label(label: &str) -> String {
        let mut unescaped = String::with_capacity(label.len());
        let mut chars = label.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                unescaped.push(chars.next().unwrap_or('\\'));
            } else {
                unescaped.push(ch);
            }
        }
        unescaped
    }

    /// Folds the label to lowercase.
    fn fold_case(label: &str) -> String {
        label.to_ascii_lowercase()
    }

    /// Collapses internal whitespace and trims the label.
    fn collapse_whitespace(label: &str) -> String {
        let mut collapsed = String::with_capacity(label.len());
        let mut last_was_space = false;

        for ch in label.chars() {
            if ch.is_whitespace() {
                if collapsed.is_empty() || last_was_space {
                    continue;
                }
                collapsed.push(' ');
                last_was_space = true;
                continue;
            }

            collapsed.push(ch);
            last_was_space = false;
        }

        if last_was_space {
            collapsed.pop();
        }

        collapsed
    }
}

/// A reference link target URL or destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceTarget(Box<str>);

impl ReferenceTarget {
    /// Returns the string slice of the target.
    #[must_use]
    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module keeps imports and nested suites grouped for \
              readability"
)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    mod reference_definitions_new {
        use super::*;

        #[test]
        fn collapses_duplicate_normalized_labels_to_single_entry() {
            let raw =
                map_from(&[("Foo   Bar", "/first"), (" foo bar ", "/second")]);

            let defs = ReferenceDefinitions::new(raw);

            assert_eq!(
                defs.0.len(),
                1,
                "duplicate labels should normalize into a single stored entry"
            );
        }
    }

    mod reference_definitions_resolve {
        use super::*;

        #[test]
        fn returns_none_when_label_is_missing() {
            let defs =
                ReferenceDefinitions::new(map_from(&[("known", "/url")]));

            assert_eq!(
                defs.resolve("unknown"),
                None,
                "resolve should return None for unknown reference labels"
            );
        }

        #[test]
        fn matches_labels_case_insensitively() {
            let defs =
                ReferenceDefinitions::new(map_from(&[("MiXeD", "/url")]));

            assert_eq!(
                defs.resolve("mixed"),
                Some("/url"),
                "resolve should treat labels as case-insensitive"
            );
        }

        #[test]
        fn collapses_internal_whitespace_and_trims_edges() {
            let defs =
                ReferenceDefinitions::new(map_from(&[("a\tb\nc", "/url")]));

            assert_eq!(
                defs.resolve("  a  b   c  "),
                Some("/url"),
                "resolve should normalize internal and edge whitespace"
            );
        }

        #[test]
        fn unescapes_backslash_sequences_in_labels() {
            let defs =
                ReferenceDefinitions::new(map_from(&[("Foo\\ Bar", "/url")]));

            assert_eq!(
                defs.resolve("foo bar"),
                Some("/url"),
                "resolve should normalize backslash-escaped label content"
            );
        }
    }

    mod reference_label {
        use super::*;

        #[test]
        fn as_str_returns_inner_slice() {
            let label = ReferenceLabel("normalized".into());

            assert_eq!(
                label.as_str(),
                "normalized",
                "label accessor should return the stored normalized slice"
            );
        }
    }

    mod reference_target {
        use super::*;

        #[test]
        fn as_str_returns_inner_slice() {
            let target = ReferenceTarget("/dest".into());

            assert_eq!(
                target.as_str(),
                "/dest",
                "target accessor should return the stored destination"
            );
        }
    }

    fn map_from(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|&(label, target)| (label.to_owned(), target.to_owned()))
            .collect()
    }
}
