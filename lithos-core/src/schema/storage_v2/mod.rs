//! Schema storage adapter implementation.

mod read;
mod write;

pub mod tables;

use std::sync::Arc;

use crate::db::Store;

/// Repository adapter for `redb`-backed schema storage.
///
/// This adapter implements the segregated repository traits using `redb`
/// as the underlying storage engine. It manages its own transaction boundaries
/// via the provided [`Store`].
#[derive(Debug)]
pub struct SchemaRedbRepository {
    pub(crate) store: Arc<Store>,
}

impl SchemaRedbRepository {
    /// Create a new repository adapter from a database store.
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

// Blanket implementation: any type that implements both Read and Write
// automatically implements the unified trait.
impl<T> crate::schema::repository::SchemaRepository for T where
    T: crate::schema::repository::SchemaReadRepository
        + crate::schema::repository::SchemaWriteRepository
{
}
