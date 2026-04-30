//! Configuration for markdown stream adaptation.
//!
//! This module provides explicit, data-driven policies for the markdown parsing
//! pipeline. By treating parser behavior (such as how to handle newlines or
//! which markdown extensions to enable) as a separate configuration object,
//! the parsing and extraction stages are kept decoupled from implicit branching
//! logic.
//! This module defines a compact policy surface for parser behavior:
//! - which pulldown-cmark extensions are enabled,
//! - how unknown events are handled,
//! - how line breaks and text merging are normalized.

#![expect(
    dead_code,
    reason = "Parser configuration surface is staged during migration"
)]

use pulldown_cmark::Options;

use crate::note::{
    error::{NoteIngestError, NoteParseError},
    position::SourceByteRange,
};

/// Event stream configuration.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EventStreamConfig {
    /// Declarative extension capability policy.
    extensions: ExtensionsPolicy,
    /// Unknown event handling policy.
    retention: EventRetentionPolicy,
    /// Line-break normalization policy.
    break_policy: BreakPolicy,
    /// Whether adjacent text events should be merged.
    merge_text: bool,
}

impl EventStreamConfig {
    /// Canonical default pulldown option set used by Lithos.
    #[must_use]
    #[inline]
    pub(crate) fn default_options() -> Options {
        ExtensionsPolicy::default().to_options()
    }

    /// Compatibility constructor from raw options.
    #[must_use]
    #[inline]
    pub(crate) fn new(
        options: Options,
        break_policy: BreakPolicy,
        merge_text: bool,
    ) -> Self {
        Self {
            extensions: ExtensionsPolicy::from_options(options),
            retention: EventRetentionPolicy::default(),
            break_policy,
            merge_text,
        }
    }

    /// Policy-first constructor.
    #[must_use]
    #[inline]
    pub(crate) const fn with_policy(
        extensions: ExtensionsPolicy,
        retention: EventRetentionPolicy,
        break_policy: BreakPolicy,
        merge_text: bool,
    ) -> Self {
        Self {
            extensions,
            retention,
            break_policy,
            merge_text,
        }
    }

    /// Resolved pulldown options.
    #[must_use]
    #[inline]
    pub(crate) const fn options(self) -> Options {
        self.extensions.to_options()
    }

    /// Extension policy.
    #[must_use]
    #[inline]
    pub(crate) const fn extensions(self) -> ExtensionsPolicy {
        self.extensions
    }

    /// Unknown event retention policy.
    #[must_use]
    #[inline]
    pub(crate) const fn retention(self) -> EventRetentionPolicy {
        self.retention
    }

    /// Line-break policy.
    #[must_use]
    #[inline]
    pub(crate) const fn break_policy(self) -> BreakPolicy {
        self.break_policy
    }

    /// Text merge toggle.
    #[must_use]
    #[inline]
    pub(crate) const fn merge_text(self) -> bool {
        self.merge_text
    }
}

impl Default for EventStreamConfig {
    #[inline]
    fn default() -> Self {
        Self {
            extensions: ExtensionsPolicy::default(),
            retention: EventRetentionPolicy::default(),
            break_policy: BreakPolicy::default(),
            merge_text: true,
        }
    }
}

/// Declarative extension policy for pulldown capabilities.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExtensionsPolicy {
    flags: ExtensionFlags,
    metadata: MetadataPolicy,
}

impl ExtensionsPolicy {
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        flags: ExtensionFlags,
        metadata: MetadataPolicy,
    ) -> Self {
        Self {
            flags,
            metadata,
        }
    }

    /// Converts extension policy to pulldown options.
    #[must_use]
    pub(crate) const fn to_options(self) -> Options {
        let mut options = Options::empty();

        if self.flags.has(ExtensionFlags::TASK_LISTS) {
            options = options.union(Options::ENABLE_TASKLISTS);
        }
        if self.flags.has(ExtensionFlags::WIKILINKS) {
            options = options.union(Options::ENABLE_WIKILINKS);
        }
        if self.flags.has(ExtensionFlags::MATH) {
            options = options.union(Options::ENABLE_MATH);
        }
        if self.flags.has(ExtensionFlags::STRIKETHROUGH) {
            options = options.union(Options::ENABLE_STRIKETHROUGH);
        }
        if self.flags.has(ExtensionFlags::TABLES) {
            options = options.union(Options::ENABLE_TABLES);
        }
        if self.flags.has(ExtensionFlags::DEFINITION_LISTS) {
            options = options.union(Options::ENABLE_DEFINITION_LIST);
        }
        if self.flags.has(ExtensionFlags::FOOTNOTES) {
            options = options.union(Options::ENABLE_FOOTNOTES);
        }

        match self.metadata {
            MetadataPolicy::None => {}
            MetadataPolicy::Yaml => {
                options =
                    options.union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
            }
            MetadataPolicy::Toml => {
                options = options
                    .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
            }
            MetadataPolicy::YamlAndToml => {
                options = options
                    .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
                    .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
            }
        }

        options
    }

    /// Converts raw pulldown options into extension policy.
    #[must_use]
    pub(crate) const fn from_options(options: Options) -> Self {
        let mut flags = ExtensionFlags::EMPTY;

        if options.contains(Options::ENABLE_TASKLISTS) {
            flags = flags.union(ExtensionFlags::TASK_LISTS);
        }
        if options.contains(Options::ENABLE_WIKILINKS) {
            flags = flags.union(ExtensionFlags::WIKILINKS);
        }
        if options.contains(Options::ENABLE_MATH) {
            flags = flags.union(ExtensionFlags::MATH);
        }
        if options.contains(Options::ENABLE_STRIKETHROUGH) {
            flags = flags.union(ExtensionFlags::STRIKETHROUGH);
        }
        if options.contains(Options::ENABLE_TABLES) {
            flags = flags.union(ExtensionFlags::TABLES);
        }
        if options.contains(Options::ENABLE_DEFINITION_LIST) {
            flags = flags.union(ExtensionFlags::DEFINITION_LISTS);
        }
        if options.contains(Options::ENABLE_FOOTNOTES) {
            flags = flags.union(ExtensionFlags::FOOTNOTES);
        }

        let has_yaml =
            options.contains(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        let has_toml =
            options.contains(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

        let metadata = match (has_yaml, has_toml) {
            (false, false) => MetadataPolicy::None,
            (true, false) => MetadataPolicy::Yaml,
            (false, true) => MetadataPolicy::Toml,
            (true, true) => MetadataPolicy::YamlAndToml,
        };

        Self {
            flags,
            metadata,
        }
    }

    /// Whether a specific extension flag is enabled.
    #[must_use]
    #[inline]
    pub(crate) const fn has(self, flag: ExtensionFlags) -> bool {
        self.flags.has(flag)
    }
}

impl Default for ExtensionsPolicy {
    fn default() -> Self {
        Self {
            flags: ExtensionFlags::TASK_LISTS
                .union(ExtensionFlags::WIKILINKS)
                .union(ExtensionFlags::MATH)
                .union(ExtensionFlags::STRIKETHROUGH),
            metadata: MetadataPolicy::default(),
        }
    }
}

/// Compact extension bitmask.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExtensionFlags(u16);

impl ExtensionFlags {
    pub(crate) const DEFINITION_LISTS: Self = Self(1 << 5);
    pub(crate) const EMPTY: Self = Self(0);
    pub(crate) const FOOTNOTES: Self = Self(1 << 6);
    pub(crate) const MATH: Self = Self(1 << 2);
    pub(crate) const STRIKETHROUGH: Self = Self(1 << 3);
    pub(crate) const TABLES: Self = Self(1 << 4);
    pub(crate) const TASK_LISTS: Self = Self(1 << 0);
    pub(crate) const WIKILINKS: Self = Self(1 << 1);

    #[must_use]
    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub(crate) const fn has(self, bit: Self) -> bool {
        (self.0 & bit.0) != 0
    }
}

/// Metadata extension policy.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum MetadataPolicy {
    None,
    Yaml,
    Toml,
    #[default]
    YamlAndToml,
}

/// Unknown event handling policy.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) struct EventRetentionPolicy {
    unknown_block: UnknownEventPolicy,
    unknown_inline: UnknownEventPolicy,
}

impl EventRetentionPolicy {
    #[must_use]
    #[inline]
    pub(crate) const fn new(
        unknown_block: UnknownEventPolicy,
        unknown_inline: UnknownEventPolicy,
    ) -> Self {
        Self {
            unknown_block,
            unknown_inline,
        }
    }

    /// Enforces policy for an unknown block-level parser event.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] when unknown block events are configured to
    /// be rejected.
    #[inline]
    pub(crate) fn enforce_unknown_block(
        self,
        observed: &'static str,
        range: Option<SourceByteRange>,
    ) -> Result<(), NoteIngestError> {
        self.unknown_block.enforce(
            "unknown_block",
            "known block parser event",
            observed,
            range,
        )
    }

    /// Enforces policy for an unknown inline parser event.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] when unknown inline events are configured to
    /// be rejected.
    #[inline]
    pub(crate) fn enforce_unknown_inline(
        self,
        observed: &'static str,
        range: Option<SourceByteRange>,
    ) -> Result<(), NoteIngestError> {
        self.unknown_inline.enforce(
            "unknown_inline",
            "known inline parser event",
            observed,
            range,
        )
    }

    #[must_use]
    #[inline]
    pub(crate) const fn reject_unknown_block(self) -> bool {
        matches!(self.unknown_block, UnknownEventPolicy::Reject)
    }

    #[must_use]
    #[inline]
    pub(crate) const fn reject_unknown_inline(self) -> bool {
        matches!(self.unknown_inline, UnknownEventPolicy::Reject)
    }
}

/// Unknown event fallback behavior.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub(crate) enum UnknownEventPolicy {
    Reject,
    #[default]
    Drop,
}

impl UnknownEventPolicy {
    /// Applies unknown-event policy and returns an explicit decision.
    ///
    /// This method centralizes policy semantics so callers do not need to
    /// duplicate `match` logic to interpret policy values.
    ///
    /// # Errors
    ///
    /// Returns [`NoteIngestError`] when policy requires rejecting unknown
    /// events.
    #[inline]
    pub(crate) fn enforce(
        self,
        policy_name: &'static str,
        expected: &'static str,
        observed: &'static str,
        range: Option<SourceByteRange>,
    ) -> Result<(), NoteIngestError> {
        match self {
            Self::Reject => Err(NoteIngestError::Domain(
                NoteParseError::PolicyViolation {
                    policy: policy_name,
                    expected,
                    observed,
                    range,
                }
                .into(),
            )),
            Self::Drop => Ok(()),
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
    /// Defaults to `NormalizeAsText`, which converts soft breaks to spaces and
    /// hard breaks to newlines for downstream text processing.
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

            assert!(config.options().contains(Options::ENABLE_TASKLISTS));
            assert!(config.options().contains(Options::ENABLE_WIKILINKS));
            assert!(config.options().contains(Options::ENABLE_MATH));
            assert!(
                config
                    .options()
                    .contains(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
            );
            assert!(
                config
                    .options()
                    .contains(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
            );
            assert!(config.options().contains(Options::ENABLE_STRIKETHROUGH));
        }

        #[test]
        fn default_normalizes_breaks_and_merges_text() {
            let config = EventStreamConfig::default();
            assert_eq!(config.break_policy(), BreakPolicy::NormalizeAsText);
            assert!(config.merge_text());
        }
    }

    mod extensions_policy {
        use super::*;

        #[test]
        fn from_options_round_trips_known_flags() {
            let options = Options::ENABLE_TASKLISTS
                .union(Options::ENABLE_MATH)
                .union(Options::ENABLE_STRIKETHROUGH)
                .union(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

            let policy = ExtensionsPolicy::from_options(options);
            let resolved = policy.to_options();

            assert!(resolved.contains(Options::ENABLE_TASKLISTS));
            assert!(resolved.contains(Options::ENABLE_MATH));
            assert!(resolved.contains(Options::ENABLE_STRIKETHROUGH));
            assert!(
                resolved.contains(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
            );
            assert!(
                !resolved
                    .contains(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
            );
        }

        #[test]
        fn from_options_round_trips_toml_only_metadata() {
            let options = Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS;
            let policy = ExtensionsPolicy::from_options(options);
            let resolved = policy.to_options();

            assert_eq!(policy.metadata, MetadataPolicy::Toml);
            assert!(
                resolved
                    .contains(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
            );
            assert!(
                !resolved.contains(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS)
            );
        }
    }

    mod unknown_event_policy {
        use super::*;

        #[test]
        fn reject_policy_returns_policy_violation() {
            let range = SourceByteRange::try_from(3..8).expect("valid range");
            let result = UnknownEventPolicy::Reject.enforce(
                "unknown_inline",
                "known inline parser event",
                "inline_math",
                Some(range),
            );

            assert!(
                matches!(&result, Err(NoteIngestError::Domain(_))),
                "reject policy must return domain error"
            );

            let Err(NoteIngestError::Domain(error)) = result else {
                return;
            };
            assert!(matches!(error, crate::note::error::NoteError::Parse(_)));
        }

        #[test]
        fn drop_policy_allows_unknown_event() {
            let result = UnknownEventPolicy::Drop.enforce(
                "unknown_inline",
                "known inline parser event",
                "inline_math",
                None,
            );
            result.unwrap();
        }
    }

    mod break_policy {
        use super::*;

        #[test]
        fn replacements_match_policy() {
            assert_eq!(BreakPolicy::Preserve.soft_break_replacement(), None);
            assert_eq!(
                BreakPolicy::SoftAsSpace.soft_break_replacement(),
                Some(" ")
            );
            assert_eq!(
                BreakPolicy::HardAsNewLine.soft_break_replacement(),
                None
            );
            assert_eq!(
                BreakPolicy::NormalizeAsText.soft_break_replacement(),
                Some(" ")
            );

            assert_eq!(BreakPolicy::Preserve.hard_break_replacement(), None);
            assert_eq!(BreakPolicy::SoftAsSpace.hard_break_replacement(), None);
            assert_eq!(
                BreakPolicy::HardAsNewLine.hard_break_replacement(),
                Some("\n")
            );
            assert_eq!(
                BreakPolicy::NormalizeAsText.hard_break_replacement(),
                Some("\n")
            );
        }
    }
}
