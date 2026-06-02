//! Root marker contract: the typed output of vault root resolution.
//!
//! [`FoundRootMarker`] carries the path and format of the marker file found
//! during ascending vault root discovery. It is the handoff type from
//! [`crate::discovery::resolver`] to the Config context.

use std::path::PathBuf;

use crate::fs::format::StructuredFileFormat;

/// A root marker file found during vault root resolution.
///
/// Carries the canonicalized path to the marker file (e.g. `lithos.toml`) and
/// the base directory it was found in. Does not include Config location
/// taxonomy; that classification is Config-owned.
#[allow(
    dead_code,
    reason = "Phase-1 contracts are defined before full pipeline integration"
)]
#[derive(Debug, PartialEq, Eq)]
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

    mod constructor {
        use super::*;

        #[test]
        fn returns_base_path_for_found_root_marker() {
            let marker = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(marker.base, PathBuf::from("/vault"));
        }

        #[test]
        fn returns_path_for_found_root_marker() {
            let marker = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(marker.path, PathBuf::from("/vault/lithos.toml"));
        }

        #[test]
        fn returns_format_for_found_root_marker() {
            let marker = FoundRootMarker {
                base: PathBuf::from("/vault"),
                path: PathBuf::from("/vault/lithos.toml"),
                format: StructuredFileFormat::Toml,
            };
            assert_eq!(marker.format, StructuredFileFormat::Toml);
        }
    }
}
