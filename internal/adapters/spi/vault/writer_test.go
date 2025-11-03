package vault

import (
	"bytes"
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	lithoslog "github.com/JackMatanky/lithos/internal/shared/logger"
)

const testNoteFilename = "test-note.md"

// Compile-time check to ensure VaultWriterAdapter implements VaultWriterPort.
var _ spi.VaultWriterPort = (*VaultWriterAdapter)(nil)

// TestNewVaultWriterAdapter tests the constructor for VaultWriterAdapter.
func TestNewVaultWriterAdapter(t *testing.T) {
	// This test will fail until NewVaultWriterAdapter is implemented
	// It verifies the constructor creates a valid adapter that implements
	// VaultWriterPort

	// Test data
	config := domain.Config{VaultPath: "/tmp/test-vault"}
	log := lithoslog.NewTest()

	// Call constructor (will fail until implemented)
	adapter := NewVaultWriterAdapter(config, log)

	// Verify config is stored
	if adapter == nil {
		t.Error("NewVaultWriterAdapter should return a non-nil adapter")
	}
}

// TestPersistCreatesNewFile tests that Persist creates a new file with note
// content.
func TestPersistCreatesNewFile(t *testing.T) {
	// Setup temporary directory
	tempDir, err := os.MkdirTemp("", "vault-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Test data
	config := domain.Config{VaultPath: tempDir}
	log := lithoslog.NewTest()
	adapter := NewVaultWriterAdapter(config, log)

	frontmatter := domain.NewFrontmatter(map[string]interface{}{
		"title":   "Test Note",
		"tags":    []string{"test"},
		"content": "This is test content",
	})
	note := domain.NewNote(domain.NewNoteID("test-note"), time.Now(), frontmatter)
	path := testNoteFilename

	// Call Persist (will fail until implemented)
	err = adapter.Persist(context.Background(), note, path)
	if err != nil {
		t.Fatalf("Persist should not return error: %v", err)
	}

	// Verify file was created
	fullPath := filepath.Join(tempDir, path)
	if _, statErr := os.Stat(fullPath); os.IsNotExist(statErr) {
		t.Error("Persist should create the file")
	}

	// Verify file content (basic check)
	content, err := os.ReadFile(fullPath)
	if err != nil {
		t.Fatalf("Failed to read created file: %v", err)
	}
	if len(content) == 0 {
		t.Error("File should contain note content")
	}
}

// TestDeleteRemovesFile tests that Delete removes a file idempotently.
func TestDeleteRemovesFile(t *testing.T) {
	// Setup temporary directory and create a test file
	tempDir, err := os.MkdirTemp("", "vault-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Create test file
	testPath := filepath.Join(tempDir, "test-note.md")
	testContent := []byte("test content")
	if writeErr := os.WriteFile(testPath, testContent, 0o600); writeErr != nil {
		t.Fatalf("Failed to create test file: %v", writeErr)
	}

	// Test data
	config := domain.Config{VaultPath: tempDir}
	log := lithoslog.NewTest()
	adapter := NewVaultWriterAdapter(config, log)

	relativePath := testNoteFilename

	// Verify file exists before delete
	if _, statErr := os.Stat(testPath); os.IsNotExist(statErr) {
		t.Fatal("Test file should exist before delete")
	}

	// Call Delete (will fail until implemented)
	err = adapter.Delete(context.Background(), relativePath)
	if err != nil {
		t.Fatalf("Delete should not return error: %v", err)
	}

	// Verify file is removed
	if _, statErr := os.Stat(testPath); !os.IsNotExist(statErr) {
		t.Error("Delete should remove the file")
	}

	// Test idempotency - delete again should not error
	err = adapter.Delete(context.Background(), relativePath)
	if err != nil {
		t.Errorf("Delete should be idempotent, got error: %v", err)
	}
}

// TestPersistOverwritesExistingFile tests that Persist overwrites an existing
// file.
func TestPersistOverwritesExistingFile(t *testing.T) {
	// Setup temporary directory
	tempDir, err := os.MkdirTemp("", "vault-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Pre-create file with old content
	testPath := filepath.Join(tempDir, testNoteFilename)
	oldContent := []byte("old content")
	if writeErr := os.WriteFile(testPath, oldContent, 0o600); writeErr != nil {
		t.Fatalf("Failed to create test file: %v", writeErr)
	}

	// Test data
	config := domain.Config{VaultPath: tempDir}
	log := lithoslog.NewTest()
	adapter := NewVaultWriterAdapter(config, log)

	note := domain.NewNote(domain.NewNoteID("test-note"),
		time.Now(),
		domain.NewFrontmatter(map[string]interface{}{
			"title":   "Updated Note",
			"content": "new content",
		}))

	// Call Persist
	err = adapter.Persist(context.Background(), note, testNoteFilename)
	if err != nil {
		t.Fatalf("Persist should not return error: %v", err)
	}

	// Verify file content was overwritten
	content, err := os.ReadFile(testPath)
	if err != nil {
		t.Fatalf("Failed to read file: %v", err)
	}
	if !bytes.Contains(content, []byte("new content")) {
		t.Error("File should contain new content")
	}
	if bytes.Contains(content, []byte("old content")) {
		t.Error("File should not contain old content")
	}
}

// TestPersistCreatesParentDirectories tests that Persist creates parent
// directories.
func TestPersistCreatesParentDirectories(t *testing.T) {
	// Setup temporary directory
	tempDir, err := os.MkdirTemp("", "vault-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Test data
	config := domain.Config{VaultPath: tempDir}
	log := lithoslog.NewTest()
	adapter := NewVaultWriterAdapter(config, log)

	nestedPath := "contacts/work/alice.md"
	note := domain.NewNote(domain.NewNoteID("test-note"),
		time.Now(),
		domain.NewFrontmatter(map[string]interface{}{
			"title":   "Alice Smith",
			"content": "Contact info",
		}))

	// Call Persist
	err = adapter.Persist(context.Background(), note, nestedPath)
	if err != nil {
		t.Fatalf("Persist should not return error: %v", err)
	}

	// Verify file was created with nested directories
	fullPath := filepath.Join(tempDir, nestedPath)
	if _, statErr := os.Stat(fullPath); os.IsNotExist(statErr) {
		t.Error("Persist should create file with nested directories")
	}

	// Verify parent directory exists
	parentDir := filepath.Dir(fullPath)
	if _, statErr := os.Stat(parentDir); os.IsNotExist(statErr) {
		t.Error("Persist should create parent directories")
	}
}

// TestPersistPreservesFrontmatter tests that Persist preserves all frontmatter
// fields (FR6).
func TestPersistPreservesFrontmatter(t *testing.T) {
	// Setup temporary directory
	tempDir, err := os.MkdirTemp("", "vault-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Test data
	config := domain.Config{VaultPath: tempDir}
	log := lithoslog.NewTest()
	adapter := NewVaultWriterAdapter(config, log)

	// Note with custom frontmatter fields
	note := domain.NewNote(domain.NewNoteID("test-note"),
		time.Now(),
		domain.NewFrontmatter(map[string]interface{}{
			"fileClass":    "contact",
			"title":        "Test Contact",
			"custom_field": "preserved value",
			"tags":         []string{"test", "contact"},
			"metadata": map[string]interface{}{
				"created": "2023-01-01",
			},
		}))

	// Call Persist
	err = adapter.Persist(context.Background(), note, testNoteFilename)
	if err != nil {
		t.Fatalf("Persist should not return error: %v", err)
	}

	// Verify file content preserves all fields
	content, err := os.ReadFile(filepath.Join(tempDir, testNoteFilename))
	if err != nil {
		t.Fatalf("Failed to read file: %v", err)
	}

	contentStr := string(content)
	if !strings.Contains(contentStr, "custom_field: preserved value") {
		t.Error("Custom field should be preserved")
	}
	if !strings.Contains(contentStr, "fileClass: contact") {
		t.Error("fileClass field should be preserved")
	}
	if !strings.Contains(contentStr, "created: \"2023-01-01\"") {
		t.Error("Nested metadata should be preserved")
	}
}
