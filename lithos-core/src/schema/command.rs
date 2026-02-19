//! Schema command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the schema command port.

use super::{
    aggregate::SchemaId,
    bank::PropertyBank,
    error::SchemaCommandError,
    ports::{self as schema_ports, SchemaRecord},
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

    /// Save a single schema record to persistence.
    ///
    /// Convenience method that delegates to `save_batch`.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save_one(
        &self,
        record: SchemaRecord,
    ) -> Result<(), SchemaCommandError> {
        self.save_batch(&[record])
    }

    /// Save multiple schema records as a batch.
    ///
    /// All saves are atomic within a single write transaction.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save_batch(
        &self,
        records: &[SchemaRecord],
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_batch(records).map_err(|error| {
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
        use crate::{
            db::Database,
            schema::aggregate::{Schema, SchemaName},
        };

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
        adapter::{command::CommandAdapter, stored::StoredSchema},
        aggregate::{SchemaId, SchemaName, Timestamp},
        bank::BankVersion,
        db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
    };

    mod persistence {
        use super::*;

        #[test]
        fn save_persists_schema_by_id_and_name_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");

            cmd.save_one(SchemaRecord::new(
                schema.clone(),
                None,
                BankVersion::initial(),
                Timestamp::now(),
                Timestamp::now(),
            ))
            .expect("Save should succeed");

            let stored = db
                .get_owned_by_uuid::<StoredSchema>(
                    SCHEMA_BY_ID,
                    schema.id().into_uuid(),
                )
                .expect("Read after save should succeed");
            let stored_schema = stored.expect("Stored schema should exist");
            assert_eq!(
                stored_schema.name.as_ref(),
                "note",
                "Stored schema name should match"
            );

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
        fn delete_removes_schema_by_id() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));

            let schema = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");
            let schema_id = schema.id();

            cmd.save_one(SchemaRecord::new(
                schema,
                None,
                BankVersion::initial(),
                Timestamp::now(),
                Timestamp::now(),
            ))
            .expect("Save should succeed");

            cmd.delete(schema_id).expect("Delete should succeed");

            let stored = db
                .get_owned_by_uuid::<StoredSchema>(
                    SCHEMA_BY_ID,
                    schema_id.into_uuid(),
                )
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted schema should not exist");

            let name = SchemaName::new("project")
                .expect("Failed to create schema name");
            let indexed = db
                .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name.as_str())
                .expect("Index lookup should succeed");
            assert!(indexed.is_none(), "Deleted schema should be unindexed");
        }

        #[test]
        fn save_batch_persists_schemas() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = RedbSchemaCommand::new(CommandAdapter::new(&db));

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");
            let schema_b = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");

            cmd.save_batch(&[
                SchemaRecord::new(
                    schema_a.clone(),
                    None,
                    BankVersion::initial(),
                    Timestamp::now(),
                    Timestamp::now(),
                ),
                SchemaRecord::new(
                    schema_b.clone(),
                    None,
                    BankVersion::initial(),
                    Timestamp::now(),
                    Timestamp::now(),
                ),
            ])
            .expect("Batch save should succeed");

            let stored_a = db
                .get_owned_by_uuid::<StoredSchema>(
                    SCHEMA_BY_ID,
                    schema_a.id().into_uuid(),
                )
                .expect("Read after batch save should succeed");
            assert!(stored_a.is_some(), "Schema A should be stored");

            let stored_b = db
                .get_owned_by_uuid::<StoredSchema>(
                    SCHEMA_BY_ID,
                    schema_b.id().into_uuid(),
                )
                .expect("Read after batch save should succeed");
            assert!(stored_b.is_some(), "Schema B should be stored");
        }
    }
}
