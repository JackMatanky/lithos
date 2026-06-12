//! Template repository persistence implementation.
//!
//! This module will provide the [`RedbRepository`] struct, which implements the
//! segregated repository traits ([`ReadRepository`], [`WriteRepository`], and
//! [`Repository`]) for template persistence using `redb` as the storage
//! engine. The redb adapter is deferred to a future slice.
//!
//! # Modules
//!
//! - [`tables`]: Public table definitions and constants for future redb adapter
//! - `read`: Internal [`ReadRepository`] implementation (deferred)
//! - `write`: Internal [`WriteRepository`] implementation (deferred)
//! - [`testing`]: Test utilities (available in `#[cfg(test)]`)
//!
//! [`ReadRepository`]: crate::template::repository::ReadRepository
//! [`WriteRepository`]: crate::template::repository::WriteRepository
//! [`Repository`]: crate::template::repository::Repository

mod read;
mod write;

pub mod tables;

#[cfg(any(test, feature = "testing"))]
pub(crate) mod testing;
