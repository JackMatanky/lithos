//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.

use crate::{
    config::paths::Paths,
    fs::source::FileSource,
    schema::{
        aggregate::Timestamp,
        raw::{RawPropertyBank, RawSchema},
    },
};

/// Errors that can occur during schema ingestion.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IngestionError {
    /// Failed to read file.
    #[error("Failed to read file {path}: {reason}")]
    ReadFailed {
        /// Path to the file.
        path: Box<str>,
        /// Reason for failure.
        reason: Box<str>,
    },

    /// Failed to parse file content.
    #[error("Failed to parse {path}: {reason}")]
    ParseFailed {
        /// Path to the file.
        path: Box<str>,
        /// Reason for failure.
        reason: Box<str>,
    },

    /// File system error.
    #[error("File system error: {0}")]
    FileSystem(Box<str>),
}

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
pub struct Ingestor<S> {
    source: S,
    paths: Paths,
}

impl<S> Ingestor<S> {
    /// Create a new ingestor with the given file source and paths.
    #[inline]
    #[must_use]
    pub const fn new(source: S, paths: Paths) -> Self {
        Self {
            source,
            paths,
        }
    }
}

impl<S: FileSource> Ingestor<S> {
    /// Load and deserialize the property bank file.
    ///
    /// # Errors
    /// Returns `IngestionError` if the file cannot be read or parsed.
    #[inline]
    pub fn load_raw_property_bank(
        &self,
    ) -> Result<RawPropertyBank, IngestionError> {
        let path = self.paths.property_bank_path();
        let content = self.source.read_to_string(&path).map_err(|e| {
            IngestionError::ReadFailed {
                path: path.to_string_lossy().into(),
                reason: e.to_string().into(),
            }
        })?;

        let raw: RawPropertyBank =
            serde_json::from_str(&content).map_err(|e| {
                IngestionError::ParseFailed {
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
    /// Returns `IngestionError` if the directory cannot be scanned or files
    /// cannot be read or parsed.
    #[inline]
    pub fn scan_raw_schemas(
        &self,
    ) -> Result<Vec<RawSchemaWithMtime>, IngestionError> {
        let schemas_dir = self.paths.schema.schemas_dir().as_path();
        let property_bank_filename = self.paths.property_bank.as_str();

        // Find all JSON files in the schemas directory
        let pattern = format!("{}/**/*.json", schemas_dir.display());
        let files = self
            .source
            .list_files(&pattern)
            .map_err(|e| IngestionError::FileSystem(e.to_string().into()))?;

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
                IngestionError::ReadFailed {
                    path: path.to_string_lossy().into(),
                    reason: e.to_string().into(),
                }
            })?;

            let raw: RawSchema =
                serde_json::from_str(&content).map_err(|e| {
                    IngestionError::ParseFailed {
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
    use std::path::Path;

    use super::*;
    use crate::{
        config::paths::{PropertyBank, Schema},
        fs::source::InMemoryFileSource,
    };

    fn test_paths() -> Paths {
        Paths::new(
            crate::config::paths::Cache::default(),
            crate::config::paths::Template::default(),
            Schema::default(),
            PropertyBank::default(),
        )
    }

    #[test]
    fn load_raw_property_bank_parses_valid_json() {
        let mut source = InMemoryFileSource::new();
        source.insert(
            Path::new("schemas/property_bank.json"),
            r#"{"properties": {}}"#.to_owned(),
        );

        let ingestor = Ingestor::new(source, test_paths());
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

        let ingestor = Ingestor::new(source, test_paths());
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_err());
        let err = result.expect_err("Should fail to parse");
        assert!(matches!(err, IngestionError::ParseFailed { .. }));
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

        let ingestor = Ingestor::new(source, test_paths());
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

        let ingestor = Ingestor::new(source, test_paths());
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert!(schemas.is_empty());
    }
}
