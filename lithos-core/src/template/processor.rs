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
    pub(crate) fn compare<R: ReadRepository>(
        self,
        repository: &R,
    ) -> DiscoveryBranch {
        let id =
            repository.find_template_id_by_path(&self.path_key).ok().flatten();
        let view =
            repository.find_raw_template_view(&self.path_key).ok().flatten();

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
    pub(crate) fn create(
        self,
        template_root: &std::path::Path,
    ) -> Result<(Template, RawTemplateView), TemplateError> {
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
        Ok((template, view))
    }

    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn persist<R: WriteRepository>(
        repository: &R,
        template: &Template,
        view: &RawTemplateView,
    ) -> Result<(), TemplateError> {
        repository
            .save_template(template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(view)
            .map_err(TemplateError::Repository)?;
        Ok(())
    }
}

impl TemplateProcessor<Construction, Changed> {
    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn update(
        self,
        id: TemplateId,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        let name =
            TemplateName::try_new(self.file.path().as_ref(), template_root)?;
        Ok(Template::new(
            id,
            self.path_key,
            name,
            crate::template::aggregate::TemplateBody::try_new(
                self.status.raw.into_inner(),
            )?,
        ))
    }

    #[cfg_attr(test, allow(dead_code, reason = "test-only method"))]
    pub(crate) fn persist<R: WriteRepository>(
        repository: &R,
        template: &Template,
        view: &RawTemplateView,
    ) -> Result<(), TemplateError> {
        repository
            .save_template(template)
            .map_err(TemplateError::Repository)?;
        repository
            .save_raw_template_view(view)
            .map_err(TemplateError::Repository)?;
        Ok(())
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
pub(crate) struct Completed;

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::{
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
        pub fn create_test_file(
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
    }

    mod state {
        use super::{fixtures, *};

        #[test]
        fn test_discovery_compares_present_when_found() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");
            let processor = TemplateProcessor::<Discovery, Discovered>::new(
                file.clone(),
                path_key.clone(),
            );

            let repo = InMemoryRepository::new();
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
            repo.save_template(&template).unwrap();

            let view = crate::template::views::RawTemplateView::new(
                PathKey::try_new("templates/test.md").unwrap(),
                crate::support::content_hash::Blake3Hash::from_bytes(b"hash"),
                processor.file().metadata().clone(),
                SystemTime::now(),
            );
            repo.save_raw_template_view(&view).unwrap();

            let branch = processor.compare(&repo);
            assert!(matches!(branch, DiscoveryBranch::Present(_)));
        }
    }

    mod content {
        use super::{fixtures::*, *};

        #[test]
        fn test_check_content_returns_mismatch_when_hash_differs() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");
            let view = crate::template::views::RawTemplateView::new(
                path_key.clone(),
                crate::support::content_hash::Blake3Hash::from_bytes(
                    b"wrong-hash",
                ),
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
            let parent = file_path.parent().expect("parent");
            let file_reader = FileReader::new(parent);
            let branch = processor.check_content(&file_reader).unwrap();
            assert!(matches!(branch, ContentBranch::Mismatch(_)));
        }
    }

    mod parse {
        use super::{fixtures::*, *};

        #[test]
        fn test_parse_returns_new_when_file_is_missing_in_repo() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "new-content");
            let processor = TemplateProcessor::<Parsed, Missing> {
                file,
                path_key,
                status: Missing,
                _phase: PhantomData,
            };

            let file_path = processor.file().path().as_ref();
            let parent = file_path.parent().expect("parent");
            let file_reader = FileReader::new(parent);

            let result = processor.parse(&file_reader).unwrap();

            assert_ne!(result.status.id, TemplateId::default());
        }

        #[test]
        fn test_parse_returns_changed_when_file_is_stale_in_repo() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "new-content");
            let view = crate::template::views::RawTemplateView::new(
                path_key.clone(),
                crate::support::content_hash::Blake3Hash::from_bytes(
                    b"old-hash",
                ),
                file.metadata().clone(),
                SystemTime::now(),
            );

            let id = TemplateId::new();

            let processor = TemplateProcessor::<Parsed, Stale> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: Stale {
                    id,
                    content_str: "new-content".to_owned(),
                    content_hash:
                        crate::support::content_hash::Blake3Hash::from_bytes(
                            b"new-hash",
                        ),
                    view,
                },
                _phase: PhantomData,
            };

            let result = processor.parse();

            assert_eq!(result.status.id, id);
        }
    }

    mod persistence {
        use super::{fixtures::*, *};
        use crate::template::storage::testing::InMemoryRepository;

        #[test]
        fn test_persists_template_and_view_successfully() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");

            let processor = TemplateProcessor::<Construction, New> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: New {
                    id: TemplateId::new(),
                    content_hash: Blake3Hash::from_bytes(b"hash"),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let (template, view) =
                processor.create(&std::path::PathBuf::from("/")).unwrap();

            let repo = InMemoryRepository::new();
            TemplateProcessor::<Construction, New>::persist(
                &repo, &template, &view,
            )
            .unwrap();

            assert!(repo.find_template_by_path(&path_key).unwrap().is_some());
            assert!(repo.find_raw_template_view(&path_key).unwrap().is_some());
        }
    }

    mod lookup {
        use super::{fixtures::*, *};
        use crate::template::storage::testing::InMemoryRepository;

        #[test]
        fn test_fetch_returns_template_when_found_in_repository() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");
            let processor = TemplateProcessor::<Construction, Fresh> {
                file,
                path_key: path_key.clone(),
                status: Fresh {
                    id: TemplateId::new(),
                },
                _phase: PhantomData,
            };

            let template = Template::new(
                TemplateId::new(),
                path_key.clone(),
                TemplateName::try_new(
                    processor.file().path().as_ref(),
                    std::path::Path::new("/"),
                )
                .unwrap(),
                crate::template::aggregate::TemplateBody::try_new(
                    "content".to_owned(),
                )
                .unwrap(),
            );

            let repo = InMemoryRepository::new();
            repo.save_template(&template).unwrap();

            let result = processor.fetch(&repo).unwrap();

            assert_eq!(result.path(), &path_key);
        }
    }
}
