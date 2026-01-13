//! Property-based testing strategies for Lithos.
//!
//! This module provides standard `proptest` strategies for domain types,
//! helping catch edge cases in filesystem logic, path normalization, and more.

use std::path::PathBuf;

use proptest::prelude::*;

/// Strategy for generating arbitrary valid-looking relative paths.
pub fn relative_path() -> impl Strategy<Value = PathBuf> {
    prop::collection::vec("[a-zA-Z0-9._-]+", 1..5).prop_map(|parts| {
        let mut path = PathBuf::new();
        for part in parts {
            path.push(part);
        }
        path
    })
}

/// Strategy for generating arbitrary filenames with extensions.
pub fn filename() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,20}\\.[a-z]{1,3}"
}

/// Strategy for generating arbitrary note content.
pub fn note_content() -> impl Strategy<Value = String> {
    prop::collection::vec(".*", 1..10).prop_map(|lines| lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_relative_path_strategy(p in relative_path()) {
            assert!(!p.is_absolute());
            assert!(p.components().count() >= 1);
        }

        #[test]
        fn test_filename_strategy(f in filename()) {
            assert!(f.contains('.'));
            assert!(!f.is_empty());
        }
    }
}
