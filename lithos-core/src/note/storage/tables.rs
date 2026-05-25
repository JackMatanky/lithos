//! Table definitions for note storage.

use crate::{
    db::{PathTable, UuidTable},
    impl_redb_uuid,
    note::aggregate::NoteId,
};

impl_redb_uuid!(NoteId);

/// Note aggregates with zero-copy serialization.
///
/// Stores full `Note` structures indexed by note ID for efficient
/// retrieval by identity.
///
/// Key: `NoteId`
/// Value: rkyv-serialized `Note`
pub const NOTES: UuidTable<NoteId, &[u8]> = UuidTable::new("notes");

/// Materialized list views indexed by note ID.
///
/// Stores cached `ListView` projections for notes containing lists.
/// This is a rebuildable cache for query-optimized list representations.
///
/// Key: `NoteId`
/// Value: rkyv-serialized `ListView`
pub const LIST_VIEWS: UuidTable<NoteId, &[u8]> = UuidTable::new("list_views");

/// Path-to-NoteId index for fast path-based lookup.
///
/// Maps vault-relative paths (e.g., "daily/2024-05-25.md") to their
/// corresponding note IDs, enabling path-based note retrieval and enforcing
/// path uniqueness constraints.
///
/// Key: vault path string
/// Value: rkyv-serialized `NoteId`
pub const NOTE_ID_BY_PATH: PathTable<&[u8]> = PathTable::new("note_id_by_path");
