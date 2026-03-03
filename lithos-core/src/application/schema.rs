//! Schema application service — orchestrates the full schema ingestion
//! pipeline.
//!
//! # Pipeline Flow
//!
//! The service uses **staleness detection** to decide whether to load from
//! files or reuse cached data from the database:
//!
//! 1. **Load existing state from DB**:
//!    - `Query::list_name_id_pairs()` → name→id map
//!    - `Query::get_property_bank()` → cached `PropertyBank` (if exists)
//!
//! 2. **`PropertyBank` staleness check**:
//!    - If no bank in DB → **load from file** (`load_raw_property_bank()`)
//!    - If bank exists:
//!      - `Query::is_bank_stale()` checks file timestamp vs DB version
//!      - Stale → **reload from file**
//!      - Fresh → **reuse cached bank**
//!
//! 3. **Scan all schema files** (always from filesystem):
//!    - `Ingestor::scan_raw_schemas()` → `Vec<(RawSchema, timestamps)>`
//!    - Schema names derived from filenames
//!
//! 4. **Schema staleness partitioning**:
//!    - `Query::batch_is_stale()` → O(1) check for all schemas
//!    - Compares file timestamps vs DB metadata
//!    - **Cascade staleness**: Parent changes mark all descendants stale
//!    - **Stale schemas** → reload + re-resolve
//!    - **Fresh schemas** → reuse from DB (fetch as `known_parents`)
//!
//! 5. **Process only stale schemas**:
//!    - `Dereferencer::deref()` → resolve property refs
//!    - `Extender::build()` → build inheritance tree
//!    - `Resolver::resolve()` → merge parent properties
//!
//! 6. **Persist changes**:
//!    - `Command::save_batch()` → save only changed schemas
//!    - `Command::save_inheritance_batch()` → track parent-child relationships
//!    - `Command::save_property_bank()` → save bank if stale
//!
//! **Key optimization**: Lightweight staleness checks (filename + timestamp)
//! avoid parsing/processing unchanged files.
//!
//! All schema-specific error types live in [`crate::schema::error`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use std::collections::HashMap;

use crate::schema::{
    adapter::{ingestor::Ingestor, stored::StoredMetadata},
    aggregate::{Schema, SchemaId, SchemaName, Timestamp},
    bank::PropertyBank,
    command::Command,
    dereferencer::Dereferencer,
    error::{
        SchemaCommandError, SchemaError, SchemaIngestionError, SchemaQueryError,
    },
    extender::Extender,
    query::Query,
    raw::RawSchema,
    resolver::Resolver,
};

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaServiceError
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during schema service operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SchemaServiceError {
    /// Ingestion (file I/O or parsing) failed.
    #[error("ingestion error: {0}")]
    Ingestion(#[from] SchemaIngestionError),

    /// Domain validation failed.
    #[error("domain error: {0}")]
    Domain(#[from] SchemaError),

    /// Storage query failed.
    #[error("query error: {0}")]
    Query(#[from] SchemaQueryError),

    /// Storage command failed.
    #[error("command error: {0}")]
    Command(#[from] SchemaCommandError),
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaService
// ─────────────────────────────────────────────────────────────────────────────

/// Thin orchestration service for schema ingestion.
///
/// Uses concrete redb adapters for production use. If testing with mocks
/// is needed in the future, this can be made generic again.
pub struct SchemaService<'db> {
    query: Query<crate::schema::adapter::query::QueryAdapter<'db>>,
    command: Command<crate::schema::adapter::command::CommandAdapter<'db>>,
}

// Type aliases for complex tuples used in service methods
type RawSchemaWithTimes = (RawSchema, Option<Timestamp>, Option<Timestamp>);
type SchemaWithTimes =
    (SchemaId, RawSchema, Option<Timestamp>, Option<Timestamp>);
type PartitionResult = (Vec<SchemaWithTimes>, Vec<SchemaId>);

impl<'db> SchemaService<'db> {
    /// Create a new `SchemaService` with query and command adapters.
    #[inline]
    #[must_use]
    pub fn new(
        query: Query<crate::schema::adapter::query::QueryAdapter<'db>>,
        command: Command<crate::schema::adapter::command::CommandAdapter<'db>>,
    ) -> Self {
        Self {
            query,
            command,
        }
    }

    /// Run the full ingestion pipeline.
    ///
    /// Reads raw files via `ingestor`, resolves schemas through the
    /// `Dereferencer → Extender → Resolver` pipeline, and persists the
    /// results.  Only stale schemas are re-resolved; fresh schemas are used
    /// as `known_parents` for the inheritance tree.
    ///
    /// Returns the set of freshly resolved schemas.  Schemas that were
    /// already fresh in the DB are not included (they were not re-resolved).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaServiceError`] on any I/O, parsing, domain, query, or
    /// command failure.
    #[inline]
    pub fn load(
        &self,
        ingestor: &Ingestor<'_>,
    ) -> Result<Vec<Schema>, SchemaServiceError> {
        // ── Step 1: read existing DB state ──────────────────────────────────
        let name_to_id = self.load_name_to_id_map()?;

        // ── Step 2: PropertyBank staleness check ────────────────────────────
        let (bank, bank_stale) = self.load_property_bank(ingestor)?;
        let current_bank_version = bank.version();

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas_with_times = ingestor.scan_raw_schemas()?;

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let (stale, fresh_ids) = self.partition_by_staleness(
            &raw_schemas_with_times,
            &name_to_id,
            current_bank_version,
            bank_stale,
        )?;

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        // Batch load: O(1) transaction for all fresh schemas
        let known_parents = self.query.batch_find_by_ids(&fresh_ids)?;

        // ── Step 6: pipeline (Dereferencer → Extender → Resolver) ──────────
        // Extract just (id, raw_schema) for dereferencer, keep timestamps
        // separate
        let stale_with_times = stale;
        let stale_for_deref: Vec<(SchemaId, RawSchema)> = stale_with_times
            .iter()
            .map(|entry| (entry.0, entry.1.clone()))
            .collect();

        let derefed = Dereferencer::new(&bank).deref(stale_for_deref)?;
        let tree = Extender::build(derefed, &known_parents)?;
        tracing::debug!(
            root_count = tree.roots().len(),
            total_count = tree.nodes().len(),
            "schema tree built"
        );
        let resolved = Resolver::resolve(&tree, &known_parents)?;

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_schemas(
                &resolved,
                stale_with_times,
                current_bank_version,
            )?;
        }

        self.command.save_property_bank(&bank)?;

        Ok(resolved)
    }

    /// Load name-to-ID mapping from database.
    fn load_name_to_id_map(
        &self,
    ) -> Result<HashMap<SchemaName, SchemaId>, SchemaServiceError> {
        let existing_pairs = self.query.list_name_id_pairs()?;
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(existing_pairs.len());
        for (name, id) in existing_pairs {
            name_to_id.insert(name, id);
        }
        Ok(name_to_id)
    }

    /// Load property bank, checking staleness and rebuilding if needed.
    fn load_property_bank(
        &self,
        ingestor: &Ingestor<'_>,
    ) -> Result<(PropertyBank, bool), SchemaServiceError> {
        let stored_bank = self.query.get_property_bank()?;

        let (bank, bank_stale) = if let Some(stored) = stored_bank {
            let stored_version = stored.version();
            let is_stale = self.query.is_bank_stale(stored_version)?;

            if is_stale {
                let raw_bank = ingestor.load_raw_property_bank()?;
                let rebuilt_bank =
                    PropertyBank::from_raw(raw_bank, Some(&stored))?;
                (rebuilt_bank, true)
            } else {
                (stored, false)
            }
        } else {
            let raw_bank = ingestor.load_raw_property_bank()?;
            let new_bank = PropertyBank::from_raw(raw_bank, None)?;
            (new_bank, true)
        };

        Ok((bank, bank_stale))
    }

    /// Partition schemas into stale and fresh based on staleness checks.
    fn partition_by_staleness(
        &self,
        raw_schemas_with_times: &[RawSchemaWithTimes],
        name_to_id: &HashMap<SchemaName, SchemaId>,
        current_bank_version: crate::schema::bank::BankVersion,
        bank_stale: bool,
    ) -> Result<PartitionResult, SchemaServiceError> {
        type StalenessCheck = (SchemaId, Option<Timestamp>, Option<Timestamp>);

        // Build schema IDs and staleness checks
        let mut schema_ids: Vec<SchemaId> =
            Vec::with_capacity(raw_schemas_with_times.len());
        let mut staleness_checks: Vec<StalenessCheck> =
            Vec::with_capacity(raw_schemas_with_times.len());

        #[expect(
            clippy::ref_patterns,
            reason = "Required for borrowing RawSchema while destructuring \
                      tuple"
        )]
        for &(ref raw_schema, modified, created) in raw_schemas_with_times {
            let schema_name = SchemaName::new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);
            schema_ids.push(id);
            staleness_checks.push((id, created, modified));
        }

        // Check staleness with cascade
        let mut staleness_map = self
            .query
            .batch_is_stale(&staleness_checks, current_bank_version)?;
        self.query.cascade_staleness(&mut staleness_map)?;

        // Partition into stale and fresh
        let mut stale = Vec::new();
        let mut fresh_ids = Vec::new();

        for (id, (raw_schema, modified, created)) in
            schema_ids.into_iter().zip(raw_schemas_with_times.iter().cloned())
        {
            let schema_stale = staleness_map.get(&id).copied().unwrap_or(true);
            let is_stale = bank_stale || schema_stale;

            if is_stale {
                stale.push((id, raw_schema, modified, created));
            } else {
                fresh_ids.push(id);
            }
        }

        Ok((stale, fresh_ids))
    }

    /// Persist resolved schemas with metadata and inheritance relationships.
    fn persist_schemas(
        &self,
        resolved: &[Schema],
        stale_with_times: Vec<SchemaWithTimes>,
        current_bank_version: crate::schema::bank::BankVersion,
    ) -> Result<(), SchemaServiceError> {
        use crate::schema::ports::InheritanceRelationship;
        type TimestampPair = (Option<Timestamp>, Option<Timestamp>);

        // Build metadata and inheritance maps
        let mut time_map: HashMap<SchemaId, TimestampPair> =
            HashMap::with_capacity(stale_with_times.len());
        let mut raw_map: HashMap<SchemaId, Vec<Box<str>>> =
            HashMap::with_capacity(stale_with_times.len());

        for (id, raw, modified, created) in stale_with_times {
            time_map.insert(id, (modified, created));
            raw_map.insert(id, raw.excludes);
        }

        // Build metadata vector
        let metadata: Vec<StoredMetadata> = resolved
            .iter()
            .map(|schema| {
                let (modified, created) =
                    time_map.get(&schema.id()).copied().unwrap_or((None, None));
                StoredMetadata::new(current_bank_version, created, modified)
            })
            .collect();

        // Save schemas with metadata
        self.command.save_batch_with_metadata(resolved, &metadata)?;

        // Build and save inheritance relationships
        let inheritance_data: Vec<InheritanceRelationship> = resolved
            .iter()
            .map(|schema| {
                let excludes =
                    raw_map.get(&schema.id()).cloned().unwrap_or_default();
                (schema.id(), schema.parent_id(), excludes)
            })
            .collect();

        self.command.save_inheritance_batch(&inheritance_data)?;

        Ok(())
    }
}
