//! Raw inline field types and token-to-field conversion.
//!
//! This module defines two representations for inline fields found in markdown:
//!
//! - [`RawInlineFieldToken`]: a zero-copy token holding the raw key and value
//!   strings exactly as extracted by the scanner.
//! - [`RawInlineField`]: a typed field where the value has been interpreted as
//!   a [`RawFieldValue`] (date, number, boolean, string, or list).
//!
//! The two conversion paths from token to typed field are:
//!
//! - [`From<RawInlineFieldToken>`] for [`RawInlineField`]: generic type
//!   inference with no task-configuration context.
//! - [`field_token_to_raw`]: task-aware conversion that looks up a
//!   date/temporal spec by normalized key before choosing the [`RawFieldValue`]
//!   variant.

use std::borrow::Cow;

use super::value::RawFieldValue;
use crate::{
    config::task::TaskConfigSpec,
    note::{inline_fields::InlineFieldKey, position::SourceByteRange},
};

/// Raw inline field token extracted from markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RawInlineFieldToken<'source> {
    pub key: Cow<'source, str>,
    pub value: Cow<'source, str>,
    pub range: SourceByteRange,
}

impl<'source> RawInlineFieldToken<'source> {
    /// Create a raw inline field token.
    #[inline]
    #[must_use]
    pub const fn new(
        key: Cow<'source, str>,
        value: Cow<'source, str>,
        range: SourceByteRange,
    ) -> Self {
        Self {
            key,
            value,
            range,
        }
    }
}

/// Converts a [`RawInlineFieldToken`] into a [`RawInlineField`] using generic
/// type inference.
///
/// Applies [`RawFieldValue::from_str_with_spec`] with no task-configuration
/// context. For task-aware conversion with date/temporal spec lookup, use
/// [`field_token_to_raw`] instead.
impl<'source> From<RawInlineFieldToken<'source>> for RawInlineField<'source> {
    #[inline]
    fn from(token: RawInlineFieldToken<'source>) -> Self {
        let RawInlineFieldToken {
            key,
            value,
            range,
        } = token;
        let typed_value = match value {
            Cow::Borrowed(text) => {
                RawFieldValue::from_str_with_spec(text, key.as_ref(), None)
            }
            Cow::Owned(text) => {
                RawFieldValue::from_str_with_spec(&text, key.as_ref(), None)
                    .into_owned()
            }
        };
        Self::new(key, typed_value, range)
    }
}

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
}

impl RawInlineField<'_> {
    /// Converts this raw inline field into an owned variant.
    ///
    /// Clones all borrowed string data into owned allocations, producing a
    /// `'static` lifetime that can outlive the source text.
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

impl RawInlineFieldToken<'_> {
    /// Converts this raw inline field token into an owned variant.
    ///
    /// Clones all borrowed string data into owned allocations, producing a
    /// `'static` lifetime that can outlive the source text.
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> RawInlineFieldToken<'static> {
        RawInlineFieldToken {
            key: Cow::Owned(self.key.into_owned()),
            value: Cow::Owned(self.value.into_owned()),
            range: self.range,
        }
    }
}

/// Converts a raw inline field token to a typed [`RawInlineField`], applying
/// any date/temporal spec from the task configuration if the key matches.
pub(crate) fn field_token_to_raw<'source>(
    token: RawInlineFieldToken<'source>,
    task_spec: &TaskConfigSpec,
) -> RawInlineField<'source> {
    let RawInlineFieldToken {
        key,
        value,
        range,
    } = token;
    let normalized = InlineFieldKey::normalize(key.as_ref());
    let date_spec = task_spec
        .temporal_specs
        .get(normalized.as_ref())
        .map(|entry| entry.1.as_ref());
    let typed_value = match value {
        Cow::Borrowed(text) => {
            RawFieldValue::from_str_with_spec(text, key.as_ref(), date_spec)
        }
        Cow::Owned(text) => {
            RawFieldValue::from_str_with_spec(&text, key.as_ref(), date_spec)
                .into_owned()
        }
    };
    RawInlineField::new(key, typed_value, range)
}
