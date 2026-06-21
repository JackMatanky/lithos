//! Template ingestion pipeline.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline that chooses the cheapest valid
//! path to a final `Template`. It uses two compile-time dimensions:
//!
//! - **Stage**: the current pipeline phase (`Init`, `Comparison`, `Parsed`,
//!   `Refresh`, `Construction`, `CompletedPhase`).
//! - **Status**: the knowledge state carrying data and invariants
//!   (`Discovered`, `Missing`, `Present`, `Suspect`, `StaleMetadata`, `New`,
//!   `Changed`, `Stale`, `Fresh`).
//!
//! The dual-typestate design prevents invalid transitions at compile time and
//! keeps orchestration in the
//! [`Repository`](crate::template::repository::Repository).
//!
//! # Flow
//!
//! ```text
//! Entry
//!   ├─ No view
//!   │   → [Parsed] parse raw file
//!   │   → [Construction] construct domain from raw → Completed
//!   └─ View found
//!       → [Comparison] check metadata
//!
//! Metadata Check
//!   ├─ [match]
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Comparison] check content hash
//!
//! Content Check
//!   ├─ [match]
//!   │   → [Refresh] sync metadata
//!   │   → [Construction] fetch cached domain → Completed
//!   └─ [mismatch]
//!       → [Parsed] parse raw template
//!       → [Construction] construct/update aggregate → Completed
//! ```
//!
//! # Maintenance Notes
//!
//! - Add new stages/statuses only when they introduce a new invariant or reduce
//!   work; each state must carry the data needed to satisfy its invariant.

use std::marker::PhantomData;

use crate::{
    fs::{FileNode, FileReader, PathKey},
    support::content_hash::{
        Blake3Hash, HasContentHash, HasContentHashMut, HashInput,
    },
    template::{
        aggregate::{Template, TemplateId, TemplateName},
        error::{TemplateError, TemplateReadError, TemplateRepositoryError},
        raw::RawTemplate,
        repository::{ReadRepository, WriteRepository},
        views::RawTemplateView,
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Processor Core
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub(crate) struct TemplateProcessor<Phase, Status> {
    file: FileNode,
    path_key: PathKey,
    status: Status,
    _phase: PhantomData<Phase>,
}

impl<Phase, Status> TemplateProcessor<Phase, Status> {
    #[inline]
    fn into_parts(self) -> (FileNode, PathKey, Status) {
        (self.file, self.path_key, self.status)
    }

    #[inline]
    fn transition_from_parts<NP, NS>(
        file: FileNode,
        path_key: PathKey,
        status: NS,
    ) -> TemplateProcessor<NP, NS> {
        TemplateProcessor {
            file,
            path_key,
            status,
            _phase: PhantomData,
        }
    }

    #[inline]
    fn transition<NP, NS>(
        self,
        _phase: NP,
        status: NS,
    ) -> TemplateProcessor<NP, NS> {
        let (file, path_key, _) = self.into_parts();
        Self::transition_from_parts(file, path_key, status)
    }

    #[cfg(test)]
    pub(crate) fn file(&self) -> &FileNode {
        &self.file
    }

    #[cfg(test)]
    pub(crate) fn path_key(&self) -> &PathKey {
        &self.path_key
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Init Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Init;
#[derive(Debug)]
pub(crate) struct Discovered {
    cache: DiscoveredCacheState,
}
#[derive(Debug)]
pub(crate) enum DiscoveredCacheState {
    New(Missing),
    Exists(Present),
    Corrupt(Corrupted),
}

#[derive(Debug)]
pub(crate) struct DiscoveredTemplate {
    file: FileNode,
    path_key: PathKey,
    cache: DiscoveredCacheState,
}

impl DiscoveredTemplate {
    pub(crate) fn new_missing(file: FileNode, path_key: PathKey) -> Self {
        Self {
            file,
            path_key,
            cache: DiscoveredCacheState::New(Missing),
        }
    }

    pub(crate) fn new_present(
        file: FileNode,
        path_key: PathKey,
        id: TemplateId,
        view: RawTemplateView,
    ) -> Self {
        Self {
            file,
            path_key,
            cache: DiscoveredCacheState::Exists(Present {
                id,
                view,
            }),
        }
    }

    pub(crate) fn new_corrupt(
        file: FileNode,
        path_key: PathKey,
        id: TemplateId,
        view: Option<RawTemplateView>,
    ) -> Self {
        Self {
            file,
            path_key,
            cache: DiscoveredCacheState::Corrupt(Corrupted {
                id,
                view,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) const fn cache(&self) -> &DiscoveredCacheState {
        &self.cache
    }
}

#[derive(Debug)]
pub(crate) struct Missing;
#[derive(Debug)]
pub(crate) struct Present {
    pub(crate) id: TemplateId,
    pub(crate) view: RawTemplateView,
}

impl TemplateProcessor<Init, Discovered> {
    pub(crate) fn new(discovered: DiscoveredTemplate) -> Self {
        Self {
            file: discovered.file,
            path_key: discovered.path_key,
            status: Discovered {
                cache: discovered.cache,
            },
            _phase: PhantomData,
        }
    }

    pub(crate) fn run<R: ReadRepository + WriteRepository>(
        self,
        repository: &R,
        source: &FileReader,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        match status.cache {
            DiscoveredCacheState::New(missing) => {
                Self::transition_from_parts(file, path_key, missing)
                    .run_missing(repository, source, template_root)
            }
            DiscoveredCacheState::Exists(present) => {
                Self::transition_from_parts(file, path_key, present)
                    .run_present(repository, source, template_root)
            }
            DiscoveredCacheState::Corrupt(corrupted) => {
                Self::transition_from_parts(file, path_key, corrupted)
                    .run_corrupt(repository, source, template_root)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Corrupted {
    pub(crate) id: TemplateId,
    pub(crate) view: Option<RawTemplateView>,
}

impl TemplateProcessor<Init, Missing> {
    fn run_missing<R: WriteRepository>(
        self,
        repository: &R,
        source: &FileReader,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        TemplateProcessor::<Parsed, Missing>::transition_from_parts(
            file, path_key, status,
        )
        .parse(source)?
        .create(repository, template_root)
    }
}

impl TemplateProcessor<Init, Present> {
    fn run_present<R: ReadRepository + WriteRepository>(
        self,
        repository: &R,
        source: &FileReader,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        let processor =
            TemplateProcessor::<Comparison, Present>::transition_from_parts(
                file, path_key, status,
            );
        match processor.check_metadata() {
            MetadataBranch::Match(fresh) => fresh.fetch(repository),
            MetadataBranch::Mismatch(suspect) => {
                match suspect.check_content(source)? {
                    ContentBranch::Match(refresh) => {
                        refresh.sync_metadata(repository)?.fetch(repository)
                    }
                    ContentBranch::Mismatch(stale) => {
                        stale.parse().update(repository, template_root)
                    }
                }
            }
        }
    }
}

impl TemplateProcessor<Init, Corrupted> {
    fn run_corrupt<R: WriteRepository>(
        self,
        repository: &R,
        source: &FileReader,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        TemplateProcessor::<Parsed, Corrupted>::transition_from_parts(
            file, path_key, status,
        )
        .parse(source)?
        .update(repository, template_root)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Comparison Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Comparison;
#[derive(Debug)]
pub(crate) struct Suspect {
    pub(crate) id: TemplateId,
    pub(crate) view: RawTemplateView,
}
pub(crate) enum MetadataBranch {
    Match(TemplateProcessor<Construction, Fresh>),
    Mismatch(TemplateProcessor<Comparison, Suspect>),
}

impl TemplateProcessor<Comparison, Present> {
    pub(crate) fn check_metadata(self) -> MetadataBranch {
        let (file, path_key, status) = self.into_parts();
        let metadata = file.metadata();
        let is_size_match =
            metadata.is_size_match(status.view.metadata().size());
        let is_timestamp_match = metadata.is_timestamp_match(
            status.view.metadata().times().created_at(),
            status.view.metadata().times().modified_at(),
        );

        if is_size_match && is_timestamp_match {
            MetadataBranch::Match(Self::transition_from_parts(
                file,
                path_key,
                Fresh {
                    id: status.id,
                },
            ))
        } else {
            MetadataBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Suspect {
                    id: status.id,
                    view: status.view,
                },
            ))
        }
    }
}

#[derive(Debug)]
pub(crate) struct Stale {
    pub(crate) id: TemplateId,
    pub(crate) content_str: String,
    pub(crate) content_hash: Blake3Hash,
    pub(crate) view: RawTemplateView,
}

pub(crate) enum ContentBranch {
    Match(TemplateProcessor<Refresh, StaleMetadata>),
    Mismatch(TemplateProcessor<Parsed, Stale>),
}

impl TemplateProcessor<Comparison, Suspect> {
    pub(crate) fn check_content(
        self,
        source: &FileReader,
    ) -> Result<ContentBranch, TemplateReadError> {
        let (file, path_key, status) = self.into_parts();
        let content =
            source.read_to_string(file.path().as_ref()).map_err(|e| {
                TemplateReadError::Read(crate::fs::ReadError::Io {
                    path: file.path().as_ref().to_path_buf(),
                    source: std::io::Error::other(e.to_string()),
                })
            })?;
        let hash = Blake3Hash::compute(HashInput::Text(content.clone()));

        if status.view.is_content_match(&hash) {
            Ok(ContentBranch::Match(Self::transition_from_parts(
                file,
                path_key,
                StaleMetadata {
                    id: status.id,
                    view: status.view,
                },
            )))
        } else {
            Ok(ContentBranch::Mismatch(Self::transition_from_parts(
                file,
                path_key,
                Stale {
                    id: status.id,
                    content_str: content,
                    content_hash: hash,
                    view: status.view,
                },
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Parsed Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Parsed;

impl TemplateProcessor<Parsed, Missing> {
    pub(crate) fn parse(
        self,
        source: &FileReader,
    ) -> Result<TemplateProcessor<Construction, New>, TemplateReadError> {
        let content =
            source.read_to_string(self.file.path().as_ref()).map_err(|e| {
                TemplateReadError::Read(crate::fs::ReadError::Io {
                    path: self.file.path().as_ref().to_path_buf(),
                    source: std::io::Error::other(e.to_string()),
                })
            })?;
        let hash = Blake3Hash::compute(HashInput::Text(content.clone()));
        Ok(self.transition(Construction, New {
            id: TemplateId::new(),
            content_hash: hash,
            raw: RawTemplate::new(content),
        }))
    }
}

impl TemplateProcessor<Parsed, Stale> {
    pub(crate) fn parse(self) -> TemplateProcessor<Construction, Changed> {
        let (file, path_key, status) = self.into_parts();
        Self::transition_from_parts(file, path_key, Changed {
            id: status.id,
            content_hash: status.content_hash,
            raw: RawTemplate::new(status.content_str),
            view: Some(status.view),
        })
    }
}

impl TemplateProcessor<Parsed, Corrupted> {
    pub(crate) fn parse(
        self,
        source: &FileReader,
    ) -> Result<TemplateProcessor<Construction, Changed>, TemplateReadError>
    {
        let (file, path_key, status) = self.into_parts();
        let content =
            source.read_to_string(file.path().as_ref()).map_err(|e| {
                TemplateReadError::Read(crate::fs::ReadError::Io {
                    path: file.path().as_ref().to_path_buf(),
                    source: std::io::Error::other(e.to_string()),
                })
            })?;
        let hash = Blake3Hash::compute(HashInput::Text(content.clone()));
        Ok(Self::transition_from_parts(file, path_key, Changed {
            id: status.id,
            content_hash: hash,
            raw: RawTemplate::new(content),
            view: status.view,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Refresh Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Refresh;
#[derive(Debug)]
pub(crate) struct StaleMetadata {
    pub(crate) id: TemplateId,
    pub(crate) view: RawTemplateView,
}

impl TemplateProcessor<Refresh, StaleMetadata> {
    pub(crate) fn sync_metadata<R: WriteRepository>(
        self,
        repository: &R,
    ) -> Result<TemplateProcessor<Construction, Fresh>, TemplateError> {
        let (file, path_key, mut status) = self.into_parts();
        status.view.update_metadata(file.metadata().clone());
        repository
            .save_raw_template_view(&status.view)
            .map_err(TemplateError::Repository)?;
        Ok(Self::transition_from_parts(file, path_key, Fresh {
            id: status.id,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Construction;

#[derive(Debug)]
pub(crate) struct New {
    pub(crate) id: TemplateId,
    pub(crate) content_hash: Blake3Hash,
    pub(crate) raw: RawTemplate,
}
#[derive(Debug)]
pub(crate) struct Changed {
    pub(crate) id: TemplateId,
    pub(crate) content_hash: Blake3Hash,
    pub(crate) raw: RawTemplate,
    pub(crate) view: Option<RawTemplateView>,
}
#[derive(Debug)]
pub(crate) struct Fresh {
    pub(crate) id: TemplateId,
}

impl TemplateProcessor<Construction, New> {
    pub(crate) fn create<R: WriteRepository>(
        self,
        repository: &R,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        let name = TemplateName::try_new(file.path().as_ref(), template_root)?;
        let template = Template::new(
            status.id,
            path_key.clone(),
            name,
            crate::template::aggregate::TemplateBody::try_new(
                status.raw.into_inner(),
            )?,
        );
        let view = RawTemplateView::new(
            path_key.clone(),
            status.content_hash,
            file.metadata().clone(),
            std::time::SystemTime::now(),
        );

        repository
            .save_template(&template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(&view)
            .map_err(TemplateError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Completed {
            template,
        }))
    }
}

impl TemplateProcessor<Construction, Changed> {
    pub(crate) fn update<R: WriteRepository>(
        self,
        repository: &R,
        template_root: &std::path::Path,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        let name = TemplateName::try_new(file.path().as_ref(), template_root)?;
        let template = Template::new(
            status.id,
            path_key.clone(),
            name,
            crate::template::aggregate::TemplateBody::try_new(
                status.raw.into_inner(),
            )?,
        );
        let view = match status.view {
            Some(mut view) => {
                view.set_content_hash(status.content_hash);
                view.update_metadata(file.metadata().clone());
                view
            }
            None => RawTemplateView::new(
                path_key.clone(),
                status.content_hash,
                file.metadata().clone(),
                std::time::SystemTime::now(),
            ),
        };

        repository
            .save_template(&template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(&view)
            .map_err(TemplateError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, Completed {
            template,
        }))
    }
}

impl TemplateProcessor<Construction, Fresh> {
    pub(crate) fn fetch<R: ReadRepository>(
        self,
        repository: &R,
    ) -> Result<TemplateProcessor<CompletedPhase, Completed>, TemplateError>
    {
        let (file, path_key, status) = self.into_parts();
        let template = repository
            .find_template_by_id(status.id)
            .map_err(TemplateError::Repository)?
            .ok_or_else(|| {
                TemplateError::Repository(
                    TemplateRepositoryError::NotFoundById(status.id),
                )
            })?;
        Ok(Self::transition_from_parts(file, path_key, Completed {
            template,
        }))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct CompletedPhase;

#[derive(Debug)]
pub(crate) struct Completed {
    template: Template,
}

impl TemplateProcessor<CompletedPhase, Completed> {
    pub(crate) fn into_template(self) -> Template {
        self.status.template
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::{
        db::testing::{
            FailureInjector, FailurePoint as FPFake, InMemoryDbError,
        },
        fs::{
            FilePath, PathKey,
            metadata::{FileMetadata, FsTimes},
        },
        template::{
            repository::WriteRepository, storage::testing::InMemoryRepository,
        },
    };

    mod fixtures {
        use super::*;

        pub fn valid_file_node(
            path_str: &str,
            content: &str,
        ) -> (FileNode, PathKey, NamedTempFile) {
            let temp_file = NamedTempFile::new().unwrap();
            fs::write(temp_file.path(), content).unwrap();

            let path =
                FilePath::try_new(temp_file.path().to_path_buf()).unwrap();
            let path_key = PathKey::try_new(path_str).unwrap();
            let times =
                FsTimes::new(Some(SystemTime::now()), Some(SystemTime::now()));
            let metadata = FileMetadata::new(
                times,
                content.len().try_into().expect("length fits in u64"),
                false,
            );
            (FileNode::new(path, metadata), path_key, temp_file)
        }

        pub struct FailOnWrite;

        impl FailureInjector for FailOnWrite {
            fn fail_at(&self, point: FPFake) -> Result<(), InMemoryDbError> {
                if point == FPFake::BeforeWrite {
                    return Err(InMemoryDbError::InjectedFailure {
                        point,
                        reason: "write injection".into(),
                    });
                }
                Ok(())
            }
        }
    }

    mod state {
        use pretty_assertions::assert_eq;

        use super::{fixtures, *};

        #[test]
        fn discovered_template_records_present_cache_state() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let id = TemplateId::new();

            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let discovered =
                DiscoveredTemplate::new_present(file, path_key, id, view);

            assert!(matches!(
                discovered.cache(),
                DiscoveredCacheState::Exists(Present { id: cached_id, .. })
                    if *cached_id == id
            ));
        }

        #[test]
        fn discovered_template_records_missing_cache_state() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let discovered = DiscoveredTemplate::new_missing(file, path_key);

            assert!(matches!(
                discovered.cache(),
                DiscoveredCacheState::New(Missing)
            ));
        }

        #[test]
        fn check_metadata_returns_match_branch_when_metadata_is_equal() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Present> {
                file,
                path_key,
                status: Present {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            let branch = processor.check_metadata();
            assert!(
                matches!(branch, MetadataBranch::Match(_)),
                "Expected Metadata Match when file metadata equals cached \
                 view metadata"
            );
        }

        #[test]
        fn check_metadata_returns_mismatch_branch_when_size_differs() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let metadata = file.metadata().clone();
            // Create a mismatch by changing the size in the view
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                FileMetadata::new(metadata.times().clone(), 9999, false),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Present> {
                file,
                path_key,
                status: Present {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            let branch = processor.check_metadata();
            assert!(
                matches!(branch, MetadataBranch::Mismatch(_)),
                "Expected Metadata Mismatch when size differs"
            );
        }

        #[test]
        fn check_content_returns_match_branch_when_hashes_are_equal() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Suspect> {
                file,
                path_key,
                status: Suspect {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            let file_path = processor.file().path().as_ref();
            let parent = file_path.parent().unwrap();
            let reader = FileReader::new(parent);

            let branch =
                processor.check_content(&reader).expect("check content");
            assert!(
                matches!(branch, ContentBranch::Match(_)),
                "Expected Content Match when disk hash equals cached view hash"
            );
        }

        #[test]
        fn check_content_returns_mismatch_branch_when_hashes_differ() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"wrong-hash"),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Suspect> {
                file,
                path_key,
                status: Suspect {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            let file_path = processor.file().path().as_ref();
            let parent = file_path.parent().unwrap();
            let reader = FileReader::new(parent);

            let branch =
                processor.check_content(&reader).expect("check content");
            assert!(
                matches!(branch, ContentBranch::Mismatch(_)),
                "Expected Content Mismatch when disk hash differs from cached \
                 view"
            );
        }

        #[test]
        fn sync_metadata_updates_repository_and_returns_fresh_processor() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let id = TemplateId::new();

            // Create a view with old metadata
            let old_metadata = FileMetadata::new(
                FsTimes::new(
                    Some(SystemTime::UNIX_EPOCH),
                    Some(SystemTime::UNIX_EPOCH),
                ),
                0,
                false,
            );
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                old_metadata,
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Refresh, StaleMetadata> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: StaleMetadata {
                    id,
                    view,
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let next = processor.sync_metadata(&repo).expect("sync metadata");

            assert_eq!(
                next.status.id, id,
                "Expected transition to keep the same TemplateId"
            );

            let persisted_view = repo
                .find_raw_template_view(next.path_key())
                .unwrap()
                .expect("Expected RawTemplateView to be persisted");

            assert_eq!(
                persisted_view.metadata().size(),
                file.metadata().size(),
                "Expected persisted view to have updated size from file"
            );
        }
    }

    mod validation {

        use super::{fixtures, *};

        #[test]
        fn check_metadata_returns_match_branch_when_size_and_timestamps_equal()
        {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"hash"),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Present> {
                file,
                path_key,
                status: Present {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            assert!(matches!(
                processor.check_metadata(),
                MetadataBranch::Match(_)
            ));
        }

        #[test]
        fn check_metadata_returns_mismatch_branch_when_size_differs() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let metadata = file.metadata();
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"hash"),
                FileMetadata::new(
                    metadata.times().clone(),
                    metadata.size() + 1,
                    false,
                ),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Present> {
                file,
                path_key,
                status: Present {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            assert!(matches!(
                processor.check_metadata(),
                MetadataBranch::Mismatch(_)
            ));
        }

        #[test]
        fn check_metadata_returns_mismatch_branch_when_timestamp_differs() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let metadata = file.metadata();
            let older_time = SystemTime::UNIX_EPOCH;
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"hash"),
                FileMetadata::new(
                    FsTimes::new(Some(older_time), Some(older_time)),
                    metadata.size(),
                    false,
                ),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Comparison, Present> {
                file,
                path_key,
                status: Present {
                    id: TemplateId::new(),
                    view,
                },
                _phase: PhantomData,
            };

            assert!(matches!(
                processor.check_metadata(),
                MetadataBranch::Mismatch(_)
            ));
        }
    }

    mod parse {
        use pretty_assertions::assert_eq;

        use super::{fixtures, *};

        #[test]
        fn parse_returns_new_status_when_missing_source_is_parsed() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let processor = TemplateProcessor::<Parsed, Missing> {
                file,
                path_key,
                status: Missing,
                _phase: PhantomData,
            };

            let parent = processor.file().path().as_ref().parent().unwrap();
            let reader = FileReader::new(parent);

            let result = processor.parse(&reader).expect("parse successful");
            assert_eq!(result.status.raw.into_inner(), "content");
            assert_ne!(result.status.id, TemplateId::default());
        }

        #[test]
        fn parse_returns_changed_status_when_stale_source_is_parsed() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "new content");
            let id = TemplateId::new();
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"old"),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Parsed, Stale> {
                file,
                path_key,
                status: Stale {
                    id,
                    content_str: "new content".to_owned(),
                    content_hash: Blake3Hash::compute(HashInput::Text(
                        "new content".to_owned(),
                    )),
                    view,
                },
                _phase: PhantomData,
            };

            let result = processor.parse();
            assert_eq!(result.status.id, id);
            assert_eq!(result.status.raw.into_inner(), "new content");
        }

        #[test]
        fn parse_propagates_read_error_when_filesystem_fails() {
            let (file, path_key, temp) =
                fixtures::valid_file_node("test.md", "content");

            let processor = TemplateProcessor::<Parsed, Missing> {
                file,
                path_key,
                status: Missing,
                _phase: PhantomData,
            };

            // Delete the file before parsing to trigger read error
            drop(temp);

            let reader = FileReader::new(std::env::temp_dir());
            let result = processor.parse(&reader);

            assert!(
                result.is_err(),
                "Expected error when filesystem read fails"
            );
            assert!(matches!(result.unwrap_err(), TemplateReadError::Read(_)));
        }

        #[test]
        fn create_rejects_parsing_when_template_body_is_empty() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "");
            let processor = TemplateProcessor::<Parsed, Missing> {
                file: file.clone(),
                path_key,
                status: Missing,
                _phase: PhantomData,
            };

            let parent = file.path().as_ref().parent().unwrap();
            let reader = FileReader::new(parent);
            let parsed = processor.parse(&reader).expect("parse content");

            let repo = InMemoryRepository::new();
            let result = parsed.create(&repo, std::path::Path::new("/"));
            assert!(
                result.is_err(),
                "Expected error when creating template from empty content"
            );
            assert!(matches!(result.unwrap_err(), TemplateError::Body(_)));
        }
    }

    mod create {
        use pretty_assertions::assert_eq;

        use super::{fixtures, *};

        #[test]
        fn create_persists_template_and_view_on_successful_execution() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let id = TemplateId::new();
            let processor = TemplateProcessor::<Construction, New> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: New {
                    id,
                    content_hash: Blake3Hash::compute(HashInput::Text(
                        "content".to_owned(),
                    )),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let template = processor
                .create(&repo, std::path::Path::new("/"))
                .expect("create template")
                .into_template();
            assert_eq!(*template.id(), id);

            assert_eq!(
                *repo.find_template_by_path(&path_key).unwrap().unwrap().id(),
                id
            );
            assert!(repo.find_raw_template_view(&path_key).unwrap().is_some());
        }

        #[test]
        fn create_returns_completed_processor_with_template() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let id = TemplateId::new();
            let processor = TemplateProcessor::<Construction, New> {
                file,
                path_key,
                status: New {
                    id,
                    content_hash: Blake3Hash::compute(HashInput::Text(
                        "content".to_owned(),
                    )),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let completed = processor
                .create(&repo, std::path::Path::new("/"))
                .expect("create template");
            let template = completed.into_template();

            assert_eq!(*template.id(), id);
        }

        #[test]
        fn create_propagates_repository_error_during_template_storage() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let processor = TemplateProcessor::<Construction, New> {
                file: file.clone(),
                path_key,
                status: New {
                    id: TemplateId::new(),
                    content_hash: Blake3Hash::compute(HashInput::Text(
                        "content".to_owned(),
                    )),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let repo_with_fail = InMemoryRepository::new()
                .with_failure_injector(Box::new(fixtures::FailOnWrite));

            let result =
                processor.create(&repo_with_fail, std::path::Path::new("/"));
            assert!(
                result.is_err(),
                "Expected repository error to be propagated"
            );
        }

        #[test]
        fn create_rejects_template_when_name_is_outside_root_boundary() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let processor = TemplateProcessor::<Construction, New> {
                file,
                path_key,
                status: New {
                    id: TemplateId::new(),
                    content_hash: Blake3Hash::from_bytes(b"hash"),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            // Root is sibling to file path, so file is not under root
            let result =
                processor.create(&repo, std::path::Path::new("/other-root"));
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), TemplateError::Name(_)));
        }
    }

    mod update {
        use pretty_assertions::assert_eq;

        use super::{fixtures, *};

        #[test]
        fn update_persists_changed_template_and_view() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "updated content");
            let id = TemplateId::new();
            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"old"),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let processor = TemplateProcessor::<Construction, Changed> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: Changed {
                    id,
                    content_hash: Blake3Hash::compute(HashInput::Text(
                        "updated content".to_owned(),
                    )),
                    raw: RawTemplate::new("updated content".to_owned()),
                    view: Some(view.clone()),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let _template = processor
                .update(&repo, std::path::Path::new("/"))
                .expect("update template");

            assert_eq!(
                *repo.find_template_by_path(&path_key).unwrap().unwrap().id(),
                id
            );
        }

        #[test]
        fn update_mutates_existing_view_hash_and_metadata() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "updated content");
            let id = TemplateId::new();
            let old_view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"old"),
                FileMetadata::new(FsTimes::new(None, None), 0, false),
                SystemTime::UNIX_EPOCH,
            );
            let new_hash = Blake3Hash::compute(HashInput::Text(
                "updated content".to_owned(),
            ));

            let processor = TemplateProcessor::<Construction, Changed> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: Changed {
                    id,
                    content_hash: new_hash,
                    raw: RawTemplate::new("updated content".to_owned()),
                    view: Some(old_view),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let completed = processor
                .update(&repo, std::path::Path::new("/"))
                .expect("update template");
            let template = completed.into_template();
            let updated_view = repo
                .find_raw_template_view(&path_key)
                .unwrap()
                .expect("view persisted");

            assert_eq!(*template.id(), id);
            assert!(updated_view.is_content_match(&new_hash));
            assert_eq!(updated_view.metadata().size(), file.metadata().size());
        }
    }

    mod lookup {

        use pretty_assertions::assert_eq;

        use super::{fixtures, *};
        use crate::template::storage::testing::InMemoryRepository;

        #[test]
        fn fetch_returns_template_when_path_is_resolved_in_repository() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let id = TemplateId::new();
            let processor = TemplateProcessor::<Construction, Fresh> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: Fresh {
                    id,
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let template = Template::new(
                id,
                path_key.clone(),
                TemplateName::try_new(
                    file.path().as_ref(),
                    std::path::Path::new("/"),
                )
                .unwrap(),
                crate::template::aggregate::TemplateBody::try_new(
                    "content".to_owned(),
                )
                .unwrap(),
            );
            repo.save_template(&template).unwrap();

            let result = processor.fetch(&repo).expect("fetch successful");
            assert_eq!(*result.into_template().id(), id);
        }

        #[test]
        fn fetch_uses_template_id_instead_of_processor_path() {
            let (file, processor_path, _temp) =
                fixtures::valid_file_node("templates/wrong.md", "content");
            let id = TemplateId::new();
            let saved_path = PathKey::try_new("templates/saved.md").unwrap();
            let processor = TemplateProcessor::<Construction, Fresh> {
                file: file.clone(),
                path_key: processor_path,
                status: Fresh {
                    id,
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            let template = Template::new(
                id,
                saved_path,
                TemplateName::try_new(
                    file.path().as_ref(),
                    std::path::Path::new("/"),
                )
                .unwrap(),
                crate::template::aggregate::TemplateBody::try_new(
                    "content".to_owned(),
                )
                .unwrap(),
            );
            repo.save_template(&template).unwrap();

            let completed = processor.fetch(&repo).expect("fetch successful");

            assert_eq!(*completed.into_template().id(), id);
        }

        #[test]
        fn fetch_returns_error_when_fresh_template_is_missing_from_repository()
        {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "content");
            let processor = TemplateProcessor::<Construction, Fresh> {
                file,
                path_key: path_key.clone(),
                status: Fresh {
                    id: TemplateId::new(),
                },
                _phase: PhantomData,
            };

            let repo = InMemoryRepository::new();
            // Repo has no template, but processor status is Fresh (phantom
            // cache scenario)
            let result = processor.fetch(&repo);

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                TemplateError::Repository(
                    TemplateRepositoryError::NotFoundById(_)
                )
            ));
        }
    }

    mod run {
        use pretty_assertions::assert_eq;

        use super::{fixtures, *};
        use crate::support::content_hash::HasContentHash;

        #[test]
        fn run_rebuilds_recoverable_corrupt_template_with_existing_id() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "rebuilt content");
            let id = TemplateId::new();
            let discovered = DiscoveredTemplate::new_corrupt(
                file,
                path_key.clone(),
                id,
                None,
            );
            let processor =
                TemplateProcessor::<Init, Discovered>::new(discovered);
            let repo = InMemoryRepository::new();
            let reader = FileReader::new(std::env::temp_dir());

            let completed = processor
                .run(&repo, &reader, std::path::Path::new("/"))
                .expect("corrupt path should rebuild");
            let template = completed.into_template();
            let view = repo
                .find_raw_template_view(&path_key)
                .unwrap()
                .expect("raw view rebuilt");
            let expected_hash = Blake3Hash::compute(HashInput::Text(
                "rebuilt content".to_owned(),
            ));

            assert_eq!(*template.id(), id);
            assert_eq!(template.body().as_str(), "rebuilt content");
            assert!(view.is_content_match(&expected_hash));
        }

        #[test]
        fn parsed_corrupt_processor_rebuilds_with_existing_id() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("test.md", "rebuilt content");
            let id = TemplateId::new();
            let processor = TemplateProcessor::<Parsed, Corrupted> {
                file,
                path_key: path_key.clone(),
                status: Corrupted {
                    id,
                    view: None,
                },
                _phase: PhantomData,
            };
            let repo = InMemoryRepository::new();
            let reader = FileReader::new(std::env::temp_dir());

            let completed = processor
                .parse(&reader)
                .expect("corrupt source should parse")
                .update(&repo, std::path::Path::new("/"))
                .expect("corrupt path should rebuild");
            let template = completed.into_template();
            let view = repo
                .find_raw_template_view(&path_key)
                .unwrap()
                .expect("raw view rebuilt");
            let expected_hash = Blake3Hash::compute(HashInput::Text(
                "rebuilt content".to_owned(),
            ));

            assert_eq!(*template.id(), id);
            assert!(view.is_content_match(&expected_hash));
        }
    }
}
