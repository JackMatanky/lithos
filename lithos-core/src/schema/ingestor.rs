//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Performs file-to-raw translation with embedded Repository for staleness
//! detection.
//!
//! ## Pattern
//!
//! Following the config ingestor pattern with caching:
//! - Single method per entity (`property_bank()`, `schema()`, `all_schemas()`)
//! - Metadata populated in Raw* types (no separate tuples)
//! - Returns `IngestResult<T>` indicating Fresh or Stale for optimization

use std::{path::Path, time::SystemTime};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::SchemaId,
        bank::PropertyBank,
        error::SchemaIngestionError,
        property::{Property, PropertyName},
        raw::{RawPropertyBank, RawSchema, RawSchemaMetadata},
        storage::Repository,
        views::raw::{RawPropertyBankView, RawSchemaView},
    },
};

/// Supported schema file extensions.
const SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"];

/// Result of ingesting a file, indicating staleness status.
///
/// Enables the loader to distinguish between cached (Fresh) and newly parsed
/// (Stale) data for performance tracking and optimization decisions.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum IngestResult<T> {
    /// Data is fresh (reused from cache without re-parsing).
    Fresh(T),
    /// Data was stale or new (re-parsed from file).
    Stale(T),
}

impl<T> IngestResult<T> {
    /// Unwraps the inner value, discarding staleness information.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        match self {
            Self::Fresh(t) | Self::Stale(t) => t,
        }
    }

    /// Returns `true` if this result is `Fresh`.
    #[inline]
    #[must_use]
    pub const fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh(_))
    }

    /// Returns `true` if this result is `Stale`.
    #[inline]
    #[must_use]
    pub const fn is_stale(&self) -> bool {
        matches!(self, Self::Stale(_))
    }

    /// Returns a reference to the inner value.
    #[inline]
    #[must_use]
    #[expect(
        clippy::ref_patterns,
        reason = "ref pattern required for const fn to extract &T from &Self"
    )]
    pub const fn as_ref(&self) -> &T {
        match self {
            &Self::Fresh(ref t) | &Self::Stale(ref t) => t,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PropertyBankResult
// ─────────────────────────────────────────────────────────────────────────────

/// Result of property bank ingestion with staleness information.
///
/// Indicates whether the property bank file is:
/// - **New**: First time seeing this file
/// - **Fresh**: File unchanged, loaded from database cache
/// - **Stale**: File changed, updated incrementally with changed properties
///   tracked
///
/// The `Stale` variant tracks which properties changed to enable incremental
/// schema resolution - only schemas using changed properties need re-expansion.
#[derive(Debug)]
#[non_exhaustive]
pub enum PropertyBankResult {
    /// Property bank file is new (first time seeing it).
    New(PropertyBank),

    /// Property bank file unchanged - loaded from database.
    Fresh(PropertyBank),

    /// Property bank file changed - updated incrementally.
    Stale {
        /// The updated property bank.
        bank: PropertyBank,
        /// Properties that changed (for incremental resolution).
        changed: Vec<PropertyName>,
    },
}

impl PropertyBankResult {
    /// Get the property bank regardless of variant.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "implicit borrow in match on &self"
    )]
    pub fn bank(&self) -> &PropertyBank {
        match self {
            Self::New(bank)
            | Self::Fresh(bank)
            | Self::Stale {
                bank,
                ..
            } => bank,
        }
    }

    /// Get the property bank, consuming self.
    #[inline]
    #[must_use]
    pub fn into_bank(self) -> PropertyBank {
        match self {
            Self::New(bank)
            | Self::Fresh(bank)
            | Self::Stale {
                bank,
                ..
            } => bank,
        }
    }

    /// Get changed properties if stale, otherwise empty slice.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "implicit borrow in match on &self"
    )]
    pub fn changed_properties(&self) -> &[PropertyName] {
        match self {
            Self::Stale {
                changed,
                ..
            } => changed,
            Self::New(_) | Self::Fresh(_) => &[],
        }
    }

    /// Check if the bank is fresh.
    #[inline]
    #[must_use]
    pub const fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh(_))
    }

    /// Check if the bank is new.
    #[inline]
    #[must_use]
    pub const fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }

    /// Check if the bank is stale.
    #[inline]
    #[must_use]
    pub const fn is_stale(&self) -> bool {
        matches!(self, Self::Stale { .. })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  SchemaIngestResult
// ─────────────────────────────────────────────────────────────────────────────

/// Result of ingesting a single schema file.
///
/// Indicates whether the schema is:
/// - **New**: First time seeing this schema file
/// - **Fresh**: File unchanged, can use cached data
/// - **Stale**: File changed or new, needs processing
#[derive(Debug)]
#[non_exhaustive]
pub enum SchemaIngestResult {
    /// Schema file is new (first time seeing it).
    New {
        /// Schema ID (newly generated).
        id: SchemaId,
        /// Parsed raw schema.
        raw: RawSchema,
    },

    /// Schema file unchanged - can use cached data.
    Fresh {
        /// Schema ID (from database).
        id: SchemaId,
        /// Cached expanded properties (if available).
        /// Enables skipping `RefExpander` when `PropertyBank` is fresh.
        expanded: Option<std::collections::HashMap<PropertyName, Property>>,
    },

    /// Schema file changed - needs processing.
    Stale {
        /// Schema ID (from database).
        id: SchemaId,
        /// Parsed raw schema.
        raw: RawSchema,
        /// Cached expanded properties (if available and still valid).
        expanded: Option<std::collections::HashMap<PropertyName, Property>>,
    },
}

impl SchemaIngestResult {
    /// Get the schema ID.
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "implicit borrow in match on &self"
    )]
    pub const fn id(&self) -> SchemaId {
        match self {
            Self::New {
                id,
                ..
            }
            | Self::Fresh {
                id,
                ..
            }
            | Self::Stale {
                id,
                ..
            } => *id,
        }
    }

    /// Check if this result is new.
    #[inline]
    #[must_use]
    pub const fn is_new(&self) -> bool {
        matches!(self, Self::New { .. })
    }

    /// Check if this result is fresh.
    #[inline]
    #[must_use]
    pub const fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    /// Check if this result is stale.
    #[inline]
    #[must_use]
    pub const fn is_stale(&self) -> bool {
        matches!(self, Self::Stale { .. })
    }

    /// Get raw schema if available (New or Stale variants).
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "implicit borrow in match on &self"
    )]
    pub const fn raw(&self) -> Option<&RawSchema> {
        match self {
            Self::New {
                raw,
                ..
            }
            | Self::Stale {
                raw,
                ..
            } => Some(raw),
            Self::Fresh {
                ..
            } => None,
        }
    }

    /// Get expanded properties if available (Fresh or Stale variants).
    #[inline]
    #[must_use]
    #[expect(
        clippy::pattern_type_mismatch,
        reason = "implicit borrow in match on &self"
    )]
    pub fn expanded(
        &self,
    ) -> Option<&std::collections::HashMap<PropertyName, Property>> {
        match self {
            Self::Fresh {
                expanded,
                ..
            }
            | Self::Stale {
                expanded,
                ..
            } => expanded.as_ref(),
            Self::New {
                ..
            } => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  IngestorResults
// ─────────────────────────────────────────────────────────────────────────────

/// Results of ingesting all schemas and property bank in a single operation.
///
/// Returned by `Ingestor::ingest_all()` to provide all ingestion results
/// pre-partitioned by staleness, eliminating the need for double-loop patterns.
#[derive(Debug)]
#[non_exhaustive]
pub struct IngestorResults {
    /// Property bank result with staleness information.
    pub property_bank: PropertyBankResult,

    /// Schema ingestion results by file path.
    pub schemas:
        std::collections::HashMap<std::path::PathBuf, SchemaIngestResult>,
}

/// Ingestor for loading raw schema files with embedded Repository for caching.
///
/// This adapter is responsible for:
/// - Loading the property bank file (JSON, TOML, or YAML)
/// - Scanning the schemas directory for schema files
/// - Per-file staleness checking to avoid unnecessary I/O
/// - Providing both Fresh and Stale variants based on file state
/// - Persisting Raw*View types (including compression)
///
/// It does NOT:
/// - Perform validation beyond deserialization
/// - Resolve references or build inheritance trees
///
/// The ingestor takes a `&Config` reference to ensure it uses the final
/// merged path values after config loading completes.
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::ingestor::Ingestor;
/// use lithos_core::fs::FsReader;
///
/// let root = std::path::PathBuf::from("/tmp");
/// let config = todo!("Provide a Config instance");
/// let repository = todo!("Provide a Repository instance");
/// let ingestor = Ingestor::new(FsReader::new(root), &config, repository);
/// let _ = ingestor;
/// ```
pub struct Ingestor<'config, R> {
    source: FsReader,
    config: &'config Config,
    repository: R,
}

impl<'config, R> Ingestor<'config, R>
where
    R: Repository,
{
    /// Create a new ingestor with the given file source, config, and
    /// repository.
    ///
    /// The config reference ensures paths are the final merged values.
    ///
    /// # Examples
    /// ```ignore
    /// use lithos_core::schema::ingestor::Ingestor;
    /// use lithos_core::fs::FsReader;
    ///
    /// let root = std::path::PathBuf::from("/tmp");
    /// let config = todo!("Provide a Config instance");
    /// let repository = todo!("Provide a Repository instance");
    /// let ingestor = Ingestor::new(FsReader::new(root), &config, repository);
    /// let _ = ingestor;
    /// ```
    #[inline]
    #[must_use]
    pub fn new(
        source: FsReader,
        config: &'config Config,
        repository: R,
    ) -> Self {
        Self {
            source,
            config,
            repository,
        }
    }

    /// Returns a reference to the embedded repository.
    ///
    /// This allows the Loader to access the repository for persistence
    /// operations.
    #[inline]
    #[must_use]
    pub const fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> Ingestor<'_, R>
where
    R: Repository,
{
    /// Get the property bank file with staleness detection.
    ///
    /// Performs optimized loading using cached data when possible:
    /// 1. Checks if file exists (returns `None` if not)
    /// 2. Tries fast staleness check via timestamps
    /// 3. Falls back to content hash if timestamps don't match
    /// 4. Only re-parses if file is truly stale
    ///
    /// Returns `Fresh` if cached data was reused, `Stale` if file was
    /// re-parsed.
    ///
    /// Supports JSON, TOML, and YAML formats (detected by extension or
    /// content).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if:
    /// - File reading fails (I/O error)
    /// - Parsing fails (syntax error)
    /// - Version validation fails
    /// - Repository access fails
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// if let Some(result) = ingestor.property_bank()? {
    ///     let bank = result.into_inner();
    /// }
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    /// Ingest property bank with staleness detection.
    ///
    /// Returns `PropertyBankResult` indicating if the bank is:
    /// - `New`: First time seeing property bank
    /// - `Fresh`: File unchanged, loaded from database
    /// - `Stale`: File changed, updated incrementally with changed properties
    ///   tracked
    ///
    /// # Errors
    ///
    /// Returns error if file reading, parsing, or repository access fails.
    pub fn property_bank(
        &self,
    ) -> Result<PropertyBankResult, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();

        // Extract timestamps once (needed for staleness check and metadata)
        let created_at = self.source.created_at(&path);
        let modified_at = self.source.modified_at(&path);

        // Load cached view if exists
        let cached_view = self
            .repository
            .get_raw_property_bank_view()
            .map_err(|e| SchemaIngestionError::Io {
                path: path.to_string_lossy().into(),
                reason: format!("Failed to query property bank view: {e}")
                    .into(),
            })?;

        // Case 1: No cached version - this is a NEW property bank
        let Some(view) = cached_view else {
            return self.ingest_new_property_bank(
                &path,
                created_at,
                modified_at,
            );
        };

        // Case 2: Check if FRESH (timestamps match)
        if view.current().is_some_and(|v| {
            v.file_times().is_timestamp_match(created_at, modified_at)
        }) {
            let bank = self
                .repository
                .get_property_bank()
                .map_err(|e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to load PropertyBank: {e}").into(),
                })?
                .ok_or_else(|| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: "PropertyBank missing from DB but view exists"
                        .into(),
                })?;
            return Ok(PropertyBankResult::Fresh(bank));
        }

        // Case 3: STALE - file changed, compute incremental update
        self.ingest_stale_property_bank(&path, created_at, modified_at, &view)
    }

    fn ingest_new_property_bank(
        &self,
        path: &Path,
        _created_at: Option<SystemTime>,
        _modified_at: Option<SystemTime>,
    ) -> Result<PropertyBankResult, SchemaIngestionError> {
        self.source.read_with(path, |path, content| {
            let _content_hash = blake3::hash(content.as_bytes());

            let raw: RawPropertyBank =
                FsReader::parse_structured_from_str(path, content)?;
            let raw = raw.validated(&path.to_string_lossy())?;

            // Create view with content for caching
            let view =
                RawPropertyBankView::try_from_with_content(&raw, content)?;

            self.repository.save_raw_property_bank_view(&view).map_err(
                |e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to save property bank view: {e}")
                        .into(),
                },
            )?;

            // Create PropertyBank
            let bank = PropertyBank::try_from(raw)?;

            self.repository.save_property_bank(&bank).map_err(|e| {
                SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to save PropertyBank: {e}").into(),
                }
            })?;

            Ok(PropertyBankResult::New(bank))
        })
    }

    fn ingest_stale_property_bank(
        &self,
        path: &Path,
        _created_at: Option<SystemTime>,
        _modified_at: Option<SystemTime>,
        cached_view: &RawPropertyBankView,
    ) -> Result<PropertyBankResult, SchemaIngestionError> {
        self.source.read_with(path, |path, content| {
            let content_hash = blake3::hash(content.as_bytes());

            // Check content hash before re-parsing
            if cached_view.current().is_some_and(|v| {
                v.hashes().is_content_match(content_hash.as_bytes())
            }) {
                // Content unchanged despite timestamp difference - treat as
                // fresh
                let bank = self
                    .repository
                    .get_property_bank()
                    .map_err(|e| SchemaIngestionError::Io {
                        path: path.to_string_lossy().into(),
                        reason: format!("Failed to load PropertyBank: {e}")
                            .into(),
                    })?
                    .ok_or_else(|| SchemaIngestionError::Io {
                        path: path.to_string_lossy().into(),
                        reason: "PropertyBank missing from DB but view exists"
                            .into(),
                    })?;
                return Ok(PropertyBankResult::Fresh(bank));
            }

            // Parse new version
            let raw: RawPropertyBank =
                FsReader::parse_structured_from_str(path, content)?;
            let raw = raw.validated(&path.to_string_lossy())?;

            // Compute new hashes and find changed properties
            let new_hashes = crate::schema::views::metadata::HashMetadata::compute_property_hashes_for_bank(&raw.properties);
            let changed = cached_view.current().map_or_else(
                || {
                    // If no current version, all properties are "changed"
                    new_hashes.keys().cloned().collect()
                },
                |v| v.hashes().changed_properties(&new_hashes),
            );

            // Create updated view
            let view =
                RawPropertyBankView::try_from_with_content(&raw, content)?;

            self.repository.save_raw_property_bank_view(&view).map_err(
                |e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to save property bank view: {e}")
                        .into(),
                },
            )?;

            // Update PropertyBank incrementally
            let mut bank = self
                .repository
                .get_property_bank()
                .map_err(|e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to load PropertyBank: {e}").into(),
                })?
                .unwrap_or_default();

            bank.update_from_raw(&raw, &changed)?;

            self.repository.save_property_bank(&bank).map_err(|e| {
                SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to save PropertyBank: {e}").into(),
                }
            })?;

            Ok(PropertyBankResult::Stale { bank, changed })
        })
    }

    /// Get a single schema file with staleness detection.
    ///
    /// Performs optimized loading using cached data when possible:
    /// 1. Tries fast staleness check via timestamps
    /// 2. Falls back to content hash if timestamps don't match
    /// 3. Only re-parses if file is truly stale
    ///
    /// Returns `Fresh` if cached data was reused, `Stale` if file was
    /// re-parsed.
    ///
    /// Supports JSON, TOML, and YAML formats (detected by extension or
    /// content).
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if:
    /// - File reading fails (I/O error)
    /// - Parsing fails (syntax error)
    /// - Version validation fails
    /// - Repository access fails
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// # let path = std::path::Path::new("schema.json");
    /// let result = ingestor.schema(path)?;
    /// let schema = result.into_inner();
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn schema(
        &self,
        path: &Path,
    ) -> Result<IngestResult<RawSchema>, SchemaIngestionError> {
        // Derive schema name from filename (without extension)
        let filename_stem = self.source.basename(path).map_err(|e| {
            SchemaIngestionError::FileSystem(
                format!(
                    "Invalid filename for schema: {} ({})",
                    path.display(),
                    e
                )
                .into(),
            )
        })?;

        let rel_path = path.to_string_lossy();
        let created_at = self.source.created_at(path);
        let modified_at = self.source.modified_at(path);

        // Load cached view if exists
        let cached_view = self
            .repository
            .find_raw_schema_view_by_path(&rel_path)
            .map_err(|e| SchemaIngestionError::Io {
                path: rel_path.to_string().into(),
                reason: format!("Failed to query schema view: {e}").into(),
            })?;

        // Fast path: Check timestamps (no file I/O)
        if let Some(view) = cached_view.as_ref()
            && view.current().is_some_and(|v| {
                v.file_times().is_timestamp_match(created_at, modified_at)
            })
            && let Some(raw) = view.to_raw()?
        {
            return Ok(IngestResult::Fresh(raw));
        }

        // Slow path: Read file once for hash check, compression, and parsing
        self.source.read_with(path, |path, content| {
            let content_hash = blake3::hash(content.as_bytes());

            // Check content hash if we have a cached view
            if let Some(view) = cached_view.as_ref()
                && view.current().is_some_and(|v| {
                    v.hashes().is_content_match(content_hash.as_bytes())
                })
                && let Some(raw) = view.to_raw()?
            {
                return Ok(IngestResult::Fresh(raw));
            }

            // Parse from the content we just read (single file read)
            let mut raw: RawSchema =
                FsReader::parse_structured_from_str(path, content)?;
            raw.name = filename_stem.into();
            let mut raw = raw.validated(&path.to_string_lossy())?;

            raw.metadata = RawSchemaMetadata {
                created_at,
                modified_at,
            };

            // Create view with content for caching
            let view =
                RawSchemaView::try_from_with_content(&raw, &rel_path, content)?;

            let schema_id = self
                .repository
                .find_schema_id_by_path(&rel_path)
                .map_err(|e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!(
                        "Failed to query schema ID for {rel_path}: {e}"
                    )
                    .into(),
                })?
                .unwrap_or_else(SchemaId::new);

            self.repository.save_raw_schema_view(schema_id, &view).map_err(
                |e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!(
                        "Failed to save schema view for {rel_path}: {e}"
                    )
                    .into(),
                },
            )?;

            Ok(IngestResult::Stale(raw))
        })
    }

    /// Scan the schemas directory for all schema files.
    ///
    /// Uses [`schema()`](Self::schema) internally for each discovered file.
    /// Supports JSON, TOML, and YAML formats. The property bank file is
    /// excluded.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if the directory cannot be scanned or
    /// any schema file cannot be read or parsed.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// let results = ingestor.all_schemas()?;
    /// for result in results {
    ///     let schema = result.into_inner();
    /// }
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn all_schemas(
        &self,
    ) -> Result<Vec<IngestResult<RawSchema>>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();

        // Property bank is always in schemas_dir (joined by
        // property_bank_path()) We exclude it from schema scanning
        // since it's loaded separately
        let property_bank_filename = paths.property_bank.as_str();

        let mut results = Vec::new();

        // Scan for each supported extension
        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = self.source.list_files(&pattern).map_err(|error| {
                SchemaIngestionError::FileSystem(error.to_string().into())
            })?;

            for path in files {
                // Exclude property bank file (glob crate doesn't support
                // negation)
                if path
                    .file_name()
                    .is_some_and(|name| name == property_bank_filename)
                {
                    continue;
                }

                let result = self.schema(&path)?;
                results.push(result);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "staleness_tests submodule placed after existing tests for \
              clarity"
)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        fs::FsReader,
        schema::{raw::RawSchemaVersion, storage::RedbRepository},
    };

    fn write_file(root: &Path, relative: &str, content: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create test dirs");
        }
        std::fs::write(&path, content).expect("write test file");
        path
    }

    fn test_config(root: &Path, property_bank_file: Option<&str>) -> Config {
        Config::build(
            &RawConfig {
                paths: crate::config::raw::RawPathsConfig {
                    property_bank_file: property_bank_file
                        .map(ToOwned::to_owned),
                    ..Default::default()
                },
                ..Default::default()
            },
            VaultId::new(),
            VaultRoot::try_new(root.to_path_buf()).expect("vault root"),
            crate::config::aggregate::Version::initial(),
        )
        .expect("failed to build test config")
    }

    fn test_repository(root: &Path) -> RedbRepository {
        let db_path = root.join(".lithos").join("test.redb");
        std::fs::create_dir_all(db_path.parent().unwrap())
            .expect("create db dir");
        let db = std::sync::Arc::new(
            crate::db::Database::open(&db_path).expect("create test database"),
        );
        RedbRepository::new(db)
    }

    #[test]
    fn property_bank_parses_valid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank_result = result.expect("Should parse property bank");
        assert!(bank_result.is_new() || bank_result.is_stale());
        assert!(bank_result.bank().all().count() == 0);
    }

    #[test]
    fn property_bank_parses_valid_yaml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.yaml",
            "$version: \"1.0\"\nproperties: {}",
        );

        let config = test_config(dir.path(), Some("property_bank.yaml"));
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank_result = result.expect("Should parse property bank");
        assert!(bank_result.is_new());
        assert!(bank_result.bank().all().count() == 0);
    }

    #[test]
    fn property_bank_parses_valid_toml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.toml",
            "\"$version\" = \"1.0\"\n[properties]",
        );

        let config = test_config(dir.path(), Some("property_bank.toml"));
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank_result = result.expect("Should parse property bank");
        assert!(bank_result.is_new() || bank_result.is_stale());
        assert!(bank_result.bank().all().count() == 0);
    }

    #[test]
    fn property_bank_returns_error_for_invalid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/property_bank.json", "not valid json");

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail to parse");
        assert!(matches!(err, SchemaIngestionError::Json { .. }));
    }

    #[test]
    fn property_bank_returns_error_for_unsupported_format() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.xml",
            "<properties></properties>",
        );

        let config = test_config(dir.path(), Some("property_bank.xml"));
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail for unsupported format");
        assert!(matches!(err, SchemaIngestionError::UnsupportedFormat { .. }));
    }

    #[test]
    fn all_schemas_returns_schemas() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/note.json",
            r#"{"$version": "1.0", "name": "note", "properties": {}}"#,
        );
        write_file(
            dir.path(),
            "schemas/task.yaml",
            "$version: \"1.0\"\nname: task\nproperties: {}",
        );
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schema_results = result.expect("Should scan schemas");
        assert_eq!(schema_results.len(), 2);

        let names: Vec<&str> =
            schema_results.iter().map(|r| r.as_ref().name.as_ref()).collect();
        assert!(names.contains(&"note"));
        assert!(names.contains(&"task"));
    }

    #[test]
    fn all_schemas_supports_toml_format() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/project.toml",
            r#""$version" = "1.0"
[properties]"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schema_results = result.expect("Should scan schemas");
        assert_eq!(schema_results.len(), 1);
        let schema_result =
            schema_results.first().expect("should have one schema");
        assert_eq!(schema_result.as_ref().name.as_ref(), "project");
    }

    #[test]
    fn all_schemas_excludes_property_bank() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schema_results = result.expect("Should scan schemas");
        assert!(schema_results.is_empty());
    }

    #[test]
    fn property_bank_defaults_version_when_omitted() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank_result = result.expect("Should parse property bank");
        assert_eq!(bank_result.bank().version().to_string(), "v0");
    }

    #[test]
    fn all_schemas_defaults_version_when_omitted() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/note.json",
            r#"{"name": "note", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let repository = test_repository(dir.path());
        let ingestor =
            Ingestor::new(FsReader::new(dir.path()), &config, repository);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schema_results = result.expect("Should scan schemas");
        assert_eq!(schema_results.len(), 1);
        let schema_result =
            schema_results.first().expect("should have one schema");
        assert_eq!(schema_result.as_ref().version.as_ref(), "1.0");
    }

    /// Staleness detection tests.
    ///
    /// These tests verify the end-to-end staleness detection optimization:
    /// - Fresh files return Fresh variant (no re-parsing needed)
    /// - Stale files return Stale variant (need re-resolution)
    /// - Content hash takes precedence over timestamps (clock skew handling)
    #[expect(
        clippy::disallowed_methods,
        reason = "Tests use std::thread::sleep for filesystem timestamp \
                  resolution"
    )]
    mod staleness_tests {
        use super::*;

        /// Helper to create a shared database for tests that need multiple
        /// repository instances.
        fn test_database(root: &Path) -> std::sync::Arc<crate::db::Database> {
            let db_path = root.join(".lithos").join("test.redb");
            std::fs::create_dir_all(db_path.parent().unwrap())
                .expect("create db dir");
            std::sync::Arc::new(
                crate::db::Database::open(&db_path)
                    .expect("create test database"),
            )
        }

        /// Test: Property bank view is persisted and can be queried.
        ///
        /// NOTE: This test verifies that views are saved correctly. The full
        /// Fresh optimization (returning cached data without re-parsing)
        /// requires `to_raw()` implementation (Phase 6 - not yet implemented).
        /// Currently `to_raw()` returns None, so files are always re-parsed.
        #[test]
        fn property_bank_view_persisted() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"text": {"type": "string"}}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // First load - should be New (new file, no cached view)
            let first_result =
                ingestor.property_bank().expect("first load should succeed");
            assert!(
                first_result.is_new(),
                "First load should be New (new file)"
            );

            // Verify view was saved to database
            let repository2 = RedbRepository::new(db);
            let cached_view = repository2
                .get_raw_property_bank_view()
                .expect("should query view");
            assert!(
                cached_view.is_some(),
                "View should be saved after first load"
            );

            // Verify view has correct structure
            let view = cached_view.unwrap();
            assert!(
                view.current().is_some(),
                "View should have current version"
            );
        }

        /// Test: Stale property bank detected by timestamp mismatch.
        #[test]
        fn stale_property_bank_by_timestamp() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // First load
            let first_result = ingestor.property_bank().expect("first load");
            assert!(first_result.is_new());

            // Modify file (change content and timestamp)
            std::thread::sleep(std::time::Duration::from_millis(10));
            write_file(
                dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"new_prop": {"type": "string"}}}"#,
            );

            // Second load - should detect staleness
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_result = ingestor2.property_bank().expect("second load");
            assert!(
                second_result.is_stale(),
                "Modified file should be detected as Stale"
            );

            // Verify content changed
            assert_eq!(first_result.bank().all().count(), 0);
            assert_eq!(second_result.bank().all().count(), 1);
        }

        /// Test: Stale property bank detected by content hash mismatch.
        #[test]
        fn stale_property_bank_by_hash() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // First load
            let first_result = ingestor.property_bank().expect("first load");
            assert!(first_result.is_new());

            // Modify file content (hash will change)
            std::thread::sleep(std::time::Duration::from_millis(10));
            write_file(
                dir.path(),
                "schemas/property_bank.json",
                r#"{"$version": "1.0", "properties": {"added": {"type": "number"}}}"#,
            );

            // Second load - should detect hash mismatch
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_result = ingestor2.property_bank().expect("second load");
            assert!(
                second_result.is_stale(),
                "Changed content should be detected by hash"
            );
        }

        /// Test: Fresh schema returns Fresh variant.
        #[test]
        fn fresh_schema_returns_fresh() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/note.json",
                r#"{"$version": "1.0", "name": "note", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // Construct path to schema file
            let schema_path = dir.path().join("schemas/note.json");

            // First load - should be Stale
            let first_result = ingestor
                .schema(&schema_path)
                .expect("first load should succeed");
            assert!(
                first_result.is_stale(),
                "First load should be Stale (new file)"
            );

            // Second load - should be Fresh
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_result = ingestor2
                .schema(&schema_path)
                .expect("second load should succeed");
            assert!(
                second_result.is_fresh(),
                "Second load should be Fresh (unchanged file)"
            );

            // Verify content is identical
            assert_eq!(first_result.as_ref().name, second_result.as_ref().name);
        }

        /// Test: Stale schema detected by modification.
        #[test]
        fn stale_schema_by_modification() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "name": "task", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // Construct path to schema file
            let schema_path = dir.path().join("schemas/task.json");

            // First load
            let first_result =
                ingestor.schema(&schema_path).expect("first load");
            assert!(first_result.is_stale());

            // Modify schema file
            std::thread::sleep(std::time::Duration::from_millis(10));
            write_file(
                dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "name": "task", "properties": {"title": {"type": "string"}}}"#,
            );

            // Second load - should detect staleness
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_result =
                ingestor2.schema(&schema_path).expect("second load");
            assert!(
                second_result.is_stale(),
                "Modified schema should be detected as Stale"
            );

            // Verify content changed
            assert_eq!(first_result.as_ref().properties.len(), 0);
            assert_eq!(second_result.as_ref().properties.len(), 1);
        }

        /// Test: New schema is detected as stale (no view exists).
        #[test]
        fn new_schema_detected() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/project.json",
                r#"{"$version": "1.0", "name": "project", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let repository = test_repository(dir.path());
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // Construct path to schema file
            let schema_path = dir.path().join("schemas/project.json");

            // First load of new schema - always Stale
            let result = ingestor.schema(&schema_path).expect("should load");
            assert!(result.is_stale(), "New schema should always be Stale");
        }

        /// Test: `all_schemas()` returns mix of Fresh and Stale.
        #[test]
        fn all_schemas_mixed_staleness() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/note.json",
                r#"{"$version": "1.0", "name": "note", "properties": {}}"#,
            );
            write_file(
                dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "name": "task", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // First load - both Stale
            let first_results =
                ingestor.all_schemas().expect("first load should succeed");
            assert_eq!(first_results.len(), 2);
            assert!(
                first_results
                    .iter()
                    .all(|r: &IngestResult<RawSchema>| r.is_stale())
            );

            // Modify only one schema
            std::thread::sleep(std::time::Duration::from_millis(10));
            write_file(
                dir.path(),
                "schemas/task.json",
                r#"{"$version": "1.0", "name": "task", "properties": {"modified": {"type": "bool"}}}"#,
            );

            // Second load - note Fresh, task Stale
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_results =
                ingestor2.all_schemas().expect("second load should succeed");
            assert_eq!(second_results.len(), 2);

            let note_result = second_results
                .iter()
                .find(|r| r.as_ref().name.as_ref() == "note")
                .expect("should find note");
            let task_result = second_results
                .iter()
                .find(|r| r.as_ref().name.as_ref() == "task")
                .expect("should find task");

            assert!(note_result.is_fresh(), "Unchanged note should be Fresh");
            assert!(task_result.is_stale(), "Modified task should be Stale");
        }

        /// Test: Path-based lookup finds correct view.
        #[test]
        fn path_based_lookup_finds_view() {
            let dir = TempDir::new().expect("tempdir");
            write_file(
                dir.path(),
                "schemas/note.json",
                r#"{"$version": "1.0", "name": "note", "properties": {}}"#,
            );

            let config = test_config(dir.path(), None);
            let db = test_database(dir.path());
            let repository = RedbRepository::new(std::sync::Arc::clone(&db));
            let ingestor =
                Ingestor::new(FsReader::new(dir.path()), &config, repository);

            // Construct path to schema file
            let schema_path = dir.path().join("schemas/note.json");

            // First load to populate view
            let first_result =
                ingestor.schema(&schema_path).expect("should load");
            assert!(first_result.is_stale());

            // Second load should find by path
            let repository2 = RedbRepository::new(db);
            let ingestor2 =
                Ingestor::new(FsReader::new(dir.path()), &config, repository2);
            let second_result =
                ingestor2.schema(&schema_path).expect("should load");
            assert!(
                second_result.is_fresh(),
                "Path lookup should return Fresh"
            );
        }
    }

    /// Unit tests for `PropertyBankResult`.
    mod property_bank_result_tests {
        use super::*;
        use crate::schema::{bank::PropertyBank, property::PropertyName};

        #[test]
        fn new_variant_returns_bank() {
            let bank = PropertyBank::default();
            let result = PropertyBankResult::New(bank.clone());

            assert_eq!(result.bank(), &bank);
            assert!(result.is_new());
            assert!(!result.is_fresh());
            assert!(!result.is_stale());
            assert_eq!(result.changed_properties(), &[]);
        }

        #[test]
        fn fresh_variant_returns_bank() {
            let bank = PropertyBank::default();
            let result = PropertyBankResult::Fresh(bank.clone());

            assert_eq!(result.bank(), &bank);
            assert!(result.is_fresh());
            assert!(!result.is_new());
            assert!(!result.is_stale());
            assert_eq!(result.changed_properties(), &[]);
        }

        #[test]
        fn stale_variant_returns_bank_and_changed() {
            let bank = PropertyBank::default();
            let prop1 = PropertyName::try_new("title").unwrap();
            let prop2 = PropertyName::try_new("status").unwrap();
            let changed = vec![prop1.clone(), prop2.clone()];

            let result = PropertyBankResult::Stale {
                bank: bank.clone(),
                changed: changed.clone(),
            };

            assert_eq!(result.bank(), &bank);
            assert!(result.is_stale());
            assert!(!result.is_new());
            assert!(!result.is_fresh());
            assert_eq!(result.changed_properties(), changed.as_slice());
        }

        #[test]
        fn into_bank_consumes_and_returns_bank() {
            let bank = PropertyBank::default();
            let result = PropertyBankResult::New(bank.clone());

            let extracted = result.into_bank();
            assert_eq!(extracted, bank);
        }

        #[test]
        fn into_bank_works_for_all_variants() {
            let bank = PropertyBank::default();

            let new_result = PropertyBankResult::New(bank.clone());
            assert_eq!(new_result.into_bank(), bank);

            let fresh_result = PropertyBankResult::Fresh(bank.clone());
            assert_eq!(fresh_result.into_bank(), bank);

            let stale_result = PropertyBankResult::Stale {
                bank: bank.clone(),
                changed: vec![],
            };
            assert_eq!(stale_result.into_bank(), bank);
        }
    }

    /// Unit tests for `SchemaIngestResult`.
    mod schema_ingest_result_tests {
        use super::*;

        #[test]
        fn new_variant_returns_id_and_raw() {
            let id = SchemaId::new();
            let raw = RawSchema {
                version: RawSchemaVersion::default(),
                name: "test".into(),
                extends: None,
                excludes: vec![],
                properties: std::collections::HashMap::new(),
                metadata: RawSchemaMetadata::default(),
            };

            let result = SchemaIngestResult::New {
                id,
                raw: raw.clone(),
            };

            assert_eq!(result.id(), id);
            assert!(result.is_new());
            assert!(!result.is_fresh());
            assert!(!result.is_stale());
            assert!(result.raw().is_some());
            assert!(result.expanded().is_none());
        }

        #[test]
        fn fresh_variant_returns_id_and_expanded() {
            let id = SchemaId::new();
            let expanded = Some(std::collections::HashMap::new());

            let result = SchemaIngestResult::Fresh {
                id,
                expanded: expanded.clone(),
            };

            assert_eq!(result.id(), id);
            assert!(result.is_fresh());
            assert!(!result.is_new());
            assert!(!result.is_stale());
            assert!(result.raw().is_none());
            assert_eq!(result.expanded(), expanded.as_ref());
        }

        #[test]
        fn stale_variant_returns_id_raw_and_expanded() {
            let id = SchemaId::new();
            let raw = RawSchema {
                version: RawSchemaVersion::default(),
                name: "test".into(),
                extends: None,
                excludes: vec![],
                properties: std::collections::HashMap::new(),
                metadata: RawSchemaMetadata::default(),
            };
            let expanded = Some(std::collections::HashMap::new());

            let result = SchemaIngestResult::Stale {
                id,
                raw: raw.clone(),
                expanded: expanded.clone(),
            };

            assert_eq!(result.id(), id);
            assert!(result.is_stale());
            assert!(!result.is_new());
            assert!(!result.is_fresh());
            assert!(result.raw().is_some());
            assert_eq!(result.expanded(), expanded.as_ref());
        }
    }
}
