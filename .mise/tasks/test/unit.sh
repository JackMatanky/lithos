#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/test/unit.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Execute all unit tests across the workspace using nextest.
# -----------------------------------------------------------------------------
#MISE description="Run all unit tests"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-v --verbose" help="Verbose output"
#USAGE arg "<filter>" help="Filter tests by name" optional=true

set -euo pipefail

#######################################
# Execute unit tests using nextest.
# Globals:
#   usage_verbose
#   usage_filter
# Arguments:
#   None
# Outputs:
#   Writes test results to stdout
#######################################
run_unit_tests() {
    local args=()
    if [[ "${usage_verbose:-false}" == "true" ]]; then
        args+=("--verbose")
    fi
    if [[ -n "${usage_filter:-}" ]]; then
        args+=("${usage_filter}")
    fi

    echo "🧪 Running unit tests..."
    cargo nextest run "${args[@]}"
    echo "✅ Unit tests complete"
}

main() {
    run_unit_tests
}

main "$@"
