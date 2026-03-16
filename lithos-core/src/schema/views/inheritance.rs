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

/// Schema inheritance metadata cache, stored in `schema_inheritance` table.
///
/// **Storage pattern:**
/// - Table: `schema_inheritance` (regular table, not multimap)
/// - Key: `SchemaId` (the schema this metadata describes)
/// - Value: `SchemaInheritanceView` (rkyv-serialized bytes)
///
/// This view enables fast-path resolution by caching precomputed inheritance
/// metadata. When a schema's inheritance chain hasn't changed, the loader can
/// skip rebuilding the `SchemaTree` and use cached ancestors directly.
///
/// **Staleness detection:** The `ancestors_hash` field is a recursive hash of
/// the parent chain. When checking if metadata is fresh, compute the current
/// hash from the parent's metadata and compare. If hashes match, the full
/// ancestor chain is unchanged (transitive staleness check).
///
/// **Storage efficiency:**
/// - Stores `Vec<SchemaId>` not `Vec<SchemaName>` (saves 33% space, avoids
///   `HashMap` lookups)
/// - No redundant `schema_id` field (it's the table key)
/// - No redundant `depth` field (derivable from `ancestors.len() + 1`)
///
/// **Size:** 113 bytes (typical case: 3 ancestors, 2 excludes).
/// - `parent`: 16 bytes (Option<SchemaId> with Some)
/// - `ancestors`: 24 bytes (Vec header) + 16 bytes/ancestor × 3 = 72 bytes
/// - `excludes`: 24 bytes (Vec header) + ~8 bytes/exclude × 2 = 40 bytes
/// - `ancestors_hash`: 8 bytes
/// - `resolved_at`: 12 bytes
/// - Total: ~172 bytes worst case, 113 bytes typical
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::views::SchemaInheritanceView;
///
/// let view = SchemaInheritanceView {
///     parent: Some(parent_id),
///     ancestors: vec![parent_id, grandparent_id],
///     excludes: vec!["created_at".into(), "internal_ref".into()],
///     ancestors_hash: compute_ancestors_hash(&parent_metadata),
///     resolved_at: SystemTime::now(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaInheritanceView {
    /// Parent schema ID, or None for root schemas.
    ///
    /// While technically redundant with `ancestors[0]`, this explicit field
    /// provides clearer API ergonomics and matches the existing
    /// `ParentSchemaView` pattern. Root detection becomes trivial:
    /// `parent.is_none()`.
    pub parent: Option<SchemaId>,

    /// Ordered list of ancestor schema IDs: [parent, grandparent, ...].
    ///
    /// Topological order from closest (parent) to farthest (root).
    /// Empty for root schemas (no parent). Note: `ancestors[0]` equals
    /// `parent` when `parent.is_some()`.
    pub ancestors: Vec<SchemaId>,

    /// Property names to exclude from inherited properties.
    ///
    /// This is the raw `excludes` field from the schema file, applied
    /// during property resolution.
    pub excludes: Vec<Box<str>>,

    /// Recursive hash of the parent chain for staleness detection.
    ///
    /// Computed as `hash(parent_id || parent_ancestors_hash)`. This enables
    /// O(1) transitive staleness checking: if the parent's hash changed, this
    /// hash won't match, indicating the full ancestor chain needs rebuilding.
    pub ancestors_hash: u64,

    /// Timestamp when this metadata was computed.
    #[rkyv(with = AsUnixTime)]
    pub resolved_at: SystemTime,
}

impl SchemaInheritanceView {
    /// Compute the ancestors hash for a schema given its parent's metadata.
    ///
    /// This is a recursive hash: `hash(parent_id || parent_ancestors_hash)`.
    /// If the parent is a root (no metadata), hash only the `parent_id`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Root schema (no parent): hash is 0
    /// let root_hash = SchemaInheritanceView::compute_hash(None);
    /// assert_eq!(root_hash, 0);
    ///
    /// // Child schema: hash parent_id + parent's hash
    /// let parent_metadata = /* ... */;
    /// let child_hash = SchemaInheritanceView::compute_hash(
    ///     Some((parent_id, &parent_metadata))
    /// );
    /// ```
    #[must_use]
    #[inline]
    pub fn compute_hash(parent_info: Option<(SchemaId, &Self)>) -> u64 {
        match parent_info {
            None => 0, // Root schema
            Some((parent_id, parent_metadata)) => {
                use std::hash::{Hash as _, Hasher as _};
                let mut hasher =
                    std::collections::hash_map::DefaultHasher::new();
                parent_id.hash(&mut hasher);
                parent_metadata.ancestors_hash.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    // Test UUIDs (v7 format)
    const BASE_UUID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0001);
    const COURSE_UUID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0002);
    const PHYSICS_UUID: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0003);

    #[test]
    fn root_schema_has_zero_hash() {
        let hash = SchemaInheritanceView::compute_hash(None);
        assert_eq!(hash, 0, "Root schemas should have hash = 0");
    }

    #[test]
    fn child_hash_includes_parent_id_and_parent_hash() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);

        // Base schema (root)
        let base_view = SchemaInheritanceView {
            parent: None,
            ancestors: vec![],
            excludes: vec![],
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };

        // Course schema (child of base)
        let course_hash =
            SchemaInheritanceView::compute_hash(Some((base_id, &base_view)));

        assert_ne!(course_hash, 0, "Child hash should be non-zero");
        assert_ne!(
            course_hash, base_view.ancestors_hash,
            "Child hash should differ from parent hash"
        );

        // Compute again with same inputs - should be deterministic
        let course_hash_2 =
            SchemaInheritanceView::compute_hash(Some((base_id, &base_view)));
        assert_eq!(course_hash, course_hash_2, "Hash should be deterministic");

        // Different parent ID should produce different hash
        let different_hash =
            SchemaInheritanceView::compute_hash(Some((course_id, &base_view)));
        assert_ne!(
            course_hash, different_hash,
            "Different parent ID should change hash"
        );
    }

    #[test]
    fn hash_changes_transitively_through_chain() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);
        let _physics_id = SchemaId::from_uuid(PHYSICS_UUID);

        // Base schema (root)
        let base_view = SchemaInheritanceView {
            parent: None,
            ancestors: vec![],
            excludes: vec![],
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };

        // Course schema (child of base)
        let course_hash =
            SchemaInheritanceView::compute_hash(Some((base_id, &base_view)));
        let course_view = SchemaInheritanceView {
            parent: Some(base_id),
            ancestors: vec![base_id],
            excludes: vec![],
            ancestors_hash: course_hash,
            resolved_at: SystemTime::now(),
        };

        // Physics schema (child of course)
        let physics_hash = SchemaInheritanceView::compute_hash(Some((
            course_id,
            &course_view,
        )));

        assert_ne!(physics_hash, 0);
        assert_ne!(physics_hash, base_view.ancestors_hash);
        assert_ne!(physics_hash, course_view.ancestors_hash);

        // If course's hash changes, physics's hash should also change
        let mut modified_course_view = course_view.clone();
        modified_course_view.ancestors_hash = 999_999; // Simulate parent chain change

        let new_physics_hash = SchemaInheritanceView::compute_hash(Some((
            course_id,
            &modified_course_view,
        )));

        assert_ne!(
            physics_hash, new_physics_hash,
            "Changing ancestor hash should cascade to child"
        );
    }

    #[test]
    fn rkyv_serialization_roundtrip() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);

        let view = SchemaInheritanceView {
            parent: Some(course_id),
            ancestors: vec![course_id, base_id],
            excludes: vec!["created_at".into(), "internal_ref".into()],
            ancestors_hash: 12345,
            resolved_at: SystemTime::UNIX_EPOCH,
        };

        // Serialize
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&view)
            .expect("serialization should succeed");

        // Deserialize
        let archived = rkyv::access::<
            ArchivedSchemaInheritanceView,
            rkyv::rancor::Error,
        >(&bytes)
        .expect("deserialization should succeed");

        // Verify fields
        assert_eq!(archived.ancestors.len(), 2);
        assert_eq!(archived.excludes.len(), 2);
        assert_eq!(archived.ancestors_hash, 12345);

        // Full roundtrip
        let deserialized: SchemaInheritanceView =
            rkyv::deserialize::<_, rkyv::rancor::Error>(archived)
                .expect("full deserialization should succeed");

        assert_eq!(deserialized, view);
    }

    #[test]
    fn parent_field_matches_ancestors_first() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);

        // Root schema: parent = None, ancestors = []
        let root_view = SchemaInheritanceView {
            parent: None,
            ancestors: vec![],
            excludes: vec![],
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(root_view.parent, None);
        assert!(root_view.ancestors.is_empty());

        // Child schema: parent = Some(base), ancestors = [base]
        let child_view = SchemaInheritanceView {
            parent: Some(base_id),
            ancestors: vec![base_id],
            excludes: vec![],
            ancestors_hash: 123,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(child_view.parent, Some(base_id));
        assert_eq!(child_view.ancestors.first(), child_view.parent.as_ref());

        // Grandchild schema: parent = Some(course), ancestors = [course, base]
        let grandchild_view = SchemaInheritanceView {
            parent: Some(course_id),
            ancestors: vec![course_id, base_id],
            excludes: vec![],
            ancestors_hash: 456,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(grandchild_view.parent, Some(course_id));
        assert_eq!(
            grandchild_view.ancestors.first(),
            grandchild_view.parent.as_ref()
        );
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
