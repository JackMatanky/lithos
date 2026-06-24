//! Marker pattern definitions for configuration discovery.
//!
//! This module defines the marker pattern constants used by the discovery
//! pipeline to identify vault root markers and global configuration markers.

/// Naming pattern used to identify a marker file family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MarkerPattern {
    /// Filename prefix before a structured file extension is applied.
    pub(crate) prefix: &'static str,
    /// Whether the pattern is nested below a config directory.
    pub(crate) is_nested: bool,
}

/// Standard marker patterns used for vault root resolution.
pub(crate) const VAULT_MARKER_PATTERNS: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "traces",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".traces",
        is_nested: false,
    },
    MarkerPattern {
        prefix: ".traces/config",
        is_nested: true,
    },
];

/// Standard marker patterns used for global config resolution.
pub(crate) const GLOBAL_MARKER_PATTERNS: &[MarkerPattern] = &[
    MarkerPattern {
        prefix: "traces",
        is_nested: false,
    },
    MarkerPattern {
        prefix: "traces/config",
        is_nested: true,
    },
];

/// Standard project boundary directory names that stop ascending traversal.
pub(crate) const BOUNDARY_MARKER_PATTERNS: &[&str] = &[".git", ".workspace"];

#[cfg(test)]
mod tests {
    use super::*;

    mod defaults {
        use super::*;

        #[test]
        fn declares_vault_marker_pattern_contract_prefix() {
            assert_eq!(
                VAULT_MARKER_PATTERNS.first().expect("vault pattern").prefix,
                "traces"
            );
        }

        #[test]
        fn declares_global_marker_pattern_contract_prefix() {
            assert_eq!(
                GLOBAL_MARKER_PATTERNS.first().expect("global pattern").prefix,
                "traces"
            );
        }

        #[test]
        fn declares_boundary_marker_patterns() {
            let patterns: Vec<&str> = BOUNDARY_MARKER_PATTERNS.to_vec();
            assert!(
                !patterns.is_empty(),
                "boundary patterns must not be empty"
            );
            assert!(
                patterns.contains(&".git"),
                "expected .git boundary marker"
            );
        }
    }
}
