//! [`WriteRepository`] trait implementation for [`RedbRepository`].
//!
//! The `RedbRepository` struct and its `WriteRepository` trait implementation
//! are deferred to the redb adapter slice. This module exists as scaffolding
//! to establish the module structure expected by the storage pattern.
//!
//! When the redb adapter is implemented, this module will contain the
//! write-side template persistence operations backed by `redb`.
//!
//! [`WriteRepository`]: crate::template::repository::WriteRepository
//! [`RedbRepository`]: crate::template::storage::RedbRepository
