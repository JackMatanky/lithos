//! Schema storage adapter implementation.

mod read;
mod write;

pub mod tables;

#[cfg(test)]
pub mod testing;

use std::sync::Arc;

use crate::{db::Store, fs::RelativePath};

/// Repository adapter for `redb`-backed schema storage.
///
/// This adapter implements the segregated repository traits using `redb`
/// as the underlying storage engine. It manages its own transaction boundaries
/// via the provided [`Store`].
#[derive(Debug)]
pub struct RedbRepository {
    pub(crate) store: Arc<Store>,
}

impl RedbRepository {
    /// Create a new repository adapter from a database store.
    #[inline]
    #[must_use]
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
        }
    }
}

#[inline]
pub(super) fn path_key(path: &RelativePath) -> String {
    path.as_path().to_string_lossy().into_owned()
}

// Blanket implementation: any type that implements both Read and Write
// automatically implements the unified trait.
impl<T> crate::schema::repository::Repository for T where
    T: crate::schema::repository::ReadRepository
        + crate::schema::repository::WriteRepository
{
}

#[cfg(test)]
mod tests {
    use super::path_key;
    use crate::fs::RelativePath;

    #[test]
    fn path_key_matches_relative_path_display() {
        let path = RelativePath::try_from("schemas/note.json").unwrap();
        assert_eq!(path_key(&path), "schemas/note.json");
    }
}
