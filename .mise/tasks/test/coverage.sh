#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/test/coverage.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Generate code coverage reports using tarpaulin.
# -----------------------------------------------------------------------------
#MISE description="Generate code coverage report"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-o --open" help="Open coverage report in browser"

set -euo pipefail

#######################################
# Generate code coverage report using tarpaulin.
# Globals:
#   usage_open
# Arguments:
#   None
# Outputs:
#   Writes coverage report to stdout
#######################################
generate_coverage() {
    echo "📊 Generating code coverage report..."
    cargo tarpaulin --ignore-tests --out Html

    if [[ "${usage_open:-false}" == "true" ]]; then
        echo "🌐 Opening coverage report..."
        open tarpaulin-report.html
    fi

    echo "✅ Coverage report generated"
}

main() {
    generate_coverage
}

main "$@"
