// Package utils provides shared utility functions for the application.
//
// This package contains common functionality used across multiple components
// to avoid code duplication and ensure consistent behavior.
package utils

import (
	"os"
)

// EnsureCacheDir creates cache directory if missing.
// Uses os.MkdirAll for recursive creation (mkdir -p semantics).
// Permissions: 0o750 (rwxr-x---).
func EnsureCacheDir(cacheDir string) error {
	return os.MkdirAll(cacheDir, 0o750)
}
