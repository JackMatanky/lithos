//! Schema loader — orchestrates the full schema ingestion pipeline.
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
//!    - `Query::are_many_stale()` → O(1) check for all schemas
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
//!    - `Command::save_many()` → save only changed schemas
//!    - `Command::save_inheritance_many()` → track parent-child relationships
//!    - `Command::save_property_bank()` → save bank if stale
//!
//! **Key optimization**: Lightweight staleness checks (filename + timestamp)
//! avoid parsing/processing unchanged files.
//!
//! All schema-specific error types live in [`crate::schema::error`].

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

use std::{collections::HashMap, time::SystemTime};

use crate::schema::{
    aggregate::{SchemaId, SchemaName},
    bank::PropertyBank,
    db_command, db_query,
    dereferencer::Dereferencer,
    error::{
        SchemaCommandError, SchemaError, SchemaIngestionError, SchemaQueryError,
    },
    events::{PropertyBankEvent, SchemaEvent, SchemaEventHandler},
    extender::Extender,
    ingestor::Ingestor,
    ports::{Command as _, Query as _},
    raw::RawSchema,
    resolver::Resolver,
    stored::{StoredMetadata, StoredSchema},
};

// ─────────────────────────────────────────────────────────────────────────────
//  LoaderError
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during schema loading operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoaderError {
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
//  Loader
// ─────────────────────────────────────────────────────────────────────────────

/// Schema loader — orchestrates file ingestion and resolution.
///
/// Uses concrete redb adapters for production use. If testing with mocks
/// is needed in the future, this can be made generic again.
pub struct Loader<'db> {
    query: db_query::Query<'db>,
    command: db_command::Command<'db>,
    event_handlers: Vec<Box<dyn SchemaEventHandler>>,
}

// Type aliases for complex tuples used in service methods
type RawSchemaWithTimes = (
    RawSchema,
    crate::schema::hash::Blake3Hash,
    Option<SystemTime>,
    Option<SystemTime>,
);
type SchemaWithTimes = (
    SchemaId,
    RawSchema,
    crate::schema::hash::Blake3Hash,
    Option<SystemTime>,
    Option<SystemTime>,
);
type PartitionResult = (Vec<SchemaWithTimes>, Vec<SchemaId>);

impl<'db> Loader<'db> {
    /// Create a new `Loader` with query and command adapters.
    #[inline]
    #[must_use]
    pub fn new(
        query: db_query::Query<'db>,
        command: db_command::Command<'db>,
    ) -> Self {
        Self {
            query,
            command,
            event_handlers: Vec::new(),
        }
    }

    /// Add an event handler to the loader.
    ///
    /// Event handlers receive notifications at each pipeline stage for
    /// observability and reactive coordination.
    #[inline]
    #[must_use]
    pub fn with_event_handler(
        mut self,
        handler: Box<dyn SchemaEventHandler>,
    ) -> Self {
        self.event_handlers.push(handler);
        self
    }

    /// Emit a schema event to all registered handlers.
    #[inline]
    fn emit_schema(&self, event: &SchemaEvent) {
        for handler in &self.event_handlers {
            handler.handle_schema(event);
        }
    }

    /// Emit a property bank event to all registered handlers.
    #[inline]
    fn emit_property_bank(&self, event: &PropertyBankEvent) {
        for handler in &self.event_handlers {
            handler.handle_property_bank(event);
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
    /// Returns [`LoaderError`] on any I/O, parsing, domain, query, or
    /// command failure.
    #[inline]
    pub fn load(
        &self,
        ingestor: &Ingestor<'_>,
    ) -> Result<Vec<StoredSchema>, LoaderError> {
        // ── Step 1: read existing DB state ──────────────────────────────────
        let name_to_id = self.load_name_to_id_map()?;

        // ── Step 2: PropertyBank staleness check ────────────────────────────
        let (bank, bank_stale) = self.load_property_bank(ingestor)?;
        let current_bank_version = bank.version();

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas_with_times = ingestor.scan_raw_schemas()?;
        self.emit_schema(&SchemaEvent::ScanCompleted {
            file_count: raw_schemas_with_times.len(),
        });

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let (stale, fresh_ids) = self.partition_by_staleness(
            &raw_schemas_with_times,
            &name_to_id,
            current_bank_version,
            bank_stale,
        )?;

        // Emit cascade event if PropertyBank change affected schemas
        if bank_stale {
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::TriggeredCascade {
                    affected_schema_count: stale.len(),
                },
            );
        }

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        // Batch load: O(1) transaction for all fresh schemas
        let known_parents = self
            .query
            .find_many_by_ids(&fresh_ids)
            .map_err(SchemaQueryError::from)?;

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

        // Emit per-schema resolution events
        for stored in &resolved {
            self.emit_schema(&SchemaEvent::SchemaResolved {
                name: stored.name.to_string().into_boxed_str(),
                id: stored.id,
            });
        }

        self.emit_schema(&SchemaEvent::SchemaResolutionCompleted {
            schema_count: resolved.len(),
        });

        // ── Step 7: persist ─────────────────────────────────────────────────
        if !resolved.is_empty() {
            self.persist_schemas(
                &resolved,
                stale_with_times,
                current_bank_version,
            )?;
        }

        self.command
            .save_property_bank(&bank)
            .map_err(SchemaCommandError::from)?;
        self.emit_property_bank(&PropertyBankEvent::Persisted {
            version: current_bank_version,
        });

        Ok(resolved)
    }

    /// Load name-to-ID mapping from database.
    fn load_name_to_id_map(
        &self,
    ) -> Result<HashMap<SchemaName, SchemaId>, LoaderError> {
        let existing_pairs =
            self.query.list_name_id_pairs().map_err(SchemaQueryError::from)?;
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
    ) -> Result<(PropertyBank, bool), LoaderError> {
        let stored_bank =
            self.query.get_property_bank().map_err(SchemaQueryError::from)?;

        let (bank, bank_stale) = if let Some(stored) = stored_bank {
            let stored_version = stored.version();
            let is_stale = self
                .query
                .is_bank_stale(stored_version)
                .map_err(SchemaQueryError::from)?;

            if is_stale {
                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::Stale {
                        reason: crate::schema::events::StalenessReason::ContentChanged,
                    },
                );
                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::ResolutionStarted,
                );
                let raw_bank = ingestor.load_raw_property_bank()?;
                let rebuilt_bank =
                    PropertyBank::try_from_raw(raw_bank, Some(&stored))?;
                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::Resolved {
                        property_count: rebuilt_bank.all().count(),
                        version: rebuilt_bank.version(),
                    },
                );
                (rebuilt_bank, true)
            } else {
                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::Fresh {
                        version: stored_version,
                    },
                );
                (stored, false)
            }
        } else {
            // New bank (no stored version) - always requires resolution
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::Stale {
                    reason: crate::schema::events::StalenessReason::New,
                },
            );
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::ResolutionStarted,
            );
            let raw_bank = ingestor.load_raw_property_bank()?;
            let new_bank = PropertyBank::try_from_raw(raw_bank, None)?;
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::Resolved {
                    property_count: new_bank.all().count(),
                    version: new_bank.version(),
                },
            );
            (new_bank, true)
        };

        Ok((bank, bank_stale))
    }

    /// Partition schemas into stale and fresh based on staleness checks.
    #[expect(
        clippy::too_many_lines,
        reason = "Event emissions add necessary observability; function is \
                  already well-structured"
    )]
    fn partition_by_staleness(
        &self,
        raw_schemas_with_times: &[RawSchemaWithTimes],
        name_to_id: &HashMap<SchemaName, SchemaId>,
        current_bank_version: crate::schema::bank::BankVersion,
        bank_stale: bool,
    ) -> Result<PartitionResult, LoaderError> {
        type StalenessCheck =
            (SchemaId, Option<SystemTime>, Option<SystemTime>);

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
        for &(ref raw_schema, _hash, modified, created) in
            raw_schemas_with_times
        {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);
            schema_ids.push(id);
            staleness_checks.push((id, created, modified));
        }

        // Step 1: Check staleness with cascade (timestamp-based fast path)
        let mut staleness_map = self
            .query
            .are_many_stale(&staleness_checks, current_bank_version)
            .map_err(SchemaQueryError::from)?;
        self.query
            .cascade_staleness(&mut staleness_map)
            .map_err(SchemaQueryError::from)?;

        // Step 2: Two-tier staleness detection - for schemas marked stale by
        // timestamp, check if hash actually changed (slow path)
        let mut hash_map: HashMap<SchemaId, crate::schema::hash::Blake3Hash> =
            HashMap::with_capacity(raw_schemas_with_times.len());

        #[expect(
            clippy::pattern_type_mismatch,
            reason = "Tuple destructuring from slice requires ref pattern"
        )]
        for (raw_schema, hash, _modified, _created) in raw_schemas_with_times {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            if let Some(&id) = name_to_id.get(&schema_name) {
                hash_map.insert(id, *hash);
            }
        }

        // For schemas marked stale by timestamp, check if content hash changed
        // Iteration over HashMap is intentional here - order doesn't matter for
        // hash comparison
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Iteration order doesn't matter for hash comparison - \
                      we're just checking each schema individually"
        )]
        for (&id, is_stale) in &mut staleness_map {
            if !*is_stale {
                continue; // Skip schemas already marked as fresh
            }

            // Check if hash actually changed (slow path)
            let Some(&current_hash) = hash_map.get(&id) else {
                continue;
            };
            let Some(stored_hash) = self
                .query
                .get_schema_hash(id)
                .map_err(SchemaQueryError::from)?
            else {
                continue;
            };

            if current_hash == stored_hash {
                // Hash unchanged - this is a touch-only change
                // Mark as fresh to avoid re-resolution
                *is_stale = false;
                tracing::debug!(
                    schema_id = %id,
                    "Touch-only change detected (hash unchanged)"
                );
            }
        }

        // Partition into stale and fresh
        let mut stale = Vec::new();
        let mut fresh_ids = Vec::new();

        for (id, (raw_schema, hash, modified, created)) in
            schema_ids.into_iter().zip(raw_schemas_with_times.iter().cloned())
        {
            let schema_stale = staleness_map.get(&id).copied().unwrap_or(true);
            let is_stale = bank_stale || schema_stale;

            if is_stale {
                // Determine staleness reason: new if ID not in name_to_id
                let is_new =
                    !name_to_id.values().any(|&existing_id| existing_id == id);
                let reason = if is_new {
                    crate::schema::events::StalenessReason::New
                } else if bank_stale {
                    crate::schema::events::StalenessReason::BankVersionChanged
                } else {
                    crate::schema::events::StalenessReason::ContentChanged
                };

                self.emit_schema(
                    &crate::schema::events::SchemaEvent::SchemaStale {
                        name: raw_schema.name.clone(),
                        reason,
                    },
                );
                stale.push((id, raw_schema, hash, modified, created));
            } else {
                self.emit_schema(
                    &crate::schema::events::SchemaEvent::SchemaFresh {
                        name: raw_schema.name.clone(),
                    },
                );
                fresh_ids.push(id);
            }
        }

        Ok((stale, fresh_ids))
    }

    /// Persist resolved schemas with metadata and inheritance relationships.
    fn persist_schemas(
        &self,
        resolved: &[StoredSchema],
        stale_with_times: Vec<SchemaWithTimes>,
        current_bank_version: crate::schema::bank::BankVersion,
    ) -> Result<(), LoaderError> {
        use crate::schema::{hash::Blake3Hash, ports::InheritanceRelationship};
        type MetadataTriple =
            (Blake3Hash, Option<SystemTime>, Option<SystemTime>);

        // Build metadata and inheritance maps
        let mut metadata_map: HashMap<SchemaId, MetadataTriple> =
            HashMap::with_capacity(stale_with_times.len());
        let mut raw_map: HashMap<SchemaId, Vec<Box<str>>> =
            HashMap::with_capacity(stale_with_times.len());

        for (id, raw, hash, modified, created) in stale_with_times {
            metadata_map.insert(id, (hash, modified, created));
            raw_map.insert(id, raw.excludes);
        }

        // Build metadata vector
        let metadata: Vec<StoredMetadata> = resolved
            .iter()
            .map(|stored| {
                let (hash, modified, created) = metadata_map
                    .get(&stored.id)
                    .copied()
                    .unwrap_or((Blake3Hash::zero(), None, None));
                StoredMetadata::new(
                    current_bank_version,
                    hash,
                    created,
                    modified,
                )
            })
            .collect();

        // Save schemas with metadata
        self.command
            .save_many_with_metadata(resolved, &metadata)
            .map_err(SchemaCommandError::from)?;

        // Emit persistence events for each schema
        for stored in resolved {
            self.emit_schema(
                &crate::schema::events::SchemaEvent::SchemaPersisted {
                    name: stored.name.to_string().into_boxed_str(),
                    id: stored.id,
                },
            );
        }

        // Build and save inheritance relationships
        let inheritance_data: Vec<InheritanceRelationship> = resolved
            .iter()
            .map(|stored| {
                let excludes =
                    raw_map.get(&stored.id).cloned().unwrap_or_default();
                (stored.id, stored.parent_id, excludes)
            })
            .collect();

        self.command
            .save_inheritance_many(&inheritance_data)
            .map_err(SchemaCommandError::from)?;

        Ok(())
    }
}
