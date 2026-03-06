//! Schema storage adapters.

pub mod command;
pub mod ingestor;

/// Type alias to remove path stuttering: `adapter::Command` vs
/// `adapter::command::Command`.
pub type Command<'db> = command::Command<'db>;
