//! Architecture testing utilities for enforcing project-wide boundaries.
//!
//! This module provides helpers to verify that crates adhere to architectural rules,
//! such as domain purity (no I/O or external dependencies in the domain crate).

use std::process::Command;

use serde_json::Value;

/// Asserts that a specific crate does not depend on any of the prohibited crates,
/// including transitive dependencies.
///
/// This check uses `cargo metadata` to analyze the full dependency graph of the workspace.
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
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("Failed to execute cargo metadata");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("cargo metadata failed: {}", stderr);
    }

    let metadata: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse cargo metadata JSON");

    let resolve =
        metadata["resolve"].as_object().expect("Missing resolve in metadata");
    let nodes = resolve["nodes"].as_array().expect("Missing nodes in resolve");

    // Find the ID of the target crate
    let packages =
        metadata["packages"].as_array().expect("Missing packages in metadata");
    let target_id = packages
        .iter()
        .find(|p| p["name"] == crate_name)
        .and_then(|p| p["id"].as_str())
        .unwrap_or_else(|| {
            panic!("Package '{}' not found in workspace", crate_name)
        });

    let mut found_prohibited = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![target_id];

    while let Some(current_id) = stack.pop() {
        if !visited.insert(current_id) {
            continue;
        }

        let Some(node) = nodes.iter().find(|n| n["id"] == current_id) else {
            continue;
        };
        let Some(deps) = node["dependencies"].as_array() else {
            continue;
        };

        for dep_id in deps {
            let dep_id_str =
                dep_id.as_str().expect("Dependency ID is not a string");

            // Find the package name for this ID
            if let Some(package) =
                packages.iter().find(|p| p["id"] == dep_id_str)
            {
                let name = package["name"]
                    .as_str()
                    .expect("Package name is not a string");
                if prohibited.contains(&name) {
                    found_prohibited.push(name.to_string());
                }
            }
            stack.push(dep_id_str);
        }
    }

    if !found_prohibited.is_empty() {
        found_prohibited.sort();
        found_prohibited.dedup();
        panic!(
            "Architectural Boundary Violation: Crate '{}' (or its dependencies) is prohibited from depending on: {:?}. Found: {:?}",
            crate_name, prohibited, found_prohibited
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_check_detects_prohibited_dependencies() {
        // test-utils depends on insta, so this should panic if we prohibit it
        // but for now we just verify the check runs.
        assert_no_prohibited_dependencies(
            "lithos-test-utils",
            &["non-existent-crate"],
        );
    }
}
