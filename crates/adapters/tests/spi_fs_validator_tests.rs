//! Integration tests for path validation utilities (Story 4.2).
//!
//! This test suite follows TDD RED-GREEN-REFACTOR cycle.

use std::{borrow::Cow, path::Path};

use lithos_adapters::spi::fs::validator::{PathValidationError, Validator};

#[cfg(test)]
mod path_traversal_tests {
    use super::*;

    #[test]
    fn rejects_double_dot_traversal() {
        // AC: Path containing `..` components should return PathTraversalError
        let validator = Validator::new_flexible();
        let result = validator.validate("../../etc/passwd");

        assert!(result.is_err(), "Should reject path with .. components");
        assert!(matches!(result, Err(PathValidationError::PathTraversal)));
    }

    #[test]
    fn rejects_single_parent_traversal() {
        let validator = Validator::new_flexible();
        let result = validator.validate("../config.toml");

        assert!(result.is_err(), "Should reject single .. traversal");
        assert!(matches!(result, Err(PathValidationError::PathTraversal)));
    }

    #[test]
    fn rejects_mid_path_traversal() {
        let validator = Validator::new_flexible();
        let result = validator.validate("valid/../../etc/passwd");

        assert!(result.is_err(), "Should reject .. in middle of path");
        assert!(matches!(result, Err(PathValidationError::PathTraversal)));
    }

    #[test]
    fn handles_encoded_characters_as_literal() {
        let validator = Validator::new_flexible();
        // URL-encoded characters are treated as literal filename characters
        // The path "..%2F..%2Fetc%2Fpasswd" is literally a file named that
        let result = validator.validate("safe%2Ffile");

        // This is a valid filename (% and chars are literal, not path
        // separators)
        assert!(result.is_ok(), "URL encoding creates literal filename chars");
    }
}

#[cfg(test)]
mod absolute_path_tests {
    use super::*;

    #[test]
    fn rejects_unix_absolute_path() {
        // AC: Absolute paths should return AbsolutePathError
        let validator = Validator::new_flexible();
        let result = validator.validate("/etc/hosts");

        assert!(result.is_err(), "Should reject Unix absolute path");
        assert!(matches!(result, Err(PathValidationError::AbsolutePath(_))));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn rejects_windows_absolute_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("C:\\Windows\\System32");

        assert!(result.is_err(), "Should reject Windows absolute path");
        assert!(matches!(result, Err(PathValidationError::AbsolutePath(_))));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn rejects_windows_unc_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("\\\\server\\share\\file");

        assert!(result.is_err(), "Should reject UNC path");
        assert!(matches!(result, Err(PathValidationError::AbsolutePath(_))));
    }

    #[test]
    fn accepts_relative_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("config/lithos.toml");

        assert!(result.is_ok(), "Should accept valid relative path");
    }
}

#[cfg(test)]
mod restricted_file_tests {
    use super::*;

    #[test]
    fn rejects_git_config() {
        // AC: Hidden/sensitive files should return RestrictedPathError
        let validator = Validator::new_flexible();
        let result = validator.validate(".git/config");

        assert!(result.is_err(), "Should reject .git directory access");
        assert!(matches!(result, Err(PathValidationError::RestrictedPath(_))));
    }

    #[test]
    fn rejects_env_file() {
        let validator = Validator::new_flexible();
        let result = validator.validate(".env");

        assert!(result.is_err(), "Should reject .env file");
        assert!(matches!(result, Err(PathValidationError::RestrictedPath(_))));
    }

    #[test]
    fn rejects_nested_hidden_file() {
        let validator = Validator::new_flexible();
        let result = validator.validate("config/.env");

        assert!(result.is_err(), "Should reject nested hidden file");
        assert!(matches!(result, Err(PathValidationError::RestrictedPath(_))));
    }

    #[test]
    fn rejects_ssh_keys() {
        let validator = Validator::new_flexible();
        let result = validator.validate(".ssh/id_rsa");

        assert!(result.is_err(), "Should reject SSH key access");
        assert!(matches!(result, Err(PathValidationError::RestrictedPath(_))));
    }

    #[test]
    fn accepts_normal_file() {
        let validator = Validator::new_flexible();
        let result = validator.validate("notes/daily.md");

        assert!(result.is_ok(), "Should accept normal file path");
    }
}

#[cfg(test)]
mod symlink_resolution_strict_tests {
    use super::*;

    #[tokio::test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses expect for clarity"
    )]
    async fn strict_rejects_escaped_symlink() {
        // AC: Strict mode should reject symlinks that escape root directory
        let temp_dir = tempfile::TempDir::new().expect("test setup failed");
        let root = temp_dir.path();

        // Create symlink pointing outside root
        let outside_target = std::env::temp_dir().join("outside.txt");
        std::fs::write(&outside_target, "outside content")
            .expect("test setup failed");

        let symlink_path = root.join("escaped_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_target, &symlink_path)
            .expect("test setup failed");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside_target, &symlink_path)
            .expect("test setup failed");

        let validator = Validator::new_strict(root.to_path_buf());
        let result = validator.resolve_safe_symlink(&symlink_path).await;

        assert!(result.is_err(), "Should reject symlink escaping root");
        assert!(matches!(result, Err(PathValidationError::SymlinkEscape)));

        // Cleanup - ignore errors as test is complete
        drop(std::fs::remove_file(&outside_target));
    }

    #[tokio::test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses expect for clarity"
    )]
    async fn strict_accepts_internal_symlink() {
        // Strict mode should accept symlinks pointing within root
        let temp_dir = tempfile::TempDir::new().expect("test setup failed");
        let root = temp_dir.path();

        let target = root.join("target.txt");
        std::fs::write(&target, "internal content").expect("test setup failed");

        let symlink_path = root.join("internal_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &symlink_path)
            .expect("test setup failed");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &symlink_path)
            .expect("test setup failed");

        let validator = Validator::new_strict(root.to_path_buf());
        let result = validator.resolve_safe_symlink(&symlink_path).await;

        assert!(result.is_ok(), "Should accept symlink within root");
        // Canonicalization resolves to actual filesystem path which may differ
        // The key invariant is that it doesn't error with SymlinkEscape
    }

    #[tokio::test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses expect for clarity"
    )]
    async fn strict_detects_symlink_loop() {
        // Should detect and reject circular symlink chains
        let temp_dir = tempfile::TempDir::new().expect("test setup failed");
        let root = temp_dir.path();

        let link_a = root.join("link_a");
        let link_b = root.join("link_b");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&link_b, &link_a)
                .expect("test setup failed");
            std::os::unix::fs::symlink(&link_a, &link_b)
                .expect("test setup failed");
        };
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(&link_b, &link_a)
                .expect("test setup failed");
            std::os::windows::fs::symlink_file(&link_a, &link_b)
                .expect("test setup failed");
        }

        let validator = Validator::new_strict(root.to_path_buf());
        let result = validator.resolve_safe_symlink(&link_a).await;

        // canonicalize should fail on circular symlinks
        assert!(result.is_err(), "Should detect symlink loop");
        assert!(matches!(result, Err(PathValidationError::IoError(_))));
    }
}

#[cfg(test)]
mod symlink_resolution_flexible_tests {
    use super::*;

    #[tokio::test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Test setup uses expect for clarity"
    )]
    async fn flexible_allows_external_symlink() {
        // AC: Flexible mode should allow symlinks pointing outside (e.g.,
        // dotfiles)
        let temp_dir = tempfile::TempDir::new().expect("test setup failed");
        let root = temp_dir.path();

        let outside_target = std::env::temp_dir().join("dotfile.toml");
        std::fs::write(&outside_target, "dotfile content")
            .expect("test setup failed");

        let symlink_path = root.join("dotfile_link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_target, &symlink_path)
            .expect("test setup failed");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside_target, &symlink_path)
            .expect("test setup failed");

        let validator = Validator::new_flexible();
        let result = validator.resolve_safe_symlink(&symlink_path).await;

        assert!(
            result.is_ok(),
            "Should allow external symlink in flexible mode"
        );

        // Cleanup - ignore errors as test is complete
        drop(std::fs::remove_file(&outside_target));
    }

    #[tokio::test]
    #[expect(
        clippy::disallowed_methods,
        reason = "matches! macro uses expect internally"
    )]
    async fn flexible_still_checks_input_traversal() {
        // Flexible mode still enforces traversal checks on input path
        let validator = Validator::new_flexible();
        let result = validator.validate("../../../dotfile");

        assert!(
            result.is_err(),
            "Flexible mode still rejects traversal in input"
        );
        assert!(matches!(result, Err(PathValidationError::PathTraversal)));
    }
}

#[cfg(test)]
mod valid_path_acceptance_tests {
    use super::*;

    #[test]
    fn accepts_simple_filename() {
        let validator = Validator::new_flexible();
        let result = validator.validate("config.toml");

        assert!(result.is_ok(), "Should accept simple filename");
    }

    #[test]
    fn accepts_nested_path() {
        let validator = Validator::new_flexible();
        let result = validator.validate("notes/2024/january/daily.md");

        assert!(result.is_ok(), "Should accept nested relative path");
    }

    #[test]
    fn returns_cow_path() {
        // AC: Should return Cow<'a, Path> to avoid unnecessary allocations
        let validator = Validator::new_flexible();
        let result = validator.validate("config.toml");

        assert!(
            result.is_ok(),
            "Validation should succeed: {:?}",
            result.err()
        );
        if let Ok(validated_path) = result {
            // Verify it's a borrowed Cow (no allocation)
            assert!(matches!(validated_path, Cow::Borrowed(_)));
            assert_eq!(validated_path.as_ref(), Path::new("config.toml"));
        }
    }

    #[test]
    fn normalization_preserves_valid_paths() {
        let validator = Validator::new_flexible();
        let result = validator.validate("./config.toml");

        // Current directory component is allowed
        assert!(result.is_ok(), "Should handle ./ prefix correctly");
    }
}

#[cfg(test)]
mod platform_specific_tests {
    use super::*;

    #[test]
    fn handles_platform_separators() {
        // AC: Should handle platform-specific separators correctly
        let validator = Validator::new_flexible();

        #[cfg(unix)]
        let path = "config/notes/file.md";
        #[cfg(windows)]
        let path = "config\\notes\\file.md";

        let result = validator.validate(path);
        assert!(result.is_ok(), "Should handle platform separators");
    }

    #[test]
    fn mixed_separators() {
        // Windows allows mixed separators, Unix only forward slash
        let validator = Validator::new_flexible();

        #[cfg(windows)]
        {
            let result = validator.validate("config/notes\\file.md");
            assert!(result.is_ok(), "Windows should handle mixed separators");
        }

        #[cfg(unix)]
        {
            // Backslash is valid filename character on Unix, but not a
            // separator This would be interpreted as: "config" /
            // "notes\file.md"
            let result = validator.validate("config/notes\\file.md");
            // This is a valid relative path on Unix
            assert!(result.is_ok(), "Unix treats backslash as filename char");
        }
    }
}
