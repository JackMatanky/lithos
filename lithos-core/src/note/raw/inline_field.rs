use std::borrow::Cow;

use super::field_value::RawFieldValue;
use crate::{config::task::TaskConfigSpec, note::position::SourceByteRange};

/// Raw inline field extracted from markdown.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct RawInlineField<'source> {
    pub key: Cow<'source, str>,
    pub value: RawFieldValue<'source>,
    pub range: SourceByteRange,
}

impl<'source> RawInlineField<'source> {
    /// Create a raw inline field entry.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: RawFieldValue<'source>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
        }
    }

    /// Map emoji key to keyword if the key is a recognized emoji in the task
    /// spec.
    ///
    /// Returns the mapped keyword if the key is a single emoji character
    /// that matches a temporal slot emoji in the spec, otherwise returns None.
    pub fn map_emoji_key(
        key: &str,
        task_spec: &TaskConfigSpec,
    ) -> Option<Box<str>> {
        // Check if key is single emoji character
        let mut chars = key.chars();
        let first = chars.next()?;
        if chars.next().is_some() {
            return None; // More than one char, not a single emoji
        }

        // Look up emoji in task spec temporal mappings
        for (keyword, (_slot, _date_spec, emoji_opt)) in
            &task_spec.temporal_specs
        {
            if let Some(emoji) = emoji_opt
                && *emoji == first
            {
                return Some(keyword.clone());
            }
        }

        None
    }
}

impl RawInlineField<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineField<'static> {
        RawInlineField {
            key: Cow::Owned(self.key.into_owned()),
            value: self.value.into_owned(),
            range: self.range,
        }
    }
}
