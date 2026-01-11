#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Running performance benchmarks..."

# Run criterion benchmarks if any exist
if [ -f "Cargo.toml" ] && grep -q "criterion" Cargo.toml; then
    echo "📊 Running criterion benchmarks..."
    cargo bench
else
    echo "ℹ️  No criterion benchmarks configured - skipping"
fi

# Run basic cargo bench for any built-in benchmarks
echo "🏃 Running cargo bench..."
cargo bench --workspace || echo "No benchmarks found"

echo "✅ Benchmarks completed"
