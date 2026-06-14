//! Inbound port for the Discovery domain.
//!
//! [`DiscoveryPort`] is the boundary trait that separates bootstrap
//! orchestration from the Discovery implementation. The application layer
//! calls this trait; the Discovery domain provides the implementation.
//! This allows the bootstrap layer to be tested with a mock without
//! running the real filesystem traversal.
use crate::discovery::{
    context::DiscoveryContext, error::DiscoveryError, service::DiscoveryResult,
};
/// Inbound port for vault and global config candidate discovery.
///
/// The application layer depends on this trait rather than on
/// [`DiscoveryEngine`] directly, keeping orchestration testable and
/// decoupled from the discovery implementation.
///
/// [`DiscoveryEngine`]: crate::discovery::engine::DiscoveryEngine
pub(crate) trait DiscoveryPort {
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
    ) -> Result<DiscoveryResult, DiscoveryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        discovery::service::CandidatePath,
        fs::{DirPath, FilePath, PathError},
    };

    // A minimal mock that returns a fixed DiscoveryResult.
    struct MockDiscoveryPort {
        result: Result<DiscoveryResult, DiscoveryError>,
    }

    impl MockDiscoveryPort {
        fn returning(result: Result<DiscoveryResult, DiscoveryError>) -> Self {
            Self {
                result,
            }
        }
    }

    impl DiscoveryPort for MockDiscoveryPort {
        fn discover(
            &self,
            _context: &DiscoveryContext<'_>,
        ) -> Result<DiscoveryResult, DiscoveryError> {
            // Clone the result for each call.
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(clone_error(e)),
            }
        }
    }

    /// Helper: reconstruct a `DiscoveryError` for testing since it is not
    /// `Clone`. We use the simplest variant that requires no filesystem state.
    fn clone_error(e: &DiscoveryError) -> DiscoveryError {
        // Reconstruct by re-parsing the display string as
        // InvalidAnchorDirectory is not available without a real path,
        // so we use a known-failing path.
        let _ = e; // silence unused warning
        DiscoveryError::InvalidAnchorDirectory {
            path: std::path::PathBuf::from("/nonexistent"),
            source: PathError::NotADirectory(std::path::PathBuf::from(
                "/nonexistent",
            )),
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

    mod discovery_port {
        use super::*;

        mod mock_returns_success {
            use super::*;

            #[test]
            fn returns_discovery_result_from_mock() {
                let root = tempfile::tempdir().expect("root");
                let candidate = make_candidate(&root, "lithos.toml");
                let expected = DiscoveryResult::new(vec![candidate], vec![]);
                let port = MockDiscoveryPort::returning(Ok(expected.clone()));
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let result =
                    port.discover(&ctx).expect("discover should succeed");

                assert_eq!(
                    result, expected,
                    "mock should return the fixed DiscoveryResult"
                );
            }

            #[test]
            fn returns_empty_result_when_mock_returns_empty() {
                let empty = DiscoveryResult::new(vec![], vec![]);
                let port = MockDiscoveryPort::returning(Ok(
                    DiscoveryResult::new(vec![], vec![]),
                ));
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let result =
                    port.discover(&ctx).expect("discover should succeed");

                assert_eq!(
                    result, empty,
                    "mock should return an empty DiscoveryResult"
                );
            }
        }

        mod mock_returns_error {
            use super::*;

            #[test]
            fn returns_error_from_mock() {
                let port = MockDiscoveryPort::returning(Err(
                    DiscoveryError::InvalidAnchorDirectory {
                        path: std::path::PathBuf::from("/bad"),
                        source: PathError::NotADirectory(
                            std::path::PathBuf::from("/bad"),
                        ),
                    },
                ));
                let anchor = tempfile::tempdir().expect("anchor");
                let ctx = DiscoveryContext::new(anchor.path())
                    .expect("valid context");

                let err =
                    port.discover(&ctx).expect_err("discover should fail");

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
