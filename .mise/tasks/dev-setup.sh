#!/usr/bin/env bash
#MISE description="Bootstraps the development environment"

set -e

echo "🚀 Bootstrapping Lithos development environment..."

# Ensure pre-commit is installed
if ! command -v pre-commit &>/dev/null; then
    echo "📦 Installing pre-commit..."
    pip install pre-commit
fi

# Install git hooks
echo "⚓ Installing git hooks..."
pre-commit install

# Verify toolchain
echo "🦀 Checking Rust toolchain..."
rustc --version
cargo --version

echo "✅ Development environment ready!"
