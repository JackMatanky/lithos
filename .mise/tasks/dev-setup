#!/usr/bin/env bash
#MISE description="Bootstraps the development environment"

set -euo pipefail

#######################################
# Install pre-commit if not already installed.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes installation progress to stdout
#######################################
install_pre_commit() {
    if ! command -v pre-commit &>/dev/null; then
        echo "📦 Installing pre-commit..."
        pip install pre-commit
    fi
}

#######################################
# Install git hooks using pre-commit.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes installation progress to stdout
#######################################
install_git_hooks() {
    echo "⚓ Installing git hooks..."
    pre-commit install
}

#######################################
# Verify that the Rust toolchain is available.
# Globals:
#   None
# Arguments:
#   None
# Outputs:
#   Writes toolchain versions to stdout
#######################################
verify_rust_toolchain() {
    echo "🦀 Checking Rust toolchain..."
    rustc --version
    cargo --version
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
    echo "🚀 Bootstrapping Lithos development environment..."
    install_pre_commit
    install_git_hooks
    verify_rust_toolchain
    echo "✅ Development environment ready!"
}

main "$@"
