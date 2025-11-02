// Package cache provides filesystem-based cache adapters for note persistence.
//
// This package implements the CQRS pattern with separate write and read
// adapters for atomic persistence and optimized querying of notes.
package cache

import (
	"encoding/base64"
	"os"
	"path/filepath"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
)

const (
	cacheFileExt         = ".json"
	cacheFilenamePrefix  = "id-"
	legacySeparatorToken = "-"
	cacheDirPerms        = 0o750
)

// noteFilePath constructs the cache file path for a given note ID using a
// reversible, collision-free encoding. The ID is normalised to forward slashes,
// Base64 URL-encoded, and prefixed to make detection of the new scheme trivial.
// Format: {cacheDir}/id-{base64(noteID)}.json.
func noteFilePath(cacheDir string, id domain.NoteID) string {
	normalized := strings.ReplaceAll(string(id), "\\", "/")
	encoded := base64.RawURLEncoding.EncodeToString([]byte(normalized))
	filename := cacheFilenamePrefix + encoded + cacheFileExt
	return filepath.Join(cacheDir, filename)
}

// legacyNoteFilePath reproduces the pre-3.10 cache file naming logic so the
// reader and writer can clean up or read existing cache entries during the
// rollout of the new encoding scheme.
func legacyNoteFilePath(cacheDir string, id domain.NoteID) string {
	safeName := strings.ReplaceAll(string(id), "/", legacySeparatorToken)
	safeName = strings.ReplaceAll(safeName, "\\", legacySeparatorToken)
	return filepath.Join(cacheDir, safeName+cacheFileExt)
}

// decodeNoteIDFromFilename attempts to recover the original NoteID from a cache
// filename. It first checks for the new Base64 encoding, then falls back to the
// legacy flat naming. The boolean return value indicates whether the new scheme
// was used.
func decodeNoteIDFromFilename(filename string) (domain.NoteID, bool) {
	base := strings.TrimSuffix(filename, cacheFileExt)
	if strings.HasPrefix(base, cacheFilenamePrefix) {
		raw := base[len(cacheFilenamePrefix):]
		decoded, err := base64.RawURLEncoding.DecodeString(raw)
		if err == nil {
			return domain.NoteID(decoded), true
		}
	}
	return domain.NoteID(base), false
}

// EnsureCacheDir creates the cache directory if missing using
// mkdir -p semantics. Permissions default to 0o750 (rwxr-x---).
func EnsureCacheDir(cacheDir string) error {
	return os.MkdirAll(cacheDir, cacheDirPerms)
}
