//! Consolidated discovery logic for config files.
//!
//! Provides the [`ConfigDiscoveryPipeline`] which performs a single atomic
//! scan of both the filesystem and database, consolidating all data needed
//! for config processing.
//!
//! Config-owned location and candidate classification live in sibling modules
//! under [`crate::config`].

use crate::{
    config::{
        error::ConfigIngestError,
        root::{ConfigDiscoveryResult, DiscoveredConfigFile},
        vault::VaultId,
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    fs::{
        FileNode, FsNode,
        metadata::{FileMetadata, FsMetadata},
        path::FilePath,
    },
};

// ═════════════════════════════════════════════════════════════════════════════
//  Discovery Result Types
// ═════════════════════════════════════════════════════════════════════════════

/// Discovery data for the global config file.
///
/// Combines filesystem metadata with optional cached view from the database.
#[derive(Debug)]
pub(crate) struct GlobalDiscovery {
    /// File entry from filesystem (path + `FileMetadata`).
    entry: Option<FsNode>,
    /// Cached view from database (None if never ingested or no file).
    view: Option<RawGlobalConfigView>,
}

impl GlobalDiscovery {
    /// Returns the file entry, if the global config file exists.
    #[inline]
    pub(crate) fn entry(&self) -> Option<&FsNode> {
        self.entry.as_ref()
    }

    /// Returns the cached view, if any.
    #[inline]
    pub(crate) fn view(&self) -> Option<&RawGlobalConfigView> {
        self.view.as_ref()
    }

    /// Returns the `FileMetadata` from the entry, if present.
    #[inline]
    pub(crate) fn info(&self) -> Option<&FileMetadata> {
        self.entry.as_ref().and_then(|entry| match entry {
            FsNode::File(file) => Some(file.metadata()),
            FsNode::Dir(_) => None,
        })
    }
}

/// Discovery data for the vault config file.
///
/// Combines filesystem metadata with optional cached view from the database.
#[derive(Debug)]
pub(crate) struct VaultDiscovery {
    /// File entry from filesystem (path + `FileMetadata`).
    entry: Option<FsNode>,
    /// Cached view from database (None if never ingested or no file).
    view: Option<RawVaultConfigView>,
}

impl VaultDiscovery {
    /// Returns the file entry, if the vault config file exists.
    #[inline]
    pub(crate) fn entry(&self) -> Option<&FsNode> {
        self.entry.as_ref()
    }

    /// Returns the cached view, if any.
    #[inline]
    pub(crate) fn view(&self) -> Option<&RawVaultConfigView> {
        self.view.as_ref()
    }

    /// Returns the `FileMetadata` from the entry, if present.
    #[inline]
    pub(crate) fn info(&self) -> Option<&FileMetadata> {
        self.entry.as_ref().and_then(|entry| match entry {
            FsNode::File(file) => Some(file.metadata()),
            FsNode::Dir(_) => None,
        })
    }
}

/// Result of atomic discovery combining filesystem scan and database state.
#[derive(Debug)]
pub(crate) struct DiscoveryResult {
    /// Discovered global config file (if exists).
    global: GlobalDiscovery,
    /// Discovered vault config file (if exists).
    vault: VaultDiscovery,
}

impl DiscoveryResult {
    /// Returns the global config discovery data.
    #[inline]
    pub(crate) fn global(&self) -> &GlobalDiscovery {
        &self.global
    }

    /// Returns the vault config discovery data.
    #[inline]
    pub(crate) fn vault(&self) -> &VaultDiscovery {
        &self.vault
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  Config Discovery Pipeline
// ═════════════════════════════════════════════════════════════════════════════

/// Reads config files from pre-discovered paths and queries database for
/// cached views, combining both into a single [`DiscoveryResult`].
///
/// The pipeline does not perform filesystem discovery itself — it receives
/// the discovered file paths via [`ConfigDiscoveryResult`] and reads
/// metadata + content from those paths.
pub(crate) struct ConfigDiscoveryPipeline;

impl ConfigDiscoveryPipeline {
    /// Performs the config discovery pipeline.
    ///
    /// This method:
    /// 1. Reads files from the paths in `discovery_result`
    /// 2. Maps them to `FsNode` with filesystem metadata
    /// 3. Queries DB for cached views
    /// 4. Combines into [`DiscoveryResult`]
    ///
    /// # Errors
    ///
    /// Returns `ConfigIngestError` if filesystem or repository operations fail.
    /// # Parameters
    ///
    /// - `vault_id`: The vault ID for DB view lookup. Pass `None` when the
    ///   vault is not yet persisted (e.g., first-time initialization); in that
    ///   case only the global view is fetched and the vault view is `None`.
    pub(crate) fn run<R>(
        discovery_result: &ConfigDiscoveryResult,
        vault_id: Option<VaultId>,
        repo: &R,
    ) -> Result<DiscoveryResult, ConfigIngestError>
    where
        R: crate::config::repository::Repository,
    {
        // Step 1: Map discovered config files to FsNode
        let global_entry = match &discovery_result.global {
            Some(file) => Some(Self::entry_from_discovered_file(file)?),
            None => None,
        };
        let vault_entry = match &discovery_result.local {
            Some(file) => Some(Self::entry_from_discovered_file(file)?),
            None => None,
        };

        // Step 2: Query cached views. When vault_id is None (vault not yet
        // persisted), only the global view is available.
        let (global_view, vault_view) = match vault_id {
            Some(vault_id) => Self::query_cached_views(repo, vault_id)?,
            None => (
                repo.get_raw_global_view().map_err(|error| {
                    ConfigIngestError::Io {
                        path: std::path::PathBuf::from("<db:raw_global_view>"),
                        source: std::io::Error::other(error.to_string()),
                    }
                })?,
                None,
            ),
        };

        // Step 3: Combine into discovery result
        Ok(Self::build_result(
            global_entry,
            vault_entry,
            global_view,
            vault_view,
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // File Mapping
    // ─────────────────────────────────────────────────────────────────────────

    /// Maps a [`DiscoveredConfigFile`] to an [`FsNode`] by reading its
    /// filesystem metadata.
    ///
    /// # Errors
    ///
    /// Returns `ConfigIngestError` if the path does not exist, is not a file,
    /// or metadata cannot be read.
    fn entry_from_discovered_file(
        file: &DiscoveredConfigFile,
    ) -> Result<FsNode, ConfigIngestError> {
        let metadata = FsMetadata::from_path(&file.path).map_err(|error| {
            ConfigIngestError::Io {
                path: file.path.clone(),
                source: std::io::Error::other(error.to_string()),
            }
        })?;

        let info = metadata.as_file().cloned().ok_or_else(|| {
            ConfigIngestError::Io {
                path: file.path.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "discovered config path is not a file",
                ),
            }
        })?;

        let file_path = FilePath::try_new(file.path.clone())
            .map_err(ConfigIngestError::from)?;

        Ok(FsNode::File(FileNode::new(file_path, info)))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Database Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Queries cached views from DB in single transaction.
    ///
    /// # Errors
    ///
    /// Returns error if database queries fail.
    #[expect(
        clippy::type_complexity,
        reason = "Return type matches discovery pattern; will be used by \
                  loader"
    )]
    fn query_cached_views<R>(
        repo: &R,
        vault_id: VaultId,
    ) -> Result<
        (Option<RawGlobalConfigView>, Option<RawVaultConfigView>),
        ConfigIngestError,
    >
    where
        R: crate::config::repository::Repository,
    {
        let global_view = repo.get_raw_global_view().map_err(|error| {
            ConfigIngestError::Io {
                path: std::path::PathBuf::from("<db:raw_global_view>"),
                source: std::io::Error::other(error.to_string()),
            }
        })?;

        let vault_view =
            repo.get_raw_vault_view(vault_id).map_err(|error| {
                ConfigIngestError::Io {
                    path: std::path::PathBuf::from("<db:raw_vault_view>"),
                    source: std::io::Error::other(error.to_string()),
                }
            })?;

        Ok((global_view, vault_view))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Result Construction
    // ─────────────────────────────────────────────────────────────────────────

    /// Builds final `DiscoveryResult` from filesystem + DB data.
    fn build_result(
        global_entry: Option<FsNode>,
        vault_entry: Option<FsNode>,
        global_view: Option<RawGlobalConfigView>,
        vault_view: Option<RawVaultConfigView>,
    ) -> DiscoveryResult {
        DiscoveryResult {
            global: GlobalDiscovery {
                entry: global_entry,
                view: global_view,
            },
            vault: VaultDiscovery {
                entry: vault_entry,
                view: vault_view,
            },
        }
    }
}
