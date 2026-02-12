//! Template bounded context models.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]

/// Template aggregate root and main entities.
pub mod aggregate;
/// Template command implementations (CQRS write operations).
pub mod command;
/// Template composition logic.
pub mod composition;
/// Template errors.
pub mod error;
/// Template domain events.
pub mod events;
/// Template ports for CQRS.
pub mod ports;
/// Template query implementations (CQRS read operations).
pub mod query;
/// Template placeholder syntax.
pub mod syntax;
/// Template validation logic.
pub mod validation;
/// Template variable definitions.
pub mod variable;

pub(crate) mod db_table {
    use redb::{MultimapTableDefinition, TableDefinition};

    pub(crate) const TEMPLATES: TableDefinition<&str, &[u8]> =
        TableDefinition::new("templates");
    pub(crate) const NAME_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("name_to_id");
}

// --- Public API ---
