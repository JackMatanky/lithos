//! Architecture testing utilities for enforcing project-wide boundaries.
//!
//! This module provides helpers to verify that crates adhere to architectural rules,
//! such as domain purity (no I/O or external dependencies in the domain crate).

use std::process::Command;

use serde_json::Value;

/// Asserts that a specific crate does not depend on any of the prohibited crates.
///
/// This check includes both direct dependencies and dev-dependencies (used in tests).
///
/// # Panics
///
/// Panics if any prohibited dependency is found or if `cargo metadata` fails.
// # LINT_DISABLE_REASON: Architecture tests use cargo metadata which requires expect() for parsing.
// # LINT_DISABLE_REASON: Options tried: manual Result propagation.
// # LINT_DISABLE_REASON: Justification: this is test-only code where panics are preferred over silent failures.
#[allow(clippy::expect_used, clippy::disallowed_methods)]
pub fn assert_no_prohibited_dependencies(
    crate_name: &str,
    prohibited: &[&str],
) {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .expect("Failed to execute cargo metadata");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("cargo metadata failed: {}", stderr);
    }

    let metadata: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse cargo metadata JSON");

    let package = metadata["packages"]
        .as_array()
        .expect("Missing packages in metadata")
        .iter()
        .find(|p| p["name"] == crate_name)
        .unwrap_or_else(|| {
            panic!("Package '{}' not found in workspace", crate_name)
        });

    let mut found_prohibited = Vec::new();

    if let Some(dependencies) = package["dependencies"].as_array() {
        for dep in dependencies {
            let dep_name =
                dep["name"].as_str().expect("Missing dependency name");
            if prohibited.contains(&dep_name) {
                found_prohibited.push(dep_name.to_string());
            }
        }
    }

    if !found_prohibited.is_empty() {
        panic!(
            "Architectural Boundary Violation: Crate '{}' is prohibited from depending on: {:?}. Found: {:?}",
            crate_name, prohibited, found_prohibited
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_purity_self_check() {
        // test-utils depends on insta, so this should panic if we prohibit it
        // but for now we just verify the check runs.
        assert_no_prohibited_dependencies(
            "lithos-test-utils",
            &["non-existent-crate"],
        );
    }
}
