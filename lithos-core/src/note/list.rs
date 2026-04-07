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
//!
//! The [`TaskExt`] trait provides a bridge between these two worlds, allowing
//! structural items to be queried for task-specific properties without
//! necessarily being promoted to a full `Task` entity.

#![expect(
    clippy::exhaustive_enums,
    reason = "rkyv generates exhaustive archived variants for enums even if \
              the base type is non_exhaustive"
)]

use std::fmt;

use super::{
    error::{ListError, NoteError},
    inline_fields::InlineField,
    position::{SourceByteOffset, SourceByteRange},
    tag::Tag,
    task::TaskRef,
};
use crate::{
    config::task::StatusSymbol,
    note::raw::{RawListDepth, RawListItem, RawListKind},
};

/// A hierarchical collection of list items.
///
/// `List` serves as a container for [`ListItem`]s that share a common
/// [`ListKind`] and are located at the same logical [`ListDepth`]. It tracks
/// the source positions of its items to maintain document order.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{List, ListItem, ListKind, ListDepth};
/// # use lithos_core::note::position::{SourceByteOffset, SourceByteRange};
/// let mut list = List::new(ListKind::Unordered);
/// let range = SourceByteRange::new(SourceByteOffset::new(0), SourceByteOffset::new(10)).unwrap();
///
/// let item = ListItem::new(
///     "Item 1".into(),
///     range,
///     range,
///     ListKind::Unordered,
///     ListDepth::root(),
///     None,
///     None,
///     Box::new([]),
///     Box::new([]),
/// );
///
/// list.add_item(item);
/// assert_eq!(list.list_kind(), ListKind::Unordered);
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct List {
    kind: ListKind,
    items: Vec<ListItem>,
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

/// A single structural unit within a markdown list.
///
/// `ListItem` represents every type of bullet point found in a note. It carries
/// its own metadata (tags and inline fields) and maintains a link to its parent
/// item to preserve hierarchy.
///
/// # Checkboxes and Tasks
///
/// If a list item has a checkbox (e.g., `- [ ]`), it will have a
/// [`StatusSymbol`] assigned to its `status` field. Such items can be
/// "promoted" to semantic [`crate::note::task::Task`] entities, at which point
/// a [`TaskRef`] is attached.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{ListItem, ListKind, ListDepth};
/// # use lithos_core::note::position::{SourceByteOffset, SourceByteRange};
/// # use lithos_core::config::task::StatusSymbol;
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(13);
/// let range = SourceByteRange::new(start, end).expect("valid range");
///
/// let item = ListItem::new(
///     "Buy groceries".into(),
///     range,
///     range,
///     ListKind::Unordered,
///     ListDepth::root(),
///     None,
///     Some(StatusSymbol::try_new(' ').unwrap()),
///     Box::new([]),
///     Box::new([]),
/// );
///
/// assert_eq!(item.text(), "Buy groceries");
/// assert!(item.task_status().is_some());
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ListItem {
    /// Raw text content.
    text: Box<str>,
    /// Source byte range in the note.
    range: SourceByteRange,
    /// Source byte range for the text content.
    text_range: SourceByteRange,
    /// List kind (ordered or unordered).
    kind: ListKind,
    /// List nesting depth.
    depth: ListDepth,
    /// Parent list item position, if any.
    parent: Option<SourceByteOffset>,
    /// Checkbox status symbol, if this is a checkbox item.
    status: Option<StatusSymbol>,
    /// Task reference if promoted to a Task.
    task_ref: Option<TaskRef>,
    /// Metadata tags attached to this item.
    tags: Box<[Tag]>,
    /// Inline metadata fields attached to this item.
    fields: Box<[InlineField]>,
}

impl ListItem {
    /// Create a new list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "List items capture full structural context"
    )]
    pub fn new(
        text: Box<str>,
        range: SourceByteRange,
        text_range: SourceByteRange,
        kind: ListKind,
        depth: ListDepth,
        parent: Option<SourceByteOffset>,
        status: Option<StatusSymbol>,
        tags: Box<[Tag]>,
        fields: Box<[InlineField]>,
    ) -> Self {
        Self {
            text,
            range,
            text_range,
            kind,
            depth,
            parent,
            status,
            task_ref: None,
            tags,
            fields,
        }
    }

    /// Returns the source byte range of this list item.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the source byte range for the text content.
    #[inline]
    #[must_use]
    pub const fn text_range(&self) -> SourceByteRange {
        self.text_range
    }

    /// Returns the start source byte position of this list item.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.range.start()
    }

    /// Returns the list kind for this item.
    #[inline]
    #[must_use]
    pub const fn list_kind(&self) -> ListKind {
        self.kind
    }

    /// Returns the list depth for this item.
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> ListDepth {
        self.depth
    }

    /// Returns the parent list item position, if any.
    #[inline]
    #[must_use]
    pub const fn parent(&self) -> Option<SourceByteOffset> {
        self.parent
    }

    /// Returns the text content of this list item.
    #[inline]
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    /// Returns the checkbox status symbol if this is a checkbox item.
    #[inline]
    #[must_use]
    pub const fn task_status(&self) -> Option<StatusSymbol> {
        self.status
    }

    /// Returns the collection of metadata tags extracted from the list item.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Returns the collection of inline metadata fields extracted from the
    /// list item.
    #[inline]
    #[must_use]
    pub fn fields(&self) -> &[InlineField] {
        &self.fields
    }

    /// Returns the task reference if this checkbox was promoted.
    #[inline]
    #[must_use]
    pub const fn promoted_task_ref(&self) -> Option<TaskRef> {
        self.task_ref
    }

    /// Sets the task reference for a promoted checkbox item.
    #[inline]
    pub fn set_task_ref(&mut self, task_ref: TaskRef) {
        self.task_ref = Some(task_ref);
    }

    /// Clears the task reference for a checkbox item.
    #[inline]
    pub fn clear_task_ref(&mut self) {
        self.task_ref = None;
    }
}

impl TryFrom<&RawListItem<'_>> for ListItem {
    type Error = NoteError;

    /// Attempts to convert a [`RawListItem`] into a validated domain
    /// [`ListItem`].
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if:
    /// - The raw depth exceeds the maximum representable depth.
    /// - A status symbol character is invalid (non-ASCII or non-printable).
    /// - A tag found on the item is malformed.
    #[inline]
    fn try_from(raw: &RawListItem<'_>) -> Result<Self, Self::Error> {
        let depth = match raw.depth {
            RawListDepth::Root => ListDepth::root(),
            RawListDepth::Nested(value) => {
                ListDepth::try_new(usize::from(value))?
            }
        };
        let kind = match raw.list_kind {
            RawListKind::Ordered(start) => ListKind::Ordered(start),
            RawListKind::Unordered => ListKind::Unordered,
        };
        let status = raw
            .task_marker
            .map(|symbol| StatusSymbol::try_new(symbol.marker.marker()))
            .transpose()?;

        let mut tags = Vec::with_capacity(raw.tags.len());
        for raw_tag in &raw.tags {
            if let Ok(tag) =
                Tag::try_new_with_range(raw_tag.value.as_ref(), raw_tag.range)
            {
                tags.push(tag);
            }
        }

        let fields = raw
            .inline_fields
            .iter()
            .map(InlineField::from_raw)
            .collect::<Vec<_>>();

        Ok(Self::new(
            raw.text.as_ref().into(),
            raw.range,
            raw.text_range,
            kind,
            depth,
            raw.parent,
            status,
            tags.into_boxed_slice(),
            fields.into_boxed_slice(),
        ))
    }
}

/// Extension trait for list items providing task-specific capabilities.
///
/// This trait allows any [`ListItem`] to be queried for task metadata without
/// requiring it to be a promoted [`crate::note::task::Task`] entity.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{ListItem, TaskExt, ListKind, ListDepth};
/// # use lithos_core::note::position::{SourceByteOffset, SourceByteRange};
/// # use lithos_core::config::task::StatusSymbol;
/// # let start = SourceByteOffset::new(0);
/// # let end = SourceByteOffset::new(13);
/// # let range = SourceByteRange::new(start, end).expect("valid range");
/// let item = ListItem::new(
///     "Test".into(),
///     range,
///     range,
///     ListKind::Unordered,
///     ListDepth::root(),
///     None,
///     Some(StatusSymbol::try_new('x').unwrap()),
///     Box::new([]),
///     Box::new([]),
/// );
///
/// if item.is_checkbox() {
///     assert_eq!(item.status().unwrap().value(), 'x');
/// }
/// ```
pub trait TaskExt {
    /// Returns true if this list item has a checkbox.
    fn is_checkbox(&self) -> bool;

    /// Returns the checkbox status symbol, if any.
    fn status(&self) -> Option<StatusSymbol>;

    /// Returns the task reference if this item was promoted.
    fn task_ref(&self) -> Option<TaskRef>;
}

impl TaskExt for ListItem {
    #[inline]
    fn is_checkbox(&self) -> bool {
        self.status.is_some()
    }

    #[inline]
    fn status(&self) -> Option<StatusSymbol> {
        self.task_status()
    }

    #[inline]
    fn task_ref(&self) -> Option<TaskRef> {
        self.promoted_task_ref()
    }
}
