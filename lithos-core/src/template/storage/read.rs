//! [`ReadRepository`] trait implementation for [`RedbRepository`].
//!
//! The `RedbRepository` struct and its `ReadRepository` trait implementation
//! are deferred to the redb adapter slice. This module exists as scaffolding
//! to establish the module structure expected by the storage pattern.
//!
//! When the redb adapter is implemented, this module will contain the
//! read-side template persistence operations backed by `redb`.
//!
//! [`ReadRepository`]: crate::template::repository::ReadRepository
//! [`RedbRepository`]: crate::template::storage::RedbRepository
