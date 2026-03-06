//! Schema storage adapters.

pub mod command;
pub mod ingestor;
pub mod query;

/// Type alias to remove path stuttering: `adapter::Command` vs
/// `adapter::command::Command`.
pub type Command<'db> = command::Command<'db>;

/// Type alias to remove path stuttering: `adapter::Query` vs
/// `adapter::query::Query`.
pub type Query<'db> = query::Query<'db>;
