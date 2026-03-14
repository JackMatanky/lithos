//! Inheritance relationship views for schema persistence.
//!
//! These types track parent-child relationships in the schema inheritance
//! graph. They are stored separately from the Schema aggregate to enable
//! efficient updates without full schema re-resolution.

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv Archive derive generates Archived* types that trigger this \
              lint - we can't control macro expansion"
)]

use std::time::SystemTime;

use rkyv::{
    Archive, Deserialize, Serialize, rancor::Error as RkyvError,
    with::AsUnixTime,
};

use crate::{db::DbError, schema::aggregate::SchemaId};

/// Child schema reference, stored in `schema_children` multimap.
///
/// **Storage pattern:**
/// - Table: `schema_children` (multimap)
/// - Key: Parent `SchemaId` (as UUID string)
/// - Value: `ChildSchemaView` (rkyv-serialized bytes)
///
/// This multimap enables O(1) lookup of "all children of parent P".
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::ChildSchemaView;
///
/// let child = ChildSchemaView {
///     child_id: child_schema_id,
///     excludes: vec!["created_at".into()],
///     resolved_at: SystemTime::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChildSchemaView {
    /// Child schema ID.
    pub child_id: SchemaId,
    /// Property names this child excludes from parent's properties.
    pub excludes: Vec<Box<str>>,
    /// Timestamp when this inheritance relationship was last resolved.
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

impl ChildSchemaView {
    /// Serialize to bytes for multimap storage.
    ///
    /// # Errors
    /// Returns serialization error if rkyv encoding fails.
    #[inline]
    pub fn to_bytes(&self) -> Result<Vec<u8>, DbError> {
        rkyv::to_bytes(self)
            .map(|bytes| bytes.to_vec())
            .map_err(|e: RkyvError| DbError::Serialization(e.to_string()))
    }
}

/// Parent schema reference, stored in `schema_parent` table.
///
/// **Storage pattern:**
/// - Table: `schema_parent` (regular table, not multimap)
/// - Key: Child `SchemaId` (as UUID string)
/// - Value: `ParentSchemaView`
///
/// This table tracks ALL schemas (both roots and children):
/// - Root schemas: `parent_id = None`
/// - Child schemas: `parent_id = Some(parent_id)`
///
/// **Update optimization:** When updating a child's parent, this table
/// provides O(1) lookup of the old parent plus the old excludes/timestamp
/// needed to reconstruct the exact bytes for removing the old entry from
/// the `schema_children` multimap.
///
/// **Data redundancy:** `excludes` and `resolved_at` are stored in both
/// `schema_parent` and `schema_children`. This trades ~10KB of storage
/// (for typical 100-schema vaults) for simpler, faster update logic.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::ParentSchemaView;
///
/// let parent = ParentSchemaView {
///     parent_id: Some(parent_schema_id),
///     excludes: vec!["created_at".into()],
///     resolved_at: SystemTime::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ParentSchemaView {
    /// Parent schema ID, or None for root schemas.
    pub parent_id: Option<SchemaId>,
    /// Property names excluded from parent (cached for multimap removal).
    pub excludes: Vec<Box<str>>,
    /// Timestamp when relationship was resolved (cached for multimap removal).
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}
