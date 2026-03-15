//! Schema loader — orchestrates the full schema ingestion pipeline.
//!
//! ## Orchestration Pattern
//!
//! The loader coordinates the file → raw → resolved → database pipeline.
//! It is the **only** place where orchestration logic lives (following the
//! single responsibility principle).
//!
//! - **No behavior in StoredSchema**: Schema is a read model (no methods)
//! - **Event emission**: Emits pipeline events for observability
//! - **Staleness detection**: Two-tier (timestamp fast path, hash slow path)
//!
//! ## Pipeline Flow
//!
//! The loader uses **staleness detection** to decide whether to load from
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
//!    - `Ingestor::all_schemas()` → `Vec<(RawSchema, timestamps)>`
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
//!    - `RefExpander::expand_all()` → resolve property refs
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
    error::{
        SchemaCommandError, SchemaError, SchemaIngestionError, SchemaQueryError,
    },
    events::{PropertyBankEvent, SchemaEvent, SchemaEventHandler},
    expander::RefExpander,
    extender::Extender,
    ingestor::Ingestor,
    ports::{Command as _, Query as _},
    raw::RawSchema,
    resolver::Resolver,
    storage::{StoredMetadata, StoredSchema},
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

// Type aliases for loader operations
// Metadata is now embedded in RawSchema.metadata, so we only track (ID,
// RawSchema)
type SchemaWithId = (SchemaId, RawSchema);
type PartitionResult = (Vec<SchemaWithId>, Vec<SchemaId>);

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
    /// `RefExpander → Extender → Resolver` pipeline, and persists the
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
        let (bank, bank_stale, changed_properties) =
            self.load_property_bank(ingestor)?;
        let current_bank_version = bank.version();

        // ── Step 3: scan raw schemas ────────────────────────────────────────
        let raw_schemas = ingestor.all_schemas()?;
        self.emit_schema(&SchemaEvent::ScanCompleted {
            file_count: raw_schemas.len(),
        });

        // ── Step 4: staleness partitioning ─────────────────────────────────
        let (stale, fresh_ids) = self.partition_by_staleness(
            &raw_schemas,
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

        // ── Step 4b: Incremental resolution for bank-only changes ──────────
        let (stale_for_full_resolution, incrementally_resolved) = self
            .apply_incremental_resolution(
                stale,
                bank_stale,
                &changed_properties,
                &bank,
                current_bank_version,
            )?;

        // ── Step 5: load fresh schemas as known_parents ─────────────────────
        // Batch load: O(1) transaction for all fresh schemas
        let known_parents = self
            .query
            .find_many_by_ids(&fresh_ids)
            .map_err(SchemaQueryError::from)?;

        // ── Step 6: pipeline (RefExpander → Extender → Resolver) ───────────
        // Extract just (id, raw_schema) for reference expansion, keep
        // timestamps separate
        let stale_with_times = stale_for_full_resolution;
        let stale_for_expand: Vec<(SchemaId, RawSchema)> = stale_with_times
            .iter()
            .map(|entry| (entry.0, entry.1.clone()))
            .collect();

        let mut resolved = if stale_for_expand.is_empty() {
            Vec::new()
        } else {
            let expanded =
                RefExpander::new(&bank).expand_all(stale_for_expand)?;
            let tree = Extender::build(expanded, &known_parents)?;
            tracing::debug!(
                root_count = tree.roots().len(),
                total_count = tree.nodes().len(),
                "schema tree built"
            );
            Resolver::resolve(&tree, &known_parents)?
        };

        // Combine full resolution and incremental resolution results
        resolved.extend(incrementally_resolved);

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
    ///
    /// Returns tuple of (bank, `was_stale`, `changed_property_names`).
    #[expect(
        clippy::type_complexity,
        reason = "Return tuple is clearest for internal pipeline orchestration"
    )]
    fn load_property_bank(
        &self,
        ingestor: &Ingestor<'_>,
    ) -> Result<
        (PropertyBank, bool, Vec<super::property::PropertyName>),
        LoaderError,
    > {
        let stored_bank =
            self.query.get_property_bank().map_err(SchemaQueryError::from)?;

        let (bank, bank_stale, changed_properties) = if let Some(stored) =
            stored_bank
        {
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
                let raw_bank = ingestor.property_bank()?.ok_or_else(|| {
                    LoaderError::Ingestion(SchemaIngestionError::FileSystem(
                        "Property bank file not found".into(),
                    ))
                })?;
                let content = String::new(); // TODO: Store raw content in metadata
                let modified = raw_bank.metadata.modified_at;
                let created = raw_bank.metadata.created_at;
                let rebuilt_bank =
                    PropertyBank::try_from_raw(raw_bank, Some(&stored))?;

                // Compute changed properties for incremental resolution
                let changed_props = rebuilt_bank.diff_property_bank(&stored);

                // Persist raw property bank file
                let raw_bank_file =
                    crate::schema::storage::RawPropertyBankFile::new(
                        &content, created, modified,
                    )
                    .map_err(|e| {
                        LoaderError::from(SchemaCommandError::Storage(
                            crate::db::DbError::Database(e.to_string()),
                        ))
                    })?;
                self.command
                    .save_raw_property_bank_file(&raw_bank_file)
                    .map_err(SchemaCommandError::from)?;

                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::Resolved {
                        property_count: rebuilt_bank.all().count(),
                        version: rebuilt_bank.version(),
                    },
                );
                (rebuilt_bank, true, changed_props)
            } else {
                self.emit_property_bank(
                    &crate::schema::events::PropertyBankEvent::Fresh {
                        version: stored_version,
                    },
                );
                (stored, false, Vec::new())
            }
        } else {
            // New bank (no stored version) - always requires resolution
            // All properties are "changed" in this case
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::Stale {
                    reason: crate::schema::events::StalenessReason::New,
                },
            );
            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::ResolutionStarted,
            );
            let raw_bank = ingestor.property_bank()?.ok_or_else(|| {
                LoaderError::Ingestion(SchemaIngestionError::FileSystem(
                    "Property bank file not found".into(),
                ))
            })?;
            let content = String::new(); // TODO: Store raw content in metadata
            let modified = raw_bank.metadata.modified_at;
            let created = raw_bank.metadata.created_at;
            let new_bank = PropertyBank::try_from_raw(raw_bank, None)?;

            // Persist raw property bank file
            let raw_bank_file =
                crate::schema::storage::RawPropertyBankFile::new(
                    &content, created, modified,
                )
                .map_err(|e| {
                    LoaderError::from(SchemaCommandError::Storage(
                        crate::db::DbError::Database(e.to_string()),
                    ))
                })?;
            self.command
                .save_raw_property_bank_file(&raw_bank_file)
                .map_err(SchemaCommandError::from)?;

            self.emit_property_bank(
                &crate::schema::events::PropertyBankEvent::Resolved {
                    property_count: new_bank.all().count(),
                    version: new_bank.version(),
                },
            );
            // For new bank, consider all properties as changed
            let all_props: Vec<_> =
                new_bank.all().map(|p| p.name().clone()).collect();
            (new_bank, true, all_props)
        };

        Ok((bank, bank_stale, changed_properties))
    }

    /// Refine staleness by checking content hash for timestamp-stale schemas.
    ///
    /// This is the "slow path" that verifies whether a timestamp-based stale
    /// detection was a real content change or just a file touch.
    fn refine_staleness_by_hash(
        &self,
        staleness_map: &mut HashMap<SchemaId, bool>,
        hash_map: &HashMap<SchemaId, [u8; 32]>,
    ) -> Result<(), LoaderError> {
        // Iteration over HashMap is intentional - order doesn't matter
        #[expect(
            clippy::iter_over_hash_type,
            reason = "Iteration order doesn't matter for hash comparison"
        )]
        for (&id, is_stale) in staleness_map {
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
                *is_stale = false;
                tracing::debug!(
                    schema_id = %id,
                    "Touch-only change detected (hash unchanged)"
                );
            }
        }
        Ok(())
    }

    /// Partition schemas into stale and fresh based on staleness checks.
    fn partition_by_staleness(
        &self,
        raw_schemas: &[RawSchema],
        name_to_id: &HashMap<SchemaName, SchemaId>,
        current_bank_version: crate::schema::bank::BankVersion,
        bank_stale: bool,
    ) -> Result<PartitionResult, LoaderError> {
        type StalenessCheck =
            (SchemaId, Option<SystemTime>, Option<SystemTime>);

        // Build schema IDs and staleness checks from embedded metadata
        let mut schema_ids: Vec<SchemaId> =
            Vec::with_capacity(raw_schemas.len());
        let mut staleness_checks: Vec<StalenessCheck> =
            Vec::with_capacity(raw_schemas.len());

        for raw_schema in raw_schemas {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            let id = name_to_id
                .get(&schema_name)
                .copied()
                .unwrap_or_else(SchemaId::new);
            schema_ids.push(id);
            staleness_checks.push((
                id,
                raw_schema.metadata.created_at,
                raw_schema.metadata.modified_at,
            ));
        }

        // Step 1: Check staleness with cascade (timestamp-based fast path)
        let mut staleness_map = self
            .query
            .are_many_stale(&staleness_checks, current_bank_version)
            .map_err(SchemaQueryError::from)?;
        self.query
            .cascade_staleness(&mut staleness_map)
            .map_err(SchemaQueryError::from)?;

        // Step 2: Build hash map for content comparison from metadata
        let mut hash_map: HashMap<SchemaId, [u8; 32]> =
            HashMap::with_capacity(raw_schemas.len());

        for raw_schema in raw_schemas {
            let schema_name = SchemaName::try_new(&raw_schema.name)?;
            if let Some(&id) = name_to_id.get(&schema_name) {
                if let Some(hash) = raw_schema.metadata.content_hash {
                    hash_map.insert(id, hash);
                }
            }
        }

        // Step 3: Refine staleness by checking content hash (slow path)
        self.refine_staleness_by_hash(&mut staleness_map, &hash_map)?;

        // Partition into stale and fresh
        let mut stale = Vec::new();
        let mut fresh_ids = Vec::new();

        for (id, raw_schema) in
            schema_ids.into_iter().zip(raw_schemas.iter().cloned())
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
                stale.push((id, raw_schema));
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
        use crate::schema::ports::InheritanceRelationship;
        type MetadataTriple =
            ([u8; 32], Option<SystemTime>, Option<SystemTime>);

        // Build metadata and inheritance maps, persist raw files
        let mut metadata_map: HashMap<SchemaId, MetadataTriple> =
            HashMap::with_capacity(stale_with_times.len());
        let mut raw_map: HashMap<SchemaId, Vec<Box<str>>> =
            HashMap::with_capacity(stale_with_times.len());

        for (id, raw, content, hash, modified, created) in stale_with_times {
            metadata_map.insert(id, (hash, modified, created));
            raw_map.insert(id, raw.excludes);

            // Persist raw file with version history
            let file_path = format!("schemas/{}.toml", raw.name);
            let raw_file = crate::schema::storage::RawSchemaFile::new(
                file_path, &content, created, modified,
            )
            .map_err(|e| {
                LoaderError::from(SchemaCommandError::Storage(
                    crate::db::DbError::Database(e.to_string()),
                ))
            })?;
            self.command
                .save_raw_schema_file(&raw_file)
                .map_err(SchemaCommandError::from)?;
        }

        // Build metadata vector
        let metadata: Vec<StoredMetadata> = resolved
            .iter()
            .map(|stored| {
                let (hash, modified, created) = metadata_map
                    .get(&stored.id)
                    .copied()
                    .unwrap_or(([0u8; 32], None, None));
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

    /// Apply incremental resolution for schemas affected by `PropertyBank`
    /// changes.
    ///
    /// This method partitions stale schemas into two groups:
    /// 1. Schemas with file changes → need full resolution
    /// 2. Schemas with only bank changes → need incremental resolution
    ///
    /// For the second group, it applies `resolve_affected_properties()` to
    /// update only the properties that reference changed bank properties.
    #[expect(
        clippy::too_many_arguments,
        reason = "Method needs bank version, changed properties, and bank to \
                  apply incremental resolution"
    )]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "Tuple destructuring requires ref patterns"
    )]
    #[expect(
        clippy::type_complexity,
        reason = "Return tuple is clearest for pipeline result partitioning"
    )]
    fn apply_incremental_resolution(
        &self,
        stale: Vec<SchemaWithTimes>,
        bank_stale: bool,
        changed_properties: &[super::property::PropertyName],
        bank: &PropertyBank,
        current_bank_version: crate::schema::bank::BankVersion,
    ) -> Result<(Vec<SchemaWithTimes>, Vec<StoredSchema>), LoaderError> {
        // Early return if bank is not stale or no properties changed
        if !bank_stale || changed_properties.is_empty() {
            return Ok((stale, Vec::new()));
        }

        // Find schemas that reference changed properties
        let affected_map = self
            .query
            .find_schemas_using_properties(changed_properties)
            .map_err(SchemaQueryError::from)?;

        // Partition stale schemas: file-changed vs. bank-only-changed
        let mut full_resolution = Vec::new();
        let mut incremental_ids = Vec::new();
        let staleness_map = self
            .query
            .are_many_stale(
                &stale
                    .iter()
                    .map(|(id, _, _, _hash, modified, created)| {
                        (*id, *created, *modified)
                    })
                    .collect::<Vec<_>>(),
                current_bank_version,
            )
            .map_err(SchemaQueryError::from)?;

        for entry in stale {
            let (id, _raw, _content, _hash, _modified, _created) = &entry;
            // Schema is file-stale if marked stale even without bank staleness
            // We check original staleness (before bank cascade)
            let file_stale = staleness_map.get(id).copied().unwrap_or(true);

            if !file_stale && affected_map.contains_key(id) {
                // Schema file is fresh but references changed bank properties
                incremental_ids.push(*id);
            } else {
                // Schema file changed OR doesn't reference changed properties
                full_resolution.push(entry);
            }
        }

        // Apply incremental resolution to bank-only-changed schemas
        let mut resolved_incremental = Vec::new();
        if !incremental_ids.is_empty() {
            let stored_schemas = self
                .query
                .find_many_by_ids(&incremental_ids)
                .map_err(SchemaQueryError::from)?;

            #[expect(
                clippy::iter_over_hash_type,
                reason = "HashMap iteration order doesn't matter - we process \
                          all affected schemas"
            )]
            for (id, affected_props) in &affected_map {
                if let Some(stored) = stored_schemas.get(id) {
                    let updated = Resolver::resolve_affected_properties(
                        stored,
                        affected_props,
                        bank,
                    )?;
                    resolved_incremental.push(updated);
                }
            }
        }

        Ok((full_resolution, resolved_incremental))
    }
}
