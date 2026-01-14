// # LINT_DISABLE_REASON: Tests use disallowed methods for setup and assertions.
// # LINT_DISABLE_REASON: Options tried: manual Result handling.
// # LINT_DISABLE_REASON: Justification: Test code clarity.
#![expect(
    clippy::disallowed_methods,
    deprecated,
    reason = "Tests use disallowed methods for setup and assertions for clarity"
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_prints_hello() {
    let mut cmd = Command::cargo_bin("lithos").expect("Binary should exist");

    cmd.assert().success().stdout(predicate::str::contains("Hello, Lithos!"));
}

#[test]
fn cli_prints_help() {
    let _vault =
        lithos_test_utils::TestVault::new().expect("Should create test vault");
    let mut cmd = Command::cargo_bin("lithos").expect("Binary should exist");

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
