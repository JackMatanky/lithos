//! List subentities for the Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use std::fmt;

use super::{error::NoteError, position::SourceByteOffset, task::TaskId};
use crate::config::task::StatusSymbol;

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
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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
    serde::Serialize,
    serde::Deserialize,
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
    /// Returns [`NoteError::ListDepthOutOfRange`] if the depth is out of range.
    #[inline]
    pub fn try_new(depth: usize) -> Result<Self, NoteError> {
        u8::try_from(depth).map(Self).map_err(|_error| {
            NoteError::ListDepthOutOfRange {
                depth,
                reason: "depth exceeds maximum allowed value of 255",
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
/// # use lithos_core::note::{list::ListItem, position::SourceByteOffset};
/// let item = ListItem::Plain {
///     text: "Buy groceries".into(),
///     position: SourceByteOffset::new(0),
/// };
/// assert_eq!(item.text(), "Buy groceries");
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
#[non_exhaustive]
pub enum ListItem {
    /// Plain list item (no checkbox).
    Plain {
        /// Raw text content.
        text: Box<str>,
        /// Source byte offset in the note.
        position: SourceByteOffset,
    },
    /// Checkbox list item.
    Checkbox {
        /// Raw text content.
        text: Box<str>,
        /// Checkbox status symbol.
        status: StatusSymbol,
        /// Source byte offset in the note.
        position: SourceByteOffset,
        /// Task id if promoted to a Task.
        task_id: Option<TaskId>,
    },
}

#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &self keep accessors concise."
)]
impl ListItem {
    /// Returns the source byte position of this list item.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Match ergonomics on &self"
    )]
    pub const fn position(&self) -> SourceByteOffset {
        match self {
            Self::Plain {
                position,
                ..
            }
            | Self::Checkbox {
                position,
                ..
            } => *position,
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

/// Markdown list type.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
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
