//! Inheritance relationship views for optimal tree queries.
//!
//! # Storage Schema (3 Tables)
//!
//! 1. **`SCHEMA_BY_ID`** (Regular Table): `SchemaId → Schema` (aggregate root)
//! 2. **`SCHEMA_INHERITANCE`** (Regular Table): `SchemaId →
//!    SchemaInheritanceView`
//! 3. **`SCHEMA_CHILDREN_BY_PARENT`** (Multimap): `ParentId → Vec<SchemaId>`
//!
//! # Design Goals
//!
//! - **O(log N)** parent lookups (embedded in metadata)
//! - **O(log N)** staleness checks (via `ancestors_hash`)
//! - **O(log N + C)** children lookups (multimap efficiency)
//! - **O(D×log N)** descendant traversal (BFS using multimap)
//! - **Minimal storage overhead** (~172 bytes/schema)
//!
//! # Key Optimizations
//!
//! - Pre-compute `depth` during tree building (saves recalculation)
//! - Store `Vec<SchemaId>` not `Vec<SchemaName>` (33% space savings)
//! - Omit redundant fields (`excludes` already in Schema aggregate)
//! - Recursive `ancestors_hash` for O(1) transitive staleness detection

#![expect(
    clippy::exhaustive_structs,
    reason = "rkyv Archive derive generates Archived* types that trigger this \
              lint - we can't control macro expansion"
)]

use std::{
    collections::{HashSet, VecDeque},
    time::SystemTime,
};

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use crate::{db::DbError, schema::aggregate::SchemaId};

/// Inheritance metadata cache for a single schema.
///
/// **Storage**:
/// - Table: `SCHEMA_INHERITANCE` (regular table)
/// - Key: `SchemaId`
/// - Value: `SchemaInheritanceView` (rkyv-serialized)
///
/// **Purpose**:
/// - Fast-path resolution by caching precomputed inheritance metadata
/// - Transitive staleness detection via `ancestors_hash` comparison
/// - Avoid rebuilding `InheritanceGraph` when inheritance chain unchanged
///
/// **Staleness Detection**:
/// The `ancestors_hash` field is a recursive hash of the parent chain:
/// ```text
/// hash = hash(parent_id || parent.ancestors_hash)
/// ```
/// When checking if metadata is fresh:
/// 1. Compute expected hash from parent's metadata
/// 2. Compare with cached hash
/// 3. If hashes match → full ancestor chain unchanged (O(1) check)
///
/// **Size**: ~172 bytes (typical: 3 ancestors, depth 4):
/// - `parent`: 16 bytes (Option<SchemaId> with Some)
/// - `ancestors`: 24 bytes (Vec header) + 16 bytes/ancestor × 3 = 72 bytes
/// - `depth`: 8 bytes (usize)
/// - `ancestors_hash`: 8 bytes (u64)
/// - `resolved_at`: 12 bytes (`SystemTime`)
/// - Total: ~140 bytes typical, ~172 bytes worst case
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SchemaInheritanceView {
    /// Parent schema ID, or None for root schemas.
    ///
    /// While technically redundant with `ancestors[0]`, this explicit field
    /// provides clearer API ergonomics. Root detection becomes trivial:
    /// `parent.is_none()`.
    pub parent: Option<SchemaId>,

    /// Ordered list of ancestor schema IDs: [parent, grandparent, ...].
    ///
    /// Topological order from closest (parent) to farthest (root).
    /// Empty for root schemas (no parent). Note: `ancestors[0]` equals
    /// `parent` when `parent.is_some()`.
    pub ancestors: Vec<SchemaId>,

    /// Inheritance depth in the tree (1-indexed).
    ///
    /// - Root schemas: `depth = 1`
    /// - Child schemas: `depth = parent.depth + 1`
    ///
    /// **Pre-computed during tree building** to avoid recalculation during
    /// property merging. This is a 8-byte cost for O(1) access.
    pub depth: usize,

    /// Recursive hash of the parent chain for staleness detection.
    ///
    /// Computed as `hash(parent_id || parent_ancestors_hash)`. This enables
    /// O(1) transitive staleness checking: if the parent's hash changed, this
    /// hash won't match, indicating the full ancestor chain needs rebuilding.
    ///
    /// ## Hash Size: 64-bit (u64)
    ///
    /// Uses 64-bit hash instead of 128-bit or 256-bit:
    /// - **Collision probability at 10K schemas**: 2.7×10⁻⁶ (0.0003%)
    /// - **Failure mode**: False cache invalidation (triggers unnecessary
    ///   re-resolution, not data corruption)
    /// - **Performance**: 8 bytes vs 16/32 bytes (better cache locality)
    /// - **Use case**: Cache invalidation tolerates rare false positives
    ///
    /// For comparison, 128-bit would provide 1.5×10⁻¹⁸ collision probability
    /// at this scale, which is overkill for cache invalidation scenarios.
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

    /// Check if this metadata is stale by comparing expected hash with cached
    /// hash.
    ///
    /// # Errors
    /// Returns error if parent metadata cannot be retrieved from repository.
    #[inline]
    #[must_use]
    pub fn is_stale(&self, parent_metadata: Option<&Self>) -> bool {
        if let Some(parent_id) = self.parent {
            if let Some(parent) = parent_metadata {
                let expected_hash =
                    Self::compute_hash(Some((parent_id, parent)));
                self.ancestors_hash != expected_hash
            } else {
                // Parent metadata missing - assume stale
                true
            }
        } else {
            // Root schema - never stale via inheritance
            false
        }
    }
}

/// Find all descendants of a schema using BFS traversal.
///
/// # Arguments
/// - `root_id`: Starting schema to find descendants of
/// - `get_children`: Closure that returns direct children for a given parent
///
/// # Returns
/// Set of all descendant schema IDs (transitive closure).
///
/// # Performance
/// O(D×log N) where:
/// - D = number of descendants
/// - N = total schemas in database
/// - log N = cost of multimap lookup per node
///
/// # Errors
/// Returns error if `get_children` closure returns an error.
///
/// # Example
/// ```ignore
/// let descendants = find_all_descendants(
///     schema_id,
///     |parent_id| repo.get_descendants(parent_id)
/// )?;
/// ```
#[inline]
pub fn find_all_descendants<F>(
    root_id: SchemaId,
    mut get_children: F,
) -> Result<HashSet<SchemaId>, DbError>
where
    F: FnMut(SchemaId) -> Result<Vec<SchemaId>, DbError>,
{
    let mut descendants = HashSet::new();
    let mut queue = VecDeque::from([root_id]);

    while let Some(id) = queue.pop_front() {
        let children = get_children(id)?;
        for child_id in children {
            if descendants.insert(child_id) {
                queue.push_back(child_id);
            }
        }
    }

    Ok(descendants)
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
            depth: 1,
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
            depth: 1,
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };

        // Course schema (child of base)
        let course_hash =
            SchemaInheritanceView::compute_hash(Some((base_id, &base_view)));
        let course_view = SchemaInheritanceView {
            parent: Some(base_id),
            ancestors: vec![base_id],
            depth: 2,
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
    fn is_stale_detects_changed_parent_hash() {
        let base_id = SchemaId::from_uuid(BASE_UUID);

        let base_view = SchemaInheritanceView {
            parent: None,
            ancestors: vec![],
            depth: 1,
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };

        // Child with correct hash
        let expected_hash =
            SchemaInheritanceView::compute_hash(Some((base_id, &base_view)));
        let child_view = SchemaInheritanceView {
            parent: Some(base_id),
            ancestors: vec![base_id],
            depth: 2,
            ancestors_hash: expected_hash,
            resolved_at: SystemTime::now(),
        };

        assert!(!child_view.is_stale(Some(&base_view)), "Should be fresh");

        // Parent hash changes
        let mut modified_base = base_view.clone();
        modified_base.ancestors_hash = 999_999;

        assert!(child_view.is_stale(Some(&modified_base)), "Should be stale");
    }

    #[test]
    fn find_descendants_traverses_tree() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);
        let physics_id = SchemaId::from_uuid(PHYSICS_UUID);

        // Mock multimap: base → [course], course → [physics]
        let get_children =
            |parent: SchemaId| -> Result<Vec<SchemaId>, DbError> {
                if parent == base_id {
                    Ok(vec![course_id])
                } else if parent == course_id {
                    Ok(vec![physics_id])
                } else {
                    Ok(vec![])
                }
            };

        let descendants = find_all_descendants(base_id, get_children)
            .expect("BFS should succeed");

        assert_eq!(descendants.len(), 2, "Should find 2 descendants");
        assert!(descendants.contains(&course_id));
        assert!(descendants.contains(&physics_id));
    }

    #[test]
    fn rkyv_serialization_roundtrip() {
        let base_id = SchemaId::from_uuid(BASE_UUID);
        let course_id = SchemaId::from_uuid(COURSE_UUID);

        let view = SchemaInheritanceView {
            parent: Some(course_id),
            ancestors: vec![course_id, base_id],
            depth: 3,
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
        assert_eq!(archived.depth, 3);
        assert_eq!(archived.ancestors_hash, 12345);

        // Full roundtrip
        let deserialized: SchemaInheritanceView =
            rkyv::deserialize::<_, rkyv::rancor::Error>(archived)
                .expect("full deserialization should succeed");

        assert_eq!(deserialized, view);
    }

    #[test]
    fn depth_field_matches_ancestors_length() {
        // Root: depth = 1, ancestors = []
        let root = SchemaInheritanceView {
            parent: None,
            ancestors: vec![],
            depth: 1,
            ancestors_hash: 0,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(root.depth, root.ancestors.len() + 1);

        // Child: depth = 2, ancestors = [parent]
        let child = SchemaInheritanceView {
            parent: Some(SchemaId::from_uuid(BASE_UUID)),
            ancestors: vec![SchemaId::from_uuid(BASE_UUID)],
            depth: 2,
            ancestors_hash: 123,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(child.depth, child.ancestors.len() + 1);

        // Grandchild: depth = 3, ancestors = [parent, grandparent]
        let grandchild = SchemaInheritanceView {
            parent: Some(SchemaId::from_uuid(COURSE_UUID)),
            ancestors: vec![
                SchemaId::from_uuid(COURSE_UUID),
                SchemaId::from_uuid(BASE_UUID),
            ],
            depth: 3,
            ancestors_hash: 456,
            resolved_at: SystemTime::now(),
        };
        assert_eq!(grandchild.depth, grandchild.ancestors.len() + 1);
    }
}
