//! `SchemaResolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent
//! properties, applying excludes, resolving $ref pointers through the
//! `PropertyBank`, and enforcing inheritance ordering.
//!
//! # Optimization Notes
//!
//! - **Sorted Merge**: Properties are merged using a two-pointer walk instead
//!   of hashing, leveraging the sorted invariant of `Schema`.
//! - **Forest Traversal**: Inheritance is treated as a forest (single
//!   inheritance), simplifying topological ordering.
//! - **Stateless Service**: The resolver holds no transient state between
//!   calls, improving thread safety and clarity.

use std::collections::{HashMap, HashSet};

use super::{
    aggregate::{
        ResolutionMetadata, Schema, SchemaHash, SchemaId, SchemaName, Timestamp,
    },
    bank::{BankVersion, PropertyBank},
    error::SchemaError,
    property::{
        Cardinality, Multiplicity, Property, PropertyId, PropertyName,
        PropertyRef,
    },
    raw::{RawPropertyEntry, RawPropertyRef, RawSchema},
};

/// Domain Service: Resolves raw schemas into fully resolved Schema entities.
pub struct SchemaResolver<'bank> {
    bank: &'bank PropertyBank,
}

impl<'bank> SchemaResolver<'bank> {
    /// Create a new resolver with a `PropertyBank` reference.
    #[inline]
    #[must_use]
    pub const fn new(bank: &'bank PropertyBank) -> Self {
        Self {
            bank,
        }
    }

    /// Process a set of raw schemas into fully resolved schemas.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails (cycles, missing parents,
    /// etc).
    #[inline]
    pub fn process(
        &self,
        raw_schemas: Vec<(SchemaId, RawSchema)>,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError> {
        // Convert to format with timestamps (all None)
        let with_mtimes =
            raw_schemas.into_iter().map(|(id, raw)| (id, raw, None)).collect();
        self.process_with_loader(with_mtimes, None, |_| Ok(None))
    }

    /// Process only changed schemas (incremental resolution).
    ///
    /// Each raw schema is accompanied by its file modification time.
    /// Schemas that are not stale (bank version, parent hash, file mtime
    /// unchanged) are skipped and loaded from the database via
    /// `parent_loader`.
    ///
    /// # Errors
    /// Returns `SchemaError` if resolution fails.
    #[inline]
    pub fn process_changed<F>(
        &self,
        raw_schemas: Vec<(SchemaId, RawSchema, Option<Timestamp>)>,
        existing_metadata: &[ResolutionMetadata],
        parent_loader: F,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        self.process_with_loader(
            raw_schemas,
            Some(existing_metadata),
            parent_loader,
        )
    }

    fn process_with_loader<F>(
        &self,
        raw_schemas: Vec<(SchemaId, RawSchema, Option<Timestamp>)>,
        existing_metadata: Option<&[ResolutionMetadata]>,
        parent_loader: F,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let mut ctx = ResolutionContext::new(self.bank, raw_schemas.len());

        // 1. Index and build forest (includes staleness checking)
        ctx.build_forest(raw_schemas, existing_metadata, &parent_loader)?;

        // 2. Compute topological order (roots to leaves)
        let order = ctx.compute_order(&parent_loader)?;

        // 3. Resolve in order
        let mut results = Vec::with_capacity(order.len());
        for id in order {
            let (schema, metadata) = ctx.resolve_one(id, &parent_loader)?;
            results.push((schema, metadata));
        }

        Ok(results)
    }
}

/// Transient state for a single resolution session.
struct ResolutionContext<'bank> {
    bank: &'bank PropertyBank,
    /// Forest of Child -> Parent relationships.
    forest: HashMap<SchemaId, Option<SchemaId>>,
    /// Lookup for `RawSchema` by ID.
    raw_by_id: HashMap<SchemaId, RawSchema>,
    /// Lookup for `SchemaName` by ID (for errors/sorting).
    names: HashMap<SchemaId, SchemaName>,
    /// Cache of resolved schemas within this session.
    resolved_cache: HashMap<SchemaId, Schema>,
    /// File modification times for incremental resolution.
    file_mtimes: HashMap<SchemaId, Option<Timestamp>>,
}

impl<'bank> ResolutionContext<'bank> {
    fn new(bank: &'bank PropertyBank, capacity: usize) -> Self {
        Self {
            bank,
            forest: HashMap::with_capacity(capacity),
            raw_by_id: HashMap::with_capacity(capacity),
            names: HashMap::with_capacity(capacity),
            resolved_cache: HashMap::with_capacity(capacity),
            file_mtimes: HashMap::new(),
        }
    }

    fn build_forest<F>(
        &mut self,
        raw_schemas: Vec<(SchemaId, RawSchema, Option<Timestamp>)>,
        existing_metadata: Option<&[ResolutionMetadata]>,
        parent_loader: &F,
    ) -> Result<(), SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        // Build lookup for existing metadata by ID
        let metadata_by_id: HashMap<SchemaId, &ResolutionMetadata> =
            existing_metadata
                .map(|m| {
                    m.iter().map(|meta| (meta.schema_id(), meta)).collect()
                })
                .unwrap_or_default();

        // Get current bank version for staleness checking
        let current_bank_version = self.bank.version();

        let mut name_to_id = HashMap::with_capacity(raw_schemas.len());

        for (id, raw, file_mtime) in raw_schemas {
            let name = SchemaName::try_from(raw.name.as_ref())?;

            if name_to_id.insert(name.clone(), id).is_some() {
                return Err(SchemaError::AlreadyExists(name.to_string()));
            }

            self.names.insert(id, name);

            // Check staleness - determine if we need to resolve this schema
            let existing_meta = metadata_by_id.get(&id).copied();
            let should_resolve = self.check_staleness(
                id,
                &raw,
                file_mtime,
                existing_meta,
                &name_to_id,
                current_bank_version,
                parent_loader,
            )?;

            if should_resolve {
                self.raw_by_id.insert(id, raw);
                self.file_mtimes.insert(id, file_mtime);
            }
        }

        // Second pass to resolve parent pointers within the batch
        let mut sorted_ids: Vec<_> = self.raw_by_id.keys().copied().collect();
        sorted_ids.sort();

        for id in sorted_ids {
            let raw = self.raw_by_id.get(&id).ok_or_else(|| {
                SchemaError::NotFound(format!("Schema ID {id} missing"))
            })?;
            let parent_id = if let Some(parent_name_str) = raw.extends.as_ref()
            {
                let parent_name =
                    SchemaName::try_from(parent_name_str.as_ref())?;
                name_to_id.get(&parent_name).copied()
            } else {
                None
            };
            self.forest.insert(id, parent_id);
        }

        Ok(())
    }

    /// Check if a schema needs to be resolved or can be loaded from DB.
    ///
    /// Returns `true` if resolution is needed, `false` if loaded from cache.
    #[expect(
        clippy::too_many_arguments,
        reason = "Internal helper with clear parameter grouping"
    )]
    fn check_staleness<F>(
        &mut self,
        id: SchemaId,
        raw: &RawSchema,
        file_mtime: Option<Timestamp>,
        existing_meta: Option<&ResolutionMetadata>,
        name_to_id: &HashMap<SchemaName, SchemaId>,
        current_bank_version: BankVersion,
        parent_loader: &F,
    ) -> Result<bool, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let Some(existing_meta) = existing_meta else {
            return Ok(true);
        };

        // Compute parent hash for staleness check
        let parent_hash = raw.extends.as_ref().and_then(|parent_name_str| {
            let parent_name = SchemaName::new(parent_name_str).ok()?;
            let parent_id = name_to_id.get(&parent_name)?;
            self.resolved_cache.get(parent_id).map(SchemaHash::compute)
        });

        let is_stale = existing_meta.is_stale(
            current_bank_version,
            parent_hash,
            file_mtime,
        );

        if !is_stale {
            // Non-stale: try to load from DB
            if let Some(schema) = parent_loader(&id)? {
                self.resolved_cache.insert(id, schema);
                return Ok(false);
            }
            // Not found in DB, must resolve
        }

        Ok(true)
    }

    fn compute_order<F>(
        &self,
        parent_loader: &F,
    ) -> Result<Vec<SchemaId>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let mut state = SortState {
            order: Vec::with_capacity(self.forest.len()),
            visited: HashSet::with_capacity(self.forest.len()),
            temp_visited: HashSet::with_capacity(8),
        };

        // Sort keys by name for deterministic ordering
        let mut ids: Vec<_> = self.forest.keys().copied().collect();
        ids.sort_by(|a, b| {
            let name_a = self.names.get(a).map_or("", SchemaName::as_str);
            let name_b = self.names.get(b).map_or("", SchemaName::as_str);
            name_a.cmp(name_b)
        });

        for id in ids {
            self.visit(id, &mut state, parent_loader)?;
        }

        Ok(state.order)
    }

    fn visit<F>(
        &self,
        id: SchemaId,
        state: &mut SortState,
        parent_loader: &F,
    ) -> Result<(), SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        if state.visited.contains(&id) {
            return Ok(());
        }

        if !state.temp_visited.insert(id) {
            let name = self
                .names
                .get(&id)
                .map_or_else(|| "unknown".to_owned(), ToString::to_string);
            return Err(SchemaError::CircularInheritance(name));
        }

        // If this schema extends another, visit the parent first
        if let Some(parent_id) = self.forest.get(&id).copied().flatten() {
            if self.forest.contains_key(&parent_id) {
                self.visit(parent_id, state, parent_loader)?;
            } else if let Ok(Some(_)) = parent_loader(&parent_id) {
                // External parent exists, nothing to do
            } else {
                // We need the name of the missing parent, but we only
                // have ID This is a cold error path.
                let extends_name =
                    self.raw_by_id.get(&id).and_then(|r| r.extends.clone());
                return Err(SchemaError::ParentNotFound(
                    extends_name
                        .map_or_else(|| "unknown".to_owned(), String::from),
                ));
            }
        }

        state.temp_visited.remove(&id);
        state.visited.insert(id);
        state.order.push(id);
        Ok(())
    }

    fn resolve_one<F>(
        &mut self,
        id: SchemaId,
        parent_loader: &F,
    ) -> Result<(Schema, ResolutionMetadata), SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let raw = self.raw_by_id.remove(&id).ok_or_else(|| {
            SchemaError::NotFound(format!(
                "Raw schema for {id} missing during resolution"
            ))
        })?;
        let parent = self.load_parent(&id, parent_loader)?;

        let schema = self.resolve_single(id, raw, parent.as_ref())?;
        let metadata = self.create_metadata(&schema, parent.as_ref());

        self.resolved_cache.insert(id, schema.clone());
        Ok((schema, metadata))
    }

    fn load_parent<F>(
        &self,
        id: &SchemaId,
        parent_loader: &F,
    ) -> Result<Option<Schema>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        if let Some(parent_id) = self.forest.get(id).copied().flatten() {
            if let Some(cached) = self.resolved_cache.get(&parent_id) {
                return Ok(Some(cached.clone()));
            }
            return parent_loader(&parent_id);
        }
        Ok(None)
    }

    fn resolve_single(
        &self,
        id: SchemaId,
        raw: RawSchema,
        parent: Option<&Schema>,
    ) -> Result<Schema, SchemaError> {
        let mut own_props = Vec::with_capacity(raw.properties.len());
        // Sort property names for deterministic ordering
        let mut sorted_entries: Vec<_> = raw.properties.into_iter().collect();
        sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, entry) in sorted_entries {
            own_props.push(self.resolve_property(&name, entry)?);
        }
        // Properties are already sorted by name

        let excludes: HashSet<PropertyName> = raw
            .excludes
            .iter()
            .map(|s| PropertyName::try_from(s.as_ref()))
            .collect::<Result<_, _>>()?;

        let final_props = if let Some(p) = parent {
            merge_sorted_properties(p.properties(), &own_props, &excludes)
        } else {
            own_props
        };

        let name = self.names.get(&id).cloned().ok_or_else(|| {
            SchemaError::NotFound(format!("Name for {id} missing"))
        })?;
        Schema::new(id, name, final_props)
    }

    fn resolve_property(
        &self,
        name: &str,
        entry: RawPropertyEntry,
    ) -> Result<Property, SchemaError> {
        match entry {
            RawPropertyEntry::Inline(inline) => {
                let prop_name = PropertyName::new(name)?;
                let spec = inline.spec.try_into_validated()?;
                let cardinality = if inline.required {
                    Cardinality::Required
                } else {
                    Cardinality::Optional
                };
                let multiplicity = if inline.multi {
                    Multiplicity::Many
                } else {
                    Multiplicity::Single
                };
                Property::new(
                    PropertyId::new(),
                    prop_name,
                    cardinality,
                    multiplicity,
                    spec,
                )
            }
            RawPropertyEntry::Ref(RawPropertyRef {
                ref_path,
                required,
                multi,
                options: _options,
                pattern: _pattern,
                min: _min,
                max: _max,
                step: _step,
                format: _format,
                directory: _directory,
                file_class: _file_class,
            }) => {
                let prop_ref = PropertyRef::try_from(ref_path.as_ref())?;
                let base_prop = match prop_ref {
                    PropertyRef::ById(id) => {
                        self.bank.get_by_id(id).cloned().ok_or_else(|| {
                            SchemaError::PropertyRefNotFound(
                                ref_path.to_string(),
                            )
                        })?
                    }
                    PropertyRef::ByName(bank_name) => {
                        self.bank.get_by_name(&bank_name).cloned().ok_or_else(
                            || {
                                SchemaError::PropertyRefNotFound(
                                    ref_path.to_string(),
                                )
                            },
                        )?
                    }
                };

                // Apply overrides
                let cardinality =
                    required.map_or(base_prop.cardinality(), |r| {
                        if r {
                            Cardinality::Required
                        } else {
                            Cardinality::Optional
                        }
                    });

                let multiplicity =
                    multi.map_or(base_prop.multiplicity(), |m| {
                        if m {
                            Multiplicity::Many
                        } else {
                            Multiplicity::Single
                        }
                    });

                // TODO: Handle type-specific overrides (options, pattern, min,
                // max, step, format, directory, file_class)
                // This requires updating PropertySpec to support merging
                // overrides

                Property::new(
                    base_prop.id(),
                    PropertyName::new(name)?,
                    cardinality,
                    multiplicity,
                    base_prop.spec().clone(),
                )
            }
        }
    }

    fn create_metadata(
        &self,
        schema: &Schema,
        parent: Option<&Schema>,
    ) -> ResolutionMetadata {
        ResolutionMetadata::new(
            schema.id(),
            Timestamp::now(),
            parent.map(SchemaHash::compute),
            self.bank.version(),
            self.file_mtimes.get(&schema.id()).copied().flatten(),
        )
    }
}

struct SortState {
    order: Vec<SchemaId>,
    visited: HashSet<SchemaId>,
    temp_visited: HashSet<SchemaId>,
}

/// Merges two sorted slices of properties into a single sorted vector.
///
/// Implements inheritance logic:
/// 1. Child properties override parent properties with the same name.
/// 2. Parent properties are omitted if they are in the `excludes` set.
fn merge_sorted_properties(
    parent: &[Property],
    child: &[Property],
    excludes: &HashSet<PropertyName>,
) -> Vec<Property> {
    // Capacity check to avoid potential overflow in with_capacity
    let capacity = parent.len().saturating_add(child.len());
    let mut result = Vec::with_capacity(capacity);
    let mut p_iter = parent.iter().peekable();
    let mut c_iter = child.iter().peekable();

    loop {
        match (p_iter.peek(), c_iter.peek()) {
            (Some(&p), Some(&c)) => {
                use std::cmp::Ordering;
                match p.name().as_str().cmp(c.name().as_str()) {
                    Ordering::Less => {
                        if !excludes.contains(p.name()) {
                            result.push((*p).clone());
                        }
                        p_iter.next();
                    }
                    Ordering::Greater => {
                        result.push((*c).clone());
                        c_iter.next();
                    }
                    Ordering::Equal => {
                        // Child overrides parent
                        result.push((*c).clone());
                        p_iter.next();
                        c_iter.next();
                    }
                }
            }
            (Some(&p), None) => {
                if !excludes.contains(p.name()) {
                    result.push((*p).clone());
                }
                p_iter.next();
            }
            (None, Some(&c)) => {
                result.push((*c).clone());
                c_iter.next();
            }
            (None, None) => break,
        }
    }

    result
}

#[cfg(test)]
mod tests {
    mod fixtures {
        use std::collections::HashMap;

        use super::*;

        pub fn parent_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("parent")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_PARENT),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn status_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("status")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_STATUS),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn excluded_property() -> Result<Property, SchemaError> {
            let name = PropertyName::new("p")?;
            Property::new(
                PropertyId::from_uuid(TEST_PROPERTY_ID_EXCLUDE),
                name,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )
        }

        pub fn parent_schema_with_property(
            property: Property,
        ) -> Result<Schema, SchemaError> {
            let name = SchemaName::new("parent")?;
            Schema::new(SchemaId::from_uuid(TEST_SCHEMA_ID_PARENT), name, vec![
                property,
            ])
        }

        pub fn child_raw_schema() -> (SchemaId, RawSchema) {
            (SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD), RawSchema {
                name: "child".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            })
        }

        pub fn child_raw_schema_with_excludes(
            exclude_name: &PropertyName,
        ) -> (SchemaId, RawSchema) {
            (SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD), RawSchema {
                name: "child".into(),
                extends: None,
                excludes: vec![exclude_name.as_str().into()],
                properties: HashMap::new(),
            })
        }

        pub fn property_bank_with(
            property: Property,
        ) -> Result<PropertyBank, SchemaError> {
            let mut bank = PropertyBank::new();
            bank.register(property)?;
            Ok(bank)
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn includes_parent_properties() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let property = fixtures::parent_property()?;
            let parent_schema =
                fixtures::parent_schema_with_property(property)?;
            let (id, raw) = fixtures::child_raw_schema();

            // Internal use of context for testing single resolution
            let mut ctx = ResolutionContext::new(&bank, 1);
            ctx.names.insert(id, SchemaName::new("child")?);
            let schema = ctx.resolve_single(id, raw, Some(&parent_schema))?;

            let name = PropertyName::new("parent")?;
            if !schema.has(&name) {
                return Err(SchemaError::ValidationFailed(
                    "Resolved schema should include parent property".to_owned(),
                ));
            }
            Ok(())
        }

        #[test]
        fn excludes_properties_listed_in_child() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let property = fixtures::excluded_property()?;
            let parent_schema =
                fixtures::parent_schema_with_property(property)?;
            let exclude_name = PropertyName::new("p")?;
            let (id, raw) =
                fixtures::child_raw_schema_with_excludes(&exclude_name);

            let mut ctx = ResolutionContext::new(&bank, 1);
            ctx.names.insert(id, SchemaName::new("child")?);
            let schema = ctx.resolve_single(id, raw, Some(&parent_schema))?;

            if schema.has(&exclude_name) {
                return Err(SchemaError::ValidationFailed(
                    "Resolved schema should exclude child-listed property"
                        .to_owned(),
                ));
            }
            Ok(())
        }
    }

    mod resolution_logic {
        use super::*;
        use crate::schema::raw::RawPropertyEntry;

        #[test]
        fn resolves_ref_property_by_plain_name() -> Result<(), SchemaError> {
            let property = fixtures::status_property()?;
            let bank = fixtures::property_bank_with(property)?;
            let entry = RawPropertyEntry::Ref(RawPropertyRef {
                ref_path: "status".into(),
                required: None,
                multi: None,
                options: None,
                pattern: None,
                min: None,
                max: None,
                step: None,
                format: None,
                directory: None,
                file_class: None,
            });
            let ctx = ResolutionContext::new(&bank, 0);
            let prop = ctx.resolve_property("status", entry)?;

            if prop.name().as_str() != "status" {
                return Err(SchemaError::ValidationFailed(
                    "Resolved property name should match".to_owned(),
                ));
            }
            Ok(())
        }
    }

    mod merge {
        use super::*;

        #[test]
        fn merge_sorted_properties_handles_overrides() -> Result<(), SchemaError>
        {
            let p1 = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new("a")?,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )?;
            let p2 = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new("b")?,
                Cardinality::Required,
                Multiplicity::Single,
                PropertySpec::Bool(BoolSpec::default()),
            )?;
            let p2_override = Property::new(
                PropertyId::from_uuid(Uuid::now_v7()),
                PropertyName::new("b")?,
                Cardinality::Optional,
                Multiplicity::Many,
                PropertySpec::Bool(BoolSpec::default()),
            )?;

            let parent = vec![p1, p2];
            let child = vec![p2_override.clone()];
            let merged =
                merge_sorted_properties(&parent, &child, &HashSet::new());

            if merged.len() != 2 {
                return Err(SchemaError::ValidationFailed(format!(
                    "Expected 2 properties, got {}",
                    merged.len()
                )));
            }
            if merged.first().map_or("", |p| p.name().as_str()) != "a" {
                return Err(SchemaError::ValidationFailed(
                    "First property should be 'a'".to_owned(),
                ));
            }
            if merged.get(1).map_or("", |p| p.name().as_str()) != "b" {
                return Err(SchemaError::ValidationFailed(
                    "Second property should be 'b'".to_owned(),
                ));
            }
            if merged.get(1).map(Property::multiplicity)
                != Some(Multiplicity::Many)
            {
                return Err(SchemaError::ValidationFailed(
                    "Second property should have Many multiplicity".to_owned(),
                ));
            }
            Ok(())
        }
    }

    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::{SchemaId, SchemaName},
        property_spec::{BoolSpec, PropertySpec},
    };

    const TEST_SCHEMA_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0501);
    const TEST_SCHEMA_ID_CHILD: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0502);
    const TEST_PROPERTY_ID_PARENT: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0503);
    const TEST_PROPERTY_ID_STATUS: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0504);
    const TEST_PROPERTY_ID_EXCLUDE: Uuid =
        Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0505);
}
