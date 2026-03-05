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
/// Template command implementations (Generic CQRS wrapper).
pub mod command;
/// Template errors.
pub mod error;
/// Template domain events.
pub mod events;
/// Template ports for CQRS.
pub mod ports;
/// Template query implementations (Generic CQRS wrapper).
pub mod query;
/// Raw template input definitions.
pub mod raw;
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
pub use command::Command;
pub use query::Query;
pub use value::InputSpec;

use self::adapter::{CommandAdapter, QueryAdapter};

/// Template query type alias (storage-agnostic).
pub type TemplateQuery<'db> = Query<QueryAdapter<'db>>;
/// Template command type alias (storage-agnostic).
pub type TemplateCommand<'db> = Command<CommandAdapter<'db>>;
