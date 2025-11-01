// Package cache provides filesystem-based cache adapters for note persistence.
//
// This package implements the CQRS pattern with separate write and read
// adapters for atomic persistence and optimized querying of notes.
package cache

import (
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
