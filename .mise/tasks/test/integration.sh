#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/test/integration.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Execute all integration tests across the workspace using nextest.
# -----------------------------------------------------------------------------
#MISE description="Run all integration tests"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-v --verbose" help="Verbose output"
#USAGE arg "<filter>" help="Filter tests by name" optional=true

set -euo pipefail

#######################################
# Execute integration tests using nextest.
# Globals:
#   usage_verbose
#   usage_filter
# Arguments:
#   None
# Outputs:
#   Writes test results to stdout
#######################################
run_integration_tests() {
    local args=("--test" "*")
    if [[ "${usage_verbose:-false}" == "true" ]]; then
        args+=("--verbose")
    fi
    if [[ -n "${usage_filter:-}" ]]; then
        args+=("${usage_filter}")
    fi

    echo "🧪 Running integration tests..."
    cargo nextest run "${args[@]}"
    echo "✅ Integration tests complete"
}

main() {
    run_integration_tests
}

main "$@"
