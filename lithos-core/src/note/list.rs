//! List subentities for the Note aggregate.
#![allow(
    missing_docs,
    clippy::exhaustive_structs,
    clippy::exhaustive_enums,
    reason = "rkyv derives generate archived/resolver items that are missing \
              docs"
)]

use super::{task::TaskId, types::SourceByteOffset};
use crate::config::task::StatusSymbol;

/// Markdown list structure.
///
/// Represents a collection of [`ListItem`]s, which can be ordered or unordered.
/// A `List` tracks its nesting depth within the document.
///
/// # Examples
///
/// ```
/// # use lithos_core::note::list::{List, ListType};
/// let list = List::new(ListType::Unordered);
/// assert_eq!(list.depth(), 0);
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
#[expect(
    clippy::struct_field_names,
    reason = "list_type is the clearest name for the list kind."
)]
pub struct List {
    list_type: ListType,
    items: Vec<ListItem>,
    depth: u8,
}

impl List {
    /// Creates a new empty list with depth 0.
    #[inline]
    #[must_use]
    pub fn new(list_type: ListType) -> Self {
        Self {
            list_type,
            items: Vec::new(),
            depth: 0,
        }
    }

    /// Creates a new empty list with an explicit depth.
    #[inline]
    #[must_use]
    pub fn with_depth(list_type: ListType, depth: u8) -> Self {
        Self {
            list_type,
            items: Vec::new(),
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
        self.list_type
    }

    /// Returns the list items in source order.
    #[inline]
    #[must_use]
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }

    /// Returns the list nesting depth (0 = top level).
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// Sets the list nesting depth.
    #[inline]
    pub fn set_depth(&mut self, depth: u8) {
        self.depth = depth;
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
/// # use lithos_core::note::{list::ListItem, types::SourceByteOffset};
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
    reason = "Accessor methods intentionally use match ergonomics on `&self` \
              to keep code concise."
)]
impl ListItem {
    /// Returns the source byte position of this list item.
    #[inline]
    #[must_use]
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
    pub fn text(&self) -> &str {
        match self {
            Self::Plain {
                text,
                ..
            }
            | Self::Checkbox {
                text,
                ..
            } => text,
        }
    }

    /// Returns the checkbox status symbol if this is a checkbox item.
    #[inline]
    #[must_use]
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

#[cfg(test)]
#[expect(
    clippy::disallowed_methods,
    reason = "Tests use expect for deterministic fixtures."
)]
mod tests {
    use super::*;

    #[test]
    fn list_adds_items_in_order() {
        let mut list = List::new(ListType::Unordered);
        list.add_item(ListItem::Plain {
            text: "first".into(),
            position: SourceByteOffset::new(0),
        });
        list.add_item(ListItem::Plain {
            text: "second".into(),
            position: SourceByteOffset::new(10),
        });

        assert_eq!(list.items().len(), 2);
        assert_eq!(list.items().first().expect("first item").text(), "first");
        assert_eq!(list.items().get(1).expect("second item").text(), "second");
    }

    #[test]
    fn checkbox_task_id_is_mutable() {
        let mut item = ListItem::Checkbox {
            text: "task".into(),
            status: StatusSymbol::try_new(' ').expect("valid status"),
            position: SourceByteOffset::new(5),
            task_id: None,
        };

        let task_id = TaskId::new();
        item.set_task_id(task_id);
        assert_eq!(item.task_id(), Some(task_id));

        item.clear_task_id();
        assert_eq!(item.task_id(), None);
    }

    #[test]
    fn list_depth_is_settable() {
        let mut list = List::new(ListType::Ordered {
            start: 1,
        });
        assert_eq!(list.depth(), 0);
        list.set_depth(2);
        assert_eq!(list.depth(), 2);
    }
}
