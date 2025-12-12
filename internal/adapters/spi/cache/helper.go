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
	"time"
)

const (
	cacheFileExt         = ".json"
	cacheFilenamePrefix  = "id-"
	legacySeparatorToken = "-"
	cacheDirPerms        = 0o750
)

// noteFilePath constructs the cache file path for a given note path.
// Since paths are now used directly as identifiers, we use a simple
// filesystem-safe encoding. Format: {cacheDir}/{base64(path)}.json.
func noteFilePath(cacheDir, path string) string {
	normalized := strings.ReplaceAll(path, "\\", "/")
	encoded := base64.RawURLEncoding.EncodeToString([]byte(normalized))
	filename := encoded + cacheFileExt
	return filepath.Join(cacheDir, filename)
}

// legacyNoteFilePath reproduces the pre-3.10 cache file naming logic so the
// reader and writer can clean up or read existing cache entries during the
// rollout of the new encoding scheme.
func legacyNoteFilePath(cacheDir, path string) string {
	safeName := strings.ReplaceAll(path, "/", legacySeparatorToken)
	safeName = strings.ReplaceAll(safeName, "\\", legacySeparatorToken)
	return filepath.Join(cacheDir, safeName+cacheFileExt)
}

// decodePathFromFilename attempts to recover the original path from a cache
// filename. It first checks for the new Base64 encoding, then falls back to the
// legacy flat naming. The boolean return value indicates whether the new scheme
// was used.
func decodePathFromFilename(filename string) (string, bool) {
	base := strings.TrimSuffix(filename, cacheFileExt)
	decoded, err := base64.RawURLEncoding.DecodeString(base)
	if err == nil {
		return string(decoded), true
	}
	// Legacy format: replace tokens back to slashes
	path := strings.ReplaceAll(base, legacySeparatorToken, "/")
	return path, false
}

// EnsureCacheDir creates the cache directory if missing using
// mkdir -p semantics. Permissions default to 0o750 (rwxr-x---).
func EnsureCacheDir(cacheDir string) error {
	return os.MkdirAll(cacheDir, cacheDirPerms)
}

// ExtractFileModTime extracts the file modification time from frontmatter
// fields.
// Looks for common field names like "file_mod_time", "modified", "updated".
// Falls back to current time if not found.
func ExtractFileModTime(fields map[string]interface{}) time.Time {
	if len(fields) == 0 {
		return time.Time{}
	}

	if ts, ok := readTimeField(fields["file_mod_time"]); ok {
		return ts
	}
	if ts, ok := readTimeField(fields["modified_at"]); ok {
		return ts
	}
	if ts, ok := readTimeField(fields["modified"]); ok {
		return ts
	}
	if ts, ok := readTimeField(fields["updated"]); ok {
		return ts
	}
	if ts, ok := readTimeField(fields["mtime"]); ok {
		return ts
	}
	return time.Time{}
}

func readTimeField(value interface{}) (time.Time, bool) {
	switch v := value.(type) {
	case time.Time:
		return v, true
	case string:
		if parsed, err := time.Parse(time.RFC3339, v); err == nil {
			return parsed, true
		}
	case int64:
		return time.Unix(v, 0), true
	case int:
		return time.Unix(int64(v), 0), true
	case float64:
		return time.Unix(int64(v), 0), true
	}
	return time.Time{}, false
}
