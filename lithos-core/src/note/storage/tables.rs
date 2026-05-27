//! Redb table definitions for Note context persistence.
//!
//! This module defines the typed table constants used by the note storage
//! layer. Each table is a strongly-typed wrapper around a redb table name,
//! ensuring key/value type safety at the storage boundary.
//!
//! # Table Inventory
//!
//! | Table | Key | Value | Purpose |
//! |-------|-----|-------|---------|
//! | [`NOTES`] | `NoteId` | `&[u8]` | Primary note aggregate table |
//! | [`LIST_VIEWS`] | `NoteId` | `&[u8]` | Cached list view projections |
//! | [`NOTE_ID_BY_PATH`] | `PathKey` | `NoteId` | Path-to-ID index (`PathUuidTable`) |
//!
//! # Consistency Model
//!
//! - `NOTES` is the source of truth.
//! - `NOTE_ID_BY_PATH` is a secondary index maintained transactionally
//!   alongside `NOTES` — path uniqueness is enforced at write time.
//! - `LIST_VIEWS` is a rebuildable cache that can be regenerated on demand.
//!
//! # Examples
//!
//! ```rust,ignore
//! use lithos_core::note::storage::tables::NOTES;
//!
//! // Open table in a read transaction
//! let table = tx.open_table(NOTES.definition())?;
//! let note_bytes = table.get(&note_id)?;
//! ```

use crate::{
    db::{PathUuidTable, UuidTable},
    impl_redb_uuid,
    note::aggregate::NoteId,
};

impl_redb_uuid!(NoteId);

/// Primary note aggregate table.
///
/// Stores full [`Note`](crate::note::aggregate::Note) structures indexed
/// by [`NoteId`](crate::note::aggregate::NoteId) for O(1) identity lookup.
/// All other note tables are secondary indexes maintained synchronously
/// during write operations.
///
/// Key: [`NoteId`](crate::note::aggregate::NoteId)
/// Value: rkyv-serialized [`Note`](crate::note::aggregate::Note)
///
/// See also: [`NOTE_ID_BY_PATH`] for path-based lookups.
pub const NOTES: UuidTable<NoteId, &[u8]> = UuidTable::new("notes");

/// Materialized list view cache.
///
/// Stores cached [`ListView`](crate::note::views::ListView) projections for
/// notes containing task lists. This is a rebuildable cache — it can be
/// dropped and regenerated from the primary [`NOTES`] table on demand.
///
/// Key: [`NoteId`](crate::note::aggregate::NoteId)
/// Value: rkyv-serialized [`ListView`](crate::note::views::ListView)
///
/// # Cache Semantics
///
/// - Written by the processor after extracting list items from a note.
/// - Read by the query layer for O(1) list view access.
/// - Not guaranteed to exist for any given note — always handle `None`.
pub const LIST_VIEWS: UuidTable<NoteId, &[u8]> = UuidTable::new("list_views");

/// Path-to-ID index for path-based note lookup.
///
/// Maps vault-relative [`PathKey`](crate::fs::PathKey) paths to their
/// corresponding [`NoteId`](crate::note::aggregate::NoteId), enabling O(1)
/// path-based note retrieval and enforcing path uniqueness at write time.
///
/// Key: [`PathKey`](crate::fs::PathKey) (vault-relative path)
/// Value: [`NoteId`](crate::note::aggregate::NoteId)
///
/// # Consistency
///
/// This index is updated transactionally alongside [`NOTES`] — if a note
/// is written, its path mapping is written in the same redb transaction
/// to prevent index drift.
///
/// # Example
///
/// ```rust,ignore
/// use lithos_core::note::storage::tables::NOTE_ID_BY_PATH;
/// use lithos_core::fs::PathKey;
///
/// let path = PathKey::try_new("daily/2024-05-25.md")?;
/// let table = tx.open_table(NOTE_ID_BY_PATH.definition())?;
/// let note_id = table.get(&path)?.expect("note exists").value();
/// ```
pub const NOTE_ID_BY_PATH: PathUuidTable<NoteId> =
    PathUuidTable::new("note_id_by_path");
