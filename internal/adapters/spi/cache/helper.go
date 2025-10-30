// Package cache provides filesystem-based cache adapters for note persistence.
//
// This package implements the CQRS pattern with separate write and read
// adapters for atomic persistence and optimized querying of notes.
package cache

import (
	"os"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
)

// noteFilePath constructs cache file path from note ID.
// Format: {cacheDir}/{noteID}.json.
func noteFilePath(cacheDir string, id domain.NoteID) string {
	return filepath.Join(cacheDir, string(id)+".json")
}

// ensureCacheDir creates cache directory if missing.
// Uses os.MkdirAll for recursive creation (mkdir -p semantics).
// Permissions: 0o750 (rwxr-x---).
func ensureCacheDir(cacheDir string) error {
	return os.MkdirAll(cacheDir, 0o750)
}
