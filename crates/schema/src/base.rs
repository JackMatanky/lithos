//! Base schema domain type.
//!
//! Provides [`BaseSchema`], the validated schema aggregate carrying a
//! file-local `extends` list (by [`SchemaName`]) prior to cross-schema
//! parent resolution.
//!
//! Relationship to other schema types:
//! - [`RawSchema`](super::raw::RawSchema) - on-disk input, fields named the
//!   same as the source file.
//! - [`BaseSchema`] - validated, name-based projection of the source file's
//!   `extends` and `excludes` lists. Multiple-inheritance ready.
//! - [`Schema`](super::aggregate::Schema) - fully resolved aggregate whose
//!   `parents` are [`SchemaId`]s after cross-schema lookup.

use std::time::SystemTime;

use rkyv::{Archive, Deserialize, Serialize, with::AsUnixTime};

use super::{
    identifier::{SchemaId, SchemaName},
    property::{PropertyMap, PropertyName},
};

/// Schema aggregate carrying file-local `extends` by name.
///
/// `BaseSchema` is the projection of a `RawSchema` after validation but before
/// cross-schema parent resolution. Its `extends` list mirrors the schema
/// source file's `extends` field (a list of [`SchemaName`]s). After resolution,
/// these names are mapped to [`SchemaId`]s on the
/// [`Schema`](super::aggregate::Schema) aggregate.
///
/// ## Fields
///
/// - `id`: Unique schema identifier
/// - `name`: Validated schema name
/// - `properties`: Direct-declared properties (pre-inheritance)
/// - `extends`: Parent schema names declared in the source file
/// - `excludes`: Inherited property names to omit from the resolved schema
/// - `recorded_at`: Ingestion timestamp (**private** - not part of public API)
///
/// # Examples
///
/// ```
/// use trace_schema::{
///     base::BaseSchema,
///     identifier::{SchemaId, SchemaName},
///     property::PropertyMap,
/// };
///
/// let id = SchemaId::new();
/// let name = SchemaName::try_new("child")?;
/// let base =
///     BaseSchema::new(id, name, PropertyMap::new(), Vec::new(), Vec::new());
/// assert_eq!(base.id(), &id);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BaseSchema {
    /// Schema identity.
    id: SchemaId,
    /// Schema name.
    name: SchemaName,
    /// Direct-declared properties on this schema (pre-inheritance).
    properties: PropertyMap,
    /// File-local parent schema names declared in the source file's
    /// `extends` list. Resolved to [`SchemaId`]s on
    /// [`Schema`](super::aggregate::Schema).
    extends: Box<[SchemaName]>,
    /// Inherited property names to omit when resolving the child schema.
    excludes: Box<[PropertyName]>,
    /// Ingestion timestamp (private - not exposed in public API).
    #[rkyv(with = AsUnixTime)]
    recorded_at: SystemTime,
}

impl BaseSchema {
    /// Creates a new `BaseSchema` with current timestamp.
    ///
    /// `extends` and `excludes` are deduplicated via sort + dedup and stored in
    /// a deterministic order.
    ///
    /// # Examples
    ///
    /// ```
    /// use trace_schema::{
    ///     base::BaseSchema,
    ///     identifier::{SchemaId, SchemaName},
    ///     property::PropertyMap,
    /// };
    ///
    /// let id = SchemaId::new();
    /// let name = SchemaName::try_new("child")?;
    /// let parent = SchemaName::try_new("parent")?;
    /// let base = BaseSchema::new(
    ///     id,
    ///     name,
    ///     PropertyMap::new(),
    ///     vec![parent.clone()],
    ///     Vec::new(),
    /// );
    /// assert_eq!(base.extends(), &[parent]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        id: SchemaId,
        name: SchemaName,
        properties: PropertyMap,
        mut extends: Vec<SchemaName>,
        mut excludes: Vec<PropertyName>,
    ) -> Self {
        extends.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        extends.dedup();

        excludes.sort();
        excludes.dedup();

        Self {
            id,
            name,
            properties,
            extends: extends.into_boxed_slice(),
            excludes: excludes.into_boxed_slice(),
            recorded_at: SystemTime::now(),
        }
    }

    /// Returns the schema ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> &SchemaId {
        &self.id
    }

    /// Returns the schema name.
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &SchemaName {
        &self.name
    }

    /// Returns the direct-declared properties (pre-inheritance).
    #[inline]
    #[must_use]
    pub const fn properties(&self) -> &PropertyMap {
        &self.properties
    }

    /// Returns the file-local parent schema names declared in the source
    /// file's `extends` list.
    ///
    /// Mirrors the schema source file's `extends` field. The resolved
    /// cross-schema concept (parent [`SchemaId`]s) is exposed on
    /// [`Schema::parents`](super::aggregate::Schema::parents) instead.
    #[inline]
    #[must_use]
    pub fn extends(&self) -> &[SchemaName] {
        &self.extends
    }

    /// Returns the inherited property names to omit when resolving the
    /// child schema.
    #[inline]
    #[must_use]
    pub fn excludes(&self) -> &[PropertyName] {
        &self.excludes
    }

    /// Replaces the direct-declared properties.
    #[inline]
    #[must_use]
    pub fn set_properties(mut self, properties: PropertyMap) -> Self {
        self.properties = properties;
        self
    }

    /// Replaces the file-local parent schema names.
    #[inline]
    #[must_use]
    pub fn set_extends(mut self, mut extends: Vec<SchemaName>) -> Self {
        extends.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        extends.dedup();
        self.extends = extends.into_boxed_slice();
        self
    }

    /// Replaces the inherited property names to omit.
    #[inline]
    #[must_use]
    pub fn set_excludes(mut self, mut excludes: Vec<PropertyName>) -> Self {
        excludes.sort();
        excludes.dedup();
        self.excludes = excludes.into_boxed_slice();
        self
    }
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    mod fixtures {
        use super::*;

        pub(crate) fn schema_name(s: &str) -> SchemaName {
            SchemaName::try_new(s).expect("valid test schema name")
        }

        pub(crate) fn property_name(s: &str) -> PropertyName {
            PropertyName::try_new(s).expect("valid test property name")
        }
    }

    mod constructor {
        use super::{fixtures, *};

        #[test]
        fn new_stores_provided_id() {
            let id = SchemaId::new();
            let base = BaseSchema::new(
                id,
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            assert_eq!(base.id(), &id);
        }

        #[test]
        fn new_stores_provided_name() {
            let name = fixtures::schema_name("child");
            let base = BaseSchema::new(
                SchemaId::new(),
                name.clone(),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            assert_eq!(base.name(), &name);
        }

        #[test]
        fn new_defaults_to_empty_extends() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            assert!(base.extends().is_empty());
        }

        #[test]
        fn new_defaults_to_empty_excludes() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            assert!(base.excludes().is_empty());
        }

        #[test]
        fn new_deduplicates_extends() {
            let parent = fixtures::schema_name("parent");
            let other = fixtures::schema_name("other");

            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                vec![parent.clone(), other.clone(), parent],
                Vec::new(),
            );

            assert_eq!(base.extends().len(), 2);
            assert!(base.extends().contains(&fixtures::schema_name("parent")));
            assert!(base.extends().contains(&fixtures::schema_name("other")));
        }

        #[test]
        fn new_deduplicates_excludes() {
            let title = fixtures::property_name("title");
            let body = fixtures::property_name("body");

            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                vec![title.clone(), body.clone(), title],
            );

            assert_eq!(base.excludes().len(), 2);
            assert!(
                base.excludes().contains(&fixtures::property_name("title"))
            );
            assert!(base.excludes().contains(&fixtures::property_name("body")));
        }

        #[test]
        fn new_accepts_empty_property_map() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            assert!(base.properties().is_empty());
        }

        #[test]
        fn new_accepts_self_in_extends_for_resolver_to_reject_later() {
            let name = fixtures::schema_name("child");

            let base = BaseSchema::new(
                SchemaId::new(),
                name.clone(),
                PropertyMap::new(),
                vec![name.clone()],
                Vec::new(),
            );

            // Phase-1 BaseSchema is file-local. Cycle detection happens
            // in the inheritance resolver (slice 02+), not here.
            assert_eq!(base.extends(), &[name]);
        }

        #[test]
        fn equal_base_schemas_compare_equal_regardless_of_extends_excludes_order()
         {
            let base1 = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                vec![
                    fixtures::schema_name("parent"),
                    fixtures::schema_name("other"),
                ],
                vec![
                    fixtures::property_name("title"),
                    fixtures::property_name("body"),
                ],
            );
            let base2 = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                vec![
                    fixtures::schema_name("other"),
                    fixtures::schema_name("parent"),
                ],
                vec![
                    fixtures::property_name("body"),
                    fixtures::property_name("title"),
                ],
            );

            assert_eq!(base1.extends(), base2.extends());
            assert_eq!(base1.excludes(), base2.excludes());
        }
    }

    mod accessors {
        use super::{fixtures, *};

        #[test]
        fn id_returns_borrowed_schema_id() {
            let id = SchemaId::new();
            let base = BaseSchema::new(
                id,
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );

            let borrowed: &SchemaId = base.id();
            assert_eq!(borrowed, &id);
        }

        #[test]
        fn name_returns_borrowed_schema_name() {
            let name = fixtures::schema_name("child");
            let base = BaseSchema::new(
                SchemaId::new(),
                name.clone(),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );

            let borrowed: &SchemaName = base.name();
            assert_eq!(borrowed, &name);
        }

        #[test]
        fn properties_returns_borrowed_property_map() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );

            let borrowed: &PropertyMap = base.properties();
            assert!(borrowed.is_empty());
        }

        #[test]
        fn extends_returns_borrowed_schema_name_slice() {
            let parent = fixtures::schema_name("parent");
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                vec![parent.clone()],
                Vec::new(),
            );

            let borrowed: &[SchemaName] = base.extends();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(
                borrowed.first().expect("extends has one entry"),
                &parent
            );
        }

        #[test]
        fn excludes_returns_borrowed_property_name_slice() {
            let title = fixtures::property_name("title");
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                vec![title.clone()],
            );

            let borrowed: &[PropertyName] = base.excludes();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(
                borrowed.first().expect("excludes has one entry"),
                &title
            );
        }
    }

    mod cloning {
        use super::{fixtures, *};

        #[test]
        fn clone_preserves_identity() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            let cloned = base.clone();
            assert_eq!(cloned.id(), base.id());
            assert_eq!(cloned.name(), base.name());
        }

        #[test]
        fn clone_preserves_extends() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                vec![fixtures::schema_name("parent")],
                Vec::new(),
            );
            let cloned = base.clone();
            assert_eq!(cloned.extends(), base.extends());
        }

        #[test]
        fn clone_preserves_excludes() {
            let base = BaseSchema::new(
                SchemaId::new(),
                fixtures::schema_name("child"),
                PropertyMap::new(),
                Vec::new(),
                vec![fixtures::property_name("title")],
            );
            let cloned = base.clone();
            assert_eq!(cloned.excludes(), base.excludes());
        }
    }

    mod serialization {
        use super::{fixtures, *};

        #[test]
        fn roundtrips_via_rkyv_archive() {
            let id = SchemaId::new();
            let name = fixtures::schema_name("child");
            let base = BaseSchema::new(
                id,
                name.clone(),
                PropertyMap::new(),
                vec![fixtures::schema_name("parent")],
                vec![fixtures::property_name("title")],
            );

            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&base)
                .expect("rkyv serialization succeeds");
            let deserialized: BaseSchema =
                rkyv::from_bytes::<BaseSchema, rkyv::rancor::Error>(&bytes)
                    .expect("rkyv deserialization succeeds");

            assert_eq!(deserialized.id(), &id);
            assert_eq!(deserialized.name().as_str(), name.as_str());
            assert_eq!(deserialized.extends().len(), 1);
            assert_eq!(deserialized.excludes().len(), 1);
        }

        #[test]
        fn roundtrips_via_rkyv_archive_when_all_collections_empty() {
            let id = SchemaId::new();
            let name = fixtures::schema_name("child");
            let base = BaseSchema::new(
                id,
                name.clone(),
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );

            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&base)
                .expect("rkyv serialization succeeds");
            let deserialized: BaseSchema =
                rkyv::from_bytes::<BaseSchema, rkyv::rancor::Error>(&bytes)
                    .expect("rkyv deserialization succeeds");

            assert_eq!(deserialized.id(), &id);
            assert_eq!(deserialized.name().as_str(), name.as_str());
            assert!(deserialized.extends().is_empty());
            assert!(deserialized.excludes().is_empty());
            assert!(deserialized.properties().is_empty());
        }
    }
}
