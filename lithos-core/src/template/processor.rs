//! Template ingestion pipeline.
//!
//! Implements a dual-typestate processor for discovering, comparing, and
//! constructing template aggregates from filesystem entries.
#![allow(dead_code, reason = "Unused during feature development")]
#![allow(unused_imports, reason = "Unused during feature development")]

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

// ============================================================================
// Typestate Markers (Visibility: private)
// ============================================================================

#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Discovery;
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Comparison;
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Parsed;
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Refresh;
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Construction;
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct StaleMetadata {
    pub(crate) view: RawTemplateView,
}
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct New {
    pub(crate) content_hash: Blake3Hash,
    pub(crate) raw: RawTemplate,
}
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Changed {
    pub(crate) content_hash: Blake3Hash,
    pub(crate) raw: RawTemplate,
    pub(crate) view: RawTemplateView,
}
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Stale {
    pub(crate) content_str: String,
    pub(crate) content_hash: Blake3Hash,
    pub(crate) view: RawTemplateView,
}
#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Completed;

#[allow(dead_code, reason = "Used in typestate pattern")]
pub(crate) struct Discovered;

#[allow(dead_code, reason = "Used in typestate pattern")]
#[derive(Debug)]
pub(crate) struct Missing;

#[allow(dead_code, reason = "Used in typestate pattern")]
#[derive(Debug)]
pub(crate) struct Present {
    pub(crate) view: RawTemplateView,
}

#[allow(dead_code, reason = "Used in typestate pattern")]
#[derive(Debug)]
pub(crate) struct Fresh;

#[allow(dead_code, reason = "Used in typestate pattern")]
#[derive(Debug)]
pub(crate) struct Suspect {
    pub(crate) view: RawTemplateView,
}

// ============================================================================
// Processor Struct
// ============================================================================

#[derive(Debug)]
pub(crate) struct TemplateProcessor<Phase, Status> {
    file: FileNode,
    path_key: PathKey,
    status: Status,
    _phase: PhantomData<Phase>,
}

impl<Phase, Status> TemplateProcessor<Phase, Status> {
    fn transition<NP, NS>(
        self,
        _phase: NP,
        status: NS,
    ) -> TemplateProcessor<NP, NS> {
        TemplateProcessor {
            file: self.file,
            path_key: self.path_key,
            status,
            _phase: PhantomData,
        }
    }

    pub fn file(&self) -> &FileNode {
        &self.file
    }

    pub fn path_key(&self) -> &PathKey {
        &self.path_key
    }
}

// ============================================================================
// Implementation
// ============================================================================

enum DiscoveryBranch {
    Missing(TemplateProcessor<Parsed, Missing>),
    Present(TemplateProcessor<Comparison, Present>),
}

impl TemplateProcessor<Discovery, Discovered> {
    pub(crate) fn new(file: FileNode, path_key: PathKey) -> Self {
        Self {
            file,
            path_key,
            status: Discovered,
            _phase: PhantomData,
        }
    }

    /// Comparison stage transition
    fn compare<R: ReadRepository>(self, repository: &R) -> DiscoveryBranch {
        let view =
            repository.find_raw_template_view(&self.path_key).ok().flatten();

        match view {
            None => DiscoveryBranch::Missing(self.transition(Parsed, Missing)),
            Some(view) => {
                DiscoveryBranch::Present(self.transition(Comparison, Present {
                    view,
                }))
            }
        }
    }
}

enum MetadataBranch {
    Match(TemplateProcessor<Construction, Fresh>),
    Mismatch(TemplateProcessor<Comparison, Suspect>),
}

impl TemplateProcessor<Comparison, Present> {
    fn view(&self) -> &RawTemplateView {
        &self.status.view
    }

    /// Classifies template against cached view.
    fn check_metadata(self) -> MetadataBranch {
        let f = self.file.metadata();
        let is_size_match = f.is_size_match(self.status.view.metadata().size());
        let is_timestamp_match = f.is_timestamp_match(
            self.status.view.metadata().times().created_at(),
            self.status.view.metadata().times().modified_at(),
        );

        if is_size_match && is_timestamp_match {
            MetadataBranch::Match(self.transition(Construction, Fresh))
        } else {
            let view = self.status.view.clone();
            MetadataBranch::Mismatch(self.transition(Comparison, Suspect {
                view,
            }))
        }
    }
}

enum ContentBranch {
    Match(TemplateProcessor<Construction, Completed>),
    Mismatch(Box<TemplateProcessor<Parsed, Stale>>),
}

impl TemplateProcessor<Comparison, Suspect> {
    fn check_content(
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
            Ok(ContentBranch::Match(self.transition(Construction, Completed)))
        } else {
            let view = self.status.view.clone();
            Ok(ContentBranch::Mismatch(Box::new(self.transition(
                Parsed,
                Stale {
                    content_str: content,
                    content_hash: hash,
                    view,
                },
            ))))
        }
    }
}

impl TemplateProcessor<Parsed, Missing> {
    fn parse(
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
            content_hash: hash,
            raw: RawTemplate::new(content),
        }))
    }
}

impl TemplateProcessor<Parsed, Stale> {
    fn parse(self) -> TemplateProcessor<Construction, Changed> {
        TemplateProcessor {
            file: self.file,
            path_key: self.path_key,
            status: Changed {
                content_hash: self.status.content_hash,
                raw: RawTemplate::new(self.status.content_str),
                view: self.status.view,
            },
            _phase: PhantomData,
        }
    }
}

impl TemplateProcessor<Refresh, StaleMetadata> {
    fn sync_metadata(
        self,
        _view: RawTemplateView,
    ) -> TemplateProcessor<Construction, Fresh> {
        self.transition(Construction, Fresh)
    }
}

impl TemplateProcessor<Construction, New> {
    fn create(
        self,
        template_root: &std::path::Path,
    ) -> Result<Template, TemplateError> {
        let name =
            TemplateName::try_new(self.file.path().as_ref(), template_root)?;
        Ok(Template::new(
            TemplateId::new(),
            self.path_key,
            name,
            crate::template::aggregate::TemplateBody::try_new(
                self.status.raw.into_inner(),
            )?,
        ))
    }

    fn persist(
        repository: &dyn WriteRepository,
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
    fn update(
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

    fn persist(
        repository: &dyn WriteRepository,
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
    fn fetch(
        &self,
        repository: &dyn ReadRepository,
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
        fn returns_present_branch_when_template_found() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");
            let processor =
                TemplateProcessor::<Discovery, Discovered>::new(file, path_key);

            let repo = InMemoryRepository::new();
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
        fn returns_mismatch_when_hash_differs() {
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
        fn returns_new_when_file_is_missing_in_repo() {
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

            assert!(matches!(result.status, New { .. }));
        }

        #[test]
        fn returns_changed_when_file_is_stale_in_repo() {
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

            let processor = TemplateProcessor::<Parsed, Stale> {
                file,
                path_key,
                status: Stale {
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

            assert!(matches!(result.status, Changed { .. }));
        }
    }

    mod persistence {
        use super::{fixtures::*, *};
        use crate::template::storage::testing::InMemoryRepository;

        #[test]
        fn persists_template_and_view_successfully() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");

            let processor = TemplateProcessor::<Construction, New> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: New {
                    content_hash: Blake3Hash::from_bytes(b"hash"),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let _processor_for_persist = TemplateProcessor::<Construction, New> {
                file: file.clone(),
                path_key: path_key.clone(),
                status: New {
                    content_hash: Blake3Hash::from_bytes(b"hash"),
                    raw: RawTemplate::new("content".to_owned()),
                },
                _phase: PhantomData,
            };

            let template =
                processor.create(&std::path::PathBuf::from("/")).unwrap();
            let view = crate::template::views::RawTemplateView::new(
                path_key.clone(),
                Blake3Hash::from_bytes(b"hash"),
                file.metadata().clone(),
                SystemTime::now(),
            );

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
        fn returns_template_when_found_in_repository() {
            let (file, path_key, _temp) =
                fixtures::create_test_file("templates/test.md", "content");
            let processor = TemplateProcessor::<Construction, Fresh> {
                file,
                path_key: path_key.clone(),
                status: Fresh,
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
