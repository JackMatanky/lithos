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

use std::collections::HashMap;

/// A normalized reference link label.
///
/// Labels are case-insensitive and have internal whitespace collapsed according
/// to the `CommonMark` specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceLabel(Box<str>);

impl ReferenceLabel {
    /// Returns the string slice of the label.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A reference link target URL or destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTarget(Box<str>);

impl ReferenceTarget {
    /// Returns the string slice of the target.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A collection of normalized reference link definitions.
///
/// This extracts references from the parser and provides O(1) resolution
/// for reference links during the event stream iteration.
#[derive(Debug, Clone)]
pub struct ReferenceDefinitions(HashMap<ReferenceLabel, ReferenceTarget>);

impl ReferenceDefinitions {
    /// Creates a new set of reference definitions from raw pulldown-cmark refs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::parser::references::ReferenceDefinitions;
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
    pub fn new(raw: HashMap<String, String>) -> Self {
        let mut normalized = HashMap::new();
        for (label, dest) in raw {
            let key = Self::normalize_label(&label);
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
    /// ```
    /// # use std::collections::HashMap;
    /// # use lithos_core::note::parser::references::ReferenceDefinitions;
    /// # let mut raw = HashMap::new();
    /// # raw.insert("label".to_string(), "/dest".to_string());
    /// # let refs = ReferenceDefinitions::new(raw);
    /// assert_eq!(refs.resolve("label"), Some("/dest"));
    /// assert_eq!(refs.resolve("missing"), None);
    /// ```
    #[must_use]
    #[inline]
    pub fn resolve(&self, label: &str) -> Option<&str> {
        let normalized = Self::normalize_label(label);
        self.0.get(&normalized).map(ReferenceTarget::as_str)
    }

    /// Orchestrates the normalization of a reference label.
    fn normalize_label(label: &str) -> ReferenceLabel {
        if !Self::needs_normalization(label) {
            return ReferenceLabel(label.to_owned().into_boxed_str());
        }

        let unescaped = Self::unescape_label(label);
        let folded = Self::fold_case(&unescaped);
        let collapsed = Self::collapse_whitespace(&folded);

        ReferenceLabel(collapsed.into_boxed_str())
    }

    /// Checks if a label requires whitespace collapsing or case folding.
    fn needs_normalization(label: &str) -> bool {
        label.chars().any(|c| c.is_ascii_uppercase())
            || label.starts_with([' ', '\t'])
            || label.ends_with([' ', '\t'])
            || label.contains("  ")
            || label.contains('\\')
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
