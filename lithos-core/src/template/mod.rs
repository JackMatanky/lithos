//! Template bounded context models.

#![allow(clippy::module_name_repetitions, reason = "Namespaced types")]
#![allow(clippy::pub_use, reason = "Re-exporting for convenience")]

/// MiniJinja adapter layer.
pub mod adapter;
/// Template aggregate root and main entities.
pub mod aggregate;
/// Template composition strategies.
pub mod block;
/// Template catalog for lifecycle orchestration.
pub mod catalog;
/// Template command implementations (CQRS write operations).
pub mod command;
/// Template errors.
pub mod error;
/// Template domain events.
pub mod events;
/// Template ports for CQRS.
pub mod ports;
/// Template query implementations (CQRS read operations).
pub mod query;
/// Template input specifications.
pub mod value;

pub(crate) mod db_table {
    use redb::{MultimapTableDefinition, TableDefinition};

    pub(crate) const TEMPLATES: TableDefinition<&str, &[u8]> =
        TableDefinition::new("templates");
    pub(crate) const NAME_TO_ID: MultimapTableDefinition<&str, &str> =
        MultimapTableDefinition::new("name_to_id");
}

pub use aggregate::{InputName, Metadata, Template, TemplateName};
pub use block::{BlockStrategy, TemplateBlock};
pub use catalog::TemplateCatalog;
pub use value::InputSpec;
