//! Indexing pipeline wiring and composition.
//!
//! This module acts as the composition root for the `trace-indexer` bounded
//! context. It is responsible for instantiating concrete adapters (like
//! `WalkdirAdapter` and `RedbRepository`), injecting them into the
//! `IndexerService` port boundary, and exposing a strictly-typed execution flow
//! (`run_index`) for CLI consumption.

use std::sync::Arc;

use trace_db::Store;
use trace_fs::DirPath;
pub use trace_indexer::{IndexOptions, IndexResult, IndexScope, ScanFilters};
use trace_indexer::{
    IndexerError, IndexerService, RedbRepository, scanner::WalkdirAdapter,
    storage::INDEX_DB_FILENAME,
};

use crate::error::AppError;

/// Command payload for the indexer pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCommand {
    scope: IndexScope,
    opts: IndexOptions,
}

impl IndexCommand {
    /// Create a new indexing command.
    #[inline]
    #[must_use]
    pub fn new(scope: IndexScope, opts: IndexOptions) -> Self {
        Self {
            scope,
            opts,
        }
    }

    /// Access the indexing scope.
    #[inline]
    #[must_use]
    pub fn scope(&self) -> &IndexScope {
        &self.scope
    }

    /// Access the indexing options.
    #[inline]
    #[must_use]
    pub fn opts(&self) -> IndexOptions {
        self.opts
    }
}

/// Run the indexing pipeline.
///
/// Constructs the `WalkdirAdapter`, `RedbRepository`, and `IndexerService`,
/// then delegates to `IndexerService::run()`. Opens store at
/// `cache_dir / INDEX_DB_FILENAME`.
///
/// # Errors
/// Returns `AppError::Indexer` if the database fails to open or if
/// an error occurs during the scan.
#[inline]
pub fn run_index(
    root: &DirPath,
    cache_dir: &DirPath,
    cmd: &IndexCommand,
) -> Result<IndexResult, AppError> {
    let db_path = cache_dir.as_path().join(INDEX_DB_FILENAME);
    let store = Store::open(&db_path)
        .map_err(|e| AppError::Indexer(IndexerError::Repository(e.into())))?;
    let repo = RedbRepository::try_new(Arc::new(store))
        .map_err(|e| AppError::Indexer(e.into()))?;
    let service = IndexerService::new(root.clone(), WalkdirAdapter, repo);
    Ok(service.run(cmd.scope(), cmd.opts())?)
}

// ----------------------------------------------------------- //
//                            Tests                            //
// ----------------------------------------------------------- //

#[cfg(test)]
mod tests {
    use trace_indexer::ScanFilters;

    use super::*;

    mod index_command {
        use super::*;

        mod constructor {
            use super::*;

            #[test]
            fn creates_command_with_scope_and_options() {
                let root =
                    DirPath::try_from(std::path::PathBuf::from("/")).unwrap();
                let scope = IndexScope::Full {
                    root,
                    filters: ScanFilters::default(),
                };
                let opts = IndexOptions::default();
                let cmd = IndexCommand::new(scope.clone(), opts);

                assert_eq!(cmd.scope(), &scope);
                assert_eq!(cmd.opts(), opts);
            }
        }
    }

    mod run_index {
        use super::*;

        #[test]
        fn returns_app_error_when_store_fails() {
            // Using a file path that doesn't exist or is invalid as a directory
            let root =
                DirPath::try_from(std::path::PathBuf::from("/")).unwrap();
            let tmp = tempfile::tempdir().unwrap();
            let file_path = tmp.path().join("file.txt");
            std::fs::write(&file_path, "test").unwrap();
            let scope = IndexScope::Full {
                root: root.clone(),
                filters: ScanFilters::default(),
            };
            let opts = IndexOptions::default();
            let cmd = IndexCommand::new(scope, opts);

            // Instead of using cache_dir, pass a file as cache_dir so it fails
            // to open DB inside it
            let bad_dir = DirPath::try_new(tmp.path().to_path_buf()).unwrap();
            let db_path = bad_dir.as_path().join("index.redb");
            // Write a directory to the db path so open fails
            std::fs::create_dir_all(&db_path).unwrap();
            let result = run_index(&root, &bad_dir, &cmd);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::Indexer(_)));
        }
    }
}
