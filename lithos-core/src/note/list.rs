//! List value objects for notes.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::fmt;

use super::{
    error::{ListError, NoteError},
    position::{SourceByteOffset, SourceByteRange},
    task::TaskId,
};
use crate::{
    config::task::StatusSymbol,
    note::raw::{RawListDepth, RawListItem},
};

/// Markdown list structure.
///
/// Represents a collection of [`ListItem`]s, which can be ordered or unordered.
/// A `List` tracks its nesting depth within the document.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{List, ListDepth, ListType};
/// let list = List::new(ListType::Unordered);
/// assert_eq!(list.depth(), ListDepth::root());
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct List {
    kind: ListType,
    items: Vec<ListItem>,
    depth: ListDepth,
}

impl List {
    /// Creates a new empty list with depth 0.
    #[inline]
    #[must_use]
    pub fn new(list_type: ListType) -> Self {
        Self {
            kind: list_type,
            items: Vec::new(),
            depth: ListDepth::root(),
        }
    }

    /// Creates a new empty list with an explicit depth.
    #[inline]
    #[must_use]
    pub fn with_depth(list_type: ListType, depth: ListDepth) -> Self {
        Self {
            kind: list_type,
            items: Vec::new(),
            depth,
        }
    }

    /// Creates a new empty list with an explicit depth and capacity hint.
    #[inline]
    #[must_use]
    pub fn with_capacity(
        list_type: ListType,
        depth: ListDepth,
        capacity: usize,
    ) -> Self {
        Self {
            kind: list_type,
            items: Vec::with_capacity(capacity),
            depth,
        }
    }

    /// Appends a list item, preserving source order.
    #[inline]
    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }

    /// Returns the list type.
    #[inline]
    #[must_use]
    pub const fn list_type(&self) -> ListType {
        self.kind
    }

    /// Returns the list items in source order.
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

/// Validated list nesting depth.
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
    /// Returns [`ListError::MaxNestingExceeded`] if the depth is out of range.
    #[inline]
    pub fn try_new(depth: usize) -> Result<Self, ListError> {
        u8::try_from(depth).map(Self).map_err(|_err| {
            ListError::MaxNestingExceeded {
                current: depth,
                limit: usize::from(u8::MAX),
            }
        })
    }

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

/// Borrowed iterator over list items.
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

/// Single item in a markdown list.
///
/// Items can be plain text or checkbox items. Checkbox items may be
/// promoted to [`crate::note::task::Task`] entities while remaining part of
/// their parent list.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::{list::ListItem, position::{SourceByteOffset, SourceByteRange}};
/// let start = SourceByteOffset::new(0);
/// let end = SourceByteOffset::new(13);
/// let range = SourceByteRange::new(start, end).expect("valid range");
/// let item = ListItem::Plain {
///     text: "Buy groceries".into(),
///     range,
/// };
/// assert_eq!(item.text(), "Buy groceries");
/// ```
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum ListItem {
    /// Plain list item (no checkbox).
    Plain {
        /// Raw text content.
        text: Box<str>,
        /// Source byte range in the note.
        range: SourceByteRange,
    },
    /// Checkbox list item.
    Checkbox {
        /// Raw text content.
        text: Box<str>,
        /// Checkbox status symbol.
        status: StatusSymbol,
        /// Source byte range in the note.
        range: SourceByteRange,
        /// Task id if promoted to a Task.
        task_id: Option<TaskId>,
    },
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &self keep accessors concise."
)]
impl ListItem {
    /// Returns the source byte range of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn range(&self) -> SourceByteRange {
        match self {
            Self::Plain {
                range,
                ..
            }
            | Self::Checkbox {
                range,
                ..
            } => *range,
        }
    }

    /// Returns the start source byte position of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn position(&self) -> SourceByteOffset {
        match self {
            Self::Plain {
                range,
                ..
            }
            | Self::Checkbox {
                range,
                ..
            } => range.start(),
        }
    }

    /// Returns the text content of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub fn text(&self) -> &str {
        match self {
            Self::Plain {
                text,
                ..
            }
            | Self::Checkbox {
                text,
                ..
            } => text.as_ref(),
        }
    }

    /// Returns the checkbox status symbol if this is a checkbox item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn status(&self) -> Option<StatusSymbol> {
        match self {
            Self::Checkbox {
                status,
                ..
            } => Some(*status),
            Self::Plain {
                ..
            } => None,
        }
    }

    /// Returns the task id if this checkbox was promoted.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn task_id(&self) -> Option<TaskId> {
        match self {
            Self::Checkbox {
                task_id,
                ..
            } => *task_id,
            Self::Plain {
                ..
            } => None,
        }
    }

    /// Sets the task id for a promoted checkbox item.
    #[inline]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &mut self"
    )]
    pub fn set_task_id(&mut self, task_id: TaskId) {
        if let Self::Checkbox {
            task_id: slot,
            ..
        } = self
        {
            *slot = Some(task_id);
        }
    }

    /// Clears the task id for a checkbox item.
    #[inline]
    pub fn clear_task_id(&mut self) {
        if let Self::Checkbox {
            task_id: slot,
            ..
        } = self
        {
            *slot = None;
        }
    }
}

/// List item metadata entry for indexing.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub struct ListItemEntry {
    range: SourceByteRange,
    depth: ListDepth,
    parent: Option<SourceByteOffset>,
    status: Option<StatusSymbol>,
    task_id: Option<TaskId>,
}

impl ListItemEntry {
    /// Creates a new list item entry.
    #[inline]
    #[must_use]
    pub fn new(
        range: SourceByteRange,
        depth: ListDepth,
        parent: Option<SourceByteOffset>,
        status: Option<StatusSymbol>,
        task_id: Option<TaskId>,
    ) -> Self {
        Self {
            range,
            depth,
            parent,
            status,
            task_id,
        }
    }

    /// Returns the list item byte range.
    #[inline]
    #[must_use]
    pub const fn range(&self) -> SourceByteRange {
        self.range
    }

    /// Returns the list item start byte position.
    #[inline]
    #[must_use]
    pub const fn position(&self) -> SourceByteOffset {
        self.range.start()
    }

    /// Returns the list item depth.
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

    /// Returns the task status symbol, if this is a checkbox item.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> Option<StatusSymbol> {
        self.status
    }

    /// Returns the task id, if this is a promoted task item.
    #[inline]
    #[must_use]
    pub const fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }
}

impl TryFrom<RawListItem> for ListItemEntry {
    type Error = NoteError;

    #[inline]
    fn try_from(raw: RawListItem) -> Result<Self, Self::Error> {
        let depth = match raw.depth() {
            RawListDepth::Root => ListDepth::root(),
            RawListDepth::Nested(value) => {
                ListDepth::try_new(usize::from(value))?
            }
        };
        let status = raw
            .task_marker()
            .map(|marker| StatusSymbol::try_new(marker.marker()))
            .transpose()?;

        Ok(ListItemEntry::new(raw.range(), depth, raw.parent(), status, None))
    }
}

/// Markdown list type.
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
pub enum ListType {
    /// Ordered list starting at the given number.
    Ordered {
        /// Starting index for the list (usually 1).
        start: u64,
    },
    /// Unordered list (bullets).
    Unordered,
}
