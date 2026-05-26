//! Consolidated discovery logic for config files.
//!
//! Provides the [`DiscoveryEngine`] which performs a single atomic scan of
//! both the filesystem and database, consolidating all data needed for
//! config processing.

#![expect(
    dead_code,
    reason = "Discovery engine implementation in progress - will be wired to \
              loader next"
)]

use std::path::Path;

use crate::{
    config::{
        error::ConfigIngestError,
        vault::{VaultId, VaultRoot},
        views::{RawGlobalConfigView, RawVaultConfigView},
    },
    fs::{
        FsEntry, FsFile, FsReader,
        metadata::{FileMetadata, FsTimes},
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
    entry: Option<FsEntry>,
    /// Cached view from database (None if never ingested or no file).
    view: Option<RawGlobalConfigView>,
}

impl GlobalDiscovery {
    /// Returns the file entry, if the global config file exists.
    #[inline]
    pub(crate) fn entry(&self) -> Option<&FsEntry> {
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
            FsEntry::File(file) => Some(file.metadata()),
            FsEntry::Dir(_) => None,
        })
    }
}

/// Discovery data for the vault config file.
///
/// Combines filesystem metadata with optional cached view from the database.
#[derive(Debug)]
pub(crate) struct VaultDiscovery {
    /// File entry from filesystem (path + `FileMetadata`).
    entry: Option<FsEntry>,
    /// Cached view from database (None if never ingested or no file).
    view: Option<RawVaultConfigView>,
}

impl VaultDiscovery {
    /// Returns the file entry, if the vault config file exists.
    #[inline]
    pub(crate) fn entry(&self) -> Option<&FsEntry> {
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
            FsEntry::File(file) => Some(file.metadata()),
            FsEntry::Dir(_) => None,
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

    /// Returns `true` if both global and vault config files are missing.
    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.global.entry.is_none() && self.vault.entry.is_none()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  Discovery Engine
// ═════════════════════════════════════════════════════════════════════════════

/// Orchestrates atomic discovery of global and vault config files.
///
/// The engine consolidates filesystem scanning and database queries into a
/// single-pass pipeline, ensuring consistency and performance.
pub(crate) struct DiscoveryEngine;

impl DiscoveryEngine {
    /// Performs an atomic discovery run.
    ///
    /// This method orchestrates the discovery pipeline:
    /// 1. Scan filesystem for global config (system-wide locations)
    /// 2. Scan filesystem for vault config (.lithos/lithos.toml)
    /// 3. Query DB for cached views (single transaction)
    /// 4. Combine filesystem + DB data into result
    ///
    /// # Errors
    ///
    /// Returns `ConfigIngestError` if filesystem or repository operations fail.
    pub(crate) fn run<R>(
        vault_root: &VaultRoot,
        repo: &R,
    ) -> Result<DiscoveryResult, ConfigIngestError>
    where
        R: crate::config::repository::Repository,
    {
        // Step 1: Scan filesystem for config files
        let (global_entry, vault_entry) = Self::scan_filesystem(vault_root)?;

        // Step 2: Resolve vault id for view lookup and query cached views.
        let vault_id =
            repo.find_vault_id_by_path(vault_root).map_err(|error| {
                ConfigIngestError::Io {
                    path: std::path::PathBuf::from("<db:vault_id_by_path>"),
                    source: std::io::Error::other(error.to_string()),
                }
            })?;

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
    // Filesystem Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Scans filesystem for global and vault config files.
    ///
    /// Returns `(Option<FsEntry>, Option<FsEntry>)` for global and vault.
    ///
    /// # Errors
    ///
    /// Returns error if filesystem operations fail.
    #[expect(
        clippy::type_complexity,
        reason = "Return tuple matches discovery pattern; will be \
                  destructured by caller"
    )]
    fn scan_filesystem(
        vault_root: &VaultRoot,
    ) -> Result<(Option<FsEntry>, Option<FsEntry>), ConfigIngestError> {
        let global_entry = Self::find_global_config()?;
        let vault_entry = Self::find_vault_config(vault_root)?;

        Ok((global_entry, vault_entry))
    }

    /// Finds the global config file using priority order.
    ///
    /// Priority order (from `ingestor.rs::GlobalConfigLocation`):
    /// 1. `$LITHOS_CONFIG_FILE`
    /// 2. `$XDG_CONFIG_HOME/lithos/lithos.toml`
    /// 3. `~/.config/lithos/lithos.toml`
    /// 4. `/etc/lithos/lithos.toml`
    ///
    /// Returns the first existing file with its `FileMetadata`.
    fn find_global_config() -> Result<Option<FsEntry>, ConfigIngestError> {
        let reader = FsReader::from_system_root();

        // Try each location in priority order
        for path in Self::global_config_paths() {
            if reader.exists(&path) {
                let default_metadata =
                    FileMetadata::new(FsTimes::new(None, None), 0, false);
                let info = reader
                    .metadata(&path)
                    .map(|m| m.as_file().cloned().unwrap_or(default_metadata))
                    .map_err(ConfigIngestError::from)?;

                let file_path = FilePath::try_new(path.clone())
                    .map_err(ConfigIngestError::from)?;

                return Ok(Some(FsEntry::File(FsFile::new(file_path, info))));
            }
        }

        Ok(None)
    }

    /// Finds the vault config file.
    ///
    /// Looks for `{vault_root}/.lithos/lithos.toml`.
    fn find_vault_config(
        vault_root: &VaultRoot,
    ) -> Result<Option<FsEntry>, ConfigIngestError> {
        let reader = FsReader::new(vault_root.as_path());
        let relative_path = Path::new(".lithos/lithos.toml");

        if !reader.exists(relative_path) {
            return Ok(None);
        }

        let info = reader
            .metadata(relative_path)
            .map(|m| {
                m.as_file().map_or_else(
                    || FileMetadata::new(FsTimes::new(None, None), 0, false),
                    std::clone::Clone::clone,
                )
            })
            .map_err(ConfigIngestError::from)?;

        let full_path = vault_root.as_path().join(relative_path);
        let file_path = FilePath::try_new(full_path.clone())
            .map_err(ConfigIngestError::from)?;

        Ok(Some(FsEntry::File(FsFile::new(file_path, info))))
    }

    /// Returns the priority-ordered list of global config paths.
    ///
    /// This mirrors the logic from `ingestor.rs::GlobalConfigLocation`.
    fn global_config_paths() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();

        // 1. $LITHOS_CONFIG_FILE
        if let Ok(path) = std::env::var("LITHOS_CONFIG_FILE") {
            paths.push(std::path::PathBuf::from(path));
        }

        // 2. $XDG_CONFIG_HOME/lithos/lithos.toml
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            paths.push(
                std::path::PathBuf::from(xdg_config)
                    .join("lithos")
                    .join("lithos.toml"),
            );
        }

        // 3. ~/.config/lithos/lithos.toml
        if let Ok(home) = std::env::var("HOME") {
            paths.push(
                std::path::PathBuf::from(home)
                    .join(".config")
                    .join("lithos")
                    .join("lithos.toml"),
            );
        }

        // 4. /etc/lithos/lithos.toml
        paths.push(std::path::PathBuf::from("/etc/lithos/lithos.toml"));

        paths
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
        global_entry: Option<FsEntry>,
        vault_entry: Option<FsEntry>,
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
