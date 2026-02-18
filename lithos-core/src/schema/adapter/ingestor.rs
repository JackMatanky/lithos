//! Ingestor adapter for loading raw schema files from the filesystem.
//!
//! Pure file-to-raw translation. No DB access. No ID assignment.

use glob::glob;

use crate::{
    config::aggregate::Config,
    fs::{Json, Toml, Yaml},
    schema::{
        aggregate::Timestamp,
        error::SchemaIngestionError,
        raw::{RawPropertyBank, RawSchema},
    },
};

/// Supported schema file extensions.
const SCHEMA_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml"];

/// A raw schema with optional file modification time.
pub type RawSchemaWithMtime = (RawSchema, Option<Timestamp>);

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
pub struct Ingestor<'config> {
    config: &'config Config,
}

impl<'config> Ingestor<'config> {
    /// Create a new ingestor with the given file source and config.
    ///
    /// The config reference ensures paths are the final merged values.
    #[inline]
    #[must_use]
    pub const fn new(config: &'config Config) -> Self {
        Self {
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
    #[inline]
    pub fn load_raw_property_bank(
        &self,
    ) -> Result<RawPropertyBank, SchemaIngestionError> {
        let relative_path = self.config.paths().property_bank_path();
        let path =
            self.config.vault_metadata().root().as_path().join(relative_path);
        let content = std::fs::read_to_string(&path).map_err(|error| {
            SchemaIngestionError::Io {
                path: path.to_string_lossy().into(),
                reason: error.to_string().into(),
            }
        })?;

        if Json::is_supported(&path) {
            return Json::parse(&path, &content)
                .map_err(SchemaIngestionError::from);
        }
        if Toml::is_supported(&path) {
            return Toml::parse(&path, &content)
                .map_err(SchemaIngestionError::from);
        }
        if Yaml::is_supported(&path) {
            return Yaml::parse(&path, &content)
                .map_err(SchemaIngestionError::from);
        }

        Err(SchemaIngestionError::UnsupportedFormat {
            path: path.to_string_lossy().into(),
            supported: "json, toml, yaml, yml".into(),
        })
    }

    /// Scan the schemas directory for all schema files.
    ///
    /// Returns a vector of (`RawSchema`, file modification time) pairs.
    /// Supports JSON, TOML, and YAML formats. The property bank file is
    /// excluded.
    ///
    /// # Errors
    /// Returns `SchemaIngestionError` if the directory cannot be scanned or
    /// files cannot be read or parsed.
    #[inline]
    pub fn scan_raw_schemas(
        &self,
    ) -> Result<Vec<RawSchemaWithMtime>, SchemaIngestionError> {
        let paths = self.config.paths();
        let schemas_dir = self
            .config
            .vault_metadata()
            .root()
            .as_path()
            .join(paths.schema.schemas_dir().as_path());
        let property_bank_filename = paths.property_bank.as_str();

        let mut results = Vec::new();

        // Scan for each supported extension
        for ext in SCHEMA_EXTENSIONS {
            let pattern = format!("{}/**/*.{}", schemas_dir.display(), ext);
            let files = glob(&pattern).map_err(|error| {
                SchemaIngestionError::FileSystem(error.to_string().into())
            })?;

            for entry in files {
                let path = entry.map_err(|error| {
                    SchemaIngestionError::FileSystem(error.to_string().into())
                })?;
                if !path.is_file() {
                    continue;
                }
                // Skip the property bank file
                if path
                    .file_name()
                    .is_some_and(|name| name == property_bank_filename)
                {
                    continue;
                }

                let content =
                    std::fs::read_to_string(&path).map_err(|error| {
                        SchemaIngestionError::Io {
                            path: path.to_string_lossy().into(),
                            reason: error.to_string().into(),
                        }
                    })?;

                let raw = if Json::is_supported(&path) {
                    Json::parse(&path, &content)
                        .map_err(SchemaIngestionError::from)?
                } else if Toml::is_supported(&path) {
                    Toml::parse(&path, &content)
                        .map_err(SchemaIngestionError::from)?
                } else if Yaml::is_supported(&path) {
                    Yaml::parse(&path, &content)
                        .map_err(SchemaIngestionError::from)?
                } else {
                    return Err(SchemaIngestionError::UnsupportedFormat {
                        path: path.to_string_lossy().into(),
                        supported: "json, toml, yaml, yml".into(),
                    });
                };

                // File modification time not available from FileSource trait
                // This is intentional - the trait is minimal and cross-platform
                results.push((raw, None));
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
    use crate::config::{
        aggregate::Config,
        raw::RawConfig,
        vault::{VaultId, VaultRoot},
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
        )
        .expect("failed to build test config")
    }

    #[test]
    fn load_raw_property_bank_parses_valid_json() {
        let dir = TempDir::new().expect("tempdir");
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(&config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_parses_valid_yaml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/property_bank.yaml", "properties: {}");

        let config = test_config(dir.path(), Some("property_bank.yaml"));
        let ingestor = Ingestor::new(&config);
        let result = ingestor.load_raw_property_bank();

        assert!(result.is_ok());
        let bank = result.expect("Should parse property bank");
        assert!(bank.properties.is_empty());
    }

    #[test]
    fn load_raw_property_bank_parses_valid_toml() {
        let dir = TempDir::new().expect("tempdir");
        write_file(dir.path(), "schemas/property_bank.toml", "[properties]");

        let config = test_config(dir.path(), Some("property_bank.toml"));
        let ingestor = Ingestor::new(&config);
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
        let ingestor = Ingestor::new(&config);
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
        let ingestor = Ingestor::new(&config);
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
            r#"{"name": "note", "properties": {}}"#,
        );
        write_file(
            dir.path(),
            "schemas/task.yaml",
            "name: task\nproperties: {}",
        );
        write_file(
            dir.path(),
            "schemas/property_bank.json",
            r#"{"properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(&config);
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
            r#"name = "project"
 [properties]"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(&config);
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
            r#"{"properties": {}}"#,
        );

        let config = test_config(dir.path(), None);
        let ingestor = Ingestor::new(&config);
        let result = ingestor.scan_raw_schemas();

        assert!(result.is_ok());
        let schemas = result.expect("Should scan schemas");
        assert!(schemas.is_empty());
    }
}
