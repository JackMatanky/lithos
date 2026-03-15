//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.
//!
//! ## Pattern
//!
//! Following the config ingestor pattern:
//! - Single method per entity (`property_bank()`, `schema()`, `all_schemas()`)
//! - Metadata populated in Raw* types (no separate tuples)
//! - Returns `Option<T>` for optional files, `Result<Vec<T>>` for collections

use std::path::Path;

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema, RawSchemaMetadata},
    },
};

/// Supported schema file extensions.
const SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"];

/// Ingestor for loading raw schema files from a file source.
///
/// This adapter is responsible for:
/// - Loading the property bank file (JSON, TOML, or YAML)
/// - Scanning the schemas directory for schema files
/// - Deserializing files into raw types using format auto-detection
///
/// It does NOT:
/// - Assign IDs
/// - Query the database
/// - Perform validation beyond deserialization
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
/// let ingestor = Ingestor::new(FsReader::new(root), &config);
/// let _ = ingestor;
/// ```
pub struct Ingestor<'config> {
    source: FsReader,
    config: &'config Config,
}

impl<'config> Ingestor<'config> {
    /// Create a new ingestor with the given file source and config.
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
    /// let ingestor = Ingestor::new(FsReader::new(root), &config);
    /// let _ = ingestor;
    /// ```
    #[inline]
    #[must_use]
    pub fn new(source: FsReader, config: &'config Config) -> Self {
        Self {
            source,
            config,
        }
    }
}

impl Ingestor<'_> {
    /// Get the property bank file with metadata extraction.
    ///
    /// Reads from the configured property bank path, then:
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Computes BLAKE3 content hash
    /// - Parses content into [`RawPropertyBank`]
    /// - Populates metadata fields on the returned type
    ///
    /// Supports JSON, TOML, and YAML formats (detected by extension or
    /// content).
    ///
    /// Returns `None` if the property bank file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if:
    /// - File reading fails (I/O error)
    /// - Parsing fails (syntax error)
    /// - Version validation fails
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// let bank = ingestor.property_bank()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn property_bank(
        &self,
    ) -> Result<Option<RawPropertyBank>, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();

        if !self.source.exists(&path) {
            return Ok(None);
        }

        // Extract timestamps
        let created_at = self.source.created_at(&path);
        let modified_at = self.source.modified_at(&path);

        // Read raw bytes for content hashing (before parsing)
        let raw_bytes = self.source.read_bytes(&path)?;
        let content_hash = blake3::hash(&raw_bytes);

        // Parse structured data
        let mut bank: RawPropertyBank = self.source.parse_structured(&path)?;
        bank.validate_version(&path.to_string_lossy())?;
        bank.validate()?;

        // Populate metadata with property hashes
        bank.metadata = RawSchemaMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
            property_hashes: RawSchemaMetadata::compute_property_hashes(
                &bank.properties,
            ),
        };

        Ok(Some(bank))
    }

    /// Load a single schema file with metadata extraction.
    ///
    /// Reads the schema file at the given path, then:
    /// - Derives schema name from filename (without extension)
    /// - Extracts filesystem timestamps (`created_at`, `modified_at`)
    /// - Computes BLAKE3 content hash
    /// - Computes per-property hashes for incremental resolution
    /// - Parses content into [`RawSchema`]
    /// - Populates metadata fields on the returned type
    ///
    /// # Errors
    ///
    /// Returns [`SchemaIngestionError`] if:
    /// - File reading fails (I/O error)
    /// - Parsing fails (syntax error)
    /// - Version validation fails
    /// - Filename is invalid
    #[inline]
    pub fn schema(
        &self,
        path: &Path,
    ) -> Result<RawSchema, SchemaIngestionError> {
        // Derive schema name from filename (without extension)
        let filename_stem =
            path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                SchemaIngestionError::FileSystem(
                    format!("Invalid filename for schema: {}", path.display())
                        .into(),
                )
            })?;

        // Extract timestamps
        let created_at = self.source.created_at(path);
        let modified_at = self.source.modified_at(path);

        // Read raw bytes for content hashing (before parsing)
        let raw_bytes = self.source.read_bytes(path)?;
        let content_hash = blake3::hash(&raw_bytes);

        // Parse structured data
        let mut raw: RawSchema = self.source.parse_structured(path)?;
        raw.validate_version(&path.to_string_lossy())?;

        // Set name from filename (always, not from file content)
        raw.name = filename_stem.into();

        // Validate syntax
        raw.validate()?;

        // Populate metadata with property hashes
        raw.metadata = RawSchemaMetadata {
            created_at,
            modified_at,
            content_hash: Some(*content_hash.as_bytes()),
            property_hashes: RawSchemaMetadata::compute_property_hashes(
                &raw.properties,
            ),
        };

        Ok(raw)
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
    /// let schemas = ingestor.all_schemas()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn all_schemas(&self) -> Result<Vec<RawSchema>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();
        
        // Property bank is always in schemas_dir (joined by property_bank_path())
        // We exclude it from schema scanning since it's loaded separately
        let property_bank_filename = paths.property_bank.as_str();

        let mut results = Vec::new();

        // Scan for each supported extension
        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = self.source.list_files(&pattern).map_err(|error| {
                SchemaIngestionError::FileSystem(error.to_string().into())
            })?;

            for path in files {
                // Exclude property bank file (glob crate doesn't support negation)
                if path.file_name().is_some_and(|name| name == property_bank_filename) {
                    continue;
                }

                let raw = self.schema(&path)?;
                results.push(raw);
            }
        }

                // Use schema() method to load each file
                let raw = self.schema(&path)?;
                results.push(raw);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
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

    #[test]
    fn property_bank_parses_valid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank.properties.is_empty());
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank.properties.is_empty());
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn property_bank_returns_error_for_invalid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/property_bank.json", "not valid json");

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> =
            schemas.iter().map(|s| s.name.as_ref()).collect();
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 1);
        let schema = schemas.first().expect("should have one schema");
        assert_eq!(schema.name.as_ref(), "project");
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert!(schemas.is_empty());
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.property_bank();

        assert!(result.is_ok());
        let bank = result
            .expect("Should parse property bank")
            .expect("Should have bank");
        assert_eq!(bank.version.as_ref(), "1.0");
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
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.all_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 1);
        let schema = schemas.first().expect("should have one schema");
        assert_eq!(schema.version.as_ref(), "1.0");
    }
}
