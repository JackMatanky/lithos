#!/usr/bin/env bash
#MISE description="Generate and open documentation for all workspace crates"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-o --open" help="Open documentation in browser"

set -euo pipefail

#######################################
# Build arguments for cargo doc.
# Globals:
#   usage_open
# Arguments:
#   Reference to an array for arguments
# Outputs:
#   None
#######################################
build_doc_args() {
    local -n ref_args=$1
    ref_args+=("--no-deps" "--all-features")
    if [[ "${usage_open:-}" == "1" ]]; then
        ref_args+=("--open")
    fi
}

#######################################
# Generate crate documentation.
# Arguments:
#   Arguments for cargo doc
# Outputs:
#   Writes doc generation progress to stdout
#######################################
run_cargo_doc() {
    echo "📚 Generating documentation..."
    cargo doc "$@"
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
    build_doc_args args
    run_cargo_doc "${args[@]}"
    echo "✅ Documentation generated"
}

main "$@"
