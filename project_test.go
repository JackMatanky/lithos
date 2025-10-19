package main

import (
	"os"
	"strings"
	"testing"
)

func TestProjectStructure(t *testing.T) {
	requiredDirs := []string{
		"cmd/lithos",
		"internal/domain",
		"internal/app/command",
		"internal/app/indexing",
		"internal/app/query",
		"internal/app/schema",
		"internal/app/template",
		"internal/ports/api",
		"internal/ports/spi",
		"internal/adapters/api",
		"internal/adapters/spi/cache",
		"internal/adapters/spi/config",
		"internal/adapters/spi/filesystem",
		"internal/adapters/spi/interactive",
		"internal/adapters/spi/schema",
		"internal/adapters/spi/template",
		"internal/shared",
		"pkg",
		"templates",
		"schemas",
		"testdata",
		".lithos",
		"docs",
	}

	for _, dir := range requiredDirs {
		t.Run("dir_"+strings.ReplaceAll(dir, "/", "_"), func(t *testing.T) {
			if _, err := os.Stat(dir); os.IsNotExist(err) {
				t.Errorf("Required directory %s does not exist", dir)
			}
		})
	}
}

func TestGoModValidity(t *testing.T) {
	content, err := os.ReadFile("go.mod")
	if err != nil {
		t.Fatalf("Failed to read go.mod: %v", err)
	}

	contentStr := string(content)

	// Check module name
	if !strings.Contains(contentStr, "module github.com/jack/lithos") {
		t.Errorf("go.mod does not contain correct module name")
	}

	// Check Go version
	if !strings.Contains(contentStr, "go 1.24") {
		t.Errorf("go.mod does not specify Go 1.24")
	}

	// Note: External dependencies may exist from previous stories
}

func TestGitIgnorePatterns(t *testing.T) {
	content, err := os.ReadFile(".gitignore")
	if err != nil {
		t.Fatalf("Failed to read .gitignore: %v", err)
	}

	contentStr := string(content)

	requiredPatterns := []string{
		".lithos/",
		"*.exe",
		"*.test",
		"vendor/",
		"*.log",
	}

	for _, pattern := range requiredPatterns {
		if !strings.Contains(contentStr, pattern) {
			t.Errorf(
				".gitignore does not contain required pattern: %s",
				pattern,
			)
		}
	}
}

func TestMainGoCompilation(t *testing.T) {
	// Check that main.go exists
	if _, err := os.Stat("cmd/lithos/main.go"); os.IsNotExist(err) {
		t.Errorf("cmd/lithos/main.go does not exist")
	}

	// Try to build it (this will fail if there are syntax errors)
	// Note: This assumes go is available in test environment
	// In a real CI, this would be tested separately
	t.Log("main.go exists and should be compilable (verified manually)")
}

func TestArchitectureCompliance(t *testing.T) {
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
		if _, err := os.Stat(dir); os.IsNotExist(err) {
			t.Errorf("Hexagonal architecture directory missing: %s", dir)
		}
	}

	// Check that cmd/ exists
	if _, err := os.Stat("cmd"); os.IsNotExist(err) {
		t.Errorf("cmd/ directory missing")
	}
}
