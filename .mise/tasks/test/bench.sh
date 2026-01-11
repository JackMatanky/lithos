#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/test/bench.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Run performance benchmarks for all workspace crates.
# -----------------------------------------------------------------------------
#MISE description="Run benchmarks using criterion"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE arg "<filter>" help="Filter benchmarks by name" optional=true

set -euo pipefail

#######################################
# Run benchmarks for all crates.
# Globals:
#   usage_filter
# Arguments:
#   None
# Outputs:
#   Writes benchmark results to stdout
#######################################
run_benchmarks() {
    local args=()
    if [[ -n "${usage_filter:-}" ]]; then
        args+=("${usage_filter}")
    fi

    echo "🧪 Running benchmarks..."
    cargo bench "${args[@]}"
    echo "✅ Benchmarking complete"
}

main() {
    run_benchmarks
}

main "$@"
