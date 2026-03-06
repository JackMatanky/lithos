//! Schema command implementations (CQRS write operations).
//!
//! This module provides the [`Command`] type, which handles write operations
//! through the schema command port.

use super::{
    aggregate::{Schema, SchemaId},
    bank::PropertyBank,
    error::SchemaCommandError,
    ports::{self as schema_ports},
};

/// Command implementation for schema write operations.
///
/// This struct is generic over a storage port to support multiple backends.
///
/// # Examples
///
/// ```ignore
/// use lithos_core::schema::{self, adapter};
///
/// let db = todo!("Provide a Database instance");
/// let command = schema::Command::new(adapter::Command::new(&db));
/// ```
pub struct Command<C> {
    command_port: C,
}

impl<C> Command<C> {
    /// Create a new `Command` with a storage port.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use lithos_core::schema::{self, adapter};
    ///
    /// let db = todo!("Provide a Database instance");
    /// let command = schema::Command::new(adapter::Command::new(&db));
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(command_port: C) -> Self {
        Self {
            command_port,
        }
    }

    /// Get a reference to the underlying command port.
    ///
    /// This allows access to adapter-specific methods not exposed via the
    /// port trait.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # let command = todo!("Provide a Command instance");
    /// let _port = command.port();
    /// ```
    #[inline]
    #[must_use]
    pub const fn port(&self) -> &C {
        &self.command_port
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # let command = todo!("Provide a Command instance");
    /// # let id = lithos_core::schema::aggregate::SchemaId::new();
    /// command.delete(id)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn delete(&self, id: SchemaId) -> Result<(), SchemaCommandError> {
        self.command_port.delete(id).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Save a single schema to persistence.
    ///
    /// Convenience method that delegates to `save_many`.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # let command = todo!("Provide a Command instance");
    /// # let schema = todo!("Provide a Schema instance");
    /// command.save(&schema)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn save(&self, schema: &Schema) -> Result<(), SchemaCommandError> {
        self.save_many(std::slice::from_ref(schema))
    }

    /// Save many schemas atomically.
    ///
    /// All saves are atomic within a single write transaction.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # let command = todo!("Provide a Command instance");
    /// # let schemas: Vec<lithos_core::schema::aggregate::Schema> = Vec::new();
    /// command.save_many(&schemas)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn save_many(
        &self,
        schemas: &[Schema],
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_many(schemas).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Save the `PropertyBank` to persistence.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # let command = todo!("Provide a Command instance");
    /// # let bank = lithos_core::schema::bank::PropertyBank::new();
    /// command.save_property_bank(&bank)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn save_property_bank(
        &self,
        bank: &PropertyBank,
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_property_bank(bank).map_err(|error| {
            SchemaCommandError::Storage(Into::<crate::db::DbError>::into(error))
        })
    }

    /// Save many inheritance relationships atomically.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if storage operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::command::Command;
    /// # use lithos_core::schema::ports::InheritanceRelationship;
    /// # let command = todo!("Provide a Command instance");
    /// # let child_id = lithos_core::schema::aggregate::SchemaId::new();
    /// # let parent_id = lithos_core::schema::aggregate::SchemaId::new();
    /// let relationships: Vec<InheritanceRelationship> =
    ///     vec![(child_id, Some(parent_id), vec!["prop".into()])];
    /// command.save_inheritance_many(&relationships)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn save_inheritance_many(
        &self,
        relationships: &[super::ports::InheritanceRelationship],
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_inheritance_many(relationships).map_err(
            |error| {
                SchemaCommandError::Storage(Into::<crate::db::DbError>::into(
                    error,
                ))
            },
        )
    }
}

impl Command<crate::schema::adapter::command::Command<'_>> {
    /// Save many schemas with filesystem timestamps.
    ///
    /// This method is only available when using the concrete redb adapter.
    /// It preserves filesystem metadata by calling the adapter's extended API.
    ///
    /// # Errors
    /// Returns `SchemaCommandError` if saving fails.
    ///
    /// # Panics
    /// Panics if `schemas.len() != metadata.len()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # use lithos_core::schema::{self, adapter};
    /// # let db = todo!("Provide database");
    /// # let command = schema::Command::new(adapter::Command::new(&db));
    /// # let schemas: Vec<lithos_core::schema::aggregate::Schema> = Vec::new();
    /// # let metadata: Vec<lithos_core::schema::stored::StoredMetadata> = Vec::new();
    /// command.save_many_with_metadata(&schemas, &metadata)?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    pub fn save_many_with_metadata(
        &self,
        schemas: &[Schema],
        metadata: &[crate::schema::stored::StoredMetadata],
    ) -> Result<(), SchemaCommandError> {
        self.command_port.save_many_with_metadata(schemas, metadata).map_err(
            |error| {
                SchemaCommandError::Storage(Into::<crate::db::DbError>::into(
                    error,
                ))
            },
        )
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
                SchemaName::try_new(name).map_err(|e| e.to_string())?;
            Schema::try_new(id, schema_name, None, vec![])
                .map_err(|e| e.to_string())
        }
    }

    use tempfile::{TempDir, tempdir};

    use crate::schema::{
        self as schema_mod, adapter,
        aggregate::{SchemaId, SchemaName},
        db_table::{SCHEMA_BY_ID, SCHEMA_ID_BY_NAME},
        stored::StoredSchema,
    };

    mod persistence {
        use super::*;

        #[test]
        fn save_persists_schema_by_id_and_name_index() {
            let (_dir, db) =
                fixtures::test_db().expect("Failed to create test db");
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));

            let schema =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");

            cmd.save(&schema).expect("Save should succeed");

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
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));

            let schema = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");
            let schema_id = schema.id();

            cmd.save(&schema).expect("Save should succeed");

            cmd.delete(schema_id).expect("Delete should succeed");

            let stored = db
                .get_owned_by_uuid::<StoredSchema>(
                    SCHEMA_BY_ID,
                    schema_id.into_uuid(),
                )
                .expect("Read after delete should succeed");
            assert!(stored.is_none(), "Deleted schema should not exist");

            let name = SchemaName::try_new("project")
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
            let cmd = schema_mod::Command::new(adapter::Command::new(&db));

            let schema_a =
                fixtures::schema_fixture(fixtures::TEST_SCHEMA_ID_NOTE, "note")
                    .expect("Failed to create schema fixture");
            let schema_b = fixtures::schema_fixture(
                fixtures::TEST_SCHEMA_ID_PROJECT,
                "project",
            )
            .expect("Failed to create schema fixture");

            cmd.save_many(&[schema_a.clone(), schema_b.clone()])
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
