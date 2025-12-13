package schema

import (
	"context"
	"errors"
	"path/filepath"
	"strings"
	"testing"

	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

const (
	testResourceFilesystem = "filesystem"
	testOperationWalk      = "walk"
	testSchemaName         = "user"
)

func TestNewSchemaLoaderAdapter(t *testing.T) {
	fs := newMockFileSystemPort()
	cfg := newMockConfigPort("/test/vault")

	adapter := NewSchemaLoaderAdapter(fs, cfg)

	if adapter == nil {
		t.Fatal("NewSchemaLoaderAdapter returned nil")
	}
	if adapter.fs != fs {
		t.Error("FileSystemPort not set correctly")
	}
	if adapter.config != cfg {
		t.Error("ConfigPort not set correctly")
	}
}

func TestLoadSchemas_Success(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup test data
	validSchema, err := loadTestData("valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}
	setupSchemaFile(fs, cfg, "user.json", validSchema)

	// Execute
	ctx := context.Background()
	schemas, err := adapter.LoadSchemas(ctx)

	// Verify
	if err != nil {
		t.Fatalf("LoadSchemas failed: %v", err)
	}
	if len(schemas) != 1 {
		t.Fatalf("Expected 1 schema, got %d", len(schemas))
	}

	schema := schemas[0]
	if schema.Name != testSchemaName {
		t.Errorf(
			"Expected schema name '%s', got '%s'",
			testSchemaName,
			schema.Name,
		)
	}
	if schema.Extends != "base" {
		t.Errorf("Expected extends 'base', got '%s'", schema.Extends)
	}
	if len(schema.Excludes) != 1 || schema.Excludes[0] != "internal_id" {
		t.Errorf("Expected excludes ['internal_id'], got %v", schema.Excludes)
	}
	if len(schema.Properties) != 3 {
		t.Errorf("Expected 3 properties, got %d", len(schema.Properties))
	}
}

func TestLoadSchemas_FileSystemError(t *testing.T) {
	adapter, fs, _ := createTestAdapter()

	// Setup walk error
	fs.SetWalkError(errors.New("permission denied"))

	// Execute
	ctx := context.Background()
	_, err := adapter.LoadSchemas(ctx)

	// Verify error
	if err == nil {
		t.Fatal("Expected error for filesystem failure")
	}

	var resErr sharederrors.ResourceError
	if !errors.As(err, &resErr) {
		t.Fatalf("Expected ResourceError, got %T", err)
	}
	if resErr.Resource() != testResourceFilesystem ||
		resErr.Operation() != testOperationWalk {
		t.Fatalf("unexpected resource metadata: %+v", resErr)
	}
}

func TestLoadSchemas_SkipsNonJSONFiles(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup mixed file types
	validSchema, err := loadTestData("valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}
	setupSchemaFile(fs, cfg, "user.json", validSchema)
	txtPath := filepath.Join(cfg.Config().SchemasDir, "readme.txt")

	fs.AddFile(txtPath, []byte("some text"))
	fs.AddWalkPath(txtPath)

	// Execute
	ctx := context.Background()
	schemas, err := adapter.LoadSchemas(ctx)

	// Verify only JSON file was processed
	if err != nil {
		t.Fatalf("LoadSchemas failed: %v", err)
	}
	if len(schemas) != 1 {
		t.Errorf("Expected 1 schema (only JSON file), got %d", len(schemas))
	}
}

func TestLoadSchemas_SkipsPropertyBankFiles(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup schema and property bank files
	validSchema, err := loadTestData("valid/complete-user.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}
	validPropertyBank, err := loadTestData("properties/bank.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}
	setupSchemaFile(fs, cfg, "user.json", validSchema)
	setupPropertyBankFile(fs, cfg, "common.json", validPropertyBank)

	// Execute
	ctx := context.Background()
	schemas, err := adapter.LoadSchemas(ctx)

	// Verify only schema file was processed
	if err != nil {
		t.Fatalf("LoadSchemas failed: %v", err)
	}
	if len(schemas) != 1 {
		t.Errorf("Expected 1 schema (skip property bank), got %d", len(schemas))
	}
	if schemas[0].Name != "user" {
		t.Errorf("Expected user schema, got %s", schemas[0].Name)
	}
}

func TestLoadPropertyBank_Success(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup property bank file
	validPropertyBank, err := loadTestData("properties/bank.json")
	if err != nil {
		t.Fatalf("Failed to load test data: %v", err)
	}
	setupPropertyBankFile(fs, cfg, "common.json", validPropertyBank)

	// Execute
	ctx := context.Background()
	bank, err := adapter.LoadPropertyBank(ctx)

	// Verify
	if err != nil {
		t.Fatalf("LoadPropertyBank failed: %v", err)
	}

	if _, exists := bank.Properties["common-email"]; !exists {
		t.Error("Expected property 'common-email' in bank")
	}
	if _, exists := bank.Properties["user-profile"]; !exists {
		t.Error("Expected property 'user-profile' in bank")
	}

	// Verify property details
	emailProp, exists := bank.Properties["common-email"]
	if !exists {
		t.Fatal("common-email property not found")
	}
	if !emailProp.Required {
		t.Error("Expected common-email to be required")
	}
}

func TestLoadPropertyBank_MalformedJSON(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup malformed property bank
	setupPropertyBankFile(
		fs,
		cfg,
		"malformed.json",
		`{"properties": {"invalid": {`,
	)

	// Execute
	ctx := context.Background()
	_, err := adapter.LoadPropertyBank(ctx)

	// Verify error - wrapped error from Walk function will be ResourceError
	if err == nil {
		t.Fatal("Expected error for malformed property bank JSON")
	}

	// Should be wrapped in ResourceError since it comes from Walk callback
	var resErr sharederrors.ResourceError
	if !errors.As(err, &resErr) {
		t.Fatalf("Expected ResourceError, got %T", err)
	}
	if resErr.Resource() != testResourceFilesystem {
		t.Fatalf("unexpected resource error domain: %+v", resErr)
	}

	// But the underlying error should contain property bank parsing info
	if !strings.Contains(err.Error(), "malformed JSON") {
		t.Errorf("Expected malformed JSON error message, got: %s", err.Error())
	}
}

// =============================================================================
// Security Validation Tests
// =============================================================================

func TestSecurity_PathTraversalPrevention(t *testing.T) {
	adapter, _, cfg := createTestAdapter()

	tests := []struct {
		name     string
		filePath string
		wantErr  bool
	}{
		{
			"valid path",
			filepath.Join(cfg.Config().SchemasDir, "user.json"),
			false,
		},
		{
			"path traversal with ..",
			cfg.Config().SchemasDir + "/../../../etc/passwd.json",
			true,
		},
		{"path traversal at start", "../../../etc/passwd.json", true},
		{
			"path traversal in middle",
			cfg.Config().SchemasDir + "/valid/../../../etc/passwd.json",
			true,
		},
		{
			"encoded traversal",
			cfg.Config().SchemasDir + "/..%2F..%2Fetc/passwd.json",
			true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := adapter.validateFilePath(
				tt.filePath,
				cfg.Config().SchemasDir,
			)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"validateFilePath(%q) error = %v, wantErr %v",
					tt.filePath,
					err,
					tt.wantErr,
				)
			}
			if tt.wantErr && err != nil &&
				!strings.Contains(err.Error(), "traversal") {
				t.Errorf("Expected traversal error, got: %s", err.Error())
			}
		})
	}
}

func TestSecurity_DirectoryBoundsChecking(t *testing.T) {
	adapter, _, cfg := createTestAdapter()

	tests := []struct {
		name     string
		filePath string
		wantErr  bool
	}{
		{
			"within bounds",
			filepath.Join(cfg.Config().SchemasDir, "user.json"),
			false,
		},
		{"outside bounds", "/etc/passwd", true},
		{"outside bounds absolute", "/tmp/malicious.json", true},
		{
			"sibling directory",
			filepath.Join(
				cfg.Config().VaultPath,
				"../other", //nolint:gocritic // intentional path traversal test case
				"file.json",
			),
			true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := adapter.validateFilePath(
				tt.filePath,
				cfg.Config().SchemasDir,
			)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"validateFilePath(%q) error = %v, wantErr %v",
					tt.filePath,
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestSecurity_FileExtensionValidation(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	tests := []struct {
		name     string
		filePath string
		wantErr  bool
	}{
		{"valid json", "user.json", false},
		{"uppercase JSON", "user.JSON", false},
		{"invalid txt", "user.txt", true},
		{"invalid executable", "user.exe", true},
		{"no extension", "user", true},
		{"empty string", "", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := adapter.checkFileExtension(tt.filePath)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"checkFileExtension(%q) error = %v, wantErr %v",
					tt.filePath,
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestSecurity_FileSizeLimits(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test data sizes
	smallData := []byte(`{"name": "test"}`) // ~18 bytes
	largeData := make([]byte, 11*1024*1024) // 11MB, over 10MB limit

	tests := []struct {
		name    string
		data    []byte
		wantErr bool
	}{
		{"small file", smallData, false},
		{"exactly 10MB", make([]byte, 10*1024*1024), false},
		{"over 10MB", largeData, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := adapter.validateFileSize("test.json", tt.data)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"validateFileSize() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
			if tt.wantErr && err != nil &&
				!strings.Contains(err.Error(), "exceeds maximum") {
				t.Errorf("Expected size limit error, got: %s", err.Error())
			}
		})
	}
}

func TestSecurity_CircularReferenceDetection(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	// Test exceeding max depth
	err := adapter.checkCircularReferenceDepth("test-prop", 11)
	if err == nil {
		t.Error("Expected max depth exceeded error")
	}
	if !strings.Contains(err.Error(), "max depth exceeded") {
		t.Errorf("Expected max depth error message, got: %s", err.Error())
	}

	// Test valid depth
	err = adapter.checkCircularReferenceDepth("test-prop", 5)
	if err != nil {
		t.Errorf("Expected no error for valid depth, got: %v", err)
	}
}

func TestSecurity_SelfReferencePrevention(t *testing.T) {
	adapter, _, _ := createTestAdapter()

	tests := []struct {
		name    string
		refName string
		wantErr bool
	}{
		{"no self reference", "other-prop", false},
		{"self reference", "user-profile", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := adapter.checkSelfReference("user-profile", tt.refName)
			if (err != nil) != tt.wantErr {
				t.Errorf(
					"checkSelfReference() error = %v, wantErr %v",
					err,
					tt.wantErr,
				)
			}
		})
	}
}

func TestSecurity_LoadSchemas_PathTraversalBlocked(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup malicious file
	maliciousPath := filepath.Join(
		cfg.Config().SchemasDir,
		"..",
		"..",
		"etc",
		"passwd",
	)
	fs.AddFile(maliciousPath, []byte(`{"name": "malicious"}`))
	fs.AddWalkPath(maliciousPath)

	// Execute
	ctx := context.Background()
	schemas, err := adapter.LoadSchemas(ctx)

	// Should succeed but skip the malicious file
	if err != nil {
		t.Fatalf("LoadSchemas should not fail on path traversal, got: %v", err)
	}

	// Should not load any schemas (malicious file should be skipped)
	if len(schemas) != 0 {
		t.Errorf(
			"Expected 0 schemas (malicious file skipped), got %d",
			len(schemas),
		)
	}
}

func TestSecurity_LoadPropertyBank_PathTraversalBlocked(t *testing.T) {
	adapter, fs, cfg := createTestAdapter()

	// Setup malicious property file
	maliciousPath := filepath.Join(
		cfg.Config().SchemasDir,
		"properties",
		"..",
		"..",
		"etc",
		"shadow",
	)
	fs.AddFile(maliciousPath, []byte(`{"properties": {}}`))
	fs.AddWalkPath(maliciousPath)

	// Execute
	ctx := context.Background()
	bank, err := adapter.LoadPropertyBank(ctx)

	// Should succeed but return empty bank
	if err != nil {
		t.Fatalf(
			"LoadPropertyBank should not fail on path traversal, got: %v",
			err,
		)
	}

	// Should have empty properties
	if len(bank.Properties) != 0 {
		t.Errorf(
			"Expected empty property bank, got %d properties",
			len(bank.Properties),
		)
	}
}
