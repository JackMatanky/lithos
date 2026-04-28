//! Configuration types for the markdown event stream.
//!
//! This module provides explicit, data-driven policies for the markdown parsing
//! pipeline. By treating parser behavior (such as how to handle newlines or
//! which markdown extensions to enable) as a separate configuration object,
//! the parsing and extraction stages are kept decoupled from implicit branching
//! logic.

use pulldown_cmark::Options;

/// Configuration for the markdown event stream.
///
/// Defines the specific behavior of the underlying `pulldown-cmark` parser
/// and the normalization rules applied by the `MarkdownEventStream`.
///
/// Fields are private to enforce immutability after construction and enable
/// future evolution without breaking changes.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub(crate) struct EventStreamConfig {
    /// Options to pass to `pulldown-cmark` (e.g., enabling task lists,
    /// wikilinks, or frontmatter).
    options: Options,
    /// The policy for normalizing line breaks in the text stream.
    break_policy: BreakPolicy,
    /// Whether the stream should eagerly merge adjacent text events together.
    merge_text: bool,
}

impl EventStreamConfig {
    /// Creates a new configuration with explicit values.
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        options: Options,
        break_policy: BreakPolicy,
        merge_text: bool,
    ) -> Self {
        Self {
            options,
            break_policy,
            merge_text,
        }
    }

    /// Returns the pulldown-cmark options for this configuration.
    #[must_use]
    #[inline]
    pub(crate) const fn options(&self) -> Options {
        self.options
    }

    /// Returns the line break normalization policy.
    #[must_use]
    #[inline]
    pub(crate) const fn break_policy(&self) -> BreakPolicy {
        self.break_policy
    }

    /// Returns whether text merging is enabled.
    #[must_use]
    #[inline]
    pub(crate) const fn merge_text(&self) -> bool {
        self.merge_text
    }
}

impl Default for EventStreamConfig {
    /// Provides the default, opinionated configuration used by Lithos.
    ///
    /// By default, task lists, wikilinks, strikethrough, and YAML/Pluses
    /// metadata blocks are enabled. Line breaks are normalized to text, and
    /// text nodes are aggressively merged to improve metadata scanning
    /// performance.
    #[inline]
    fn default() -> Self {
        Self {
            options: Options::ENABLE_TASKLISTS
                | Options::ENABLE_WIKILINKS
                | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
                | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
                | Options::ENABLE_STRIKETHROUGH,
            break_policy: BreakPolicy::default(),
            merge_text: true,
        }
    }
}

/// Policy for handling soft and hard line breaks in the event stream.
///
/// In standard markdown, lines wrapped with a single newline are `SoftBreak`s,
/// while lines wrapped with two spaces or a trailing backslash are
/// `HardBreak`s. Depending on the pipeline stage, it may be beneficial to
/// preserve these events, or normalize them into `Text` events so that adjacent
/// text nodes can be seamlessly merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum BreakPolicy {
    /// Leave `SoftBreak` and `HardBreak` events exactly as they are.
    Preserve,
    /// Convert `SoftBreak` to `Text(" ")`, leave `HardBreak` alone.
    SoftAsSpace,
    /// Convert `HardBreak` to `Text("\n")`, leave `SoftBreak` alone.
    HardAsNewLine,
    /// Convert `SoftBreak` to `Text(" ")` and `HardBreak` to `Text("\n")`.
    NormalizeAsText,
}

impl BreakPolicy {
    /// Returns the replacement text for a `SoftBreak` if this policy normalizes
    /// them.
    #[must_use]
    #[inline]
    pub(crate) const fn soft_break_replacement(self) -> Option<&'static str> {
        match self {
            Self::SoftAsSpace | Self::NormalizeAsText => Some(" "),
            Self::Preserve | Self::HardAsNewLine => None,
        }
    }

    /// Returns the replacement text for a `HardBreak` if this policy normalizes
    /// them.
    #[must_use]
    #[inline]
    pub(crate) const fn hard_break_replacement(self) -> Option<&'static str> {
        match self {
            Self::HardAsNewLine | Self::NormalizeAsText => Some("\n"),
            Self::Preserve | Self::SoftAsSpace => None,
        }
    }
}

impl Default for BreakPolicy {
    /// Returns the default break policy for Lithos.
    ///
    /// Defaults to `NormalizeAsText`, which converts both soft breaks to spaces
    /// and hard breaks to newlines. This enables aggressive text merging for
    /// efficient metadata scanning while preserving semantic line breaks.
    #[inline]
    fn default() -> Self {
        Self::NormalizeAsText
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod event_stream_config {
        use super::*;

        #[test]
        fn default_enables_expected_markdown_extensions() {
            let config = EventStreamConfig::default();

            assert!(
                config.options().contains(Options::ENABLE_TASKLISTS),
                "default config should enable task lists"
            );
            assert!(
                config.options().contains(Options::ENABLE_WIKILINKS),
                "default config should enable wikilinks"
            );
            assert!(
                config
                    .options()
                    .contains(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS),
                "default config should enable YAML metadata blocks"
            );
            assert!(
                config
                    .options()
                    .contains(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS),
                "default config should enable plus-delimited metadata blocks"
            );
            assert!(
                config.options().contains(Options::ENABLE_STRIKETHROUGH),
                "default config should enable strikethrough"
            );
        }

        #[test]
        fn default_normalizes_breaks_and_merges_text() {
            let config = EventStreamConfig::default();

            assert_eq!(
                config.break_policy(),
                BreakPolicy::NormalizeAsText,
                "default config should normalize both soft and hard breaks"
            );
            assert!(
                config.merge_text(),
                "default config should merge adjacent text events"
            );
        }
    }

    mod break_policy {
        use super::*;

        #[test]
        fn default_is_normalize_as_text() {
            assert_eq!(
                BreakPolicy::default(),
                BreakPolicy::NormalizeAsText,
                "default break policy should normalize both soft and hard \
                 breaks"
            );
        }
    }

    mod break_policy_soft_break_replacement {
        use super::*;

        #[test]
        fn preserve_returns_none() {
            assert_eq!(
                BreakPolicy::Preserve.soft_break_replacement(),
                None,
                "preserve policy should not replace soft breaks"
            );
        }

        #[test]
        fn soft_as_space_returns_space() {
            assert_eq!(
                BreakPolicy::SoftAsSpace.soft_break_replacement(),
                Some(" "),
                "soft-as-space policy should map soft breaks to a single space"
            );
        }

        #[test]
        fn hard_as_newline_returns_none() {
            assert_eq!(
                BreakPolicy::HardAsNewLine.soft_break_replacement(),
                None,
                "hard-as-newline policy should not replace soft breaks"
            );
        }

        #[test]
        fn normalize_as_text_returns_space() {
            assert_eq!(
                BreakPolicy::NormalizeAsText.soft_break_replacement(),
                Some(" "),
                "normalize-as-text policy should map soft breaks to a space"
            );
        }
    }

    mod break_policy_hard_break_replacement {
        use super::*;

        #[test]
        fn preserve_returns_none() {
            assert_eq!(
                BreakPolicy::Preserve.hard_break_replacement(),
                None,
                "preserve policy should not replace hard breaks"
            );
        }

        #[test]
        fn soft_as_space_returns_none() {
            assert_eq!(
                BreakPolicy::SoftAsSpace.hard_break_replacement(),
                None,
                "soft-as-space policy should not replace hard breaks"
            );
        }

        #[test]
        fn hard_as_newline_returns_newline() {
            assert_eq!(
                BreakPolicy::HardAsNewLine.hard_break_replacement(),
                Some("\n"),
                "hard-as-newline policy should map hard breaks to newlines"
            );
        }

        #[test]
        fn normalize_as_text_returns_newline() {
            assert_eq!(
                BreakPolicy::NormalizeAsText.hard_break_replacement(),
                Some("\n"),
                "normalize-as-text policy should map hard breaks to newlines"
            );
        }
    }
}
