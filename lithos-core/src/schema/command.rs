//! Schema command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the schema command port.

use super::{
    aggregate::{Schema, SchemaId},
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
        self.command_port
            .delete(id)
            .map_err(|error| SchemaCommandError::Storage(error.into()))
    }

    /// Save a schema to persistence.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    #[inline]
    pub fn save(&self, schema: &Schema) -> Result<(), SchemaCommandError> {
        self.command_port
            .save(schema)
            .map_err(|error| SchemaCommandError::Storage(error.into()))
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
        aggregate::{SchemaId, SchemaName, SchemaNameKey},
        db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
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

            cmd.save(&schema).expect("Save should succeed");

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

            let name_key = SchemaNameKey::from(schema.name());
            let indexed = db
                .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())
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
            cmd.save(&schema).expect("Save should succeed");

            cmd.delete(schema_id).expect("Delete should succeed");

            let id_key = schema_id.as_uuid().to_string();
            let stored = db
                .get_owned::<Schema>(SCHEMA_BY_ID, &id_key)
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted schema should not exist");

            let name = SchemaName::new("project")
                .expect("Failed to create schema name");
            let name_key = SchemaNameKey::from(&name);
            let indexed = db
                .get_owned::<SchemaId>(SCHEMA_ID_BY_NAME, name_key.as_str())
                .expect("Index lookup should succeed");
            assert!(indexed.is_none(), "Deleted schema should be unindexed");
        }
    }
}
