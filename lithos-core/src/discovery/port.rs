//! Inbound port for the Discovery domain.
//!
//! [`DiscoveryPort`] is the boundary trait that separates bootstrap
//! orchestration from the Discovery implementation. The application layer
//! calls this trait; the Discovery domain provides the implementation.
//! This allows the bootstrap layer to be tested with a mock without
//! running the real filesystem traversal.
use crate::discovery::{
    context::DiscoveryContext, error::DiscoveryError, report::DiscoveryReport,
    service::DiscoveryResult,
};

/// Inbound port for vault and global config candidate discovery.
///
/// The application layer depends on this trait rather than on
/// [`DiscoveryEngine`] directly, keeping orchestration testable and
/// decoupled from the discovery implementation.
///
/// [`DiscoveryEngine`]: crate::discovery::engine::DiscoveryEngine
pub trait DiscoveryPort {
    /// Runs discovery using the provided invocation context.
    ///
    /// # Errors
    ///
    /// Returns [`DiscoveryError`] if the anchor directory is invalid, the
    /// ascending traversal fails, or a global config directory cannot be
    /// read.
    fn discover(
        &self,
        context: &DiscoveryContext<'_>,
    ) -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>;
}

#[cfg(test)]
mod tests {
    use mockall::{mock, predicate::always};

    use super::*;
    use crate::{
        discovery::{
            report::{DiscoveryReport, LocalTraversalStopReason},
            service::CandidatePath,
        },
        fs::{DirPath, FilePath, PathError},
    };

    // Generate a mock using mock! so the production trait keeps `'_`.
    mock! {
        pub(crate) DiscoveryPort {}
        impl DiscoveryPort for DiscoveryPort {
            fn discover<'ctx>(
                &self,
                context: &DiscoveryContext<'ctx>,
            ) -> Result<(DiscoveryResult, DiscoveryReport), DiscoveryError>;
        }
    }

    fn make_candidate(root: &tempfile::TempDir, name: &str) -> CandidatePath {
        let path = root.path().join(name);
        std::fs::write(&path, "").expect("write candidate");
        CandidatePath::new(
            DirPath::try_new(root.path().to_path_buf())
                .expect("valid base dir"),
            FilePath::try_new(path).expect("valid file"),
        )
    }

    fn placeholder_cache_root() -> crate::discovery::location::CacheRoot {
        use crate::discovery::location::{CacheLocation, GlobalCacheLocation};
        crate::discovery::location::CacheRoot {
            location: CacheLocation::Global(
                GlobalCacheLocation::PlatformUserCache,
            ),
            path: std::path::PathBuf::from("/tmp/placeholder-cache"),
        }
    }

    mod discovery_port {
        use super::*;

        mod mock_returns_success {
            use super::*;

            #[test]
            fn returns_discovery_result_from_mock() {
                let root = tempfile::tempdir().expect("root");
                let candidate = make_candidate(&root, "lithos.toml");
                let expected = DiscoveryResult::new(
                    vec![candidate],
                    vec![],
                    placeholder_cache_root(),
                );
                let report = DiscoveryReport {
                    skipped_ceilings: vec![],
                    local_traversal_stop_reason:
                        LocalTraversalStopReason::FilesystemRoot,
                    global_resolution_skip_reason: None,
                };
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let mut mock = MockDiscoveryPort::new();
                let ret = expected.clone();
                let rep = report.clone();
                mock.expect_discover()
                    .with(always())
                    .once()
                    .returning(move |_| Ok((ret.clone(), rep.clone())));

                let (result, _) =
                    mock.discover(&ctx).expect("discover should succeed");

                assert_eq!(result, expected);
            }

            #[test]
            fn returns_empty_result_when_mock_returns_empty() {
                let empty = DiscoveryResult::new(
                    vec![],
                    vec![],
                    placeholder_cache_root(),
                );
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let mut mock = MockDiscoveryPort::new();
                mock.expect_discover().with(always()).once().returning(
                    move |_| {
                        Ok((
                            DiscoveryResult::new(
                                vec![],
                                vec![],
                                placeholder_cache_root(),
                            ),
                            DiscoveryReport {
                                skipped_ceilings: vec![],
                                local_traversal_stop_reason:
                                    LocalTraversalStopReason::FilesystemRoot,
                                global_resolution_skip_reason: None,
                            },
                        ))
                    },
                );

                let (result, _) =
                    mock.discover(&ctx).expect("discover should succeed");

                assert_eq!(result, empty);
            }
        }

        mod mock_returns_error {
            use super::*;

            #[test]
            fn returns_error_from_mock() {
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let mut mock = MockDiscoveryPort::new();
                mock.expect_discover().with(always()).once().returning(|_| {
                    Err(DiscoveryError::InvalidAnchorDirectory {
                        path: std::path::PathBuf::from("/bad"),
                        source: PathError::NotADirectory(
                            std::path::PathBuf::from("/bad"),
                        ),
                    })
                });

                let err =
                    mock.discover(&ctx).expect_err("discover should fail");

                assert!(
                    matches!(
                        err,
                        DiscoveryError::InvalidAnchorDirectory { .. }
                    ),
                    "expected InvalidAnchorDirectory, got: {err:?}"
                );
            }
        }
    }
}
