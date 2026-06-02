//! Typed processing pipeline for building or fetching a `BaseSchema`.
//!
//! # Purpose
//!
//! This module implements a typestate pipeline with two paths:
//!
//! - **Missing path**: When no cached view exists, construct a new `BaseSchema`
//!   and persist it.
//! - **Present path**: When a view exists, check timestamps for freshness; if
//!   fresh, fetch the cached `BaseSchema`.

#![expect(
    dead_code,
    reason = "Types used internally and in tests; Builder integration \
              deferred to Phase 3"
)]

use std::marker::PhantomData;

use crate::{
    fs::{DirPath, FileReader, FsFile, PathKey},
    schema::{
        base::BaseSchema,
        error::{
            SchemaFileError, SchemaIngestionError, SchemaLoaderError,
            SchemaRepositoryError,
        },
        identifier::{SchemaId, SchemaName},
        property::{PropertyMap, PropertyName},
        repository::{Repository, WriteRepository},
        views::{RawViewRead, raw::RawSchemaView},
    },
};

// ─────────────────────────────────────────────────────────────────────────────
//  Processor Core
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
#[must_use]
pub(crate) struct BaseSchemaProcessor<P, S> {
    file: FsFile,
    path_key: PathKey,
    status: S,
    _stage: PhantomData<P>,
}

impl<P, S> BaseSchemaProcessor<P, S> {
    #[inline]
    fn into_parts(self) -> (FsFile, PathKey, S) {
        (self.file, self.path_key, self.status)
    }

    #[inline]
    fn transition_from_parts<NP, NS>(
        file: FsFile,
        path_key: PathKey,
        status: NS,
    ) -> BaseSchemaProcessor<NP, NS> {
        BaseSchemaProcessor {
            file,
            path_key,
            status,
            _stage: PhantomData,
        }
    }

    #[inline]
    fn transition<NP, NS>(
        self,
        _stage: NP,
        status: NS,
    ) -> BaseSchemaProcessor<NP, NS> {
        let (file, path_key, _) = self.into_parts();
        Self::transition_from_parts(file, path_key, status)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Resolution
// ─────────────────────────────────────────────────────────────────────────────

/// The result of a base schema resolution attempt.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum BaseSchemaResolution {
    /// Base schema was already fresh in the repository.
    Fresh {
        /// The schema identifier.
        schema_id: SchemaId,
        /// The cached base schema.
        base_schema: BaseSchema,
    },
    /// Base schema was newly constructed.
    New {
        /// The newly constructed base schema.
        base_schema: BaseSchema,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
//  Entry Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Initial state before any knowledge has been gathered.
#[derive(Debug)]
pub(crate) struct Unknown;

/// Entry-point stage: processor created from discovery data.
#[derive(Debug)]
pub(crate) struct Init;

/// Entry-state operations that bootstrap the pipeline.
impl BaseSchemaProcessor<Init, Unknown> {
    #[inline]
    pub(crate) fn from_discovery(
        file: FsFile,
        root: &DirPath,
    ) -> Result<Self, crate::fs::PathError> {
        let path_key = file.path().as_key(root)?;
        Ok(Self {
            file,
            path_key,
            status: Unknown,
            _stage: PhantomData,
        })
    }

    /// Derive a `SchemaName` from the file's basename.
    fn schema_name_from_path(
        file: &FsFile,
    ) -> Result<SchemaName, SchemaLoaderError> {
        let basename = file.path().basename().ok_or_else(|| {
            SchemaLoaderError::Ingestion(SchemaIngestionError::File(
                SchemaFileError::InvalidFileName {
                    path: file.path().as_path().to_path_buf(),
                    reason: "missing file stem".into(),
                },
            ))
        })?;
        Ok(SchemaName::try_new(basename.as_str())?)
    }

    /// Run the full base schema pipeline.
    ///
    /// When `view` is `None`, the processor runs the missing path
    /// (construct from defaults → persist).
    /// When `view` is `Some(...)`, the processor runs the present path
    /// (check timestamps → fetch if fresh).
    pub(crate) fn run<R: Repository>(
        self,
        view: Option<&RawSchemaView>,
        _source: &FileReader,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        if let Some(view) = view {
            self.run_present(view, repository)
        } else {
            self.run_missing(repository)
        }
    }

    /// Internal helper for the missing path (no cached view exists).
    fn run_missing<R: Repository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        let (file, path_key, _status) = self.into_parts();
        let schema_name = Self::schema_name_from_path(&file)?;
        let constructed = Self::transition_from_parts(file, path_key, New {
            id: SchemaId::new(),
            schema_name,
            properties: PropertyMap::new(),
            extends: Vec::new(),
            excludes: Vec::new(),
        });
        let completed = constructed.create(repository)?;
        Ok(BaseSchemaResolution::New {
            base_schema: completed.into_base(),
        })
    }

    /// Internal helper for the present path (cached view exists).
    fn run_present<R: Repository>(
        self,
        view: &RawSchemaView,
        repository: &R,
    ) -> Result<BaseSchemaResolution, SchemaLoaderError> {
        let timestamps_match = view.is_timestamp_match(
            self.file.metadata().times().created_at(),
            self.file.metadata().times().modified_at(),
        );

        if timestamps_match {
            let schema_id = repository
                .find_schema_id_by_path(view.path())?
                .ok_or_else(|| {
                    SchemaLoaderError::Repository(
                        SchemaRepositoryError::NotFoundByPath(
                            view.path().clone(),
                        ),
                    )
                })?;

            let base_schema = repository
                .find_base_schema_by_id(schema_id)?
                .ok_or_else(|| {
                    SchemaLoaderError::Repository(
                        SchemaRepositoryError::NotFoundById(schema_id),
                    )
                })?;

            Ok(BaseSchemaResolution::Fresh {
                schema_id,
                base_schema,
            })
        } else {
            let (file, path_key, _status) = self.into_parts();
            let schema_name = Self::schema_name_from_path(&file)?;
            let constructed =
                Self::transition_from_parts(file, path_key, New {
                    id: SchemaId::new(),
                    schema_name,
                    properties: PropertyMap::new(),
                    extends: Vec::new(),
                    excludes: Vec::new(),
                });
            let completed = constructed.create(repository)?;
            Ok(BaseSchemaResolution::New {
                base_schema: completed.into_base(),
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Construction Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Building phase: terminal domain construction.
///
/// Activated after the entry stage resolves into either a missing (`New`) or
/// present (`Fresh`) path. Constructing the `BaseSchema` requires a
/// `WriteRepository`.
#[derive(Debug)]
struct Construction;

/// Raw construction inputs for building a new `BaseSchema` from scratch.
///
/// Entered from the missing-view path or stale-timestamp path when no cached
/// view exists or timestamps have drifted. Produces a `Completed<NewReady>`.
#[derive(Debug)]
struct New {
    id: SchemaId,
    schema_name: SchemaName,
    properties: PropertyMap,
    extends: Vec<SchemaName>,
    excludes: Vec<PropertyName>,
}

/// Proven: identity is fully synchronized; schema can be fetched without
/// rebuild.
///
/// Entered from the fresh-timestamp path when a cached view exists and
/// timestamps match. Currently unused until 04 activates the stale-analysis
/// pipeline which transitions into `Construction<Fresh>` for the fetch.
/// See `.scratch/base-schema/
/// 04-base-processor-stale-analysis-and-normalization.md`.
#[derive(Debug)]
struct Fresh {
    id: SchemaId,
    base: BaseSchema,
}

/// Construction operations that build the base schema.
impl BaseSchemaProcessor<Construction, New> {
    #[inline]
    fn create<R: WriteRepository>(
        self,
        repository: &R,
    ) -> Result<BaseSchemaProcessor<Completed, NewReady>, SchemaLoaderError>
    {
        let (file, path_key, status) = self.into_parts();

        let base = BaseSchema::new(
            status.id,
            status.schema_name,
            status.properties,
            status.extends,
            status.excludes,
        );

        repository
            .save_base_schema(&base)
            .map_err(SchemaLoaderError::Repository)?;

        Ok(Self::transition_from_parts(file, path_key, NewReady {
            base,
        }))
    }
}

/// Construction operations that fetch the cached base schema.
///
/// Dead code until 04 activates the stale-analysis pipeline, which
/// transitions through the `Fresh` status after verifying timestamps match.
/// See `.scratch/base-schema/
/// 04-base-processor-stale-analysis-and-normalization.md`.
impl BaseSchemaProcessor<Construction, Fresh> {
    #[inline]
    fn fetch(self) -> BaseSchemaProcessor<Completed, FreshReady> {
        let (file, path_key, status) = self.into_parts();

        Self::transition_from_parts(file, path_key, FreshReady {
            id: status.id,
            base: status.base,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Completed Stage
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal phase: the `BaseSchema` is ready and owned.
#[derive(Debug)]
struct Completed;

/// Proven: terminal ingestion goal reached with newly built schema.
#[derive(Debug)]
struct NewReady {
    base: BaseSchema,
}

/// Proven: terminal ingestion goal reached with freshly fetched schema.
#[derive(Debug)]
struct FreshReady {
    id: SchemaId,
    base: BaseSchema,
}

/// Completed operations that expose the final base schema.
impl BaseSchemaProcessor<Completed, NewReady> {
    #[inline]
    #[must_use]
    fn into_base(self) -> BaseSchema {
        self.status.base
    }
}

impl BaseSchemaProcessor<Completed, FreshReady> {
    #[inline]
    #[must_use]
    fn into_base(self) -> BaseSchema {
        self.status.base
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{
        fs::{DirPath, FsFile},
        schema::{
            repository::ReadRepository, storage::testing::InMemoryRepository,
            views::RawView,
        },
    };

    /// Helper to create a [`RawSchemaView`] with the fixture file's timestamps.
    fn matching_view(fixture: &fixtures::Fixture) -> RawSchemaView {
        let metadata = fixture.file.metadata().clone();
        let raw = crate::schema::raw::RawSchema {
            version: crate::schema::raw::RawSchemaVersion::default(),
            name: "test-schema".into(),
            extends: vec![],
            excludes: vec![],
            properties: crate::schema::raw::property::RawPropertyMap::from_map(
                std::collections::HashMap::new(),
            ),
            metadata: metadata.clone(),
        };
        let hash = crate::support::content_hash::Blake3Hash::new([0; 32]);
        let hashes = crate::schema::views::hashes::HashRecord::new(
            hash,
            crate::schema::views::hashes::RawPropertyHashIndex::default(),
        );
        let mut view = RawSchemaView::try_from_raw_with_hashes(
            &raw,
            fixture.key.clone(),
            hashes,
        )
        .expect("view");
        view.update_metadata(fixture.file.metadata().clone());
        view
    }

    mod fixtures {
        use super::*;

        pub(super) struct Fixture {
            pub(super) repository: InMemoryRepository,
            pub(super) source: FileReader,
            pub(super) vault_root: DirPath,
            pub(super) _vault_dir: TempDir,
            pub(super) file: FsFile,
            pub(super) key: PathKey,
        }

        pub(super) fn make_fixture() -> Fixture {
            let vault_dir = TempDir::new().expect("temp dir");
            let vault_root = DirPath::try_new(vault_dir.path().to_path_buf())
                .expect("vault root");
            let relative = std::path::PathBuf::from("schemas/test-schema.yaml");
            let absolute = vault_dir.path().join(&relative);
            std::fs::create_dir_all(absolute.parent().expect("parent"))
                .expect("mkdir");
            std::fs::write(&absolute, "name: test-schema\nproperties: {}")
                .expect("write file");

            let source = FileReader::new(vault_dir.path());
            let file_path = crate::fs::FilePath::try_new(absolute.clone())
                .expect("file path");
            let metadata =
                crate::fs::metadata::FsMetadata::from_path(file_path.as_path())
                    .expect("metadata")
                    .as_file()
                    .cloned()
                    .expect("file metadata");
            let file = FsFile::new(file_path.clone(), metadata.clone());
            let key = file.path().as_key(&vault_root).expect("path key");

            Fixture {
                repository: InMemoryRepository::new(),
                source,
                vault_root,
                _vault_dir: vault_dir,
                file,
                key,
            }
        }
    }

    mod constructor {
        use super::*;

        #[test]
        fn from_discovery_returns_processor_with_unknown() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            assert!(matches!(processor.status, Unknown));
        }
    }

    mod run {
        use super::*;

        macro_rules! expect_new {
            ($resolution:expr) => {{
                let resolution = $resolution;
                let BaseSchemaResolution::New {
                    base_schema,
                } = resolution
                else {
                    panic!("Expected New resolution");
                };
                base_schema
            }};
        }

        macro_rules! expect_fresh {
            ($resolution:expr) => {{
                let resolution = $resolution;
                let BaseSchemaResolution::Fresh {
                    schema_id,
                    base_schema,
                } = resolution
                else {
                    panic!("Expected Fresh resolution");
                };
                (schema_id, base_schema)
            }};
        }

        #[test]
        fn missing_constructs_and_persists_base_schema() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            assert!(matches!(resolution, BaseSchemaResolution::New { .. }));
        }

        #[test]
        fn new_resolution_derives_name_from_file_basename() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert_eq!(base_schema.name().as_str(), "test-schema");
        }

        #[test]
        fn new_resolution_constructs_with_empty_defaults() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);
            assert!(base_schema.properties().is_empty());
            assert!(base_schema.extends().is_empty());
            assert!(base_schema.excludes().is_empty());
        }

        #[test]
        fn present_returns_fresh_when_timestamps_match() {
            let fixture = fixtures::make_fixture();

            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            fixture.repository.save_base_schema(&base).expect("save base");

            let view = matching_view(&fixture);
            fixture
                .repository
                .save_raw_schema_view(schema_id, &view)
                .expect("save view");

            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");

            let (sid, bs) = expect_fresh!(resolution);
            assert_eq!(sid, schema_id);
            assert_eq!(bs.name().as_str(), "test-schema");
        }

        #[test]
        fn present_does_not_write_when_fresh() {
            let fixture = fixtures::make_fixture();

            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            fixture.repository.save_base_schema(&base).expect("save base");

            let view = matching_view(&fixture);
            fixture
                .repository
                .save_raw_schema_view(schema_id, &view)
                .expect("save view");

            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(Some(&view), &fixture.source, &fixture.repository)
                .expect("run");

            let (_sid, bs) = expect_fresh!(resolution);
            assert_eq!(bs, base);
        }

        #[test]
        fn missing_persisted_base_schema_is_retrievable() {
            let fixture = fixtures::make_fixture();
            let processor =
                BaseSchemaProcessor::<Init, Unknown>::from_discovery(
                    fixture.file,
                    &fixture.vault_root,
                )
                .expect("processor");

            let resolution = processor
                .run(None, &fixture.source, &fixture.repository)
                .expect("run");

            let base_schema = expect_new!(resolution);

            let found = fixture
                .repository
                .find_base_schema_by_id(*base_schema.id())
                .expect("find base")
                .expect("base schema should exist");

            assert_eq!(found, base_schema);
        }
    }

    mod terminal {
        use super::*;

        #[test]
        fn new_ready_into_base_returns_constructed_schema() {
            let fixture = fixtures::make_fixture();
            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            let expected = base.clone();

            let processor = BaseSchemaProcessor::<Completed, NewReady> {
                file: fixture.file,
                path_key: fixture.key,
                status: NewReady {
                    base: expected.clone(),
                },
                _stage: PhantomData,
            };

            let result = processor.into_base();
            assert_eq!(result, expected);
        }

        #[test]
        fn fresh_ready_into_base_returns_fetched_schema() {
            let fixture = fixtures::make_fixture();
            let schema_id = crate::schema::identifier::SchemaId::new();
            let schema_name =
                crate::schema::identifier::SchemaName::try_new("test-schema")
                    .expect("name");
            let base = BaseSchema::new(
                schema_id,
                schema_name,
                PropertyMap::new(),
                Vec::new(),
                Vec::new(),
            );
            let expected = base.clone();

            let processor = BaseSchemaProcessor::<Completed, FreshReady> {
                file: fixture.file,
                path_key: fixture.key,
                status: FreshReady {
                    id: schema_id,
                    base: expected.clone(),
                },
                _stage: PhantomData,
            };

            let result = processor.into_base();
            assert_eq!(result, expected);
        }
    }
}
