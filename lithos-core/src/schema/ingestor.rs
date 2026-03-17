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

use std::{collections::BTreeMap, path::Path};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema, RawSchemaMetadata},
        storage::Repository,
        views::raw::RawFileVersion,
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
    pub fn property_bank(
        &self,
    ) -> Result<Option<IngestResult<RawPropertyBank>>, SchemaIngestionError>
    {
        let path = self.config.paths().property_bank_path();

        if !self.source.exists(&path) {
            return Ok(None);
        }

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

        // Fast path: Check timestamps (no file I/O)
        if let Some(view) = cached_view.as_ref()
            && view
                .current()
                .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
            && let Some(raw) = view.to_raw()
        {
            return Ok(Some(IngestResult::Fresh(raw)));
        }

        // Slow path: Read file once for hash check, compression, and parsing
        self.source.read_with(&path, |path, content| {
            let content_hash = blake3::hash(content.as_bytes());

            // Check content hash if we have a cached view
            if let Some(view) = cached_view.as_ref()
                && view.current().is_some_and(|v| {
                    v.is_content_match(content_hash.as_bytes())
                })
                && let Some(raw) = view.to_raw()
            {
                return Ok(Some(IngestResult::Fresh(raw)));
            }

            // File is stale or no cache - compress and parse from the content
            // we already read
            let compressed_content = RawFileVersion::compress_content(content)
                .map_err(|e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Compression failed: {e}").into(),
                })?;

            // Parse from the content we just read (single file read)
            let raw: RawPropertyBank =
                FsReader::parse_structured_from_str(path, content)?;
            let mut raw = raw.validated(&path.to_string_lossy())?;

            raw.metadata = crate::schema::raw::RawSchemaMetadata {
                created_at,
                modified_at,
                content_hash: Some(*content_hash.as_bytes()),
                property_hashes: BTreeMap::new(),
            };

            let view = crate::schema::views::raw::RawPropertyBankView::new(
                *content_hash.as_bytes(),
                BTreeMap::new(),
                created_at,
                modified_at,
                Some(compressed_content),
            );

            self.repository.save_raw_property_bank_view(&view).map_err(
                |e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Failed to save property bank view: {e}")
                        .into(),
                },
            )?;

            Ok(Some(IngestResult::Stale(raw)))
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
    #[expect(
        clippy::too_many_lines,
        reason = "Optimized flow to eliminate duplicate I/O - complex but \
                  linear"
    )]
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
            && view
                .current()
                .is_some_and(|v| v.is_timestamp_match(created_at, modified_at))
            && let Some(raw) = view.to_raw()
        {
            return Ok(IngestResult::Fresh(raw));
        }

        // Slow path: Read file once for hash check, compression, and parsing
        self.source.read_with(path, |path, content| {
            let content_hash = blake3::hash(content.as_bytes());

            // Check content hash if we have a cached view
            if let Some(view) = cached_view.as_ref()
                && view.current().is_some_and(|v| {
                    v.is_content_match(content_hash.as_bytes())
                })
                && let Some(raw) = view.to_raw()
            {
                return Ok(IngestResult::Fresh(raw));
            }

            // File is stale or no cache - compress and parse from the content
            // we already read
            let compressed_content = RawFileVersion::compress_content(content)
                .map_err(|e| SchemaIngestionError::Io {
                    path: path.to_string_lossy().into(),
                    reason: format!("Compression failed: {e}").into(),
                })?;

            // Parse from the content we just read (single file read)
            let mut raw: RawSchema =
                FsReader::parse_structured_from_str(path, content)?;
            raw.name = filename_stem.into();
            let mut raw = raw.validated(&path.to_string_lossy())?;

            let property_hashes =
                RawSchemaMetadata::compute_property_hashes(&raw.properties);

            raw.metadata = RawSchemaMetadata {
                created_at,
                modified_at,
                content_hash: Some(*content_hash.as_bytes()),
                property_hashes: property_hashes.clone(),
            };

            let extends = raw.extends.as_ref().and_then(|name| {
                crate::schema::aggregate::SchemaName::try_new(name.as_ref())
                    .ok()
            });

            let excludes: Vec<crate::schema::property::PropertyName> = raw
                .excludes
                .iter()
                .filter_map(|name| {
                    crate::schema::property::PropertyName::try_new(
                        name.as_ref(),
                    )
                    .ok()
                })
                .collect();

            let view = crate::schema::views::raw::RawSchemaView::new(
                rel_path.to_string().into_boxed_str(),
                extends,
                excludes,
                *content_hash.as_bytes(),
                property_hashes
                    .into_iter()
                    .filter_map(|(k, v)| {
                        crate::schema::property::PropertyName::try_new(
                            k.as_ref(),
                        )
                        .ok()
                        .map(|name| (name, v))
                    })
                    .collect(),
                created_at,
                modified_at,
                Some(compressed_content),
            );

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
                .unwrap_or_else(crate::schema::aggregate::SchemaId::new);

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
        schema::storage::RedbRepository,
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
        let bank_result = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank_result.is_stale()); // First load is always stale
        assert!(bank_result.as_ref().properties.is_empty());
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
        let bank_result = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank_result.is_stale());
        assert!(bank_result.as_ref().properties.is_empty());
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
        let bank_result = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank_result.is_stale());
        assert!(bank_result.as_ref().properties.is_empty());
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
        let bank_result = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert_eq!(bank_result.as_ref().version.as_ref(), "1.0");
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

            // First load - should be Stale (new file, no cached view)
            let first_result = ingestor
                .property_bank()
                .expect("first load should succeed")
                .expect("should have property bank");
            assert!(
                first_result.is_stale(),
                "First load should be Stale (new file)"
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
            let first_result = ingestor
                .property_bank()
                .expect("first load")
                .expect("should have bank");
            assert!(first_result.is_stale());

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
            let second_result = ingestor2
                .property_bank()
                .expect("second load")
                .expect("should have bank");
            assert!(
                second_result.is_stale(),
                "Modified file should be detected as Stale"
            );

            // Verify content changed
            assert_eq!(first_result.as_ref().properties.len(), 0);
            assert_eq!(second_result.as_ref().properties.len(), 1);
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
            let first_result = ingestor
                .property_bank()
                .expect("first load")
                .expect("should have bank");
            assert!(first_result.is_stale());

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
            let second_result = ingestor2
                .property_bank()
                .expect("second load")
                .expect("should have bank");
            assert!(
                second_result.is_stale(),
                "Changed content should be detected by hash"
            );
        }

        /// Test: Fresh schema returns Fresh variant.
        ///
        /// NOTE: Requires `to_raw()` implementation (Phase 6 - not yet
        /// implemented). Currently `to_raw()` returns None, so this
        /// test is ignored.
        #[test]
        #[ignore = "Requires to_raw() implementation (Phase 6)"]
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
        ///
        /// NOTE: Requires `to_raw()` implementation (Phase 6 - not yet
        /// implemented). Currently `to_raw()` returns None, so this
        /// test is ignored.
        #[test]
        #[ignore = "Requires to_raw() implementation (Phase 6)"]
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
        ///
        /// NOTE: Requires `to_raw()` implementation (Phase 6 - not yet
        /// implemented). Currently `to_raw()` returns None, so this
        /// test is ignored.
        #[test]
        #[ignore = "Requires to_raw() implementation (Phase 6)"]
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
}
