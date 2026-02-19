//! Schema application service — orchestrates the full schema ingestion
//! pipeline.
//!
//! # Pipeline
//!
//! ```text
//! Ingestor
//!   load_raw_property_bank()       → RawPropertyBank
//!   scan_raw_schemas()             → Vec<(RawSchema, Option<Timestamp>)>
//! Query
//!   list_name_id_pairs()           → existing name → id map
//!   find_property_bank()           → Option<PropertyBank>
//! PropertyBank::from_raw()         → PropertyBank
//! Staleness partitioning
//!   is_bank_stale() / is_schema_stale()
//! Query
//!   find_by_id() for each fresh id → known_parents map
//! Dereferencer::deref()            → Vec<(SchemaId, DereferencedSchema)>
//! Extender::build()                → SchemaTree
//! Resolver::resolve()              → Vec<Schema>
//! Command
//!   save_batch()
//!   save_property_bank()
//! ```
//!
//! All schema-specific error types live in [`crate::schema::error`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use std::collections::HashMap;

use crate::{
    db::DbError,
    schema::{
        adapter::ingestor::Ingestor,
        aggregate::{Schema, SchemaId, SchemaName, Timestamp},
        bank::PropertyBank,
        command::Command,
        dereferencer::Dereferencer,
        error::{
            SchemaCommandError, SchemaError, SchemaIngestionError,
            SchemaQueryError,
        },
        extender::Extender,
        ports::{self as schema_ports, SchemaRecord},
        query::Query,
        raw::RawSchema,
        resolver::Resolver,
    },
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
/// Generic over storage ports so it can be tested with in-memory fakes and
/// used in production with the redb adapters.
pub struct SchemaService<Q, C> {
    query: Query<Q>,
    command: Command<C>,
}

impl<Q, C> SchemaService<Q, C> {
    /// Create a new `SchemaService` with the given query and command ports.
    #[inline]
    #[must_use]
    pub fn new(query: Query<Q>, command: Command<C>) -> Self {
        Self {
            query,
            command,
        }
    }
}

impl<Q, C> SchemaService<Q, C>
where
    Q: schema_ports::Query,
    Q::Error: Into<DbError>,
    C: schema_ports::Command,
    C::Error: Into<DbError>,
{
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
        // ── Step 1: file ingestion ──────────────────────────────────────────
        let raw_bank = ingestor.load_raw_property_bank()?;
        let raw_schemas: Vec<(RawSchema, Option<Timestamp>)> =
            ingestor.scan_raw_schemas()?;

        // ── Step 2: read existing DB state ──────────────────────────────────
        let existing_pairs = self.query.list_name_id_pairs()?;
        let stored_bank = self.query.find_property_bank()?;

        // Build name → id lookup from DB pairs.
        let mut name_to_id: HashMap<SchemaName, SchemaId> =
            HashMap::with_capacity(existing_pairs.len());
        for (name, id) in existing_pairs {
            name_to_id.insert(name, id);
        }

        // ── Step 3: build PropertyBank ──────────────────────────────────────
        let bank = PropertyBank::from_raw(raw_bank, stored_bank.as_ref())?;
        let current_bank_version = bank.version();

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let bank_stale = self.query.is_bank_stale(current_bank_version)?;

        let mut stale: Vec<(SchemaId, RawSchema)> = Vec::new();
        let mut fresh_ids: Vec<SchemaId> = Vec::new();

        for (raw_schema, mtime) in raw_schemas {
            let schema_name = SchemaName::new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);

            let is_stale = bank_stale
                || self.query.is_schema_stale(
                    id,
                    mtime,
                    current_bank_version,
                )?;

            if is_stale {
                stale.push((id, raw_schema));
            } else {
                fresh_ids.push(id);
            }
        }

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        let mut known_parents: HashMap<SchemaId, Schema> =
            HashMap::with_capacity(fresh_ids.len());
        for fresh_id in fresh_ids {
            if let Some(schema) = self.query.find_by_id(fresh_id)? {
                known_parents.insert(fresh_id, schema);
            }
        }

        // ── Step 6: pipeline (Dereferencer → Extender → Resolver) ──────────
        let derefed = Dereferencer::new(&bank).deref(stale)?;
        let tree = Extender::build(derefed, &known_parents)?;
        tracing::debug!(
            root_count = tree.roots().len(),
            total_count = tree.nodes().len(),
            "schema tree built"
        );
        let resolved = Resolver::resolve(&tree, &known_parents)?;

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            let now = Timestamp::now();
            let records: Vec<SchemaRecord> = resolved
                .iter()
                .map(|schema| {
                    let parent_id =
                        tree.get(schema.id()).and_then(|node| node.parent_id);
                    SchemaRecord::new(
                        schema.clone(),
                        parent_id,
                        current_bank_version,
                        now,
                        now,
                    )
                })
                .collect();
            self.command.save_batch(&records)?;
        }

        self.command.save_property_bank(&bank)?;

        Ok(resolved)
    }
}
