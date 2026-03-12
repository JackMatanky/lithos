//! Config storage adapters.

pub mod command;
pub mod query;
pub(crate) mod stored;

/// Type alias to remove path stuttering: `adapter::Command` vs
/// `adapter::command::Command`.
pub type Command<'db> = command::Command<'db>;

/// Type alias to remove path stuttering: `adapter::Query` vs
/// `adapter::query::Query`.
pub type Query<'db> = query::Query<'db>;
