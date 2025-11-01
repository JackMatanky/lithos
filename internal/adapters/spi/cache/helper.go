// Package cache provides filesystem-based cache adapters for note persistence.
//
// This package implements the CQRS pattern with separate write and read
// adapters for atomic persistence and optimized querying of notes.
package cache

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

// noteFilePath constructs cache file path from note ID.
// Converts NoteID to filesystem-safe filename by replacing path separators.
// Format: {cacheDir}/{safeNoteID}.json.
//
// Path separators in NoteID (/, \) are replaced with hyphens (-) to create
// unique, filesystem-safe cache filenames.
func noteFilePath(cacheDir string, id domain.NoteID) string {
	// Convert NoteID to safe filename by replacing path separators
	safeName := strings.ReplaceAll(string(id), "/", "-")
	safeName = strings.ReplaceAll(safeName, "\\", "-")
	return filepath.Join(cacheDir, safeName+".json")
}

// ensureCacheDir creates cache directory if missing.
// Uses os.MkdirAll for recursive creation (mkdir -p semantics).
// Permissions: 0o750 (rwxr-x---).
func ensureCacheDir(cacheDir string) error {
	return os.MkdirAll(cacheDir, 0o750)
}
