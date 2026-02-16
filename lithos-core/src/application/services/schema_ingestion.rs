//! Schema ingestion service.
//!
//! This service orchestrates the file → raw → domain → database pipeline for
//! schema definitions.

use std::{io, path::Path};

use tracing::instrument;

use crate::{
    application::error::IngestionError,
    fs::{parsers::parse_file, source::FileSource},
    schema::{
        aggregate::{ResolutionMetadata, Schema, SchemaId},
        command::Command,
        ports as schema_ports,
        query::Query,
        raw::RawSchema,
    },
};

/// Schema file ingestion service.
///
/// This service handles reading schema definition files, parsing them into
/// domain aggregates, and persisting them to the database.
///
/// # Architecture
///
/// The service follows a strict pipeline:
///
/// 1. **Read**: Use `FileSource` to read file contents
/// 2. **Parse**: Deserialize JSON/TOML/YAML into `RawSchema`
/// 3. **Validate**: Convert `RawSchema` → `Schema` (domain validation)
/// 4. **Persist**: Save `Schema` via command port
///
/// # Type Parameters
///
/// - `Q`: Query port implementation for reading schemas
/// - `C`: Command port implementation for writing schemas
///
/// # Examples
///
/// ```rust,ignore
/// use std::path::Path;
///
/// use lithos_core::{
///     application::services::SchemaIngestionService,
///     fs::source::FsFileSource,
///     schema::{command::Command, query::Query},
/// };
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Set up infrastructure
/// # let db = lithos_core::db::Database::open_in_memory()?;
/// let query = Query::new(todo!());
/// let command = Command::new(todo!());
/// let source = FsFileSource::new(Path::new("/vault"));
///
/// // Create service
/// let service = SchemaIngestionService::new(&query, &command);
///
/// // Ingest a single schema file
/// let schema_id =
///     service.ingest_file(&source, Path::new("schemas/person.json"))?;
/// println!("Ingested schema: {}", schema_id);
///
/// // Ingest all schemas in a directory
/// let ids = service.ingest_directory(&source, "schemas/*.json")?;
/// println!("Ingested {} schemas", ids.len());
/// # Ok(())
/// # }
/// ```
#[expect(
    clippy::module_name_repetitions,
    reason = "Context-specific service name is intentional"
)]
pub struct SchemaIngestionService<'svc, Q, C> {
    /// Query port for reading existing schemas.
    #[expect(
        dead_code,
        reason = "Reserved for future incremental ingestion features (mtime \
                  tracking)"
    )]
    query: &'svc Query<Q>,
    /// Command port for persisting schemas.
    command: &'svc Command<C>,
}

impl<'svc, Q, C> SchemaIngestionService<'svc, Q, C>
where
    Q: schema_ports::Query,
    C: schema_ports::Command,
{
    /// Creates a new schema ingestion service.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::{
    ///     application::services::SchemaIngestionService,
    ///     fs::source::FsFileSource,
    ///     schema::{command::Command, query::Query},
    /// };
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = FsFileSource::new(Path::new("/vault"));
    /// let query = Query::new(todo!());
    /// let command = Command::new(todo!());
    /// let service = SchemaIngestionService::new(&query, &command);
    ///
    /// // Ingest all JSON schema files
    /// let ids = service.ingest_directory(&source, "schemas/*.json")?;
    /// println!("Successfully ingested {} schemas", ids.len());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(query: &'svc Query<Q>, command: &'svc Command<C>) -> Self {
        Self {
            query,
            command,
        }
    }

    /// Ingests a single schema file.
    ///
    /// This method reads the file, parses it into a `RawSchema`, validates it
    /// into a `Schema` aggregate, and persists it to the database.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (I/O error)
    /// - The file format is invalid (parse error)
    /// - The schema fails domain validation (validation error)
    /// - Database persistence fails (command error)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use lithos_core::{
    ///     application::services::SchemaIngestionService,
    ///     fs::source::InMemoryFileSource,
    ///     schema::{command::Command, query::Query},
    /// };
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut source = InMemoryFileSource::new();
    /// source.insert(
    ///     Path::new("person.json"),
    ///     r#"{"id": "...", "name": "Person", "properties": []}"#.to_owned(),
    /// );
    ///
    /// let query = Query::new(todo!());
    /// let command = Command::new(todo!());
    /// let service = SchemaIngestionService::new(&query, &command);
    ///
    /// let schema_id = service.ingest_file(&source, Path::new("person.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(skip(self, source), fields(path = %path.display()))]
    pub fn ingest_file<S>(
        &self,
        source: &S,
        path: &Path,
    ) -> Result<SchemaId, IngestionError>
    where
        S: FileSource<Error = io::Error>,
        C::Error: Into<crate::db::DbError>,
    {
        // Step 1: Parse file (FileSource → RawSchema)
        let raw: RawSchema = parse_file(source, path)?;

        // Step 2: Validate (RawSchema → Schema)
        // TODO: This is a simplified version. In production, this should go
        // through the schema resolver to handle inheritance, property
        // bank lookups, etc.
        let schema_name = crate::schema::aggregate::SchemaName::new(&raw.name)
            .map_err(|e| IngestionError::Validation(e.to_string()))?;

        // For now, create schema with empty properties (no resolution)
        // TODO: Add proper property resolution via SchemaResolver
        let schema_id = SchemaId::from_uuid(raw.id);
        let schema = Schema::new(schema_id, schema_name, vec![])
            .map_err(|e| IngestionError::Validation(e.to_string()))?;

        // Step 3: Persist (Schema → Database)
        let id = schema.id();
        let metadata = ResolutionMetadata::new(
            id,
            crate::schema::aggregate::Timestamp::now(),
            None,
            crate::schema::bank::BankVersion::initial(),
            None,
        );
        self.command.save_with_metadata(&schema, &metadata)?;

        tracing::info!(
            schema_id = %id,
            schema_name = %schema.name(),
            "Schema ingested successfully"
        );

        Ok(id)
    }

    /// Ingests all schema files matching a glob pattern.
    ///
    /// This method lists all files matching the pattern, then ingests each one.
    /// Failures are logged but do not stop processing (partial failure
    /// tolerance).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The glob pattern is invalid
    /// - Directory traversal fails
    ///
    /// Note: Individual file ingestion failures are logged but do not cause
    /// this method to fail. Check the returned vector length against the
    /// expected number of files to detect partial failures.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::path::Path;
    ///
    /// use lithos_core::{
    ///     application::services::SchemaIngestionService,
    ///     fs::source::FsFileSource,
    ///     schema::{command::Command, query::Query},
    /// };
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = FsFileSource::new(Path::new("/vault"));
    /// let query = Query::new(todo!());
    /// let command = Command::new(todo!());
    /// let service = SchemaIngestionService::new(&query, &command);
    ///
    /// // Ingest all JSON schema files
    /// let ids = service.ingest_directory(&source, "schemas/*.json")?;
    /// println!("Successfully ingested {} schemas", ids.len());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[instrument(skip(self, source))]
    pub fn ingest_directory<S>(
        &self,
        source: &S,
        pattern: &str,
    ) -> Result<Vec<SchemaId>, IngestionError>
    where
        S: FileSource<Error = io::Error>,
        C::Error: Into<crate::db::DbError>,
    {
        let paths = source.list_files(pattern).map_err(|e| {
            IngestionError::Parse(crate::fs::ParseError::Io {
                path: Path::new(pattern).into(),
                source: e,
            })
        })?;

        let mut ids = Vec::new();

        for path in paths {
            match self.ingest_file(source, &path) {
                Ok(id) => {
                    tracing::info!(
                        schema_id = %id,
                        path = %path.display(),
                        "Schema ingested from directory scan"
                    );
                    ids.push(id);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        path = %path.display(),
                        "Failed to ingest schema file (continuing with others)"
                    );
                    // Continue processing other files
                }
            }
        }

        tracing::info!(
            total_ingested = ids.len(),
            pattern = %pattern,
            "Directory ingestion complete"
        );

        Ok(ids)
    }

    /// Checks if a schema file needs to be re-ingested.
    ///
    /// This is a stub implementation that always returns `true`. In a future
    /// iteration, this should compare file modification time with the database
    /// timestamp to enable incremental ingestion.
    ///
    /// # Errors
    ///
    /// Currently never returns an error. Future implementations may return
    /// errors if:
    /// - File metadata cannot be read
    /// - Database query fails
    #[inline]
    #[instrument(skip(self, source))]
    pub fn needs_update<S>(
        &self,
        source: &S,
        path: &Path,
        id: SchemaId,
    ) -> Result<bool, IngestionError>
    where
        S: FileSource<Error = io::Error>,
    {
        // TODO: Compare file mtime with DB timestamp
        // For now, always return true (always re-ingest)
        let _: (&S, SchemaId) = (source, id); // Suppress unused warnings for stub implementation
        tracing::debug!(
            path = %path.display(),
            "needs_update stub: always returning true (no mtime tracking yet)"
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::{
        db::Database,
        fs::source::InMemoryFileSource,
        schema::{RedbSchemaCommand, RedbSchemaQuery},
    };

    fn test_db() -> Result<(TempDir, Database), String> {
        let dir = tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("schema.redb");
        let db = Database::open(&path).map_err(|e| e.to_string())?;
        Ok((dir, db))
    }

    #[test]
    fn schema_ingestion_service_new_creates_service() {
        let (_dir, db) = test_db().expect("Failed to create test db");
        let query = RedbSchemaQuery::new_redb(&db);
        let command = RedbSchemaCommand::new_redb(&db);

        let _service = SchemaIngestionService::new(&query, &command);
        // If we get here, construction succeeded
    }

    #[test]
    fn schema_ingestion_service_ingests_valid_json_file() {
        let (_dir, db) = test_db().expect("Failed to create test db");
        let query = RedbSchemaQuery::new_redb(&db);
        let command = RedbSchemaCommand::new_redb(&db);
        let service = SchemaIngestionService::new(&query, &command);

        let mut source = InMemoryFileSource::new();
        let schema_json = r#"{
            "id": "01933333-3333-7333-8333-333333333333",
            "name": "person",
            "properties": []
        }"#;
        source.insert(Path::new("person.json"), schema_json.to_owned());

        let schema_id = service
            .ingest_file(&source, Path::new("person.json"))
            .expect("Failed to ingest schema");

        // Verify schema was persisted
        let schemas = query.list().expect("Failed to list schemas");
        assert_eq!(schemas.len(), 1, "Should have exactly one schema");
        let stored_schema = schemas.first().expect("Schema should exist");
        assert_eq!(
            stored_schema.id(),
            schema_id,
            "Returned ID should match stored schema"
        );
        assert_eq!(
            stored_schema.name().as_str(),
            "person",
            "Schema name should be 'person'"
        );
    }

    #[test]
    fn schema_ingestion_service_rejects_invalid_schema() {
        let (_dir, db) = test_db().expect("Failed to create test db");
        let query = RedbSchemaQuery::new_redb(&db);
        let command = RedbSchemaCommand::new_redb(&db);
        let service = SchemaIngestionService::new(&query, &command);

        let mut source = InMemoryFileSource::new();
        // Invalid: empty name
        let invalid_json = r#"{
            "id": "01933333-3333-7333-8333-333333333333",
            "name": "",
            "properties": []
        }"#;
        source.insert(Path::new("invalid.json"), invalid_json.to_owned());

        let result = service.ingest_file(&source, Path::new("invalid.json"));

        assert!(result.is_err(), "Should reject schema with empty name");

        // Verify nothing was persisted
        let schemas = query.list().expect("Failed to list schemas");
        assert!(schemas.is_empty(), "Should not persist invalid schema");
    }

    #[test]
    fn schema_ingestion_service_handles_partial_directory_failures() {
        let (_dir, db) = test_db().expect("Failed to create test db");
        let query = RedbSchemaQuery::new_redb(&db);
        let command = RedbSchemaCommand::new_redb(&db);
        let service = SchemaIngestionService::new(&query, &command);

        let mut source = InMemoryFileSource::new();

        // Valid schema
        let valid_json = r#"{
            "id": "01933333-3333-7333-8333-333333333333",
            "name": "valid",
            "properties": []
        }"#;
        source.insert(Path::new("valid.json"), valid_json.to_owned());

        // Invalid schema (empty name)
        let invalid_json = r#"{
            "id": "01933333-4444-7444-8444-444444444444",
            "name": "",
            "properties": []
        }"#;
        source.insert(Path::new("invalid.json"), invalid_json.to_owned());

        let ids = service.ingest_directory(&source, "*.json").expect(
            "Directory ingestion should succeed despite partial failures",
        );

        // Only the valid schema should have been ingested
        assert_eq!(
            ids.len(),
            1,
            "Should ingest exactly one schema (the valid one)"
        );

        let schemas = query.list().expect("Failed to list schemas");
        assert_eq!(
            schemas.len(),
            1,
            "Should have exactly one schema in database"
        );
        let stored_schema = schemas.first().expect("Schema should exist");
        assert_eq!(
            stored_schema.name().as_str(),
            "valid",
            "Persisted schema should be the valid one"
        );
    }

    #[test]
    fn schema_ingestion_service_needs_update_always_returns_true() {
        let (_dir, db) = test_db().expect("Failed to create test db");
        let query = RedbSchemaQuery::new_redb(&db);
        let command = RedbSchemaCommand::new_redb(&db);
        let service = SchemaIngestionService::new(&query, &command);

        let source = InMemoryFileSource::new();
        let schema_id = SchemaId::new();

        let needs_update = service
            .needs_update(&source, Path::new("test.json"), schema_id)
            .expect("needs_update should not fail");

        assert!(needs_update, "Stub implementation should always return true");
    }
}
