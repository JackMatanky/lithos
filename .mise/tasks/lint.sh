#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/lint.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Run clippy lints on all crates in the workspace.
# -----------------------------------------------------------------------------
#MISE description="Run clippy lints on all crates"
#MISE sources=["**/*.rs", "Cargo.toml", "clippy.toml"]
#USAGE flag "-f --fix" help="Automatically apply lint fixes"
#USAGE flag "-v --verbose" help="Verbose output"

set -euo pipefail

#######################################
# Run clippy lints on the workspace.
# Globals:
#   usage_fix
#   usage_verbose
# Arguments:
#   None
# Outputs:
#   Writes lint results to stdout/stderr
#######################################
run_clippy() {
    local args=("--all-targets" "--all-features")
    if [[ "${usage_fix:-false}" == "true" ]]; then
        args+=("--fix" "--allow-dirty" "--allow-staged")
    fi
    if [[ "${usage_verbose:-false}" == "true" ]]; then
        args+=("--verbose")
    fi

    echo "🔍 Running clippy lints..."
    cargo clippy "${args[@]}"
    echo "✅ Linting complete"
}

main() {
    run_clippy
}

main "$@"
