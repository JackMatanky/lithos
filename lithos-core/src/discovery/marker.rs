//! Root marker contract: the typed output of vault root resolution.
//!
//! [`FoundRootMarker`] carries the path and format of the marker file found
//! during ascending vault root discovery. It is the handoff type from
//! [`crate::discovery::engine`] to the Config context.

use std::path::PathBuf;

use crate::fs::format::StructuredFileFormat;

/// A root marker file found during vault root resolution.
///
/// Carries the canonicalized path to the marker file (e.g. `lithos.toml`) and
/// the base directory it was found in. Does not include Config location
/// taxonomy; that classification is Config-owned.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FoundRootMarker {
    /// Base directory the marker was found in (the vault root candidate).
    pub(crate) base: PathBuf,
    /// Absolute canonicalized path to the marker file.
    pub(crate) path: PathBuf,
    /// The detected structured format of the marker file.
    pub(crate) format: StructuredFileFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod equality {
        use super::*;

        #[test]
        fn returns_true_when_markers_have_same_path_and_format() {
            let a = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(a, b);
        }

        #[test]
        fn returns_false_when_marker_paths_differ() {
            let a = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/.lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_ne!(a, b);
        }

        #[test]
        fn returns_false_when_marker_formats_differ() {
            let a = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            let b = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Json,
            };
            assert_ne!(a, b);
        }
    }
}
