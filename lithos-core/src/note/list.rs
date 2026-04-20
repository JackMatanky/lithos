//! List value objects and structural management for notes.
//!
//! This module defines the core types for representing hierarchical lists in
//! markdown notes. It follows a unified structural model where every bullet
//! point is a [`ListItem`], regardless of whether it is a plain text bullet, an
//! ordered list item, or a checkbox.
//!
//! # Lists and Tasks
//!
//! In the Lithos domain, there is a clear distinction between **Structural**
//! list items and **Semantic** tasks:
//!
//! 1. **Structural ([`ListItem`]):** Manages the physical presence of a bullet
//!    in the source file, including its nesting [`ListDepth`], its [`ListKind`]
//!    (ordered vs unordered), and raw metadata like tags and inline fields.
//! 2. **Semantic ([`crate::note::task::Task`]):** A "promoted" entity that
//!    represents a checkbox item with validated domain logic (e.g., due dates,
//!    priorities).

#![allow(
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived variants for enums even if \
              the base type is non_exhaustive"
)]

use std::fmt;

use uuid::Uuid;

use super::{
    error::{ListError, NoteError},
    inline_fields::InlineField,
    position::{SourceByteOffset, SourceByteRange},
    tag::Tag,
};
use crate::{
    config::task::TaskConfigSpec,
    note::{
        inline_fields::InlineFieldKey,
        raw::{RawInlineFieldToken, RawListDepth, RawListItem, RawListKind},
    },
};

/// Unique identifier for a list item (UUID v7).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq, Hash))]
pub struct ListItemId(Uuid);

impl ListItemId {
    /// Creates a new random `ListItemId` (UUID v7).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ListItemId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ListItemId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core structural metadata for any list-based item.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ListItemBase {
    /// Unique identifier for the list item.
    pub id: ListItemId,
    /// Source byte range of the full list item.
    pub range: SourceByteRange,
    /// Raw text content of the list item.
    pub text: Box<str>,
    /// Metadata tags attached to the item.
    pub tags: Box<[Tag]>,
    /// Nesting depth of the item.
    pub depth: ListDepth,
    /// Type of list (ordered or unordered).
    pub kind: ListKind,
    /// Source position of the parent item, if nested.
    pub parent: Option<SourceByteOffset>,
    /// Whether the parser identified this as a checkbox (GFM).
    pub is_checkbox: Option<bool>,
}

impl ListItemBase {
    /// Create a new list item base.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "Base metadata requires all components"
    )]
    pub fn new(
        id: ListItemId,
        range: SourceByteRange,
        text: Box<str>,
        tags: Box<[Tag]>,
        depth: ListDepth,
        kind: ListKind,
        parent: Option<SourceByteOffset>,
        is_checkbox: Option<bool>,
    ) -> Self {
        Self {
            id,
            range,
            text,
            tags,
            depth,
            kind,
            parent,
            is_checkbox,
        }
    }
}

/// A hierarchical collection of list items.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct List {
    /// Type of list.
    kind: ListKind,
    /// Collection of items in the list.
    items: Vec<ListItem>,
    /// Nesting depth of the list.
    depth: ListDepth,
}

impl List {
    /// Creates a new empty list with depth 0.
    #[inline]
    #[must_use]
    pub fn new(list_kind: ListKind) -> Self {
        Self {
            kind: list_kind,
            items: Vec::new(),
            depth: ListDepth::root(),
        }
    }

    /// Creates a new empty list with an explicit depth.
    #[inline]
    #[must_use]
    pub fn with_depth(list_kind: ListKind, depth: ListDepth) -> Self {
        Self {
            kind: list_kind,
            items: Vec::new(),
            depth,
        }
    }

    /// Creates a new empty list with an explicit depth and capacity hint.
    #[inline]
    #[must_use]
    pub fn with_capacity(
        list_kind: ListKind,
        depth: ListDepth,
        capacity: usize,
    ) -> Self {
        Self {
            kind: list_kind,
            items: Vec::with_capacity(capacity),
            depth,
        }
    }

    /// Appends a list item, preserving source order.
    #[inline]
    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    /// Returns the list type (Ordered or Unordered).
    #[inline]
    #[must_use]
    pub const fn list_kind(&self) -> ListKind {
        self.kind
    }

    /// Returns an iterator over the list items in source order.
    #[inline]
    #[must_use]
    pub fn items(&self) -> ListItems<'_> {
        ListItems {
            inner: self.items.iter(),
        }
    }

    /// Returns the list nesting depth (0 = top level).
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> ListDepth {
        self.depth
    }

    /// Sets the list nesting depth.
    #[inline]
    pub fn set_depth(&mut self, depth: ListDepth) {
        self.depth = depth;
    }
}

/// The type of list extracted from markdown.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum ListKind {
    /// Ordered list starting at the given number (e.g., `1.`).
    Ordered(u64),
    /// Unordered list (bullets like `-`, `*`, or `+`).
    Unordered,
}

/// The nesting level of a list or item within a document.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct ListDepth(u8);

impl ListDepth {
    /// Returns the root list depth (0).
    #[inline]
    #[must_use]
    pub const fn root() -> Self {
        Self(0)
    }

    /// Creates a validated list depth from a numeric value.
    ///
    /// # Errors
    ///
    /// Returns [`ListError::MaxNestingExceeded`] if the depth is out of range
    /// (currently max 255).
    #[inline]
    pub fn try_new(depth: usize) -> Result<Self, ListError> {
        u8::try_from(depth).map(Self).map_err(|_err| {
            ListError::MaxNestingExceeded {
                current: depth,
                limit: usize::from(u8::MAX),
            }
        })
    }

    /// Returns the raw depth value as a `u8`.
    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl fmt::Display for ListDepth {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A borrowed iterator over [`ListItem`]s.
pub struct ListItems<'list> {
    inner: std::slice::Iter<'list, ListItem>,
}

#[expect(
    clippy::missing_trait_methods,
    reason = "Iterator wrapper forwards core methods only."
)]
impl<'list> Iterator for ListItems<'list> {
    type Item = &'list ListItem;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// A structural bullet point or numbered item.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ListItem {
    /// Structural core metadata.
    pub base: ListItemBase,
    /// Typed metadata fields.
    pub fields: Box<[InlineField]>,
}

impl ListItem {
    /// Create a new structural list item.
    #[inline]
    #[must_use]
    pub fn new(base: ListItemBase, fields: Box<[InlineField]>) -> Self {
        Self {
            base,
            fields,
        }
    }

    /// Converts a [`RawListItem`] into a validated domain [`ListItem`].
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if list depth validation fails or tag creation
    /// fails.
    #[inline]
    pub fn from_raw(
        raw: &RawListItem<'_>,
        spec: &TaskConfigSpec,
    ) -> Result<Self, NoteError> {
        let depth = match raw.depth {
            RawListDepth::Root => ListDepth::root(),
            RawListDepth::Nested(value) => {
                ListDepth::try_new(usize::from(value))?
            }
        };
        let kind = match raw.kind {
            RawListKind::Ordered(start) => ListKind::Ordered(start),
            RawListKind::Unordered => ListKind::Unordered,
        };

        let mut tags = Vec::with_capacity(raw.tags.len());
        for raw_tag in &raw.tags {
            if let Ok(tag) =
                Tag::try_new_with_range(raw_tag.value.as_ref(), raw_tag.range)
            {
                tags.push(tag);
            }
        }

        // Type inference for fields
        let mut fields = Vec::with_capacity(raw.inline_fields.len());
        for token in &raw.inline_fields {
            let field = InlineField::from_token(token, spec);
            fields.push(field);
        }

        let base = ListItemBase::new(
            ListItemId::new(),
            raw.range,
            raw.text.text.as_ref().into(),
            tags.into_boxed_slice(),
            depth,
            kind,
            raw.parent,
            raw.is_checkbox,
        );

        Ok(Self::new(base, fields.into_boxed_slice()))
    }

    /// Returns the raw text content of this list item.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.base.text
    }

    /// Returns the source byte range of this list item.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.base.range
    }

    /// Returns the collection of metadata tags extracted from the list item.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.base.tags
    }

    /// Returns the collection of inline metadata fields extracted from the
    /// list item.
    #[inline]
    #[must_use]
    pub fn fields(&self) -> &[InlineField] {
        &self.fields
    }

    /// Returns the start source byte position of this list item.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.base.range.start()
    }

    /// Returns the list kind for this item.
    #[inline]
    #[must_use]
    pub const fn list_kind(&self) -> ListKind {
        self.base.kind
    }

    /// Returns the list depth for this item.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> ListDepth {
        self.base.depth
    }

    /// Returns the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.base.parent
    }
}

impl InlineField {
    /// Create a domain field from a raw token using task configuration.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics are preferred for value references"
    )]
    pub fn from_token(
        token: &RawInlineFieldToken<'_>,
        spec: &TaskConfigSpec,
    ) -> Self {
        use crate::note::raw::value::RawFieldValue;

        let key = InlineFieldKey::new(token.key.as_ref());
        let date_spec = spec
            .temporal_specs
            .get(key.as_kebab())
            .map(|entry| entry.1.as_ref());

        let typed_value = match &token.value {
            std::borrow::Cow::Borrowed(text) => {
                RawFieldValue::from_str_with_spec(
                    text,
                    token.key.as_ref(),
                    date_spec,
                )
            }
            std::borrow::Cow::Owned(text) => RawFieldValue::from_str_with_spec(
                text,
                token.key.as_ref(),
                date_spec,
            )
            .into_owned(),
        };

        Self::new(key, typed_value.into(), token.range)
    }
}
