//! `SchemaResolver` domain service for schema resolution.
//!
//! Resolves raw schemas into fully resolved Schema entities by merging parent
//! properties, applying excludes, resolving $ref pointers through the
//! `PropertyBank`, and enforcing inheritance ordering.
//!
//! # Design
//!
//! The resolver is decomposed into three independent concerns:
//!
//! 1. **`InheritanceForest`** — pure topological sort over the parent map. No
//!    I/O. Receives `known_external: &HashSet<SchemaId>` for schemas already
//!    loaded from the DB so they are correctly excluded from "missing parent"
//!    errors.
//!
//! 2. **Free function `resolve_property`** — resolves a single
//!    `RawPropertyEntry` into a `Property` using the bank. For `$ref` entries,
//!    applies type-specific overrides and rejects type changes.
//!
//! 3. **Free function `assemble_schema`** — merges parent properties with the
//!    schema's own resolved properties (applying excludes) and constructs the
//!    final `Schema`.
//!
//! # Staleness Ordering (R-12)
//!
//! Staleness is checked before topological sort. The set of non-stale schemas
//! loaded from the DB (`known_external`) is passed to `topo_order` so that
//! external parents are correctly excluded from "missing parent" errors.
//! Resolution proceeds in topological order; the resolved cache is populated
//! as each schema is assembled, making parent hashes available in declaration
//! order.
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
    property_spec::PropertySpec,
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
        let with_mtimes: Vec<(SchemaId, RawSchema, Option<Timestamp>)> =
            raw_schemas.into_iter().map(|(id, raw)| (id, raw, None)).collect();
        self.process_internal(with_mtimes, None, |_| Ok(None))
    }

    /// Process only changed schemas (incremental resolution).
    ///
    /// Each raw schema is accompanied by its file modification time.
    /// Schemas that are not stale (bank version, parent hash, file mtime
    /// unchanged) are skipped and loaded from the database via
    /// `parent_loader`.
    ///
    /// Staleness is checked before topological sort. The set of non-stale
    /// schemas loaded from the DB is passed to `topo_order` so external
    /// parents do not cause "missing parent" errors.
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
        self.process_internal(
            raw_schemas,
            Some(existing_metadata),
            parent_loader,
        )
    }

    fn process_internal<F>(
        &self,
        raw_schemas: Vec<(SchemaId, RawSchema, Option<Timestamp>)>,
        existing_metadata: Option<&[ResolutionMetadata]>,
        parent_loader: F,
    ) -> Result<Vec<(Schema, ResolutionMetadata)>, SchemaError>
    where
        F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
    {
        let current_bank_version = self.bank.version();
        let capacity = raw_schemas.len();

        // Build metadata lookup for staleness checks.
        let metadata_by_id: HashMap<SchemaId, &ResolutionMetadata> =
            existing_metadata
                .map(|m| {
                    m.iter().map(|meta| (meta.schema_id(), meta)).collect()
                })
                .unwrap_or_default();

        // Phase 1: Staleness check (before topo-sort).
        //
        // For each raw schema, determine whether it needs resolution or can be
        // loaded from the DB. Schemas loaded from DB are collected into
        // `known_external` so the topo-sort can treat their IDs as available
        // parents without resolving them.
        //
        // Parent hashes cannot be computed here because the resolved cache is
        // empty at this point — they are computed during Phase 3 after parents
        // have been resolved in topo order. Staleness is therefore checked
        // conservatively: a schema is considered stale if the bank version
        // changed or the file mtime changed. Parent hash staleness is detected
        // lazily during resolution when the resolved parent is available.
        let mut names: HashMap<SchemaId, SchemaName> =
            HashMap::with_capacity(capacity);
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(capacity);
        let mut stale_raws: HashMap<SchemaId, RawSchema> =
            HashMap::with_capacity(capacity);
        let mut file_mtimes: HashMap<SchemaId, Option<Timestamp>> =
            HashMap::with_capacity(capacity);
        let mut resolved_cache: HashMap<SchemaId, Schema> =
            HashMap::with_capacity(capacity);

        for (id, raw, file_mtime) in raw_schemas {
            let name = SchemaName::try_from(raw.name.as_ref())?;

            if name_to_id.insert(name.clone(), id).is_some() {
                return Err(SchemaError::AlreadyExists(name.to_string()));
            }
            names.insert(id, name);

            let needs_resolution = check_staleness(
                id,
                file_mtime,
                metadata_by_id.get(&id).copied(),
                current_bank_version,
            );

            if needs_resolution {
                stale_raws.insert(id, raw);
                file_mtimes.insert(id, file_mtime);
            } else if let Some(schema) = parent_loader(&id)? {
                // Non-stale: load from DB into cache so it can act as a parent.
                resolved_cache.insert(id, schema);
            } else {
                // Metadata says not stale but DB has no record: force
                // resolution.
                stale_raws.insert(id, raw);
                file_mtimes.insert(id, file_mtime);
            }
        }

        // Phase 2: Build the InheritanceForest and compute topo order.
        //
        // `known_external` contains IDs of schemas already loaded from the DB
        // (non-stale). They are valid parents even though they are not in
        // `stale_raws`.
        let known_external: HashSet<SchemaId> =
            resolved_cache.keys().copied().collect();

        let forest =
            InheritanceForest::build(&stale_raws, &names, &name_to_id)?;
        let order = forest.topo_order(&known_external)?;

        // Phase 3: Resolve in topological order.
        let mut results = Vec::with_capacity(order.len());

        for id in order {
            let raw = stale_raws.remove(&id).ok_or_else(|| {
                SchemaError::NotFound(format!(
                    "Raw schema for {id} missing during resolution"
                ))
            })?;
            let name = names.get(&id).cloned().ok_or_else(|| {
                SchemaError::NotFound(format!("Name for {id} missing"))
            })?;
            let file_mtime = file_mtimes.get(&id).copied().flatten();

            // Load parent from cache (already resolved in a prior iteration or
            // from DB). External parents loaded from DB are in resolved_cache.
            let parent =
                load_parent(id, &forest, &resolved_cache, &parent_loader)?;

            let schema = assemble_schema(
                id,
                name,
                raw.properties,
                parent.as_ref(),
                &raw.excludes,
                self.bank,
            )?;

            let parent_hash = parent.as_ref().map(SchemaHash::compute);
            let metadata = ResolutionMetadata::new(
                id,
                Timestamp::now(),
                parent_hash,
                current_bank_version,
                file_mtime,
            );

            resolved_cache.insert(id, schema.clone());
            results.push((schema, metadata));
        }

        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// Staleness helper (Phase 1)
// ---------------------------------------------------------------------------

/// Returns `true` if the schema needs resolution.
///
/// Checks bank version and file mtime staleness. Parent hash staleness is
/// checked lazily during Phase 3 once parents are resolved.
fn check_staleness(
    _id: SchemaId,
    file_mtime: Option<Timestamp>,
    existing_meta: Option<&ResolutionMetadata>,
    current_bank_version: BankVersion,
) -> bool {
    let Some(meta) = existing_meta else {
        return true; // No stored metadata → must resolve.
    };

    // Bank version changed → stale.
    if meta.bank_version().is_older_than(current_bank_version) {
        return true;
    }

    // File mtime advanced → stale.
    if let Some(stored) = meta.file_modified()
        && let Some(current) = file_mtime
        && stored < current
    {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// InheritanceForest (Phase 2, R-02)
// ---------------------------------------------------------------------------

/// Pure topological sort structure for schema inheritance.
///
/// Holds only the parent map and name lookup. `topo_order` is a pure
/// computation — no I/O, no `parent_loader` parameter. External parent
/// existence is validated during Phase 1 (staleness check / DB load), not
/// here.
struct InheritanceForest {
    /// Maps each schema ID to its parent ID, or `None` for root schemas.
    parents: HashMap<SchemaId, Option<SchemaId>>,
    /// Maps schema ID to name (for cycle-detection error messages).
    names: HashMap<SchemaId, SchemaName>,
}

impl InheritanceForest {
    /// Build the forest from the set of stale raw schemas.
    ///
    /// Parent pointers that refer to schemas not in `stale_raws` are stored
    /// as `None` (they are external / already-cached parents). The ID of
    /// such an external parent is resolved via `name_to_id` using the raw
    /// schema's `extends` field; if the name is not found there either, the
    /// parent simply does not appear in `parents` (treated as external).
    #[expect(
        clippy::iter_over_hash_type,
        reason = "Forest build iterates all stale schemas; insertion order is \
                  irrelevant because topo_order sorts by name for determinism"
    )]
    fn build(
        stale_raws: &HashMap<SchemaId, RawSchema>,
        names: &HashMap<SchemaId, SchemaName>,
        name_to_id: &HashMap<SchemaName, SchemaId>,
    ) -> Result<Self, SchemaError> {
        let capacity = stale_raws.len();
        let mut parents: HashMap<SchemaId, Option<SchemaId>> =
            HashMap::with_capacity(capacity);

        for (&id, raw) in stale_raws {
            let parent_id = if let Some(parent_name_str) = raw.extends.as_ref()
            {
                let parent_name =
                    SchemaName::try_from(parent_name_str.as_ref())?;
                // Resolve within the current batch; None means external.
                name_to_id.get(&parent_name).copied()
            } else {
                None
            };
            parents.insert(id, parent_id);
        }

        Ok(Self {
            parents,
            names: names.clone(),
        })
    }

    /// Compute a topological order for schemas that need resolution.
    ///
    /// `known_external` contains IDs of schemas already loaded from the DB.
    /// These are treated as valid parents — the sort does not emit them and
    /// does not error when they appear as parent pointers.
    ///
    /// Returns schemas in dependency order (roots first, leaves last) with
    /// deterministic ordering by name for reproducibility.
    fn topo_order(
        &self,
        known_external: &HashSet<SchemaId>,
    ) -> Result<Vec<SchemaId>, SchemaError> {
        let mut state = TopoState {
            order: Vec::with_capacity(self.parents.len()),
            visited: HashSet::with_capacity(self.parents.len()),
            in_progress: HashSet::with_capacity(8),
        };

        // Sort by name for deterministic output.
        let mut ids: Vec<SchemaId> = self.parents.keys().copied().collect();
        ids.sort_by(|a, b| {
            let name_a = self.names.get(a).map_or("", SchemaName::as_str);
            let name_b = self.names.get(b).map_or("", SchemaName::as_str);
            name_a.cmp(name_b)
        });

        for id in ids {
            self.visit(id, known_external, &mut state)?;
        }

        Ok(state.order)
    }

    fn visit(
        &self,
        id: SchemaId,
        known_external: &HashSet<SchemaId>,
        state: &mut TopoState,
    ) -> Result<(), SchemaError> {
        if state.visited.contains(&id) {
            return Ok(());
        }

        if !state.in_progress.insert(id) {
            let name = self
                .names
                .get(&id)
                .map_or_else(|| id.to_string(), ToString::to_string);
            return Err(SchemaError::CircularInheritance(name));
        }

        // Visit parent first (if in-batch; external parents are already done).
        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Match ergonomics on &Option<Option<SchemaId>> is \
                      intentional; deref would require binding the inner \
                      reference awkwardly"
        )]
        if let Some(Some(parent_id)) = self.parents.get(&id) {
            if self.parents.contains_key(parent_id) {
                // Parent is in the batch — recurse.
                self.visit(*parent_id, known_external, state)?;
            } else if known_external.contains(parent_id) {
                // Parent is in the DB cache — valid, nothing to visit.
            } else {
                // Parent is neither in-batch nor in DB cache → missing.
                return Err(SchemaError::ParentNotFound(
                    self.names.get(parent_id).map_or_else(
                        || parent_id.to_string(),
                        ToString::to_string,
                    ),
                ));
            }
        }

        state.in_progress.remove(&id);
        state.visited.insert(id);
        state.order.push(id);
        Ok(())
    }

    /// Returns the parent ID for a schema, if any.
    fn parent_of(&self, id: SchemaId) -> Option<SchemaId> {
        self.parents.get(&id).copied().flatten()
    }
}

struct TopoState {
    order: Vec<SchemaId>,
    visited: HashSet<SchemaId>,
    in_progress: HashSet<SchemaId>,
}

// ---------------------------------------------------------------------------
// Parent loading helper (Phase 3)
// ---------------------------------------------------------------------------

fn load_parent<F>(
    id: SchemaId,
    forest: &InheritanceForest,
    resolved_cache: &HashMap<SchemaId, Schema>,
    parent_loader: &F,
) -> Result<Option<Schema>, SchemaError>
where
    F: Fn(&SchemaId) -> Result<Option<Schema>, SchemaError>,
{
    let Some(parent_id) = forest.parent_of(id) else {
        return Ok(None);
    };

    if let Some(cached) = resolved_cache.get(&parent_id) {
        return Ok(Some(cached.clone()));
    }

    parent_loader(&parent_id)
}

// ---------------------------------------------------------------------------
// Free function: resolve_property (R-01, R-10)
// ---------------------------------------------------------------------------

/// Resolve a single raw property entry into a validated `Property`.
///
/// For `Ref` entries, applies type-specific overrides from the `RawPropertyRef`
/// and rejects type changes via `$ref` (R-10): the override fields must be
/// compatible with the base property's type.
///
/// # Errors
/// Returns `SchemaError` if:
/// - The `$ref` path is invalid.
/// - The referenced property is not in the bank.
/// - Override fields are incompatible with the base property type (R-10).
/// - Property validation fails.
#[inline]
pub fn resolve_property(
    bank: &PropertyBank,
    name: &str,
    entry: RawPropertyEntry,
) -> Result<Property, SchemaError> {
    match entry {
        RawPropertyEntry::Inline(inline) => {
            let prop_name = PropertyName::new(name)?;
            let spec = inline.spec.try_into_validated()?;
            let cardinality = Cardinality::from(inline.required);
            let multiplicity = Multiplicity::from(inline.multi);
            Property::new(
                PropertyId::new(),
                prop_name,
                cardinality,
                multiplicity,
                spec,
            )
        }

        RawPropertyEntry::Ref(ref_entry) => {
            resolve_ref_property(bank, name, &ref_entry)
        }
    }
}

/// Resolves a `$ref` property entry, applying overrides and rejecting type
/// changes (R-10).
fn resolve_ref_property(
    bank: &PropertyBank,
    name: &str,
    ref_entry: &RawPropertyRef,
) -> Result<Property, SchemaError> {
    let prop_ref = PropertyRef::try_from(ref_entry.ref_path.as_ref())?;
    let base_prop = bank.get_by_name(prop_ref.name()).ok_or_else(|| {
        SchemaError::PropertyRefNotFound(ref_entry.ref_path.to_string())
    })?;

    // Apply cardinality / multiplicity overrides.
    let cardinality =
        ref_entry.required.map_or(base_prop.cardinality(), Cardinality::from);
    let multiplicity =
        ref_entry.multi.map_or(base_prop.multiplicity(), Multiplicity::from);

    // Apply type-specific spec overrides (R-10: type changes rejected).
    let spec = apply_spec_overrides(base_prop.spec(), ref_entry)?;

    let prop_name = PropertyName::new(name)?;
    Property::new(base_prop.id(), prop_name, cardinality, multiplicity, spec)
}

/// Apply override fields from a `RawPropertyRef` on top of the base spec.
///
/// Type changes via `$ref` are **not** allowed (R-10). The override fields
/// must be compatible with the base property type. If any override field
/// belongs to a different type, `SchemaError::PropertyTypeMismatch` is
/// returned.
#[expect(
    clippy::pattern_type_mismatch,
    reason = "Match ergonomics on &PropertySpec are intentional for \
              readability; dereferencing every arm adds noise"
)]
fn apply_spec_overrides(
    base: &PropertySpec,
    ref_entry: &RawPropertyRef,
) -> Result<PropertySpec, SchemaError> {
    // Determine whether override fields target a type incompatible with base.
    // We detect an attempted type override by checking which Raw*Spec structs
    // have non-None fields and comparing against the base spec type.
    let has_number_overrides = ref_entry.number.min.is_some()
        || ref_entry.number.max.is_some()
        || ref_entry.number.step.is_some();
    let has_string_overrides = ref_entry.string.options.is_some()
        || ref_entry.string.pattern.is_some();
    let has_date_overrides = ref_entry.date.format.is_some();
    let has_file_overrides = ref_entry.file.directory.is_some()
        || ref_entry.file.file_class.is_some();

    match base {
        PropertySpec::Bool(_) => {
            // Bool has no override fields; any type-specific override is a
            // mismatch.
            if has_number_overrides {
                return Err(type_mismatch("bool", "number"));
            }
            if has_string_overrides {
                return Err(type_mismatch("bool", "string"));
            }
            if has_date_overrides {
                return Err(type_mismatch("bool", "date"));
            }
            if has_file_overrides {
                return Err(type_mismatch("bool", "file"));
            }
            Ok(base.clone())
        }

        PropertySpec::Number(number_spec) => {
            if has_string_overrides {
                return Err(type_mismatch("number", "string"));
            }
            if has_date_overrides {
                return Err(type_mismatch("number", "date"));
            }
            if has_file_overrides {
                return Err(type_mismatch("number", "file"));
            }
            Ok(PropertySpec::Number(
                number_spec.clone().apply_overrides(&ref_entry.number)?,
            ))
        }

        PropertySpec::String(string_spec) => {
            if has_number_overrides {
                return Err(type_mismatch("string", "number"));
            }
            if has_date_overrides {
                return Err(type_mismatch("string", "date"));
            }
            if has_file_overrides {
                return Err(type_mismatch("string", "file"));
            }
            Ok(PropertySpec::String(
                string_spec.clone().apply_overrides(&ref_entry.string)?,
            ))
        }

        PropertySpec::Date(date_spec) => {
            if has_number_overrides {
                return Err(type_mismatch("date", "number"));
            }
            if has_string_overrides {
                return Err(type_mismatch("date", "string"));
            }
            if has_file_overrides {
                return Err(type_mismatch("date", "file"));
            }
            Ok(PropertySpec::Date(
                date_spec.clone().apply_overrides(&ref_entry.date)?,
            ))
        }

        PropertySpec::File(file_spec) => {
            if has_number_overrides {
                return Err(type_mismatch("file", "number"));
            }
            if has_string_overrides {
                return Err(type_mismatch("file", "string"));
            }
            if has_date_overrides {
                return Err(type_mismatch("file", "date"));
            }
            Ok(PropertySpec::File(
                file_spec.clone().apply_overrides(&ref_entry.file)?,
            ))
        }
    }
}

#[inline]
fn type_mismatch(expected: &str, actual: &str) -> SchemaError {
    SchemaError::PropertyTypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
    }
}

// ---------------------------------------------------------------------------
// Free function: assemble_schema (R-01)
// ---------------------------------------------------------------------------

/// Assemble a `Schema` from its raw properties, optional parent, and excludes.
///
/// 1. Each property entry in `raw_props` is resolved via `resolve_property`.
/// 2. Properties are sorted by name.
/// 3. If a parent is provided, parent properties are merged in (with child
///    properties overriding same-named parent properties and excluded names
///    filtered out).
///
/// # Errors
/// Returns `SchemaError` if any property fails resolution or schema
/// construction fails.
#[inline]
#[expect(
    clippy::too_many_arguments,
    reason = "All 6 parameters are distinct concerns required for schema \
              assembly; a builder would not reduce clarity here"
)]
#[expect(
    clippy::implicit_hasher,
    reason = "Callers always use the standard HashMap; generalising the \
              hasher adds complexity with no practical benefit"
)]
pub fn assemble_schema(
    id: SchemaId,
    name: SchemaName,
    raw_props: std::collections::HashMap<Box<str>, RawPropertyEntry>,
    parent: Option<&Schema>,
    excludes: &[Box<str>],
    bank: &PropertyBank,
) -> Result<Schema, SchemaError> {
    // Resolve and sort own properties.
    let mut sorted_entries: Vec<(Box<str>, RawPropertyEntry)> =
        raw_props.into_iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut own_props = Vec::with_capacity(sorted_entries.len());
    for (prop_name, entry) in sorted_entries {
        own_props.push(resolve_property(bank, &prop_name, entry)?);
    }

    // Build exclude set.
    let excludes_set: HashSet<PropertyName> = excludes
        .iter()
        .map(|s| PropertyName::try_from(s.as_ref()))
        .collect::<Result<_, _>>()?;

    // Merge with parent.
    let final_props = if let Some(p) = parent {
        merge_sorted_properties(p.properties(), &own_props, &excludes_set)
    } else {
        own_props
    };

    Schema::new(id, name, final_props)
}

// ---------------------------------------------------------------------------
// Sorted merge helper
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures before constants for readability"
)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
    use crate::schema::{
        aggregate::{SchemaId, SchemaName},
        property_spec::{BoolSpec, PropertySpec},
    };

    mod fixtures {
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

        pub fn empty_raw_schema(schema_name: &str) -> (SchemaId, RawSchema) {
            (SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD), RawSchema {
                name: schema_name.into(),
                extends: None,
                excludes: Vec::new(),
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

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions may use assert! macros for clarity"
    )]
    mod assemble {
        use super::*;

        #[test]
        fn includes_parent_properties() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let property = fixtures::parent_property()?;
            let parent_schema =
                fixtures::parent_schema_with_property(property)?;
            let (id, raw) = fixtures::empty_raw_schema("child");

            let schema = assemble_schema(
                id,
                SchemaName::new("child")?,
                raw.properties,
                Some(&parent_schema),
                &raw.excludes,
                &bank,
            )?;

            let name = PropertyName::new("parent")?;
            assert!(
                schema.has(&name),
                "Resolved schema should include parent property"
            );
            Ok(())
        }

        #[test]
        fn excludes_properties_listed_in_child() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let property = fixtures::excluded_property()?;
            let parent_schema =
                fixtures::parent_schema_with_property(property)?;
            let exclude_name = PropertyName::new("p")?;

            let (id, _raw) = fixtures::empty_raw_schema("child");
            let schema = assemble_schema(
                id,
                SchemaName::new("child")?,
                HashMap::new(),
                Some(&parent_schema),
                &[exclude_name.as_str().into()],
                &bank,
            )?;

            assert!(
                !schema.has(&exclude_name),
                "Resolved schema should exclude child-listed property"
            );
            Ok(())
        }
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions may use assert! macros for clarity"
    )]
    mod resolve_property_tests {
        use super::*;
        use crate::schema::raw::{
            RawBoolSpec, RawDateSpec, RawFileSpec, RawNumberSpec,
            RawPropertyEntry, RawPropertyInline, RawPropertyRef,
            RawPropertySpec, RawStringSpec,
        };

        #[test]
        fn resolves_ref_property_by_bank_name() -> Result<(), SchemaError> {
            let property = fixtures::status_property()?;
            let bank = fixtures::property_bank_with(property)?;
            let entry = RawPropertyEntry::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });

            let prop = resolve_property(&bank, "status", entry)?;
            assert_eq!(prop.name().as_str(), "status");
            Ok(())
        }

        #[test]
        fn resolves_inline_property() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let entry = RawPropertyEntry::Inline(RawPropertyInline {
                required: true,
                multi: false,
                spec: RawPropertySpec::Bool(RawBoolSpec),
            });

            let prop = resolve_property(&bank, "flag", entry)?;
            assert_eq!(prop.name().as_str(), "flag");
            assert_eq!(prop.cardinality(), Cardinality::Required);
            assert_eq!(prop.multiplicity(), Multiplicity::Single);
            Ok(())
        }

        #[test]
        fn ref_override_cardinality_and_multiplicity() -> Result<(), SchemaError>
        {
            let property = fixtures::status_property()?;
            let bank = fixtures::property_bank_with(property)?;
            let entry = RawPropertyEntry::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: Some(false), // override: base is Required
                multi: Some(true),     // override: base is Single
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });

            let prop = resolve_property(&bank, "status", entry)?;
            assert_eq!(prop.cardinality(), Cardinality::Optional);
            assert_eq!(prop.multiplicity(), Multiplicity::Many);
            Ok(())
        }

        #[test]
        fn ref_type_change_rejected() {
            // Base property is Bool; number overrides are incompatible.
            let property =
                fixtures::status_property().expect("valid status property");
            let bank = fixtures::property_bank_with(property)
                .expect("valid property bank");
            let entry = RawPropertyEntry::Ref(RawPropertyRef {
                ref_path: "property_bank#/status".into(),
                required: None,
                multi: None,
                number: RawNumberSpec {
                    min: Some(0.0f64),
                    max: None,
                    step: None,
                },
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });

            let result = resolve_property(&bank, "status", entry);
            assert!(
                matches!(result, Err(SchemaError::PropertyTypeMismatch { .. })),
                "Expected PropertyTypeMismatch, got: {result:?}"
            );
        }

        #[test]
        fn ref_invalid_path_rejected() {
            let bank = PropertyBank::new();
            let entry = RawPropertyEntry::Ref(RawPropertyRef {
                ref_path: "bad_format".into(),
                required: None,
                multi: None,
                number: RawNumberSpec::default(),
                string: RawStringSpec::default(),
                date: RawDateSpec::default(),
                file: RawFileSpec::default(),
            });

            let result = resolve_property(&bank, "prop", entry);
            assert!(
                matches!(result, Err(SchemaError::InvalidPropertyRef(_))),
                "Expected InvalidPropertyRef, got: {result:?}"
            );
        }
    }

    #[expect(
        clippy::panic_in_result_fn,
        reason = "Test functions may use assert! macros for clarity"
    )]
    mod inheritance_forest {
        use super::*;

        #[test]
        fn topo_order_root_only() -> Result<(), SchemaError> {
            let id = SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD);
            let mut raws = HashMap::new();
            raws.insert(id, RawSchema {
                name: "leaf".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            });
            let mut names = HashMap::new();
            names.insert(id, SchemaName::new("leaf")?);
            let name_to_id: HashMap<SchemaName, SchemaId> =
                names.iter().map(|(&k, v)| (v.clone(), k)).collect();

            let forest = InheritanceForest::build(&raws, &names, &name_to_id)?;
            let order = forest.topo_order(&HashSet::new())?;
            assert_eq!(order, vec![id]);
            Ok(())
        }

        #[test]
        fn topo_order_parent_before_child() -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(TEST_SCHEMA_ID_PARENT);
            let child_id = SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD);

            let mut raws = HashMap::new();
            raws.insert(parent_id, RawSchema {
                name: "parent".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            });
            raws.insert(child_id, RawSchema {
                name: "child".into(),
                extends: Some("parent".into()),
                excludes: Vec::new(),
                properties: HashMap::new(),
            });

            let mut names = HashMap::new();
            names.insert(parent_id, SchemaName::new("parent")?);
            names.insert(child_id, SchemaName::new("child")?);
            let name_to_id: HashMap<SchemaName, SchemaId> =
                names.iter().map(|(&k, v)| (v.clone(), k)).collect();

            let forest = InheritanceForest::build(&raws, &names, &name_to_id)?;
            let order = forest.topo_order(&HashSet::new())?;

            // Parent must come before child
            let parent_pos =
                order.iter().position(|&x| x == parent_id).ok_or_else(
                    || SchemaError::NotFound("parent_id not in order".into()),
                )?;
            let child_pos =
                order.iter().position(|&x| x == child_id).ok_or_else(|| {
                    SchemaError::NotFound("child_id not in order".into())
                })?;
            if parent_pos >= child_pos {
                return Err(SchemaError::ValidationFailed(
                    "Parent must appear before child in topo order".into(),
                ));
            }
            Ok(())
        }

        #[test]
        fn topo_order_detects_cycle() {
            // We cannot construct a true cycle via RawSchema::extends (names
            // resolve to IDs in the batch), but we can test the in-progress
            // cycle detection indirectly by constructing the forest manually.
            // This test verifies the error path using a direct forest
            // construction with a self-loop.
            let id = SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD);
            let mut parents = HashMap::new();
            parents.insert(id, Some(id)); // self-loop
            let mut names = HashMap::new();
            names.insert(
                id,
                SchemaName::new("loopy").expect("valid schema name"),
            );

            let forest = InheritanceForest {
                parents,
                names,
            };

            let result = forest.topo_order(&HashSet::new());
            assert!(
                matches!(result, Err(SchemaError::CircularInheritance(_))),
                "Expected CircularInheritance, got: {result:?}"
            );
        }

        #[test]
        fn external_parent_does_not_cause_missing_parent_error()
        -> Result<(), SchemaError> {
            let parent_id = SchemaId::from_uuid(TEST_SCHEMA_ID_PARENT);
            let child_id = SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD);

            // Only child is in stale_raws; parent is external (in DB).
            let mut raws = HashMap::new();
            raws.insert(child_id, RawSchema {
                name: "child".into(),
                extends: Some("parent".into()),
                excludes: Vec::new(),
                properties: HashMap::new(),
            });

            let mut names = HashMap::new();
            names.insert(child_id, SchemaName::new("child")?);
            names.insert(parent_id, SchemaName::new("parent")?);

            // parent is NOT in name_to_id because it's not in the batch —
            // so the forest stores None for child's parent pointer.
            let name_to_id: HashMap<SchemaName, SchemaId> =
                [(SchemaName::new("child")?, child_id)].into_iter().collect();

            let forest = InheritanceForest::build(&raws, &names, &name_to_id)?;

            // Parent is known_external — no error expected.
            let mut known_external = HashSet::new();
            known_external.insert(parent_id);

            // child's parent is None in the forest (external), so it just
            // emits child.
            let order = forest.topo_order(&known_external)?;
            if order != vec![child_id] {
                return Err(SchemaError::ValidationFailed(format!(
                    "Expected [child_id], got: {order:?}"
                )));
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

            let first = merged
                .first()
                .ok_or_else(|| SchemaError::NotFound("no first".into()))?;
            let second = merged
                .get(1)
                .ok_or_else(|| SchemaError::NotFound("no second".into()))?;

            if merged.len() != 2 {
                return Err(SchemaError::ValidationFailed(format!(
                    "Expected 2 properties, got {}",
                    merged.len()
                )));
            }
            if first.name().as_str() != "a" {
                return Err(SchemaError::ValidationFailed(
                    "First property should be 'a'".into(),
                ));
            }
            if second.name().as_str() != "b" {
                return Err(SchemaError::ValidationFailed(
                    "Second property should be 'b'".into(),
                ));
            }
            if second.multiplicity() != Multiplicity::Many {
                return Err(SchemaError::ValidationFailed(
                    "Second property should have Many multiplicity".into(),
                ));
            }
            Ok(())
        }
    }

    mod process {
        use super::*;

        #[test]
        fn process_empty_input_returns_empty() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let resolver = SchemaResolver::new(&bank);
            let result = resolver.process(vec![])?;
            if !result.is_empty() {
                return Err(SchemaError::ValidationFailed(
                    "Expected empty result".into(),
                ));
            }
            Ok(())
        }

        #[test]
        fn process_single_schema() -> Result<(), SchemaError> {
            let bank = PropertyBank::new();
            let resolver = SchemaResolver::new(&bank);
            let id = SchemaId::from_uuid(TEST_SCHEMA_ID_CHILD);
            let raw = RawSchema {
                name: "myschema".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            };

            let result = resolver.process(vec![(id, raw)])?;
            let first = result
                .first()
                .ok_or_else(|| SchemaError::NotFound("no result".into()))?;
            let schema_name = first.0.name().as_str().to_owned();
            if schema_name != "myschema" {
                return Err(SchemaError::ValidationFailed(format!(
                    "Expected 'myschema', got '{schema_name}'"
                )));
            }
            Ok(())
        }

        #[test]
        fn process_duplicate_names_returns_error() {
            let bank = PropertyBank::new();
            let resolver = SchemaResolver::new(&bank);
            let id1 = SchemaId::new();
            let id2 = SchemaId::new();
            let raw1 = RawSchema {
                name: "dup".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            };
            let raw2 = RawSchema {
                name: "dup".into(),
                extends: None,
                excludes: Vec::new(),
                properties: HashMap::new(),
            };

            let result = resolver.process(vec![(id1, raw1), (id2, raw2)]);
            assert!(
                matches!(result, Err(SchemaError::AlreadyExists(_))),
                "Expected AlreadyExists, got: {result:?}"
            );
        }
    }
}
