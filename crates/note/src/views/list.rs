//! Hierarchical view and query utilities for list items.
//!
//! [`ListView`] is a **persistable projection** stored in the database as a
//! query-optimized representation of list item hierarchies. It provides fast
//! lookup via binary search for common LSP and UI query patterns.
//!
//! # Architecture
//!
//! - **Source of Truth**: `Note.list_items` (flat collection with parent
//!   references)
//! - **Derived Projection**: `ListView` (persisted items + in-memory build)
//! - **Storage**: Separate database table (`LIST_VIEWS_BY_NOTE_ID`)
//! - **Rebuild Strategy**: Reconstructed on every Note save
//!
//! # Query Patterns
//!
//! `ListView` optimizes for:
//! - Position-based lookup (LSP: "item at cursor") - O(log n) binary search
//! - Depth filtering (outline views) - O(n) scan
//! - Hierarchy traversal (parent/child relationships) - O(n) scan
//! - Kind filtering (checkboxes only, ordered lists only) - O(n) scan

use super::super::{
    aggregate::NoteId,
    list::{ListDepth, ListItem, ListKind},
    position::SourceByteOffset,
};

/// Hierarchical projection of list items optimized for queries.
///
/// This is a **persistable cache** built from `Note.list_items` and stored in
/// the database. Items are sorted by position for binary search.
#[derive(
    Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[non_exhaustive]
pub struct ListView {
    /// Source note for this view.
    note_id: NoteId,

    /// All list items (owned, flattened) - sorted by position.
    items: Box<[ListItem]>,
}

impl ListView {
    /// Builds a `ListView` from a Note's flat list items.
    ///
    /// Items are sorted by position for binary search.
    #[inline]
    #[must_use]
    pub fn from_note_items(note_id: NoteId, items: &[ListItem]) -> Self {
        let mut items_vec = items.to_vec();
        items_vec.sort_by_key(super::super::list::ListItem::position);
        Self {
            note_id,
            items: items_vec.into_boxed_slice(),
        }
    }

    /// Returns the source note ID for this view.
    #[inline]
    #[must_use]
    pub const fn note_id(&self) -> NoteId {
        self.note_id
    }

    /// Returns all list items in source order (sorted by position).
    #[inline]
    #[must_use]
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }

    /// Returns root-level items (depth 0, no parent).
    #[inline]
    #[must_use]
    pub fn roots(&self) -> Vec<&ListItem> {
        self.items
            .iter()
            .filter(|item| {
                item.depth() == ListDepth::root() && item.parent().is_none()
            })
            .collect()
    }

    /// Finds item at exact position using binary search.
    #[inline]
    #[must_use]
    pub fn find_at_position(&self, pos: SourceByteOffset) -> Option<&ListItem> {
        let idx = self.position_lower_bound(pos)?;
        let item = self.items.get(idx)?;
        (item.position() == pos).then_some(item)
    }

    fn position_lower_bound(&self, pos: SourceByteOffset) -> Option<usize> {
        self.items
            .binary_search_by_key(&pos, super::super::list::ListItem::position)
            .ok()
    }

    /// Returns direct children of item.
    #[inline]
    #[must_use]
    pub fn children_of(&self, item: &ListItem) -> Vec<&ListItem> {
        let item_pos = item.position();
        let item_depth = item.depth();
        self.items
            .iter()
            .filter(|child| {
                child.parent() == Some(item_pos) && child.depth() > item_depth
            })
            .collect()
    }

    /// Returns all descendants of item (recursive, depth-first).
    #[inline]
    #[must_use]
    pub fn descendants_of(&self, item: &ListItem) -> Vec<&ListItem> {
        let mut result = Vec::new();
        let mut stack = self.children_of(item);

        while let Some(child) = stack.pop() {
            result.push(child);
            stack.extend(self.children_of(child));
        }

        result
    }

    /// Returns the parent of item, if any.
    #[inline]
    #[must_use]
    pub fn parent_of(&self, item: &ListItem) -> Option<&ListItem> {
        let parent_pos = item.parent()?;
        self.find_at_position(parent_pos)
    }

    /// Returns all items at a specific depth level.
    #[inline]
    #[must_use]
    pub fn items_at_depth(&self, depth: ListDepth) -> Vec<&ListItem> {
        self.items.iter().filter(|item| item.depth() == depth).collect()
    }

    /// Returns all ordered list items.
    #[inline]
    #[must_use]
    pub fn ordered_items(&self) -> Vec<&ListItem> {
        self.items
            .iter()
            .filter(|item| matches!(item.list_kind(), ListKind::Ordered(_)))
            .collect()
    }

    /// Returns all unordered list items.
    #[inline]
    #[must_use]
    pub fn unordered_items(&self) -> Vec<&ListItem> {
        self.items
            .iter()
            .filter(|item| matches!(item.list_kind(), ListKind::Unordered))
            .collect()
    }

    /// Returns all checkbox items.
    #[inline]
    #[must_use]
    pub fn checkbox_items(&self) -> Vec<&ListItem> {
        self.items
            .iter()
            .filter(|item| item.base.is_checkbox.is_some())
            .collect()
    }
}
