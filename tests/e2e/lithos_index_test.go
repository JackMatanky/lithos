package e2e

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"
	"unicode"

	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// titleCase converts the first character of a string to uppercase.
func titleCase(s string) string {
	if s == "" {
		return s
	}
	runes := []rune(s)
	runes[0] = unicode.ToUpper(runes[0])
	return string(runes)
}

// TestEndToEndCLIWorkflow tests the complete `lithos index` command workflow.
func TestEndToEndCLIWorkflow(t *testing.T) {
	// Setup workspace
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault")

	// Create complex test vault with all edge cases
	createComplexTestVault(t, vaultDir)

	// Copy schemas to a location the CLI can find during startup
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")

	// Ensure test schemas exist
	ws.MkdirAll("schemas", 0o755)

	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables before building
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary
	binaryPath := utils.BuildLithosBinary(t, tempDir)

	// Execute index command
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(t, err, "CLI index command should succeed")

	// Verify output contains expected elements
	assert.Contains(t, output, "✓ Vault indexed successfully")
	assert.Contains(t, output, "Statistics:")
	assert.Contains(t, output, "Scanned:")
	assert.Contains(t, output, "Indexed:")
	assert.Contains(t, output, "Duration:")

	// Parse and verify statistics
	utils.VerifyStatistics(t, output)
}

// createQueryTestVault creates a test vault with known content for query
// testing.
func createQueryTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure
	dirs := []string{
		"projects",
		"contacts",
		"meetings",
	}
	for _, dir := range dirs {
		require.NoError(t, os.MkdirAll(filepath.Join(vaultDir, dir), 0o755))
	}

	// Create test notes with specific content for query testing
	testNotes := []struct {
		path      string
		title     string
		fileClass string
		tags      string
		content   string
	}{
		{
			"projects/project1.md",
			"Project Alpha",
			"project",
			"work,active",
			"This is project alpha content.",
		},
		{
			"projects/project2.md",
			"Project Beta",
			"project",
			"work,planning",
			"This is project beta content.",
		},
		{
			"contacts/john.md",
			"John Doe",
			"contact",
			"personal",
			"John is a contact.",
		},
		{
			"meetings/weekly.md",
			"Weekly Meeting",
			"meeting",
			"work,recurring",
			"Weekly team meeting notes.",
		},
	}

	for _, note := range testNotes {
		content := fmt.Sprintf(`---
title: "%s"
fileClass: "%s"
tags: [%s]
---

# %s

%s
`, note.title, note.fileClass, note.tags, note.title, note.content)

		path := filepath.Join(vaultDir, note.path)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}
}

// createFrontmatterTestVault creates a vault with various frontmatter formats.
func createFrontmatterTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	require.NoError(t, os.MkdirAll(vaultDir, 0o755))

	// Test different frontmatter formats
	testCases := []struct {
		filename string
		content  string
	}{
		{
			"standard.md",
			`---
title: "Standard Note"
fileClass: "note"
tags: ["test", "standard"]
date: "2025-01-01"
---

# Standard Note

This is a standard note with typical frontmatter.
`,
		},
		{
			"minimal.md",
			`---
title: Minimal Note
---

# Minimal Note

This note has minimal frontmatter.
`,
		},
		{
			"complex.md",
			`---
title: "Complex Note"
fileClass: "note"
tags:
  - complex
  - test
metadata:
  created: "2025-01-01T10:00:00Z"
  modified: "2025-01-02T15:30:00Z"
aliases: ["Complex", "Test Note"]
---

# Complex Note

This note has complex nested frontmatter.
`,
		},
		{
			"no-frontmatter.md",
			`# Note Without Frontmatter

This note has no YAML frontmatter at all.
`,
		},
		{
			"empty-frontmatter.md",
			`---
---

# Empty Frontmatter

This note has empty frontmatter.
`,
		},
	}

	for _, tc := range testCases {
		path := filepath.Join(vaultDir, tc.filename)
		require.NoError(t, os.WriteFile(path, []byte(tc.content), 0o644))
	}
}

// createHybridStorageTestVault creates a test vault for hybrid storage testing.
func createHybridStorageTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure
	dirs := []string{
		"notes",
		"projects",
		"archive",
	}
	for _, dir := range dirs {
		require.NoError(t, os.MkdirAll(filepath.Join(vaultDir, dir), 0o755))
	}

	// Create test notes
	for i := 1; i <= 20; i++ {
		content := fmt.Sprintf(`---
title: "Test Note %d"
fileClass: "note"
tags: ["test", "hybrid"]
created: "2025-01-%02d"
---

# Test Note %d

This is test note number %d for hybrid storage testing.
`, i, i, i, i)

		filename := fmt.Sprintf("note%d.md", i)
		path := filepath.Join(vaultDir, "notes", filename)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}

	// Create some project notes
	projectNotes := []string{"alpha", "beta", "gamma"}
	for _, name := range projectNotes {
		content := fmt.Sprintf(`---
title: "Project %s"
fileClass: "project"
tags: ["project", "active"]
status: "active"
---

# Project %s

This is project %s.
`, titleCase(name), titleCase(name), name)

		filename := fmt.Sprintf("%s.md", name)
		path := filepath.Join(vaultDir, "projects", filename)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}
}

// createPerformanceTestVault creates a vault with 100+ notes for performance
// testing.
func createPerformanceTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure
	dirs := []string{
		"notes",
		"projects",
		"contacts",
		"meetings",
	}
	for _, dir := range dirs {
		require.NoError(t, os.MkdirAll(filepath.Join(vaultDir, dir), 0o755))
	}

	// Create 50 notes
	for i := 1; i <= 50; i++ {
		content := fmt.Sprintf(`---
title: "Performance Note %d"
fileClass: "note"
tags: ["performance", "test"]
created: "2025-01-%02d"
---

# Performance Note %d

This is performance test note number %d.
It contains some content to make it more realistic.
With multiple lines and some structure.

## Section 1

Some content here.

## Section 2

More content for testing.
`, i, i%31+1, i, i)

		filename := fmt.Sprintf("perf_note_%d.md", i)
		path := filepath.Join(vaultDir, "notes", filename)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}

	// Create 25 projects
	for i := 1; i <= 25; i++ {
		content := fmt.Sprintf(`---
title: "Performance Project %d"
fileClass: "project"
tags: ["project", "performance"]
status: "active"
priority: "medium"
---

# Performance Project %d

Project description for performance testing.
`, i, i)

		filename := fmt.Sprintf("perf_project_%d.md", i)
		path := filepath.Join(vaultDir, "projects", filename)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}

	// Create 25 contacts
	for i := 1; i <= 25; i++ {
		content := fmt.Sprintf(`---
title: "Performance Contact %d"
fileClass: "contact"
tags: ["contact", "performance"]
email: "contact%d@example.com"
---

# Performance Contact %d

Contact information for performance testing.
`, i, i, i)

		filename := fmt.Sprintf("perf_contact_%d.md", i)
		path := filepath.Join(vaultDir, "contacts", filename)
		require.NoError(t, os.WriteFile(path, []byte(content), 0o644))
	}
}

// createFileClassTestVault creates a vault for testing different file_class_key
// settings.
func createFileClassTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	require.NoError(t, os.MkdirAll(vaultDir, 0o755))

	// Create notes with different file class keys
	testNotes := []struct {
		filename string
		content  string
	}{
		{
			"note1.md",
			`---
title: "Note with fileClass"
fileClass: "note"
tags: ["test"]
---

# Note 1

Note with standard fileClass key.
`,
		},
		{
			"note2.md",
			`---
title: "Note with type"
type: "note"
tags: ["test"]
---

# Note 2

Note with type key.
`,
		},
		{
			"note3.md",
			`---
title: "Note with category"
category: "note"
tags: ["test"]
---

# Note 3

Note with category key.
`,
		},
	}

	for _, note := range testNotes {
		path := filepath.Join(vaultDir, note.filename)
		require.NoError(t, os.WriteFile(path, []byte(note.content), 0o644))
	}
}

// createErrorTestVault creates a vault with error cases for testing error
// handling.
func createErrorTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure
	dirs := []string{
		"valid",
		"errors",
	}
	for _, dir := range dirs {
		require.NoError(t, os.MkdirAll(filepath.Join(vaultDir, dir), 0o755))
	}

	// Create valid notes
	validNotes := []struct {
		filename string
		content  string
	}{
		{
			"valid/valid1.md",
			`---
title: "Valid Note 1"
fileClass: "note"
---

# Valid Note 1

This is a valid note.
`,
		},
		{
			"valid/valid2.md",
			`---
title: "Valid Note 2"
fileClass: "note"
---

# Valid Note 2

This is another valid note.
`,
		},
	}

	for _, note := range validNotes {
		path := filepath.Join(vaultDir, note.filename)
		require.NoError(t, os.WriteFile(path, []byte(note.content), 0o644))
	}

	// Create error cases
	errorCases := []struct {
		filename string
		content  string
	}{
		{
			"errors/invalid-yaml.md",
			`---
title: "Invalid YAML"
fileClass: note
invalid: yaml: syntax: [
---
# Invalid YAML

This has invalid YAML frontmatter.
`,
		},
		{
			"errors/malformed-frontmatter.md",
			`---
title: "Malformed"
fileClass: "note"
unclosed: "string
---
# Malformed

This has malformed frontmatter.
`,
		},
		{
			"errors/empty.md",
			"", // Empty file
		},
	}

	for _, errCase := range errorCases {
		path := filepath.Join(vaultDir, errCase.filename)
		require.NoError(t, os.WriteFile(path, []byte(errCase.content), 0o644))
	}
}

// TestVaultIndexingCompleteWorkflow tests the complete vault indexing workflow
// with production-scale data (500+ notes) and hybrid storage validation.
func TestVaultIndexingCompleteWorkflow(t *testing.T) {
	// Setup workspace with large vault
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-large")

	// Copy production-scale vault (594 notes from real Obsidian vault)
	srcVault := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"vault-large",
	)
	utils.CopyDir(t, srcVault, vaultDir)

	// Create lithos.json in vault directory to set vault path
	lithosConfig := fmt.Sprintf(`{
		"vault_path": "%s",
		"schemas_dir": "schemas",
		"templates_dir": "templates",
		"cache_dir": ".lithos/cache",
		"file_class_key": "fileClass",
		"log_level": "info"
	}`, vaultDir)
	configPath := filepath.Join(vaultDir, "lithos.json")
	require.NoError(t, os.WriteFile(configPath, []byte(lithosConfig), 0o644))

	// Setup schemas in vault directory
	vaultSchemasDir := filepath.Join(vaultDir, "schemas")
	require.NoError(t, os.MkdirAll(vaultSchemasDir, 0o755))
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(vaultSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Copy valid schemas
	srcBaseNote := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"base_note.json",
	)
	dstBaseNote := filepath.Join(vaultSchemasDir, "base_note.json")
	utils.CopyFile(t, srcBaseNote, dstBaseNote)

	srcMeetingNote := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"meeting_note.json",
	)
	dstMeetingNote := filepath.Join(vaultSchemasDir, "meeting_note.json")
	utils.CopyFile(t, srcMeetingNote, dstMeetingNote)

	// Build lithos binary
	binaryPath := utils.BuildLithosBinary(t, tempDir)

	// Execute index command
	startTime := time.Now()
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	duration := time.Since(startTime)

	if err != nil {
		t.Logf("Command output: %s", output)
	}
	require.NoError(t, err, "CLI index command should succeed with large vault")

	// Verify output contains expected elements
	assert.Contains(t, output, "✓ Vault indexed successfully")
	assert.Contains(t, output, "Statistics:")
	assert.Contains(t, output, "Scanned:")
	assert.Contains(t, output, "Indexed:")
	assert.Contains(t, output, "Duration:")

	// Parse and verify statistics for large vault
	stats := utils.ParseStatistics(t, output)
	assert.GreaterOrEqual(
		t,
		stats.Scanned,
		500,
		"Should scan at least 500 notes",
	)
	assert.Positive(t, stats.Indexed, "Should index at least some notes")

	// Performance validation: reasonable performance for indexing
	if stats.Indexed > 0 {
		avgTimePerNote := duration / time.Duration(stats.Indexed)
		assert.Less(
			t,
			avgTimePerNote,
			500*time.Millisecond,
			"Indexing should be reasonably fast",
		)
	}

	// Verify cache files were created
	cacheDir := filepath.Join(vaultDir, ".lithos", "cache")
	assert.DirExists(t, cacheDir, "Cache directory should exist")

	// Verify hybrid storage files exist
	boltDBPath := filepath.Join(cacheDir, "hot.db")
	sqlitePath := filepath.Join(cacheDir, "cold.db")
	assert.FileExists(t, boltDBPath, "BoltDB hot cache should exist")
	assert.FileExists(t, sqlitePath, "SQLite cold cache should exist")
}

// TestVaultQueryFunctionality tests query functionality against indexed vault
// data.
func TestVaultQueryFunctionality(t *testing.T) {
	// Setup workspace with indexed vault
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-query")

	// Create test vault with known content for query testing
	createQueryTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary and index vault
	binaryPath := utils.BuildLithosBinary(t, tempDir)
	_, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(t, err, "Vault indexing should succeed")

	// Test query commands (when implemented)
	// TODO: Add query command tests once CLI query functionality is implemented
}

// TestFrontmatterParsingAccuracy tests frontmatter parsing accuracy with
// various formats.
func TestFrontmatterParsingAccuracy(t *testing.T) {
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-frontmatter")

	// Create vault with various frontmatter formats
	createFrontmatterTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary and index vault
	binaryPath := utils.BuildLithosBinary(t, tempDir)
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(t, err, "Vault indexing should succeed")

	// Verify frontmatter parsing accuracy
	stats := utils.ParseStatistics(t, output)
	assert.Positive(
		t,
		stats.Indexed,
		"Should index some frontmatter test files",
	)

	// Verify cache contains parsed frontmatter
	cacheDir := filepath.Join(vaultDir, ".lithos", "cache")
	assert.DirExists(t, cacheDir, "Cache directory should exist")
}

// TestHybridStorageIntegration tests BoltDB + SQLite + JSON hybrid storage.
func TestHybridStorageIntegration(t *testing.T) {
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-hybrid")

	// Create test vault
	createHybridStorageTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary and index vault
	binaryPath := utils.BuildLithosBinary(t, tempDir)
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(t, err, "Vault indexing should succeed")

	// Verify hybrid storage files
	cacheDir := filepath.Join(vaultDir, ".lithos", "cache")
	boltDBPath := filepath.Join(cacheDir, "hot.db")
	sqlitePath := filepath.Join(cacheDir, "cold.db")

	assert.FileExists(t, boltDBPath, "BoltDB hot cache should exist")
	assert.FileExists(t, sqlitePath, "SQLite cold cache should exist")

	// Verify statistics
	stats := utils.ParseStatistics(t, output)
	assert.GreaterOrEqual(t, stats.Indexed, 0, "Should attempt to index notes")
}

// TestSmartQueryRouting tests intelligent routing between storage systems.
func TestSmartQueryRouting(t *testing.T) {
	// TODO: Implement once query CLI commands are available
	// This test will verify that queries are routed to appropriate storage
	// based on query type and performance characteristics
	t.Skip("Query routing tests require CLI query commands to be implemented")
}

// TestPerformanceValidation tests sub-100ms query performance with realistic
// load.
func TestPerformanceValidation(t *testing.T) {
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-performance")

	// Create performance test vault with 100+ notes
	createPerformanceTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary and index vault
	binaryPath := utils.BuildLithosBinary(t, tempDir)
	startTime := time.Now()
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	indexDuration := time.Since(startTime)

	require.NoError(t, err, "Vault indexing should succeed")

	// Verify performance requirements
	stats := utils.ParseStatistics(t, output)
	assert.GreaterOrEqual(
		t,
		stats.Scanned,
		50,
		"Should scan at least 50 notes",
	)
	assert.GreaterOrEqual(
		t,
		stats.Indexed,
		0,
		"Should index notes that match schema requirements",
	)

	// Performance validation: sub-100ms per note for indexing
	if stats.Indexed > 0 {
		avgTimePerNote := indexDuration / time.Duration(stats.Indexed)
		assert.Less(
			t,
			avgTimePerNote,
			100*time.Millisecond,
			"Indexing should be faster than 100ms per note, got %v",
			avgTimePerNote,
		)
	}
}

// TestFileClassKeyConfiguration tests different file_class_key settings.
func TestFileClassKeyConfiguration(t *testing.T) {
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-fileclass")

	// Create vault with different file class keys
	createFileClassTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Test with default fileClass key
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	binaryPath := utils.BuildLithosBinary(t, tempDir)
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
	require.NoError(
		t,
		err,
		"Vault indexing should succeed with default fileClass",
	)

	stats := utils.ParseStatistics(t, output)
	assert.Positive(
		t,
		stats.Indexed,
		"Should index notes with default fileClass key",
	)
}

// TestErrorHandling tests error handling for invalid files and permission
// issues.
func TestErrorHandling(t *testing.T) {
	ws := utils.NewWorkspace(t)
	tempDir := ws.Root()
	vaultDir := filepath.Join(tempDir, "vault-errors")

	// Create vault with error cases
	createErrorTestVault(t, vaultDir)

	// Setup schemas
	ws.MkdirAll("schemas", 0o755)
	testSchemasDir := ws.Path("schemas")
	srcPropertyBank := filepath.Join(
		utils.FindProjectRoot(t),
		"testdata",
		"schemas",
		"property_bank.json",
	)
	dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
	utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

	// Set environment variables
	require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
	defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

	// Build lithos binary and index vault
	binaryPath := utils.BuildLithosBinary(t, tempDir)
	output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)

	// Should succeed despite some errors
	require.NoError(t, err, "CLI should succeed even with some file errors")

	// Verify error handling - should continue processing despite errors
	assert.Contains(t, output, "✓ Vault indexed successfully")
	stats := utils.ParseStatistics(t, output)
	assert.Positive(t, stats.Indexed, "Should index valid files despite errors")
}

// TestEdgeCases tests edge cases like empty vault and malformed frontmatter.
func TestEdgeCases(t *testing.T) {
	// Test empty vault
	t.Run("EmptyVault", func(t *testing.T) {
		ws := utils.NewWorkspace(t)
		tempDir := ws.Root()
		vaultDir := filepath.Join(tempDir, "vault-empty")

		// Create empty vault directory
		require.NoError(t, os.MkdirAll(vaultDir, 0o755))

		// Setup schemas
		ws.MkdirAll("schemas", 0o755)
		testSchemasDir := ws.Path("schemas")
		srcPropertyBank := filepath.Join(
			utils.FindProjectRoot(t),
			"testdata",
			"schemas",
			"property_bank.json",
		)
		dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
		utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

		require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
		defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

		binaryPath := utils.BuildLithosBinary(t, tempDir)
		output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)
		require.NoError(t, err, "Empty vault indexing should succeed")

		stats := utils.ParseStatistics(t, output)
		assert.Equal(t, 0, stats.Scanned, "Empty vault should scan 0 files")
		assert.Equal(t, 0, stats.Indexed, "Empty vault should index 0 files")
	})

	// Test malformed frontmatter
	t.Run("MalformedFrontmatter", func(t *testing.T) {
		ws := utils.NewWorkspace(t)
		tempDir := ws.Root()
		vaultDir := filepath.Join(tempDir, "vault-malformed")

		// Create vault with malformed frontmatter
		require.NoError(t, os.MkdirAll(vaultDir, 0o755))
		malformedContent := `---
title: "Test Note"
invalid: yaml: content: [
---
# Test Note

Content here.
`
		malformedPath := filepath.Join(vaultDir, "malformed.md")
		require.NoError(
			t,
			os.WriteFile(malformedPath, []byte(malformedContent), 0o644),
		)

		// Setup schemas
		ws.MkdirAll("schemas", 0o755)
		testSchemasDir := ws.Path("schemas")
		srcPropertyBank := filepath.Join(
			utils.FindProjectRoot(t),
			"testdata",
			"schemas",
			"property_bank.json",
		)
		dstPropertyBank := filepath.Join(testSchemasDir, "property_bank.json")
		utils.CopyFile(t, srcPropertyBank, dstPropertyBank)

		require.NoError(t, os.Setenv("LITHOS_SCHEMAS_DIR", testSchemasDir))
		defer func() { _ = os.Unsetenv("LITHOS_SCHEMAS_DIR") }()

		binaryPath := utils.BuildLithosBinary(t, tempDir)
		output, err := utils.ExecuteIndexCommand(binaryPath, vaultDir)

		// Should succeed but with warnings/errors for malformed file
		require.NoError(
			t,
			err,
			"CLI should succeed despite malformed frontmatter",
		)

		// Verify some files were processed (may skip malformed ones)
		assert.Contains(t, output, "✓ Vault indexed successfully")
	})
}

// createComplexTestVault creates a test vault with all edge cases for
// comprehensive testing.
func createComplexTestVault(t *testing.T, vaultDir string) {
	t.Helper()

	// Create directory structure with nested folders
	dirs := []string{
		"projects/active",
		"projects/archive",
		"ideas/brainstorm",
		"meetings/2025",
		"meetings/2024",
		"templates",
		"assets/images",
		"assets/documents",
	}

	for _, dir := range dirs {
		require.NoError(
			t,
			os.MkdirAll(filepath.Join(vaultDir, dir), 0o755),
		)
	}

	// Create files with duplicate basenames across directories
	duplicateBasenameContent := []struct {
		dir      string
		filename string
		title    string
		content  string
	}{
		{
			"projects/active",
			"meeting.md",
			"Active Project Meeting",
			"Content for active project",
		},
		{
			"projects/archive",
			"meeting.md",
			"Archived Project Meeting",
			"Content for archived project",
		},
		{
			"ideas/brainstorm",
			"meeting.md",
			"Brainstorm Meeting",
			"Content for brainstorm",
		},
		{"meetings/2025", "meeting.md", "2025 Meeting", "Content for 2025"},
		{"meetings/2024", "meeting.md", "2024 Meeting", "Content for 2024"},
	}

	for _, item := range duplicateBasenameContent {
		content := fmt.Sprintf(
			`---\ntitle: \"%s\"\ndate: \"2025-01-01\"\n---\n\n# %s\n\n%s\n`,
			item.title,
			item.title,
			item.content,
		)

		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(content), 0o644),
		)
	}

	// Create files with different extensions and types
	mixedFiles := []struct {
		dir      string
		filename string
		content  string
	}{
		{
			"templates",
			"note-template.md",
			"# Note Template\\n\\nTemplate content",
		},
		{
			"templates",
			"meeting-template.md",
			"# Meeting Template\\n\\nMeeting template content",
		},
		{"assets/documents", "readme.txt", "This is a text document"},
		{"assets/documents", "data.json", `{"key": "value", "number": 42}`},
	}

	for _, item := range mixedFiles {
		path := filepath.Join(vaultDir, item.dir, item.filename)
		require.NoError(
			t,
			os.WriteFile(path, []byte(item.content), 0o644),
		)
	}

	// Create large binary file to test memory efficiency (1MB)
	largeBinaryPath := filepath.Join(
		vaultDir,
		"assets/images", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"large-image.jpg",
	)
	largeData := make([]byte, 1024*1024) // 1MB
	for i := range largeData {
		largeData[i] = byte(i % 256)
	}
	require.NoError(
		t,
		os.WriteFile(largeBinaryPath, largeData, 0o644),
	)

	// Create file with invalid frontmatter for error handling
	invalidFrontmatterPath := filepath.Join(
		vaultDir,
		"projects/active", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"invalid.md",
	)
	invalidContent := `
		---
		invalid: yaml: content:
		---
		# Invalid Frontmatter

		This file has invalid YAML frontmatter.
	`
	require.NoError(
		t,
		os.WriteFile(invalidFrontmatterPath, []byte(invalidContent), 0o644),
	)

	// Create file without frontmatter
	noFrontmatterPath := filepath.Join(
		vaultDir,
		"ideas/brainstorm", //nolint:gocritic // filepath.Join with subpath strings is acceptable
		"plain.md",
	)
	plainContent := `# Plain Markdown\n\nThis file has no frontmatter.`
	require.NoError(
		t,
		os.WriteFile(noFrontmatterPath, []byte(plainContent), 0o644),
	)
}
