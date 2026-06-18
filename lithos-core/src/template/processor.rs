//! Template ingestion pipeline.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline that chooses the cheapest valid
//! path to a final `Template`. It uses two compile-time dimensions:
//!
//! - **Stage**: the current pipeline phase (`Discovery`, `Comparison`,
//!   `Parsed`, `Refresh`, `Construction`, `Completed`).
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

#![allow(
    dead_code,
    unused_imports,
    reason = "Template pipeline is work-in-progress and unused until further \
              development."
)]

use std::marker::PhantomData;

use crate::{
    fs::{FileNode, FileReader, PathKey},
    support::content_hash::{Blake3Hash, HashInput},
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
//  Discovery Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Discovery;
#[derive(Debug)]
pub(crate) struct Discovered;
#[derive(Debug)]
pub(crate) struct Missing;
#[derive(Debug)]
pub(crate) struct Present {
    pub(crate) id: TemplateId,
    pub(crate) view: RawTemplateView,
}

#[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
pub(crate) enum DiscoveryBranch {
    Missing(TemplateProcessor<Parsed, Missing>),
    Present(TemplateProcessor<Comparison, Present>),
}

impl TemplateProcessor<Discovery, Discovered> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn new(file: FileNode, path_key: PathKey) -> Self {
        Self {
            file,
            path_key,
            status: Discovered,
            _phase: PhantomData,
        }
    }

    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn compare(
        self,
        id: Option<TemplateId>,
        view: Option<RawTemplateView>,
    ) -> DiscoveryBranch {
        match (id, view) {
            (Some(id), Some(view)) => {
                DiscoveryBranch::Present(self.transition(Comparison, Present {
                    id,
                    view,
                }))
            }
            _ => DiscoveryBranch::Missing(self.transition(Parsed, Missing)),
        }
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
#[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
pub(crate) enum MetadataBranch {
    Match(TemplateProcessor<Construction, Fresh>),
    Mismatch(TemplateProcessor<Comparison, Suspect>),
}

impl TemplateProcessor<Comparison, Present> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn view(&self) -> &RawTemplateView {
        &self.status.view
    }

    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn check_metadata(self) -> MetadataBranch {
        let f = self.file.metadata();
        let is_size_match = f.is_size_match(self.status.view.metadata().size());
        let is_timestamp_match = f.is_timestamp_match(
            self.status.view.metadata().times().created_at(),
            self.status.view.metadata().times().modified_at(),
        );

        let id = self.status.id;
        if is_size_match && is_timestamp_match {
            MetadataBranch::Match(self.transition(Construction, Fresh {
                id,
            }))
        } else {
            let view = self.status.view.clone();
            MetadataBranch::Mismatch(self.transition(Comparison, Suspect {
                id,
                view,
            }))
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

#[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
pub(crate) enum ContentBranch {
    Match(TemplateProcessor<Refresh, StaleMetadata>),
    Mismatch(TemplateProcessor<Parsed, Stale>),
}

impl TemplateProcessor<Comparison, Suspect> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn check_content(
        self,
        source: &FileReader,
    ) -> Result<ContentBranch, TemplateReadError> {
        let content =
            source.read_to_string(self.file.path().as_ref()).map_err(|e| {
                TemplateReadError::Read(crate::fs::ReadError::Io {
                    path: self.file.path().as_ref().to_path_buf(),
                    source: std::io::Error::other(e.to_string()),
                })
            })?;
        let hash = Blake3Hash::compute(HashInput::Text(content.clone()));

        if self.status.view.content_hash().is_match(&hash) {
            let (file, path_key, status) = self.into_parts();
            Ok(ContentBranch::Match(Self::transition_from_parts(
                file,
                path_key,
                StaleMetadata {
                    id: status.id,
                    view: status.view,
                },
            )))
        } else {
            let view = self.status.view.clone();
            let id = self.status.id;
            Ok(ContentBranch::Mismatch(self.transition(Parsed, Stale {
                id,
                content_str: content,
                content_hash: hash,
                view,
            })))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Parsed Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Parsed;

impl TemplateProcessor<Parsed, Missing> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
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
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn parse(self) -> TemplateProcessor<Construction, Changed> {
        let (file, path_key, status) = self.into_parts();
        Self::transition_from_parts(file, path_key, Changed {
            id: status.id,
            content_hash: status.content_hash,
            raw: RawTemplate::new(status.content_str),
            view: status.view,
        })
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
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
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
    pub(crate) view: RawTemplateView,
}
#[derive(Debug)]
pub(crate) struct Fresh {
    pub(crate) id: TemplateId,
}

impl TemplateProcessor<Construction, New> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn create<R: WriteRepository>(
        self,
        repository: &R,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        let name =
            TemplateName::try_new(self.file.path().as_ref(), template_root)?;
        let template = Template::new(
            self.status.id,
            self.path_key.clone(),
            name,
            crate::template::aggregate::TemplateBody::try_new(
                self.status.raw.into_inner(),
            )?,
        );
        let view = RawTemplateView::new(
            self.path_key,
            self.status.content_hash,
            self.file.metadata().clone(),
            std::time::SystemTime::now(),
        );

        repository
            .save_template(&template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(&view)
            .map_err(TemplateError::Repository)?;

        Ok(template)
    }
}

impl TemplateProcessor<Construction, Changed> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn update<R: WriteRepository>(
        self,
        repository: &R,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        let name =
            TemplateName::try_new(self.file.path().as_ref(), template_root)?;
        let template = Template::new(
            self.status.id,
            self.path_key.clone(),
            name,
            crate::template::aggregate::TemplateBody::try_new(
                self.status.raw.into_inner(),
            )?,
        );
        let view = RawTemplateView::new(
            self.path_key,
            self.status.content_hash,
            self.file.metadata().clone(),
            std::time::SystemTime::now(),
        );

        repository
            .save_template(&template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(&view)
            .map_err(TemplateError::Repository)?;

        Ok(template)
    }
}

impl TemplateProcessor<Construction, Fresh> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn fetch<R: ReadRepository>(
        &self,
        repository: &R,
    ) -> Result<Template, TemplateError> {
        repository
            .find_template_by_path(&self.path_key)
            .map_err(TemplateError::Repository)?
            .ok_or_else(|| {
                TemplateError::Repository(
                    TemplateRepositoryError::NotFoundByPath(
                        self.path_key.clone(),
                    ),
                )
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug)]
pub(crate) struct Completed;

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc, time::SystemTime};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::{
        db::testing::{
            FailureInjector, FailurePoint as FPFake, InMemoryDbError,
            InMemoryHarness,
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

        pub fn valid_path_key(path: &str) -> PathKey {
            PathKey::try_new(path).expect("valid path key")
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
        fn compare_returns_present_branch_when_view_and_id_exist_in_repository()
        {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let processor = TemplateProcessor::<Discovery, Discovered>::new(
                file.clone(),
                path_key.clone(),
            );

            let template = Template::new(
                TemplateId::new(),
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

            let view = RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::compute(HashInput::Text("content".to_owned())),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let branch = processor.compare(Some(*template.id()), Some(view));
            assert!(
                matches!(branch, DiscoveryBranch::Present(_)),
                "Expected Present branch when both ID and view exist in \
                 repository"
            );
        }

        #[test]
        fn compare_returns_missing_branch_when_view_absent_from_repository() {
            let (file, path_key, _temp) =
                fixtures::valid_file_node("templates/test.md", "content");
            let processor =
                TemplateProcessor::<Discovery, Discovered>::new(file, path_key);

            let branch = processor.compare(None, None);
            assert!(
                matches!(branch, DiscoveryBranch::Missing(_)),
                "Expected Missing branch when repository is empty"
            );
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
        use pretty_assertions::assert_eq;

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
                .expect("create template");
            assert_eq!(*template.id(), id);

            assert_eq!(
                *repo.find_template_by_path(&path_key).unwrap().unwrap().id(),
                id
            );
            assert!(repo.find_raw_template_view(&path_key).unwrap().is_some());
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
                    view: view.clone(),
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
            assert_eq!(*result.id(), id);
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
                    TemplateRepositoryError::NotFoundByPath(_)
                )
            ));
        }
    }
}
