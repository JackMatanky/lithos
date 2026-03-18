#!/usr/bin/env bash
# Test Reorganization Script for Phase 6.2
# This script reorganizes tests in ingestor.rs and loader.rs into proper submodules

set -euo pipefail

INGESTOR_FILE="lithos-core/src/schema/ingestor.rs"
LOADER_FILE="lithos-core/src/schema/loader.rs"

echo "=== Phase 6.2: Test Reorganization ==="

echo "Step 1: Creating backup files..."
cp "$INGESTOR_FILE" "${INGESTOR_FILE}.backup"
cp "$LOADER_FILE" "${LOADER_FILE}.backup"

echo "Step 2: Reorganizing ingestor.rs tests..."
# This will be done manually via Edit tool due to complexity

echo "Step 3: Reorganizing loader.rs tests..."
# This will be done manually via Edit tool due to complexity

echo "Step 4: Verification..."
cargo nextest run --lib -p lithos-core -E 'test(schema)' --no-fail-fast

echo "=== Reorganization Complete ==="
