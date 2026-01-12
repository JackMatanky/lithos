#!/usr/bin/env bash
#MISE description="Clean the workspace by removing build artifacts and cache files"

set -euo pipefail

#######################################
# Remove cargo build artifacts.
# Arguments:
#   None
# Outputs:
#   Writes progress to stdout
#######################################
run_cargo_clean() {
    echo "🧹 Cleaning cargo build artifacts..."
    cargo clean
}

#######################################
# Remove custom task stamp files and reports.
# Arguments:
#   None
# Outputs:
#   Writes progress to stdout
#######################################
remove_custom_artifacts() {
    echo "🧹 Removing custom task artifacts and stamps..."
    rm -f target/*.stamp
    rm -f tarpaulin-report.html
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
    run_cargo_clean
    remove_custom_artifacts
    echo "✅ Cleanup complete"
}

main "$@"
