//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.

use crate::{
    config::aggregate::Config,
    fs::source::FileSource,
    schema::{
        aggregate::Timestamp,
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema},
    },
};

/// A raw schema with optional file modification time.
pub type RawSchemaWithMtime = (RawSchema, Option<Timestamp>);

/// Ingestor for loading raw schema files from a file source.
///
/// This adapter is responsible for:
/// - Loading the property bank JSON file
/// - Scanning the schemas directory for schema files
/// - Deserializing files into raw types
///
/// It does NOT:
/// - Assign IDs
/// - Query the database
/// - Perform validation beyond deserialization
///
/// The ingestor takes a `&Config` reference to ensure it uses the final
/// merged path values after config loading completes.
pub struct Ingestor<'config, S> {
    source: S,
    config: &'config Config,
}

impl<'config, S> Ingestor<'config, S> {
    /// Create a new ingestor with the given file source and config.
    ///
    /// The config reference ensures paths are the final merged values.
    #[inline]
    #[must_use]
    pub const fn new(source: S, config: &'config Config) -> Self {
        Self {
            source,
            config,
        }
    }
}

impl<S: FileSource> Ingestor<'_, S> {
    /// Load and deserialize the property bank file.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if the file cannot be read or parsed.
    #[inline]
    pub fn load_raw_property_bank(
        &self,
    ) -> Result<RawPropertyBank, SchemaIngestionError> {
        let path = self.config.paths().property_bank_path();
        let content = self.source.read_to_string(&path).map_err(|e| {
            SchemaIngestionError::ReadFailed {
                path: path.to_string_lossy().into(),
                reason: e.to_string().into(),
            }
        })?;

        let raw: RawPropertyBank =
            serde_json::from_str(&content).map_err(|e| {
                SchemaIngestionError::ParseFailed {
                    path: path.to_string_lossy().into(),
                    reason: e.to_string().into(),
                }
            })?;

        Ok(raw)
    }

    /// Scan the schemas directory for all schema files.
    ///
    /// Returns a vector of (`RawSchema`, file modification time) pairs.
    /// The property bank file is excluded from the results.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if the directory cannot be scanned or
    /// files cannot be read or parsed.
    #[inline]
    pub fn scan_raw_schemas(
        &self,
    ) -> Result<Vec<RawSchemaWithMtime>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = paths.schema.schemas_dir().as_path();
        let property_bank_filename = paths.property_bank.as_str();

        // Find all JSON files in the schemas directory
        let pattern = format!("{}/**/*.json", schemas_dir.display());
        let files = self.source.list_files(&pattern).map_err(|e| {
            SchemaIngestionError::FileSystem(e.to_string().into())
        })?;

        let mut results = Vec::with_capacity(files.len());

        for path in files {
            // Skip the property bank file
            if path
                .file_name()
                .is_some_and(|name| name == property_bank_filename)
            {
                continue;
            }

            let content = self.source.read_to_string(&path).map_err(|e| {
                SchemaIngestionError::ReadFailed {
                    path: path.to_string_lossy().into(),
                    reason: e.to_string().into(),
                }
            })?;

            let raw: RawSchema =
                serde_json::from_str(&content).map_err(|e| {
                    SchemaIngestionError::ParseFailed {
                        path: path.to_string_lossy().into(),
                        reason: e.to_string().into(),
                    }
                })?;

            // File modification time not available from FileSource trait
            // This is intentional - the trait is minimal and cross-platform
            results.push((raw, None));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::{
        config::{
            aggregate::Config,
            raw::RawConfig,
            vault::{VaultId, VaultRoot},
        },
        fs::source::InMemoryFileSource,
    };

    fn test_config() -> Config {
        Config::build(
            &RawConfig::default(),
            VaultId::new(),
            VaultRoot::try_new(PathBuf::from("/vault")).expect("vault root"),
        )
        .expect("failed to build test config")
    }

    #[test]
    fn load_raw_property_bank_parses_valid_json() {
        let mut source = InMemoryFileSource::new();
        source.insert(
            Path::new("schemas/property_bank.json"),
            r#"{"properties": {}}"#.to_owned(),
        );

        let config = test_config();
        let ingestor = Ingestor::new(source, &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_returns_error_for_invalid_json() {
        let mut source = InMemoryFileSource::new();
        source.insert(
            Path::new("schemas/property_bank.json"),
            "not valid json".to_owned(),
        );

        let config = test_config();
        let ingestor = Ingestor::new(source, &config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail to parse");
        assert!(matches!(err, SchemaIngestionError::ParseFailed { .. }));
    }

    #[test]
    fn scan_raw_schemas_returns_schemas() {
        let mut source = InMemoryFileSource::new();
        source.insert(
            Path::new("schemas/note.json"),
            r#"{"name": "note", "properties": {}}"#.to_owned(),
        );
        source.insert(
            Path::new("schemas/task.json"),
            r#"{"name": "task", "properties": {}}"#.to_owned(),
        );
        // Property bank should be excluded
        source.insert(
            Path::new("schemas/property_bank.json"),
            r#"{"properties": {}}"#.to_owned(),
        );

        let config = test_config();
        let ingestor = Ingestor::new(source, &config);
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
    fn scan_raw_schemas_excludes_property_bank() {
        let mut source = InMemoryFileSource::new();
        source.insert(
            Path::new("schemas/property_bank.json"),
            r#"{"properties": {}}"#.to_owned(),
        );

        let config = test_config();
        let ingestor = Ingestor::new(source, &config);
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert!(schemas.is_empty());
    }
}
