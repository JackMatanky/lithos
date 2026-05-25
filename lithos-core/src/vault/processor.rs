//! Vault processing pipeline and routing entry point.

#![expect(
    clippy::module_name_repetitions,
    reason = "Vault processor naming is explicit and scoped"
)]

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::Path,
};

use super::error::{VaultFileError, VaultProcessError};
use crate::{
    config::aggregate::Config,
    fs::{
        DirMetadata, DirName, FileFormat, FileMetadata, FileName, FsReader,
        NormalizedPath,
    },
    note::{
        error::NoteProcessError,
        paths::NotePath,
        processor::{NoteFileInfo, NoteProcessAction, NoteProcessor},
        storage_legacy::RedbRepository as NoteRepository,
    },
    vault::{
        model::{DirId, DirView, FileId, FileView},
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
    files: Vec<ScannedFile>,
    dirs: Vec<ScannedDir>,
    path_set: HashSet<NormalizedPath>,
}

/// Status after comparison with routing candidates.
#[derive(Debug)]
pub struct Compared {
    mode: ScanMode,
    path_set: HashSet<NormalizedPath>,
    markdown_candidates: Vec<NormalizedPath>,
    report: VaultProcessReport,
}

/// Status after routing markdown files.
#[derive(Debug)]
pub struct Routed {
    mode: ScanMode,
    path_set: HashSet<NormalizedPath>,
    report: VaultProcessReport,
}

/// Terminal status carrying the final report.
#[derive(Debug)]
pub struct Ready {
    report: VaultProcessReport,
}

struct CompareOutcome {
    markdown_candidates: Vec<NormalizedPath>,
    file_updates: Vec<ScannedFile>,
    dir_updates: Vec<ScannedDir>,
}

#[derive(Debug, Clone)]
struct ScannedFile {
    path: NormalizedPath,
    view: FileView,
}

#[derive(Debug, Clone)]
struct ScannedDir {
    path: NormalizedPath,
    view: DirView,
}

type ScanViews = (Vec<ScannedDir>, Vec<ScannedFile>);

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
        paths: &[NormalizedPath],
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
        let (dirs, files) = Self::scan_views(source)?;
        let mut path_set =
            HashSet::with_capacity(files.len().saturating_add(dirs.len()));
        for file in &files {
            path_set.insert(file.path.clone());
        }
        for dir in &dirs {
            path_set.insert(dir.path.clone());
        }

        Ok(Self::transition(Comparison, Scanned {
            mode,
            files,
            dirs,
            path_set,
        }))
    }

    #[inline]
    #[expect(
        clippy::too_many_lines,
        reason = "Partial path handling performs validation, scan conversion, \
                  and parent backfill in one transition stage"
    )]
    fn discover_partial(
        self,
        source: &FsReader,
        paths: &[NormalizedPath],
    ) -> Result<VaultProcessor<Comparison, Scanned>, VaultProcessError> {
        drop(self);
        let mut files = Vec::with_capacity(paths.len());
        let mut dirs = Vec::new();
        let mut known_dirs = HashMap::<NormalizedPath, DirId>::new();

        for path in paths {
            source.validate_path(Path::new(path.as_str())).map_err(
                |error| VaultFileError::InvalidPath {
                    path: path.as_str().into(),
                    reason: error.to_string().into(),
                },
            )?;
            let metadata = source
                .std_metadata(Path::new(path.as_str()))
                .map_err(|error| VaultFileError::MetadataFailed {
                    path: path.as_str().into(),
                    message: error.to_string().into(),
                })?;

            let normalized =
                NormalizedPath::try_new(path.as_str()).map_err(|error| {
                    VaultFileError::InvalidPath {
                        path: path.as_str().into(),
                        reason: error.to_string().into(),
                    }
                })?;
            if metadata.is_dir() {
                let parent = parent_path(&normalized)?;
                let parent_id = parent
                    .as_ref()
                    .and_then(|parent| known_dirs.get(parent))
                    .copied();
                let name = last_component(Path::new(path.as_str()))?;
                let dir = ScannedDir {
                    path: normalized.clone(),
                    view: DirView::new(
                        DirId::new(),
                        parent_id,
                        DirName::new(name),
                        DirMetadata::from(&metadata),
                    ),
                };
                known_dirs.insert(normalized, dir.view.id());
                dirs.push(dir);
            } else {
                let parent = parent_path(&normalized)?;
                let parent_id = parent
                    .as_ref()
                    .and_then(|parent| known_dirs.get(parent))
                    .copied();
                let filename = last_component(Path::new(path.as_str()))?;
                let format = Path::new(path.as_str())
                    .extension()
                    .map_or(FileFormat::Unknown, FileFormat::from_extension);
                files.push(ScannedFile {
                    path: normalized,
                    view: FileView::new(
                        FileId::new(),
                        parent_id,
                        FileName::new(filename),
                        format,
                        FileMetadata::from(&metadata),
                        [0u8; 32],
                    ),
                });
            }
        }

        let dir_ids_by_path = dirs
            .iter()
            .map(|dir| (dir.path.clone(), dir.view.id()))
            .collect::<HashMap<_, _>>();

        for dir in &mut dirs {
            let parent = parent_path(&dir.path)?;
            let parent_id = parent
                .as_ref()
                .and_then(|key| dir_ids_by_path.get(key))
                .copied();
            if parent_id == dir.view.parent_id() {
                continue;
            }
            dir.view = DirView::new(
                dir.view.id(),
                parent_id,
                dir.view.name().clone(),
                dir.view.metadata().clone(),
            );
        }

        for file in &mut files {
            let parent = parent_path(&file.path)?;
            let parent_id = parent
                .as_ref()
                .and_then(|key| dir_ids_by_path.get(key))
                .copied();
            if parent_id == file.view.parent_id() {
                continue;
            }
            file.view = FileView::new(
                file.view.id(),
                parent_id,
                file.view.name().clone(),
                file.view.format(),
                file.view.metadata().clone(),
                *file.view.content_hash(),
            );
        }

        let mut path_set =
            HashSet::with_capacity(files.len().saturating_add(dirs.len()));
        for file in &files {
            path_set.insert(file.path.clone());
        }
        for dir in &dirs {
            path_set.insert(dir.path.clone());
        }

        Ok(Self::transition(Comparison, Scanned {
            mode: ScanMode::Partial,
            files,
            dirs,
            path_set,
        }))
    }

    fn scan_views(source: &FsReader) -> Result<ScanViews, VaultFileError> {
        // Pre-compute relative paths for all directories before sorting
        // This handles the fallible as_relative() call before sorting
        let mut dir_entries: Vec<(
            crate::fs::path::RelativePath,
            crate::fs::entry::FsDir,
        )> = source
            .filter_dir_entries("**/*")
            .map_err(|error| VaultFileError::ReadFailed {
                path: "<vault>".into(),
                message: error.to_string().into(),
            })?
            .into_iter()
            .map(|entry| {
                let relative = entry
                    .path()
                    .as_relative(source.root())
                    .map_err(|error| VaultFileError::ReadFailed {
                        path: "<vault>".into(),
                        message: error.to_string().into(),
                    })?;
                Ok((relative, entry))
            })
            .collect::<Result<Vec<_>, VaultFileError>>()?;

        // Sort by depth (component count), then by path
        dir_entries.sort_by(|(rel_a, _), (rel_b, _)| {
            let depth_a = rel_a.as_path().components().count();
            let depth_b = rel_b.as_path().components().count();
            depth_a
                .cmp(&depth_b)
                .then_with(|| rel_a.as_path().cmp(rel_b.as_path()))
        });

        let mut dirs = Vec::with_capacity(dir_entries.len());
        let mut dir_ids_by_path = HashMap::with_capacity(dir_entries.len());
        for (relative, entry) in dir_entries {
            let path = normalized_path_from_relative(relative.as_path())?;
            let parent = parent_path(&path)?;
            let parent_id = parent
                .as_ref()
                .and_then(|key| dir_ids_by_path.get(key))
                .copied();
            let dir = ScannedDir {
                path: path.clone(),
                view: DirView::new(
                    DirId::new(),
                    parent_id,
                    DirName::new(last_component(relative.as_path())?),
                    entry.metadata().clone(),
                ),
            };
            dir_ids_by_path.insert(path, dir.view.id());
            dirs.push(dir);
        }

        let file_entries =
            source.filter_file_entries("**/*").map_err(|error| {
                VaultFileError::ReadFailed {
                    path: "<vault>".into(),
                    message: error.to_string().into(),
                }
            })?;

        let mut files = Vec::with_capacity(file_entries.len());
        for file_entry in file_entries {
            let relative = file_entry
                .path()
                .as_relative(source.root())
                .map_err(|error| VaultFileError::ReadFailed {
                    path: "<vault>".into(),
                    message: error.to_string().into(),
                })?;
            let path = normalized_path_from_relative(relative.as_path())?;
            let parent = parent_path(&path)?;
            let parent_id = parent
                .as_ref()
                .and_then(|key| dir_ids_by_path.get(key))
                .copied();
            let format = relative
                .as_path()
                .extension()
                .map_or(FileFormat::Unknown, FileFormat::from_extension);
            let file = ScannedFile {
                path,
                view: FileView::new(
                    FileId::new(),
                    parent_id,
                    FileName::new(last_component(relative.as_path())?),
                    format,
                    file_entry.metadata().clone(),
                    [0u8; 32],
                ),
            };
            files.push(file);
        }

        Ok((dirs, files))
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
            folders_scanned: self.status.dirs.len(),
            ..VaultProcessReport::default()
        };

        let outcome = match self.status.mode {
            ScanMode::Full => self.compare_full(repository, &mut report)?,
            ScanMode::Partial => {
                self.compare_partial(repository, &mut report)?
            }
        };

        if !outcome.file_updates.is_empty() || !outcome.dir_updates.is_empty() {
            for file in &outcome.file_updates {
                repository.save_file_view(&file.path, &file.view)?;
            }
            for dir in &outcome.dir_updates {
                repository.save_dir_view(&dir.path, &dir.view)?;
            }
        }

        Ok(Self::transition(Routing, Compared {
            mode: self.status.mode,
            path_set: self.status.path_set,
            markdown_candidates: outcome.markdown_candidates,
            report,
        }))
    }

    fn compare_full(
        &self,
        repository: &VaultRepository<'_>,
        report: &mut VaultProcessReport,
    ) -> Result<CompareOutcome, VaultProcessError> {
        let existing_file_paths = repository.list_file_paths()?;
        let existing_dir_paths = repository.list_dir_paths()?;

        let mut file_map = HashMap::new();
        for path in existing_file_paths {
            if let Some(view) = repository.find_file_view_by_path(&path)? {
                file_map.insert(path, view);
            }
        }
        let mut dir_map = HashMap::new();
        for path in existing_dir_paths {
            if let Some(view) = repository.find_dir_view_by_path(&path)? {
                dir_map.insert(path, view);
            }
        }

        let mut file_updates = Vec::new();
        let mut markdown_candidates = Vec::new();
        for file in &self.status.files {
            let existing = file_map.get(&file.path);
            let (should_update, should_route) =
                evaluate_file(existing, &file.view, report);
            if should_update {
                file_updates.push(file.clone());
            }
            if should_route {
                markdown_candidates.push(file.path.clone());
            }
        }

        let mut dir_updates = Vec::new();
        for dir in &self.status.dirs {
            let existing = dir_map.get(&dir.path);
            if evaluate_dir(existing, &dir.view, report) {
                dir_updates.push(dir.clone());
            }
        }

        Ok(CompareOutcome {
            markdown_candidates,
            file_updates,
            dir_updates,
        })
    }

    fn compare_partial(
        &self,
        repository: &VaultRepository<'_>,
        report: &mut VaultProcessReport,
    ) -> Result<CompareOutcome, VaultProcessError> {
        let mut file_updates = Vec::new();
        let mut markdown_candidates = Vec::new();
        for file in &self.status.files {
            let existing = repository.find_file_view_by_path(&file.path)?;
            let (should_update, should_route) =
                evaluate_file(existing.as_ref(), &file.view, report);
            if should_update {
                file_updates.push(file.clone());
            }
            if should_route {
                markdown_candidates.push(file.path.clone());
            }
        }

        let mut dir_updates = Vec::new();
        for dir in &self.status.dirs {
            let existing = repository.find_dir_view_by_path(&dir.path)?;
            if evaluate_dir(existing.as_ref(), &dir.view, report) {
                dir_updates.push(dir.clone());
            }
        }

        Ok(CompareOutcome {
            markdown_candidates,
            file_updates,
            dir_updates,
        })
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
            let info = NoteFileInfo::try_from_path(file.as_str(), true)
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

        let paths = repository.list_file_paths()?;
        for path in paths {
            if self.status.path_set.contains(&path) {
                continue;
            }
            let Some(file) = repository.find_file_view_by_path(&path)? else {
                continue;
            };
            repository.delete_file_view(file.id())?;
            bump(&mut report.files_deleted);
            if !is_markdown_path(Path::new(path.as_str())) {
                continue;
            }
            let note_path = NotePath::try_new(path.as_str())
                .map_err(NoteProcessError::from)?;
            let note_report = NoteProcessor::new()
                .record_deleted(note_repository, &note_path)?;
            if note_report.action() == NoteProcessAction::Deleted {
                bump(&mut report.notes_deleted);
            }
        }

        let dir_paths = repository.list_dir_paths()?;
        for path in dir_paths {
            if self.status.path_set.contains(&path) {
                continue;
            }
            if let Some(dir) = repository.find_dir_view_by_path(&path)? {
                repository.delete_dir_view(dir.id())?;
            }
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

fn normalized_path_from_relative(
    relative: &Path,
) -> Result<NormalizedPath, VaultFileError> {
    let raw = relative.to_str().ok_or_else(|| VaultFileError::InvalidPath {
        path: "<invalid>".into(),
        reason: "path is not valid utf-8".into(),
    })?;
    NormalizedPath::try_new(raw).map_err(|error| VaultFileError::InvalidPath {
        path: raw.into(),
        reason: error.to_string().into(),
    })
}

fn parent_path(
    path: &NormalizedPath,
) -> Result<Option<NormalizedPath>, VaultFileError> {
    let parent = Path::new(path.as_str()).parent();
    let Some(parent) = parent else {
        return Ok(None);
    };
    if parent.as_os_str().is_empty() {
        return Ok(None);
    }
    let parent_str =
        parent.to_str().ok_or_else(|| VaultFileError::InvalidPath {
            path: path.as_str().into(),
            reason: "parent path is not valid utf-8".into(),
        })?;
    Ok(Some(NormalizedPath::try_new(parent_str).map_err(|error| {
        VaultFileError::InvalidPath {
            path: parent_str.into(),
            reason: error.to_string().into(),
        }
    })?))
}

fn last_component(path: &Path) -> Result<Box<str>, VaultFileError> {
    let name =
        path.file_name().and_then(|value| value.to_str()).ok_or_else(|| {
            VaultFileError::InvalidPath {
                path: path.to_string_lossy().into_owned().into_boxed_str(),
                reason: "missing terminal path component".into(),
            }
        })?;
    Ok(name.into())
}

#[inline]
fn bump(value: &mut usize) {
    *value = value.saturating_add(1);
}

#[inline]
fn metadata_match(stored: &FileView, current: &FileView) -> bool {
    let size_match = stored.metadata().size() == current.metadata().size();
    let mtime_match = match (
        stored.metadata().times().modified_at(),
        current.metadata().times().modified_at(),
    ) {
        (Some(stored), Some(current)) => stored == current,
        (None, None) => true,
        _ => false,
    };
    size_match && mtime_match
}

#[inline]
fn dir_metadata_match(stored: &DirView, current: &DirView) -> bool {
    match (
        stored.metadata().times().modified_at(),
        current.metadata().times().modified_at(),
    ) {
        (Some(stored), Some(current)) => stored == current,
        (None, None) => true,
        _ => false,
    }
}

fn evaluate_file(
    existing: Option<&FileView>,
    file: &FileView,
    report: &mut VaultProcessReport,
) -> (bool, bool) {
    let is_markdown = file.format() == FileFormat::Markdown;
    let should_update = match existing {
        Some(existing) if metadata_match(existing, file) => {
            bump(&mut report.files_fresh);
            false
        }
        Some(_) => {
            bump(&mut report.files_updated);
            true
        }
        None => {
            bump(&mut report.files_added);
            true
        }
    };

    (should_update, should_update && is_markdown)
}

fn evaluate_dir(
    existing: Option<&DirView>,
    dir: &DirView,
    report: &mut VaultProcessReport,
) -> bool {
    match existing {
        Some(existing) if dir_metadata_match(existing, dir) => false,
        Some(_) => {
            bump(&mut report.folders_updated);
            true
        }
        None => {
            bump(&mut report.folders_added);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{DirMetadata, DirName, FileMetadata, FileName, FsTimes};

    mod discovery_tests {
        use super::*;

        #[test]
        fn normalized_path_helper_rejects_non_utf8_or_invalid() {
            assert!(NormalizedPath::try_new("../outside").is_err());
        }
    }

    mod parent_link_tests {
        use super::*;

        #[test]
        fn parent_path_returns_none_for_root_file() {
            let path = NormalizedPath::try_new("note.md").expect("path");
            assert!(parent_path(&path).expect("parent").is_none());
        }

        #[test]
        fn parent_path_returns_parent_for_nested_path() {
            let path =
                NormalizedPath::try_new("notes/a/note.md").expect("path");
            let parent =
                parent_path(&path).expect("parent").expect("some parent");
            assert_eq!(parent.as_str(), "notes/a");
        }
    }

    mod compare_tests {
        use super::*;

        fn sample_file(
            size: u64,
            modified: Option<std::time::SystemTime>,
        ) -> FileView {
            FileView::new(
                FileId::new(),
                None,
                FileName::new("note.md".into()),
                FileFormat::Markdown,
                FileMetadata::new(FsTimes::new(None, modified), size, false),
                [1u8; 32],
            )
        }

        #[test]
        fn evaluate_file_counts_add_when_missing() {
            let mut report = VaultProcessReport::default();
            let file = sample_file(10, None);
            let (update, route) = evaluate_file(None, &file, &mut report);
            assert!(update);
            assert!(route);
            assert_eq!(report.files_added(), 1);
        }
    }

    mod route_tests {
        use super::*;

        #[test]
        fn markdown_detection_is_extension_based() {
            assert!(is_markdown_path(Path::new("notes/a.md")));
            assert!(is_markdown_path(Path::new("notes/a.markdown")));
            assert!(!is_markdown_path(Path::new("notes/a.txt")));
        }
    }

    mod prune_tests {
        use super::*;

        #[test]
        fn evaluate_dir_counts_add_when_missing() {
            let mut report = VaultProcessReport::default();
            let dir = DirView::new(
                DirId::new(),
                None,
                DirName::new("notes".into()),
                DirMetadata::new(FsTimes::new(None, None), false),
            );
            assert!(evaluate_dir(None, &dir, &mut report));
            assert_eq!(report.folders_added(), 1);
        }
    }
}
