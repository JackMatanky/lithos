//! Logic for selecting a single winning marker from multiple discovery
//! candidates.
//!
//! When a probe identifies multiple markers in a single directory (e.g., both
//! `lithos.toml` and `lithos.json`), this module provides the logic to
//! deterministically select the primary marker based on format and path
//! precedence.
//!
//! # Selection Criteria
//!
//! 1. **Format Rank**: Markers are prioritized based on their file format. TOML
//!    is generally preferred over JSON or YAML.
//! 2. **Lexical Tie-break**: If formats are identical, the path that comes
//!    first alphabetically is selected.

use super::engine::DiscoveredMarker;
use crate::fs::format::StructuredFileFormat;

/// Picks the highest-precedence marker from a slice of discovered markers.
///
/// Precedence is determined by:
/// 1. **File Format**: Formats are ranked according to
///    [`StructuredFileFormat::rank`] (typically TOML > JSON > YAML).
/// 2. **Path Lexicographical Order**: If formats are identical, the path closer
///    to the start of the alphabet wins (tie-breaker).
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) fn select_candidate(
    markers: &[DiscoveredMarker],
) -> Option<&DiscoveredMarker> {
    markers.iter().min_by(|a, b| {
        a.format.rank().cmp(&b.format.rank()).then_with(|| a.path.cmp(&b.path))
    })
}

/// Attempts to find a marker with the preferred format, falling back to
/// standard selection.
#[allow(dead_code, reason = "Phase-1 seam; wired in once orchestration lands")]
pub(crate) fn promote_alternative(
    markers: &[DiscoveredMarker],
    preferred_format: StructuredFileFormat,
) -> Option<&DiscoveredMarker> {
    let preferred = markers.iter().find(|m| m.format == preferred_format);
    preferred.or_else(|| select_candidate(markers))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn marker(
        base: &str,
        path: &str,
        format: StructuredFileFormat,
    ) -> DiscoveredMarker {
        DiscoveredMarker {
            base: PathBuf::from(base),
            path: PathBuf::from(path),
            format,
        }
    }

    mod select_candidate {
        use super::*;

        #[test]
        fn returns_none_for_empty_slice() {
            assert_eq!(crate::discovery::selector::select_candidate(&[]), None);
        }

        #[test]
        fn returns_single_marker() {
            let markers = [marker(
                "/vault",
                "/vault/lithos.toml",
                StructuredFileFormat::Toml,
            )];

            let selected =
                crate::discovery::selector::select_candidate(&markers);

            assert_eq!(selected, Some(&markers[0]));
        }

        #[test]
        fn returns_highest_precedence_format() {
            let markers = [
                marker(
                    "/vault",
                    "/vault/lithos.json",
                    StructuredFileFormat::Json,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.toml",
                    StructuredFileFormat::Toml,
                ),
            ];

            let selected =
                crate::discovery::selector::select_candidate(&markers);

            assert_eq!(selected, Some(&markers[1]));
        }

        #[test]
        fn returns_toml_over_yaml() {
            let markers = [
                marker(
                    "/vault",
                    "/vault/lithos.yaml",
                    StructuredFileFormat::Yaml,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.toml",
                    StructuredFileFormat::Toml,
                ),
            ];

            let selected =
                crate::discovery::selector::select_candidate(&markers);

            assert_eq!(selected, Some(&markers[1]));
        }

        #[test]
        fn returns_yaml_over_yml() {
            let markers = [
                marker(
                    "/vault",
                    "/vault/lithos.yml",
                    StructuredFileFormat::Yml,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.yaml",
                    StructuredFileFormat::Yaml,
                ),
            ];

            let selected =
                crate::discovery::selector::select_candidate(&markers);

            assert_eq!(selected, Some(&markers[1]));
        }
    }

    mod promote_alternative {
        use super::*;

        #[test]
        fn returns_matching_format() {
            let markers = [
                marker(
                    "/vault",
                    "/vault/lithos.toml",
                    StructuredFileFormat::Toml,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.json",
                    StructuredFileFormat::Json,
                ),
            ];

            let selected = crate::discovery::selector::promote_alternative(
                &markers,
                StructuredFileFormat::Json,
            );

            assert_eq!(selected, Some(&markers[1]));
        }

        #[test]
        fn falls_back_to_select_candidate() {
            let markers = [
                marker(
                    "/vault",
                    "/vault/lithos.json",
                    StructuredFileFormat::Json,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.yaml",
                    StructuredFileFormat::Yaml,
                ),
                marker(
                    "/vault",
                    "/vault/lithos.toml",
                    StructuredFileFormat::Toml,
                ),
            ];

            let selected = crate::discovery::selector::promote_alternative(
                &markers,
                StructuredFileFormat::Yml,
            );

            assert_eq!(selected, Some(&markers[2]));
        }

        #[test]
        fn returns_none_for_empty_slice() {
            let selected = crate::discovery::selector::promote_alternative(
                &[],
                StructuredFileFormat::Toml,
            );

            assert_eq!(selected, None);
        }
    }
}
