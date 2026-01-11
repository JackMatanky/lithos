#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/fmt.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Format all Rust files in the workspace using rustfmt.
# -----------------------------------------------------------------------------
#MISE description="Format all Rust files in the workspace"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-c --check" help="Check formatting without making changes"

set -euo pipefail

#######################################
# Format the codebase using rustfmt.
# Globals:
#   usage_check
# Arguments:
#   None
# Outputs:
#   Writes formatting progress to stdout
#######################################
format_code() {
    local args=()
    if [[ "${usage_check:-false}" == "true" ]]; then
        args+=("--check")
    fi

    echo "🚀 Formatting codebase..."
    cargo fmt --all "${args[@]}"
    echo "✅ Formatting complete"
}

main() {
    format_code
}

main "$@"
