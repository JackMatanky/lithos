//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    schema::{
        aggregate::Timestamp,
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema},
    },
};

/// Supported schema file extensions.
const SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"];

/// A raw schema with optional filesystem timestamps (modified, created).
///
/// # Examples
/// ```ignore
/// use lithos_core::schema::adapter::ingestor::RawSchemaWithFileTimes;
///
/// let _tuple: RawSchemaWithFileTimes = todo!("Provide raw schema data");
/// ```
pub type RawSchemaWithFileTimes =
    (RawSchema, Option<Timestamp>, Option<Timestamp>);

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
/// use lithos_core::schema::adapter::ingestor::Ingestor;
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
    /// use lithos_core::schema::adapter::ingestor::Ingestor;
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
    /// Load and deserialize the property bank file.
    ///
    /// Supports JSON, TOML, and YAML formats (detected by extension or
    /// content).
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if the file cannot be read or parsed.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::adapter::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// let _bank = ingestor.load_raw_property_bank()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn load_raw_property_bank(
        &self,
    ) -> Result<RawPropertyBank, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();
        let bank: RawPropertyBank = self
            .source
            .parse_structured(&path)
            .map_err(SchemaIngestionError::from)?;
        bank.validate_version(&path.to_string_lossy())?;
        Ok(bank)
    }

    /// Scan the schemas directory for all schema files.
    ///
    /// Returns a vector of (`RawSchema`, modified time, created time) tuples.
    /// Supports JSON, TOML, and YAML formats. The property bank file is
    /// excluded.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if the directory cannot be scanned or
    /// files cannot be read or parsed.
    ///
    /// # Examples
    /// ```ignore
    /// # use lithos_core::schema::adapter::ingestor::Ingestor;
    /// # let ingestor = todo!("Provide an Ingestor instance");
    /// let _schemas = ingestor.scan_raw_schemas()?;
    /// # Ok::<_, lithos_core::schema::error::SchemaIngestionError>(())
    /// ```
    #[inline]
    pub fn scan_raw_schemas(
        &self,
    ) -> Result<Vec<RawSchemaWithFileTimes>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();
        let property_bank_filename = paths.property_bank.as_str();

        let mut results = Vec::new();

        // Scan for each supported extension
        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = self.source.list_files(&pattern).map_err(|error| {
                SchemaIngestionError::FileSystem(error.to_string().into())
            })?;

            for path in files {
                // Skip the property bank file
                if path
                    .file_name()
                    .is_some_and(|name| name == property_bank_filename)
                {
                    continue;
                }

                // Derive schema name from filename (without extension)
                let filename_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| {
                    SchemaIngestionError::FileSystem(
                        format!(
                            "Invalid filename for schema: {}",
                            path.display()
                        )
                        .into(),
                    )
                })?;

                // Parse the schema file
                let mut raw: RawSchema = self
                    .source
                    .parse_structured(&path)
                    .map_err(SchemaIngestionError::from)?;
                raw.validate_version(&path.to_string_lossy())?;

                // Set name from filename (always, not from file content)
                raw.name = filename_stem.into();

                // Extract timestamps using FsReader methods
                let modified =
                    self.source.modified_at(&path).map(Timestamp::from_secs);
                let created =
                    self.source.created_at(&path).map(Timestamp::from_secs);

                results.push((raw, modified, created));
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
    fn load_raw_property_bank_parses_valid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_parses_valid_yaml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.yaml",
            "$version: \"1.0\"\nproperties: {}",
        );

        let config = test_config(dir.path(), Some("property_bank.yaml"));
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_parses_valid_toml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.toml",
            "\"$version\" = \"1.0\"\n[properties]",
        );

        let config = test_config(dir.path(), Some("property_bank.toml"));
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_returns_error_for_invalid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/property_bank.json", "not valid json");

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail to parse");
        assert!(matches!(err, SchemaIngestionError::Json { .. }));
    }

    #[test]
    fn load_raw_property_bank_returns_error_for_unsupported_format() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.xml",
            "<properties></properties>",
        );

        let config = test_config(dir.path(), Some("property_bank.xml"));
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail for unsupported format");
        assert!(matches!(err, SchemaIngestionError::UnsupportedFormat { .. }));
    }

    #[test]
    fn scan_raw_schemas_returns_schemas() {
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
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> =
            schemas.iter().map(|tuple| tuple.0.name.as_ref()).collect();
        assert!(names.contains(&"note"));
        assert!(names.contains(&"task"));
    }

    #[test]
    fn scan_raw_schemas_supports_toml_format() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/project.toml",
            r#""$version" = "1.0"
[properties]"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 1);
        let schema = schemas.first().expect("should have one schema");
        assert_eq!(schema.0.name.as_ref(), "project");
    }

    #[test]
    fn scan_raw_schemas_excludes_property_bank() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"$version": "1.0", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert!(schemas.is_empty());
    }

    #[test]
    fn load_raw_property_bank_defaults_version_when_omitted() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert_eq!(bank.version.as_ref(), "1.0");
    }

    #[test]
    fn scan_raw_schemas_defaults_version_when_omitted() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/note.json",
            r#"{"name": "note", "properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(FsReader::new(dir.path()), &config);
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert_eq!(schemas.len(), 1);
        let schema = schemas.first().expect("should have one schema");
        assert_eq!(schema.0.version.as_ref(), "1.0");
    }
}
