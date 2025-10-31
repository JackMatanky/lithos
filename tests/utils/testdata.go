package utils

import (
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

var (
	allowedRoots = map[string]struct{}{
		"schemas":   {},
		"templates": {},
		"vault":     {},
		"golden":    {},
	}

	snakeCaseSegment = regexp.MustCompile(`^[a-z0-9]+(?:_[a-z0-9]+)*$`)
)

func isSnakeCase(value string) bool {
	return snakeCaseSegment.MatchString(value)
}

// Root returns the absolute path to the project's testdata directory.
func Root(t *testing.T) string {
	t.Helper()

	dir, err := os.Getwd()
	require.NoError(t, err)

	for {
		if _, statErr := os.Stat(filepath.Join(dir, "go.mod")); statErr == nil {
			break
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			t.Fatalf(
				"could not locate project root containing go.mod from %s",
				dir,
			)
		}
		dir = parent
	}

	return filepath.Join(dir, "testdata")
}

// Path resolves a fixture path under testdata, enforcing allowed roots and
// snake_case naming.
func Path(t *testing.T, segments ...string) string {
	t.Helper()

	require.NotEmpty(t, segments, "testdata path segments must not be empty")

	first := segments[0]
	root := Root(t)

	if _, ok := allowedRoots[first]; ok {
		validateSegments(t, segments)
		full := filepath.Join(append([]string{root}, segments...)...)
		ensurePathExists(t, full)
		return full
	}

	// Allow direct top-level files (e.g., basic fixtures)
	require.Len(
		t,
		segments,
		1,
		"top-level fixtures must be referenced with a single segment",
	)
	validateTestdataSegment(t, first)

	full := filepath.Join(root, first)
	ensurePathExists(t, full)
	return full
}

// Open returns an *os.File for the requested fixture, failing the test on
// error.
func Open(t *testing.T, segments ...string) *os.File {
	t.Helper()

	path := Path(t, segments...)
	cleanPath := filepath.Clean(path)
	file, err := os.Open(cleanPath)
	require.NoError(t, err, "open fixture %s", path)
	return file
}

func validateSegments(t *testing.T, segments []string) {
	for _, seg := range segments {
		validateTestdataSegment(t, seg)
	}
}

func validateTestdataSegment(t *testing.T, segment string) {
	name := trimSegment(segment)
	require.Truef(
		t,
		isSnakeCase(name),
		"segment %q must be snake_case",
		segment,
	)
}

func trimSegment(segment string) string {
	trimmed := strings.TrimPrefix(segment, ".")
	if ext := filepath.Ext(trimmed); ext != "" {
		trimmed = trimmed[:len(trimmed)-len(ext)]
	}
	return trimmed
}

func ensurePathExists(t *testing.T, path string) {
	t.Helper()

	_, err := os.Stat(path)
	require.NoErrorf(t, err, "fixture %s does not exist", path)
}
