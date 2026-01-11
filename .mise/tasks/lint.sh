#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename:    .mise/tasks/lint.sh
# Description: Run clippy lints on all crates in the workspace.
# -----------------------------------------------------------------------------
#MISE description="Run clippy lints on all crates"
#MISE sources=["**/*.rs", "Cargo.toml", "clippy.toml"]
#MISE outputs=["target/lint.stamp"]
#USAGE flag "-v --verbose" help="Verbose output"
#USAGE flag "-n --no-fix" help="Do not automatically apply lint fixes"

set -euo pipefail

#######################################
# Build arguments for clippy.
# Globals:
#   usage_no_fix
#   usage_verbose
# Arguments:
#   Reference to an array for arguments
# Outputs:
#   None
#######################################
build_clippy_args() {
    local -n ref_args=$1
    ref_args+=("--all-targets" "--all-features")

    # Default to fixing unless --no-fix is passed
    if [[ "${usage_no_fix:-}" != "1" ]]; then
        ref_args+=("--fix" "--allow-dirty" "--allow-staged")
    fi

    if [[ "${usage_verbose:-}" == "1" ]]; then
        ref_args+=("--verbose")
    fi
}

#######################################
# Run clippy lints on the workspace.
# Arguments:
#   Arguments for cargo clippy
# Outputs:
#   Writes lint results to stdout/stderr
#######################################
run_clippy() {
    echo "🔍 Running clippy lints..."
    cargo clippy "$@"
}

#######################################
# Create a stamp file for mise caching.
# Arguments:
#   None
# Outputs:
#   None
#######################################
create_stamp_file() {
    mkdir -p target
    touch target/lint.stamp
}

#######################################
# Main entry point.
# Globals:
#   None
# Arguments:
#   $@
# Outputs:
#   None
#######################################
main() {
    local args=()
    build_clippy_args args
    run_clippy "${args[@]}"
    create_stamp_file
    echo "✅ Linting complete"
}

main "$@"
