#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename:    .mise/tasks/fmt.sh
# Description: Format all Rust files in the workspace using rustfmt.
# -----------------------------------------------------------------------------
#MISE description="Format all Rust files in the workspace"
#MISE sources=["**/*.rs", "Cargo.toml", "rustfmt.toml", "rust-toolchain.toml"]
#MISE outputs=["target/fmt.stamp"]
#USAGE flag "-c --check" help="Check formatting without making changes"

set -euo pipefail

# Ensure we are in the project root
cd "$(git rev-parse --show-toplevel)"

#######################################
# Build arguments for rustfmt.
# Globals:
#   usage_check
# Arguments:
#   Reference to an array for arguments
# Outputs:
#   None
#######################################
build_fmt_args() {
    local -n ref_args=$1
    if [[ "${usage_check:-}" == "1" ]]; then
        ref_args+=("--check")
    fi
}

#######################################
# Format the codebase using rustfmt.
# Arguments:
#   Arguments for cargo fmt
# Outputs:
#   Writes formatting progress to stdout
#######################################
run_rustfmt() {
    echo "🚀 Formatting codebase (using nightly via rust-toolchain.toml)..."
    # Using 'cargo fmt' directly as rust-toolchain.toml handles the channel
    # Passing --unstable-features to the underlying rustfmt call
    cargo fmt --all "$@" -- --unstable-features
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
    touch target/fmt.stamp
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
    build_fmt_args args
    run_rustfmt "${args[@]}"
    create_stamp_file
    echo "✅ Formatting complete"
}

main "$@"
