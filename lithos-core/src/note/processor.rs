//! Note processing pipeline.
//!
//! Provides a typestate-driven pipeline for processing markdown notes using
//! file metadata and the note repository.

use std::{marker::PhantomData, sync::Arc};

use crate::{
    config::aggregate::Config,
    fs::FsReader,
    note::{
        aggregate::{Note, NoteId},
        error::{
            NoteError, NoteFileError, NoteIngestError, NoteProcessError,
            NoteRepositoryError,
        },
        parser::MarkdownParser,
        paths::NotePath,
        raw::RawNote,
        storage::Repository,
    },
};

/// Core state machine tracking the current pipeline stage and knowledge status.
#[derive(Debug)]
#[must_use]
pub struct NoteProcessor<P, S> {
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> NoteProcessor<P, S> {
    #[inline]
    fn transition<NP, NS>(_stage: NP, status: NS) -> NoteProcessor<NP, NS> {
        NoteProcessor {
            status,
            _stage: PhantomData,
        }
    }
}

/// Entry phase: repository discovery.
#[derive(Debug)]
#[non_exhaustive]
pub struct Discovery;

/// Comparison phase: metadata checks.
#[derive(Debug)]
#[non_exhaustive]
pub struct Comparison;

/// Analysis phase: parse markdown into raw note.
#[derive(Debug)]
#[non_exhaustive]
pub struct Analysis;

/// Construction phase: build and persist note domain data.
#[derive(Debug)]
#[non_exhaustive]
pub struct Construction;

/// Terminal phase: completed processing.
#[derive(Debug)]
#[non_exhaustive]
pub struct Completed;

/// Initial state before any knowledge has been gathered.
#[derive(Debug)]
#[non_exhaustive]
pub struct Unknown;

/// Status for a missing note in the repository.
#[derive(Debug)]
pub struct Missing {
    info: NoteFileInfo,
}

/// Status for a present note in the repository.
#[derive(Debug)]
pub struct Present {
    info: NoteFileInfo,
    note_id: NoteId,
}

/// Status for suspected changes with loaded content.
#[derive(Debug)]
pub struct Suspect {
    info: NoteFileInfo,
    content: String,
    is_new: bool,
}

/// Status for parsed raw note in the new path.
#[derive(Debug)]
pub struct New {
    raw: RawNote<'static>,
}

/// Status for parsed raw note in the update path.
#[derive(Debug)]
pub struct Changed {
    raw: RawNote<'static>,
}

/// Terminal status for completed processing.
#[derive(Debug)]
pub struct Ready {
    report: NoteProcessReport,
}

/// File metadata provided by the vault processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteFileInfo {
    path: NotePath,
    /// Indicates whether the vault detected this note as stale.
    is_stale: bool,
}

impl NoteFileInfo {
    /// Creates a new note file info record.
    #[inline]
    #[must_use]
    pub const fn new(path: NotePath, is_stale: bool) -> Self {
        Self {
            path,
            is_stale,
        }
    }

    /// Creates a new note file info record from a path string.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] if the path is invalid.
    #[inline]
    pub fn try_from_path(
        path: &str,
        is_stale: bool,
    ) -> Result<Self, NoteError> {
        let note_path = NotePath::try_new(path)?;
        Ok(Self::new(note_path, is_stale))
    }

    /// Returns the note path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    /// Returns whether the vault marked this note as stale.
    #[inline]
    #[must_use]
    pub const fn is_stale(&self) -> bool {
        self.is_stale
    }
}

/// Result of discovery in the note pipeline.
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "branch outcomes must be handled"]
pub enum ComparisonBranch {
    /// Note missing in repository.
    Missing(NoteProcessor<Comparison, Missing>),
    /// Note present in repository.
    Present(NoteProcessor<Comparison, Present>),
}

/// Result of metadata comparison.
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "branch outcomes must be handled"]
pub enum MetadataBranch {
    /// Note is fresh and unchanged.
    Fresh(NoteProcessor<Completed, Ready>),
    /// Note might be stale; content should be parsed.
    Stale(NoteProcessor<Analysis, Suspect>),
}

/// Outcome of parsing in the analysis stage.
#[derive(Debug)]
#[non_exhaustive]
#[must_use = "branch outcomes must be handled"]
pub enum AnalysisBranch {
    /// New note content parsed.
    New(NoteProcessor<Construction, New>),
    /// Updated note content parsed.
    Changed(NoteProcessor<Construction, Changed>),
}

/// Structured note processing report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NoteProcessReport {
    note_id: Option<NoteId>,
    path: NotePath,
    action: NoteProcessAction,
    reason: NoteProcessReason,
}

impl NoteProcessReport {
    /// Returns the processed note ID, if available.
    #[inline]
    #[must_use]
    pub const fn note_id(&self) -> Option<NoteId> {
        self.note_id
    }

    /// Returns the note path for this report.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &NotePath {
        &self.path
    }

    /// Returns the action taken by the processor.
    #[inline]
    #[must_use]
    pub const fn action(&self) -> NoteProcessAction {
        self.action
    }

    /// Returns the reason for the processor action.
    #[inline]
    #[must_use]
    pub const fn reason(&self) -> NoteProcessReason {
        self.reason
    }
}

/// High-level note processing action outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NoteProcessAction {
    /// Note created.
    Created,
    /// Note updated.
    Updated,
    /// Note unchanged.
    Unchanged,
    /// Note deleted.
    Deleted,
}

/// Reason for the note processing outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NoteProcessReason {
    /// Metadata matches cached values.
    Fresh,
    /// Metadata indicates change.
    MetadataChanged,
    /// Note missing from repository.
    Missing,
    /// Note removed from vault.
    Deleted,
}

impl NoteProcessor<Discovery, Unknown> {
    /// Creates a new note processor in the discovery stage.
    #[inline]
    pub const fn new() -> Self {
        NoteProcessor {
            status: Unknown,
            _stage: PhantomData,
        }
    }

    /// Processes a single markdown file using the typestate pipeline.
    ///
    /// # Errors
    ///
    /// Returns [`NoteProcessError`] if ingestion or persistence fails.
    #[inline]
    pub fn process_file<R: Repository<Error = NoteRepositoryError>>(
        self,
        repository: &R,
        config: &Config,
        source: &FsReader,
        info: NoteFileInfo,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let branch = self.discover(repository, info)?;
        let metadata = match branch {
            ComparisonBranch::Missing(state) => {
                state.load_content(repository, source, config)?
            }
            ComparisonBranch::Present(state) => match state.check_metadata() {
                MetadataBranch::Fresh(state) => state.report(),
                MetadataBranch::Stale(state) => {
                    state.load_content(repository, source, config)?
                }
            },
        };
        Ok(metadata)
    }

    /// Records deletion of a note by path.
    ///
    /// # Errors
    ///
    /// Returns [`NoteProcessError`] if repository access fails.
    #[inline]
    pub fn record_deleted<R: Repository<Error = NoteRepositoryError>>(
        self,
        repository: &R,
        path: &NotePath,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let stored = repository.find_by_path(path)?;
        if let Some(note) = stored {
            repository.delete_note(note.id())?;
            return Ok(NoteProcessReport {
                note_id: Some(note.id()),
                path: path.clone(),
                action: NoteProcessAction::Deleted,
                reason: NoteProcessReason::Deleted,
            });
        }

        Ok(NoteProcessReport {
            note_id: None,
            path: path.clone(),
            action: NoteProcessAction::Unchanged,
            reason: NoteProcessReason::Missing,
        })
    }

    #[inline]
    fn discover<R: Repository<Error = NoteRepositoryError>>(
        self,
        repository: &R,
        info: NoteFileInfo,
    ) -> Result<ComparisonBranch, NoteProcessError> {
        drop(self);
        let stored = repository.find_by_path(info.path())?;
        if let Some(note) = stored {
            Ok(ComparisonBranch::Present(Self::transition(
                Comparison,
                Present {
                    info,
                    note_id: note.id(),
                },
            )))
        } else {
            Ok(ComparisonBranch::Missing(Self::transition(
                Comparison,
                Missing {
                    info,
                },
            )))
        }
    }
}

impl Default for NoteProcessor<Discovery, Unknown> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NoteProcessor<Comparison, Present> {
    #[inline]
    fn check_metadata(self) -> MetadataBranch {
        if self.status.info.is_stale() {
            let info = self.status.info;
            MetadataBranch::Stale(Self::transition(Analysis, Suspect {
                info,
                content: String::new(),
                is_new: false,
            }))
        } else {
            MetadataBranch::Fresh(Self::transition(Completed, Ready {
                report: NoteProcessReport {
                    note_id: Some(self.status.note_id),
                    path: self.status.info.path.clone(),
                    action: NoteProcessAction::Unchanged,
                    reason: NoteProcessReason::Fresh,
                },
            }))
        }
    }
}

impl NoteProcessor<Comparison, Missing> {
    #[inline]
    fn load_content(
        self,
        repository: &impl Repository<Error = NoteRepositoryError>,
        source: &FsReader,
        config: &Config,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let content = source
            .read_to_string(std::path::Path::new(
                self.status.info.path.as_str(),
            ))
            .map_err(|error| {
                NoteIngestError::from(NoteFileError::ReadFailed {
                    path: self.status.info.path.clone(),
                    message: error.to_string().into(),
                })
            })?;
        let suspect = Self::transition(Analysis, Suspect {
            info: self.status.info,
            content,
            is_new: true,
        });
        let analysis = suspect.parse(config)?;
        match analysis {
            AnalysisBranch::New(state) => state.persist(repository, config),
            AnalysisBranch::Changed(state) => state.persist(repository, config),
        }
    }
}

impl NoteProcessor<Analysis, Suspect> {
    #[inline]
    fn load_content(
        self,
        repository: &impl Repository<Error = NoteRepositoryError>,
        source: &FsReader,
        config: &Config,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let content = source
            .read_to_string(std::path::Path::new(
                self.status.info.path.as_str(),
            ))
            .map_err(|error| {
                NoteIngestError::from(NoteFileError::ReadFailed {
                    path: self.status.info.path.clone(),
                    message: error.to_string().into(),
                })
            })?;
        let suspect = Self::transition(Analysis, Suspect {
            info: self.status.info,
            content,
            is_new: self.status.is_new,
        });
        let analysis = suspect.parse(config)?;
        match analysis {
            AnalysisBranch::New(state) => state.persist(repository, config),
            AnalysisBranch::Changed(state) => state.persist(repository, config),
        }
    }

    #[inline]
    fn parse(
        self,
        config: &Config,
    ) -> Result<AnalysisBranch, NoteProcessError> {
        let task_spec = Arc::new(config.to_task_spec());
        let raw = MarkdownParser::parse(
            &self.status.content,
            self.status.info.path.clone(),
            &task_spec,
        )
        .map(RawNote::into_owned)?;

        if self.status.is_new {
            Ok(AnalysisBranch::New(Self::transition(Construction, New {
                raw,
            })))
        } else {
            Ok(AnalysisBranch::Changed(Self::transition(
                Construction,
                Changed {
                    raw,
                },
            )))
        }
    }
}

impl NoteProcessor<Construction, New> {
    #[inline]
    fn persist<R: Repository<Error = NoteRepositoryError>>(
        self,
        repository: &R,
        config: &Config,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let path = self.status.raw.path.clone();
        let frontmatter_spec = config.to_frontmatter_spec();
        let task_spec = config.to_task_spec();
        let note_id = repository
            .find_by_path(&path)?
            .map_or_else(NoteId::new, |note| note.id());
        let facts = Note::try_from((
            self.status.raw,
            note_id,
            &frontmatter_spec,
            &task_spec,
        ))?;
        let saved_id = repository.save(&facts)?;
        Ok(NoteProcessReport {
            note_id: Some(saved_id),
            path,
            action: NoteProcessAction::Created,
            reason: NoteProcessReason::Missing,
        })
    }
}

impl NoteProcessor<Construction, Changed> {
    #[inline]
    fn persist<R: Repository<Error = NoteRepositoryError>>(
        self,
        repository: &R,
        config: &Config,
    ) -> Result<NoteProcessReport, NoteProcessError> {
        let path = self.status.raw.path.clone();
        let frontmatter_spec = config.to_frontmatter_spec();
        let task_spec = config.to_task_spec();
        let note_id = repository
            .find_by_path(&path)?
            .map_or_else(NoteId::new, |note| note.id());
        let facts = Note::try_from((
            self.status.raw,
            note_id,
            &frontmatter_spec,
            &task_spec,
        ))?;
        let saved_id = repository.save(&facts)?;
        Ok(NoteProcessReport {
            note_id: Some(saved_id),
            path,
            action: NoteProcessAction::Updated,
            reason: NoteProcessReason::MetadataChanged,
        })
    }
}

impl NoteProcessor<Completed, Ready> {
    #[inline]
    fn report(self) -> NoteProcessReport {
        self.status.report
    }
}
