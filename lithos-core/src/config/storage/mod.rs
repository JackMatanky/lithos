//! Configuration storage persistence implementation.

mod read;
pub(crate) mod tables;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod write;

use std::sync::Arc;

use crate::db::Store;

/// Repository implementation for `redb`-backed configuration storage.
#[derive(Debug, Clone)]
pub struct RedbRepository {
    pub(crate) store: Arc<Store>,
}

/// Legacy alias for [`RedbRepository`].
pub type RedbStorage = RedbRepository;

impl RedbRepository {
    /// Creates a new repository adapter from a database store.
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}
