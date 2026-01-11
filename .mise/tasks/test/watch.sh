#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Filename: .mise/tasks/test/watch.sh
# Docs: https://mise.jdx.dev/tasks/
# Description: Watch for file changes and automatically run tests.
# -----------------------------------------------------------------------------
#MISE description="Watch for changes and run tests"
#MISE sources=["**/*.rs", "Cargo.toml"]
#USAGE arg "[args]..." help="Arguments to pass to cargo test"

set -euo pipefail

#######################################
# Watch for changes and run tests using cargo-watch.
# Globals:
#   usage_args
# Arguments:
#   None
# Outputs:
#   Writes test results to stdout on every change
#######################################
watch_tests() {
    local watch_cmd="test"
    if [[ -n "${usage_args:-}" ]]; then
        watch_cmd="test -- ${usage_args}"
    fi

    echo "👀 Watching for changes..."
    cargo watch -x "${watch_cmd}"
}

main() {
    watch_tests
}

main "$@"
