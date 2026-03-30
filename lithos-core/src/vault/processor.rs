//! Vault processing pipeline and routing entry point.

#![expect(
    clippy::module_name_repetitions,
    reason = "Vault processor naming is explicit and scoped"
)]

use std::{collections::HashSet, marker::PhantomData, path::Path};

use super::error::{VaultFileError, VaultProcessError};
use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{
        error::NoteProcessError,
        processor::{NoteFileInfo, NoteProcessAction, NoteProcessor},
        storage::RedbRepository as NoteRepository,
    },
    vault::{
        model::{VaultFile, VaultFolder, VaultPath},
        storage::{RedbRepository as VaultRepository, Repository as _},
    },
};

/// Core state machine tracking the current pipeline stage and knowledge status.
#[derive(Debug)]
#[must_use]
pub struct VaultProcessor<P, S> {
    status: S,
    _stage: PhantomData<P>,
}

#[inline]
#[must_use]
fn is_markdown_path(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
    })
}

impl<P, S> VaultProcessor<P, S> {
    #[inline]
    fn transition<NP, NS>(_stage: NP, status: NS) -> VaultProcessor<NP, NS> {
        VaultProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

/// Entry phase: scanning the filesystem.
#[derive(Debug)]
#[non_exhaustive]
pub struct Discovery;

/// Comparison phase: update vault repository entries.
#[derive(Debug)]
#[non_exhaustive]
pub struct Comparison;

/// Routing phase: route markdown files to the note processor.
#[derive(Debug)]
#[non_exhaustive]
pub struct Routing;

/// Prune phase: remove missing entries from the vault repository.
#[derive(Debug)]
#[non_exhaustive]
pub struct Prune;

/// Terminal phase: processing complete.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed;

/// Initial status before any discovery.
#[derive(Debug)]
#[non_exhaustive]
pub struct Unknown;

/// Status after discovery with scan results.
#[derive(Debug)]
pub struct Scanned {
    mode: ScanMode,
    files: Vec<VaultFile>,
    folders: Vec<VaultFolder>,
    path_set: HashSet<VaultPath>,
}

/// Status after comparison with routing candidates.
#[derive(Debug)]
pub struct Compared {
    mode: ScanMode,
    path_set: HashSet<VaultPath>,
    markdown_candidates: Vec<VaultFile>,
    report: VaultProcessReport,
}

/// Status after routing markdown files.
#[derive(Debug)]
pub struct Routed {
    mode: ScanMode,
    path_set: HashSet<VaultPath>,
    report: VaultProcessReport,
}

/// Terminal status carrying the final report.
#[derive(Debug)]
pub struct Ready {
    report: VaultProcessReport,
}

/// Scan mode indicating whether pruning should run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScanMode {
    /// Full scan of the vault root.
    Full,
    /// Partial scan of provided paths only.
    Partial,
}

/// Structured processing report for vault scans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct VaultProcessReport {
    files_scanned: usize,
    files_added: usize,
    files_updated: usize,
    files_fresh: usize,
    files_deleted: usize,
    folders_scanned: usize,
    folders_added: usize,
    folders_updated: usize,
    folders_deleted: usize,
    markdown_routed: usize,
    notes_created_or_updated: usize,
    notes_deleted: usize,
    errors: Vec<VaultProcessErrorSummary>,
}

impl VaultProcessReport {
    /// Returns the total number of scanned files.
    #[inline]
    #[must_use]
    pub const fn files_scanned(&self) -> usize {
        self.files_scanned
    }

    /// Returns the number of added files.
    #[inline]
    #[must_use]
    pub const fn files_added(&self) -> usize {
        self.files_added
    }

    /// Returns the number of updated files.
    #[inline]
    #[must_use]
    pub const fn files_updated(&self) -> usize {
        self.files_updated
    }

    /// Returns the number of fresh files.
    #[inline]
    #[must_use]
    pub const fn files_fresh(&self) -> usize {
        self.files_fresh
    }

    /// Returns the number of deleted files.
    #[inline]
    #[must_use]
    pub const fn files_deleted(&self) -> usize {
        self.files_deleted
    }

    /// Returns the total number of scanned folders.
    #[inline]
    #[must_use]
    pub const fn folders_scanned(&self) -> usize {
        self.folders_scanned
    }

    /// Returns the number of added folders.
    #[inline]
    #[must_use]
    pub const fn folders_added(&self) -> usize {
        self.folders_added
    }

    /// Returns the number of updated folders.
    #[inline]
    #[must_use]
    pub const fn folders_updated(&self) -> usize {
        self.folders_updated
    }

    /// Returns the number of deleted folders.
    #[inline]
    #[must_use]
    pub const fn folders_deleted(&self) -> usize {
        self.folders_deleted
    }

    /// Returns the count of markdown files routed to the note processor.
    #[inline]
    #[must_use]
    pub const fn markdown_routed(&self) -> usize {
        self.markdown_routed
    }

    /// Returns the count of created or updated notes.
    #[inline]
    #[must_use]
    pub const fn notes_created_or_updated(&self) -> usize {
        self.notes_created_or_updated
    }

    /// Returns the count of deleted notes.
    #[inline]
    #[must_use]
    pub const fn notes_deleted(&self) -> usize {
        self.notes_deleted
    }

    /// Returns any errors captured during processing.
    #[inline]
    #[must_use]
    pub fn errors(&self) -> &[VaultProcessErrorSummary] {
        &self.errors
    }
}

/// Lightweight error summary for reports.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct VaultProcessErrorSummary {
    path: Option<Box<str>>,
    message: Box<str>,
}

impl VaultProcessErrorSummary {
    /// Creates a new error summary.
    #[inline]
    #[must_use]
    pub fn new(path: Option<Box<str>>, message: Box<str>) -> Self {
        Self {
            path,
            message,
        }
    }

    /// Returns the optional path associated with the error.
    #[inline]
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the error message.
    #[inline]
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl VaultProcessor<Discovery, Unknown> {
    /// Creates a new vault processor in the discovery stage.
    #[inline]
    pub const fn new() -> Self {
        VaultProcessor {
            status: Unknown,
            _stage: PhantomData,
        }
    }

    /// Runs a full vault scan and processing pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`VaultProcessError`] if discovery, storage, or note processing
    /// fails.
    #[inline]
    #[tracing::instrument(level = "info", skip(self, db, config))]
    pub fn process_full(
        self,
        db: &crate::db::Database,
        config: &Config,
    ) -> Result<VaultProcessReport, VaultProcessError> {
        let source = FsReader::new(config.vault_metadata().root().as_path());
        let repository = VaultRepository::new(db);
        let note_repository = NoteRepository::new(db);

        let compared = self
            .discover(&source, ScanMode::Full)?
            .compare(&repository)?
            .route(&note_repository, config, &source)?;
        let completed = compared.prune(&repository, &note_repository)?;
        Ok(completed.report())
    }

    /// Runs a partial vault scan for the provided paths.
    ///
    /// # Errors
    ///
    /// Returns [`VaultProcessError`] if discovery, storage, or note processing
    /// fails.
    #[inline]
    #[tracing::instrument(level = "info", skip(self, db, config, paths))]
    pub fn process_partial(
        self,
        db: &crate::db::Database,
        config: &Config,
        paths: &[VaultPath],
    ) -> Result<VaultProcessReport, VaultProcessError> {
        let source = FsReader::new(config.vault_metadata().root().as_path());
        let repository = VaultRepository::new(db);
        let note_repository = NoteRepository::new(db);

        let compared = self
            .discover_partial(&source, paths)?
            .compare(&repository)?
            .route(&note_repository, config, &source)?;
        let completed = compared.complete_partial();
        Ok(completed.report())
    }

    #[inline]
    fn discover(
        self,
        source: &FsReader,
        mode: ScanMode,
    ) -> Result<VaultProcessor<Comparison, Scanned>, VaultProcessError> {
        drop(self);
        let files = Self::scan_files(source)?;
        let folders = Self::collect_folders(source, &files)?;
        let mut path_set =
            HashSet::with_capacity(files.len().saturating_add(folders.len()));
        for file in &files {
            path_set.insert(file.path().clone());
        }
        for folder in &folders {
            path_set.insert(folder.path().clone());
        }

        Ok(Self::transition(Comparison, Scanned {
            mode,
            files,
            folders,
            path_set,
        }))
    }

    #[inline]
    fn discover_partial(
        self,
        source: &FsReader,
        paths: &[VaultPath],
    ) -> Result<VaultProcessor<Comparison, Scanned>, VaultProcessError> {
        drop(self);
        let mut files = Vec::with_capacity(paths.len());
        let mut folders = Vec::new();
        for path in paths {
            source.validate_path(path.as_path()).map_err(|error| {
                VaultFileError::InvalidPath {
                    path: path.as_str().into(),
                    reason: error.to_string().into(),
                }
            })?;
            let metadata =
                source.metadata(path.as_path()).map_err(|error| {
                    VaultFileError::MetadataFailed {
                        path: path.as_str().into(),
                        message: error.to_string().into(),
                    }
                })?;
            if metadata.is_dir() {
                let folder = VaultFolder::try_new(path.clone(), &metadata)
                    .map_err(|error| VaultFileError::InvalidPath {
                        path: path.as_str().into(),
                        reason: error.to_string().into(),
                    })?;
                folders.push(folder);
            } else {
                let file = VaultFile::try_new(path.clone(), &metadata)
                    .map_err(|error| VaultFileError::InvalidPath {
                        path: path.as_str().into(),
                        reason: error.to_string().into(),
                    })?;
                files.push(file);
            }
        }

        let mut path_set =
            HashSet::with_capacity(files.len().saturating_add(folders.len()));
        for file in &files {
            path_set.insert(file.path().clone());
        }
        for folder in &folders {
            path_set.insert(folder.path().clone());
        }

        Ok(Self::transition(Comparison, Scanned {
            mode: ScanMode::Partial,
            files,
            folders,
            path_set,
        }))
    }

    fn scan_files(source: &FsReader) -> Result<Vec<VaultFile>, VaultFileError> {
        let pattern = "**/*";
        let paths = source.list_files(pattern).map_err(|error| {
            VaultFileError::ReadFailed {
                path: "<vault>".into(),
                message: error.to_string().into(),
            }
        })?;

        let mut files = Vec::with_capacity(paths.len());
        for relative in paths {
            source.validate_path(relative.as_path()).map_err(|error| {
                VaultFileError::InvalidPath {
                    path: relative.to_str().unwrap_or("<invalid>").into(),
                    reason: error.to_string().into(),
                }
            })?;

            let vault_path =
                VaultPath::try_from_path(&relative).map_err(|error| {
                    VaultFileError::InvalidPath {
                        path: relative.to_str().unwrap_or("<invalid>").into(),
                        reason: error.to_string().into(),
                    }
                })?;

            let metadata = source
                .metadata(Path::new(vault_path.as_str()))
                .map_err(|error| VaultFileError::MetadataFailed {
                    path: vault_path.as_str().into(),
                    message: error.to_string().into(),
                })?;
            let file =
                VaultFile::try_new(vault_path, &metadata).map_err(|error| {
                    VaultFileError::InvalidPath {
                        path: relative.to_str().unwrap_or("<invalid>").into(),
                        reason: error.to_string().into(),
                    }
                })?;
            files.push(file);
        }
        Ok(files)
    }

    fn collect_folders(
        source: &FsReader,
        files: &[VaultFile],
    ) -> Result<Vec<VaultFolder>, VaultFileError> {
        let mut seen = HashSet::new();
        let mut folders = Vec::new();
        for file in files {
            let mut current = file.path().as_path();
            while let Some(parent) = current.parent() {
                if parent.as_os_str().is_empty() {
                    break;
                }
                let parent_path =
                    VaultPath::try_from_path(parent).map_err(|error| {
                        VaultFileError::InvalidPath {
                            path: parent.to_string_lossy().into_owned().into(),
                            reason: error.to_string().into(),
                        }
                    })?;
                if !seen.insert(parent_path.clone()) {
                    current = parent;
                    continue;
                }
                let metadata = source.metadata(parent).map_err(|error| {
                    VaultFileError::MetadataFailed {
                        path: parent_path.as_str().into(),
                        message: error.to_string().into(),
                    }
                })?;
                let folder = VaultFolder::try_new(parent_path, &metadata)
                    .map_err(|error| VaultFileError::InvalidPath {
                        path: parent.to_string_lossy().into_owned().into(),
                        reason: error.to_string().into(),
                    })?;
                folders.push(folder);
                current = parent;
            }
        }
        Ok(folders)
    }
}

impl VaultProcessor<Comparison, Scanned> {
    #[inline]
    fn compare(
        self,
        repository: &VaultRepository<'_>,
    ) -> Result<VaultProcessor<Routing, Compared>, VaultProcessError> {
        let mut report = VaultProcessReport {
            files_scanned: self.status.files.len(),
            folders_scanned: self.status.folders.len(),
            ..VaultProcessReport::default()
        };

        let mut markdown_candidates = Vec::new();

        for file in &self.status.files {
            let stored = repository.get_file(file.path())?;
            let is_markdown = is_markdown_path(file.path().as_path());
            let mut should_route = false;

            if let Some(existing) = stored {
                if metadata_match(&existing, file) {
                    bump(&mut report.files_fresh);
                } else {
                    repository.save_file(file)?;
                    bump(&mut report.files_updated);
                    should_route = true;
                }
            } else {
                repository.save_file(file)?;
                bump(&mut report.files_added);
                should_route = true;
            }

            if should_route && is_markdown {
                markdown_candidates.push(file.clone());
            }
        }

        for folder in &self.status.folders {
            let stored = repository.get_folder(folder.path())?;
            match stored {
                None => {
                    repository.save_folder(folder)?;
                    bump(&mut report.folders_added);
                }
                Some(existing) => {
                    if folder_metadata_match(&existing, folder) {
                        // no report count for fresh folders
                    } else {
                        repository.save_folder(folder)?;
                        bump(&mut report.folders_updated);
                    }
                }
            }
        }

        Ok(Self::transition(Routing, Compared {
            mode: self.status.mode,
            path_set: self.status.path_set,
            markdown_candidates,
            report,
        }))
    }
}

impl VaultProcessor<Routing, Compared> {
    #[inline]
    fn route(
        self,
        note_repository: &NoteRepository<'_>,
        config: &Config,
        source: &FsReader,
    ) -> Result<VaultProcessor<Prune, Routed>, VaultProcessError> {
        let mut report = self.status.report;
        for file in &self.status.markdown_candidates {
            let info = NoteFileInfo::try_from_path(
                file.path().as_str(),
                file.size(),
                file.created_at(),
                file.modified_at(),
            )
            .map_err(NoteProcessError::from)?;
            let note_report = NoteProcessor::new().process_file(
                note_repository,
                config,
                source,
                info,
            )?;
            bump(&mut report.markdown_routed);
            match note_report.action() {
                NoteProcessAction::Created | NoteProcessAction::Updated => {
                    bump(&mut report.notes_created_or_updated);
                }
                NoteProcessAction::Unchanged => {}
                NoteProcessAction::Deleted => {
                    bump(&mut report.notes_deleted);
                }
            }
        }

        Ok(Self::transition(Prune, Routed {
            mode: self.status.mode,
            path_set: self.status.path_set,
            report,
        }))
    }
}

impl VaultProcessor<Prune, Routed> {
    #[inline]
    fn prune(
        self,
        repository: &VaultRepository<'_>,
        note_repository: &NoteRepository<'_>,
    ) -> Result<VaultProcessor<Completed, Ready>, VaultProcessError> {
        let mut report = self.status.report;
        if self.status.mode != ScanMode::Full {
            return Ok(Self::transition(Completed, Ready {
                report,
            }));
        }

        let files = repository.list_files()?;
        for file in files {
            if self.status.path_set.contains(file.path()) {
                continue;
            }
            repository.delete_file(file.path())?;
            bump(&mut report.files_deleted);
            if !is_markdown_path(file.path().as_path()) {
                continue;
            }
            let info = NoteFileInfo::try_from_path(
                file.path().as_str(),
                file.size(),
                file.created_at(),
                file.modified_at(),
            )
            .map_err(NoteProcessError::from)?;
            let note_report = NoteProcessor::new()
                .record_deleted(note_repository, info.path())?;
            if note_report.action() == NoteProcessAction::Deleted {
                bump(&mut report.notes_deleted);
            }
        }

        let folders = repository.list_folders()?;
        for folder in folders {
            if self.status.path_set.contains(folder.path()) {
                continue;
            }
            repository.delete_folder(folder.path())?;
            bump(&mut report.folders_deleted);
        }

        Ok(Self::transition(Completed, Ready {
            report,
        }))
    }

    #[inline]
    fn complete_partial(self) -> VaultProcessor<Completed, Ready> {
        Self::transition(Completed, Ready {
            report: self.status.report,
        })
    }
}

impl VaultProcessor<Completed, Ready> {
    #[inline]
    #[must_use]
    /// Returns the vault process report.
    pub fn report(self) -> VaultProcessReport {
        self.status.report
    }
}

impl Default for VaultProcessor<Discovery, Unknown> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn bump(value: &mut usize) {
    *value = value.saturating_add(1);
}

#[inline]
fn metadata_match(stored: &VaultFile, current: &VaultFile) -> bool {
    let size_match = stored.size() == current.size();
    let mtime_match = match (stored.modified_at(), current.modified_at()) {
        (Some(stored), Some(current)) => stored == current,
        (None, None) => true,
        _ => false,
    };
    size_match && mtime_match
}

#[inline]
fn folder_metadata_match(stored: &VaultFolder, current: &VaultFolder) -> bool {
    match (stored.modified_at(), current.modified_at()) {
        (Some(stored), Some(current)) => stored == current,
        (None, None) => true,
        _ => false,
    }
}
