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
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "Internal configuration fields are accessible within the crate"
)]
pub(crate) struct EventStreamConfig {
    /// Options to pass to `pulldown-cmark` (e.g., enabling task lists,
    /// wikilinks, or frontmatter).
    pub(crate) options: Options,
    /// The policy for normalizing line breaks in the text stream.
    pub(crate) break_policy: BreakPolicy,
    /// Whether the stream should eagerly merge adjacent text events together.
    pub(crate) merge_text: bool,
}

impl EventStreamConfig {
    /// Creates a new configuration with explicit values.
    #[must_use]
    #[inline]
    #[expect(dead_code, reason = "New adapter layer not yet integrated")]
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
            break_policy: BreakPolicy::NormalizeAsText,
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
#[expect(dead_code, reason = "New adapter layer not yet integrated")]
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
