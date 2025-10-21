// Package testutils provides shared test utilities for filesystem operations.
package testutils

import (
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

// FindProjectRoot finds the project root directory by looking for go.mod.
func FindProjectRoot(t *testing.T) string {
	_, filename, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("Could not get caller information")
	}

	// Start from the directory containing this test file
	dir := filepath.Dir(filename)

	// Walk up directories until we find go.mod
	for {
		if _, err := os.Stat(filepath.Join(dir, "go.mod")); err == nil {
			return dir
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			t.Fatal("Could not find project root (go.mod)")
		}
		dir = parent
	}
}
