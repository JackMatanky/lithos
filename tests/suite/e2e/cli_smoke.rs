// # LINT_DISABLE_REASON: Tests use disallowed methods for setup and assertions.
// # LINT_DISABLE_REASON: Options tried: manual Result handling.
// # LINT_DISABLE_REASON: Justification: Test code clarity.
#![expect(
    clippy::disallowed_methods,
    reason = "Tests use disallowed methods for setup and assertions for clarity"
)]

use std::{
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    sync::OnceLock,
};

use assert_cmd::Command;
use predicates::prelude::*;

static LITHOS_BINARY: OnceLock<PathBuf> = OnceLock::new();

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("..")
        .join("..")
        .canonicalize()
        .expect("Failed to resolve workspace root from CARGO_MANIFEST_DIR")
}

fn lithos_binary() -> PathBuf {
    LITHOS_BINARY
        .get_or_init(|| {
            if let Ok(path) = std::env::var("CARGO_BIN_EXE_lithos") {
                return PathBuf::from(path);
            }

            let workspace = workspace_root();
            let manifest_path = workspace.join("Cargo.toml");

            let status = ProcessCommand::new("cargo")
                .args([
                    "build",
                    "-p",
                    "lithos",
                    "--manifest-path",
                    manifest_path
                        .to_str()
                        .expect("Workspace manifest path is not valid UTF-8"),
                ])
                .current_dir(&workspace)
                .status()
                .expect("Failed to invoke cargo build for lithos");
            assert!(status.success(), "Cargo build for lithos failed");

            let target_dir = std::env::var("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| workspace.join("target"));
            let mut path = target_dir.join("debug");
            if cfg!(windows) {
                path = path.join("lithos.exe");
            } else {
                path = path.join("lithos");
            }
            path
        })
        .clone()
}

#[test]
fn cli_prints_hello() {
    // GIVEN: the lithos binary is available for execution
    let mut cmd = Command::new(lithos_binary());

    // WHEN: the CLI is run without arguments
    let assertion = cmd.assert().success();

    // THEN: a welcome message is printed for the user
    assertion.stdout(predicate::str::contains("Hello, Lithos!"));
}

#[test]
fn cli_prints_help() {
    // GIVEN: a test vault exists for the CLI to reference
    let _vault =
        lithos_test_utils::TestVault::new().expect("Should create test vault");
    let mut cmd = Command::new(lithos_binary());

    // WHEN: the user requests CLI help output
    let assertion = cmd.arg("--help").assert().success();

    // THEN: the usage text is included in help output
    assertion.stdout(predicate::str::contains("Usage"));
}
