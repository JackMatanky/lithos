//! Schema command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the schema command port.

use super::{
    aggregate::{ResolutionMetadata, Schema, SchemaId},
    bank::PropertyBank,
    error::SchemaCommandError,
    ports as schema_ports,
};

/// Command implementation for schema write operations.
///
/// This struct is generic over a storage port to support multiple backends.
pub struct Command<C> {
    command_port: C,
}

impl<C> Command<C> {
    /// Create a new `Command` with a storage port.
    #[inline]
    #[must_use]
    pub const fn new(command_port: C) -> Self {
        Self {
            command_port,
        }
    }
}

impl<C> Command<C>
where
    C: schema_ports::Command,
    C::Error: Into<crate::db::DbError>,
{
    /// Delete a schema by ID.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if deletion fails.
    #[inline]
    pub fn delete(&self, id: SchemaId) -> Result<(), SchemaCommandError> {
        self.command_port.delete(id).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Save a schema to persistence.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save_with_metadata(
        &self,
        schema: &Schema,
        metadata: &ResolutionMetadata,
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_with_metadata(schema, metadata).map_err(
            |error| {
                SchemaCommandError::Storage(Into::<crate::db::DbError>::into(
                    error,
                ))
            },
        )
    }

    /// Save multiple schemas and metadata entries as a batch.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save_batch(
        &self,
        schemas: &[(Schema, ResolutionMetadata)],
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_batch(schemas).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Save the `PropertyBank` to persistence.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_property_bank(bank).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Test module groups fixtures and submodules for readability."
)]
mod tests {
    mod fixtures {
        use uuid::Uuid;

        use super::*;
        use crate::db::Database;

        pub const TEST_SCHEMA_ID_NOTE: SchemaId = SchemaId::from_uuid(
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0101),
        );
        pub const TEST_SCHEMA_ID_PROJECT: SchemaId = SchemaId::from_uuid(
            Uuid::from_u128(0x018C_0000_0000_7000_8000_0000_0000_0102),
        );

        pub fn test_db() -> Result<(TempDir, Database), String> {
            let dir = tempdir().map_err(|e| e.to_string())?;
            let path = dir.path().join("schema.redb");
            let db = Database::open(&path).map_err(|e| e.to_string())?;
            Ok((dir, db))
        }

        pub fn schema_fixture(
            id: SchemaId,
            name: &str,
        ) -> Result<Schema, String> {
            let schema_name =
                SchemaName::new(name).map_err(|e| e.to_string())?;
            Schema::new(id, schema_name, vec![]).map_err(|e| e.to_string())
        }
    }

    use tempfile::{TempDir, tempdir};

    use super::*;
    use crate::schema::{
        RedbSchemaCommand,
        aggregate::{SchemaId, SchemaName, Timestamp},
        bank::BankVersion,
        db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME, SCHEMA_METADATA},
    };

    mod persistence {
        use super::*;

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn save_persists_schema_by_id_and_name_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new_redb(&db);

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");

            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

            let id_key = schema.id().as_uuid().to_string();
            let stored = db
                .get_owned::<Schema>(SCHEMA_BY_ID, &id_key)
                .expect("Read after save should succeed");
            let stored_schema = stored.expect("Stored schema should exist");
            assert_eq!(
                stored_schema.name().as_ref(),
                "note",
                "Stored schema name should match"
            );

            let metadata_key = schema.id().as_uuid().to_string();
            let stored_metadata = db
                .get_owned::<ResolutionMetadata>(
                    SCHEMA_METADATA,
                    metadata_key.as_str(),
                )
                .expect("Metadata read should succeed");
            assert!(stored_metadata.is_some(), "Metadata should be stored");

            let indexed = db
                .get_owned::<SchemaId>(
                    SCHEMA_ID_BY_NAME,
                    schema.name().as_str(),
                )
                .expect("Index lookup should succeed");
            assert_eq!(
                indexed,
                Some(schema.id()),
                "Name index should map to schema id"
            );
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn delete_removes_schema_by_id() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new_redb(&db);

            let schema = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");
            let schema_id = schema.id();
            let metadata = ResolutionMetadata::new(
                schema.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            cmd.save_with_metadata(&schema, &metadata)
                .expect("Save should succeed");

            cmd.delete(schema_id).expect("Delete should succeed");

            let id_key = schema_id.as_uuid().to_string();
            let stored = db
                .get_owned::<Schema>(SCHEMA_BY_ID, &id_key)
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted schema should not exist");

            let stored_metadata = db
                .get_owned::<ResolutionMetadata>(
                    SCHEMA_METADATA,
                    id_key.as_str(),
                )
                .expect("Read metadata after delete should succeed");
            assert!(
                stored_metadata.is_none(),
                "Deleted schema metadata removed"
            );

            let name = SchemaName::new("project")
                .expect("Failed to create schema name");
            let indexed = db
                .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())
                .expect("Index lookup should succeed");
            assert!(indexed.is_none(), "Deleted schema should be unindexed");
        }

        #[test]
        #[expect(
            clippy::disallowed_methods,
            reason = "Test uses expect for deterministic fixture setup and \
                      value extraction."
        )]
        fn save_batch_persists_schemas_and_metadata() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new_redb(&db);

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");
            let schema_b = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");

            let metadata_a = ResolutionMetadata::new(
                schema_a.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );
            let metadata_b = ResolutionMetadata::new(
                schema_b.id(),
                Timestamp::now(),
                None,
                BankVersion::initial(),
                None,
            );

            cmd.save_batch(&[
                (schema_a.clone(), metadata_a.clone()),
                (schema_b.clone(), metadata_b.clone()),
            ])
            .expect("Batch save should succeed");

            let schema_key = schema_a.id().as_uuid().to_string();
            let stored_schema = db
                .get_owned::<Schema>(SCHEMA_BY_ID, &schema_key)
                .expect("Read after batch save should succeed");
            assert!(stored_schema.is_some(), "Schema should be stored");

            let metadata_key = schema_b.id().as_uuid().to_string();
            let stored_metadata = db
                .get_owned::<ResolutionMetadata>(
                    SCHEMA_METADATA,
                    metadata_key.as_str(),
                )
                .expect("Metadata read should succeed");
            assert!(stored_metadata.is_some(), "Metadata should be stored");
        }
    }
}
