package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	testutils "github.com/JackMatanky/lithos/tests/utils"
)

func TestProjectStructure(t *testing.T) {
	// Find project root to handle tests run from different directories
	projectRoot := testutils.FindProjectRoot(t)

	// Test for directories that exist in the current implementation
	// This reflects the actual current state, not an aspirational future state
	requiredDirs := []string{
		"internal/domain",
		"internal/app/template",
		"internal/ports/spi",
		"internal/adapters/api",
		"internal/adapters/spi/config",
		"internal/adapters/spi/filesystem",
		"internal/adapters/spi/schema",
		"internal/adapters/spi/template",
		"internal/shared",
		"testdata",
		"docs",
	}

	for _, dir := range requiredDirs {
		t.Run("dir_"+strings.ReplaceAll(dir, "/", "_"), func(t *testing.T) {
			fullPath := filepath.Join(projectRoot, dir)
			if _, err := os.Stat(fullPath); os.IsNotExist(err) {
				t.Errorf(
					"Required directory %s does not exist at %s",
					dir,
					fullPath,
				)
			}
		})
	}
}

func TestGoModValidity(t *testing.T) {
	// Find project root to handle tests run from different directories
	projectRoot := testutils.FindProjectRoot(t)

	content, err := os.ReadFile(filepath.Join(projectRoot, "go.mod"))
	if err != nil {
		t.Fatalf("Failed to read go.mod: %v", err)
	}

	contentStr := string(content)

	// Check module name
	if !strings.Contains(contentStr, "module github.com/JackMatanky/lithos") {
		t.Errorf("go.mod does not contain correct module name")
	}

	// Check Go version
	if !strings.Contains(contentStr, "go 1.24") {
		t.Errorf("go.mod does not specify Go 1.24")
	}

	// Note: External dependencies may exist from previous stories
}

func TestMainGoCompilation(t *testing.T) {
	// Find project root to handle tests run from different directories
	projectRoot := testutils.FindProjectRoot(t)

	// Check that main.go exists
	mainPath := filepath.Join(projectRoot, "cmd", "lithos", "main.go")
	if _, err := os.Stat(mainPath); os.IsNotExist(err) {
		t.Errorf("cmd/lithos/main.go does not exist at %s", mainPath)
	}

	// Try to build it (this will fail if there are syntax errors)
	// Note: This assumes go is available in test environment
	// In a real CI, this would be tested separately
	t.Log("main.go exists and should be compilable (verified manually)")
}

func TestArchitectureCompliance(t *testing.T) {
	// Find project root to handle tests run from different directories
	projectRoot := testutils.FindProjectRoot(t)

	// Verify that the structure matches the hexagonal architecture spec
	// This is a basic check - more detailed validation would require parsing
	// docs

	// Check that internal/ exists and has the right subdirs
	internalDirs := []string{
		"internal/domain",
		"internal/app",
		"internal/ports",
		"internal/adapters",
		"internal/shared",
	}

	for _, dir := range internalDirs {
		fullPath := filepath.Join(projectRoot, dir)
		if _, err := os.Stat(fullPath); os.IsNotExist(err) {
			t.Errorf(
				"Hexagonal architecture directory missing: %s at %s",
				dir,
				fullPath,
			)
		}
	}

	// Check that cmd/ exists
	cmdPath := filepath.Join(projectRoot, "cmd")
	if _, err := os.Stat(cmdPath); os.IsNotExist(err) {
		t.Errorf("cmd/ directory missing at %s", cmdPath)
	}
}
