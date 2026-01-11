#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/doc.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Generate and open crate documentation for all workspace crates.
# -----------------------------------------------------------------------------
#MISE description="Generate and open documentation for all crates"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE flag "-o --open" help="Open documentation in browser"

set -euo pipefail

#######################################
# Generate crate documentation.
# Globals:
#   usage_open
# Arguments:
#   None
# Outputs:
#   Writes doc generation progress to stdout
#######################################
generate_docs() {
    local args=("--no-deps" "--all-features")
    if [[ "${usage_open:-false}" == "true" ]]; then
        args+=("--open")
    fi

    echo "📚 Generating documentation..."
    cargo doc "${args[@]}"
    echo "✅ Documentation generated"
}

main() {
    generate_docs
}

main "$@"
