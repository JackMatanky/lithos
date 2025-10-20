package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestNewConfig(t *testing.T) {
	tests := []struct {
		name       string
		vaultPath  string
		wantFields map[string]string // field name to expected value pattern
	}{
		{
			name:      "empty vault path uses current directory",
			vaultPath: "",
			wantFields: map[string]string{
				"LogLevel": "info",
			},
		},
		{
			name:      "explicit vault path",
			vaultPath: "/tmp/test-vault",
			wantFields: map[string]string{
				"VaultPath":    "/tmp/test-vault",
				"TemplatesDir": "/tmp/test-vault/templates",
				"SchemasDir":   "/tmp/test-vault/schemas",
				"CacheDir":     "/tmp/test-vault/.lithos/cache",
				"LogLevel":     "info",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := NewConfig(tt.vaultPath)

			checkFieldExpectations(t, config, tt.wantFields)
			verifyConfigPaths(t, config)
		})
	}
}

// logic.
//
//nolint:gocognit,nestif // test function with multiple test cases and nested
func TestConfig_Validate(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-config-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	tests := []struct {
		name    string
		config  *Config
		wantErr bool
		errMsg  string
	}{
		{
			name: "valid config with existing vault path",
			config: &Config{
				VaultPath:    tempDir,
				TemplatesDir: filepath.Join(tempDir, "templates"),
				SchemasDir:   filepath.Join(tempDir, "schemas"),
				CacheDir:     filepath.Join(tempDir, ".lithos", "cache"),
				LogLevel:     "info",
			},
			wantErr: false,
		},
		{
			name: "empty vault path",
			config: &Config{
				VaultPath: "",
				LogLevel:  "info",
			},
			wantErr: true,
			errMsg:  "vault path cannot be empty",
		},
		{
			name: "non-existent vault path",
			config: &Config{
				VaultPath: "/non/existent/path",
				LogLevel:  "info",
			},
			wantErr: true,
			errMsg:  "vault path does not exist",
		},
		{
			name: "invalid log level",
			config: &Config{
				VaultPath: tempDir,
				LogLevel:  "invalid",
			},
			wantErr: true,
			errMsg:  "invalid log level",
		},
		{
			name: "log level normalization",
			config: &Config{
				VaultPath: tempDir,
				LogLevel:  "INFO", // Should be normalized to lowercase
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			originalLogLevel := tt.config.LogLevel
			validationErr := tt.config.Validate()

			if tt.wantErr {
				if validationErr == nil {
					t.Errorf(
						"Config.Validate() expected error containing %q, got nil",
						tt.errMsg,
					)
					return
				}
				if tt.errMsg != "" &&
					!contains(validationErr.Error(), tt.errMsg) {
					t.Errorf(
						"Config.Validate() error = %q, want error containing %q",
						validationErr.Error(),
						tt.errMsg,
					)
				}
			} else {
				if validationErr != nil {
					t.Errorf("Config.Validate() unexpected error = %v", validationErr)
				}

				// Check log level normalization
				if originalLogLevel == "INFO" && tt.config.LogLevel != "info" {
					t.Errorf("Config.Validate() should normalize log level to lowercase, got %q",
						tt.config.LogLevel)
				}
			}
		})
	}
}

// checkFieldExpectations verifies that config fields match expected values.
func checkFieldExpectations(
	t *testing.T,
	config *Config,
	wantFields map[string]string,
) {
	t.Helper()

	for field, expected := range wantFields {
		actual := getConfigField(config, field)

		if expected != "" && actual != expected {
			t.Errorf(
				"NewConfig() field %s = %q, want %q",
				field,
				actual,
				expected,
			)
		}
	}
}

// getConfigField returns the value of a config field by name.
func getConfigField(config *Config, field string) string {
	switch field {
	case "VaultPath":
		return config.VaultPath
	case "TemplatesDir":
		return config.TemplatesDir
	case "SchemasDir":
		return config.SchemasDir
	case "CacheDir":
		return config.CacheDir
	case "LogLevel":
		return config.LogLevel
	default:
		return ""
	}
}

// verifyConfigPaths checks that all config paths are properly set and valid.
func verifyConfigPaths(t *testing.T, config *Config) {
	t.Helper()

	// Verify VaultPath is absolute
	if !filepath.IsAbs(config.VaultPath) {
		t.Errorf(
			"NewConfig() VaultPath should be absolute, got %q",
			config.VaultPath,
		)
	}

	// Verify all paths are set
	if config.VaultPath == "" {
		t.Error("NewConfig() VaultPath should not be empty")
	}
	if config.TemplatesDir == "" {
		t.Error("NewConfig() TemplatesDir should not be empty")
	}
	if config.SchemasDir == "" {
		t.Error("NewConfig() SchemasDir should not be empty")
	}
	if config.CacheDir == "" {
		t.Error("NewConfig() CacheDir should not be empty")
	}
	if config.LogLevel == "" {
		t.Error("NewConfig() LogLevel should not be empty")
	}
}

//nolint:nestif // test function with nested error checking logic
func TestConfig_validateVaultPath(t *testing.T) {
	// Create a temporary directory and file for testing
	tempDir, err := os.MkdirTemp("", "lithos-config-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	tempFile := filepath.Join(tempDir, "testfile")
	if writeErr := os.WriteFile(tempFile, []byte("test"), 0o600); writeErr != nil {
		t.Fatalf("Failed to create temp file: %v", writeErr)
	}

	tests := []struct {
		name      string
		vaultPath string
		wantErr   bool
		errMsg    string
	}{
		{
			name:      "valid directory",
			vaultPath: tempDir,
			wantErr:   false,
		},
		{
			name:      "empty path",
			vaultPath: "",
			wantErr:   true,
			errMsg:    "vault path cannot be empty",
		},
		{
			name:      "non-existent path",
			vaultPath: "/non/existent/path",
			wantErr:   true,
			errMsg:    "vault path does not exist",
		},
		{
			name:      "path is file not directory",
			vaultPath: tempFile,
			wantErr:   true,
			errMsg:    "vault path is not a directory",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := &Config{VaultPath: tt.vaultPath}
			validateErr := config.validateVaultPath()

			if tt.wantErr {
				if validateErr == nil {
					t.Errorf(
						"validateVaultPath() expected error containing %q, got nil",
						tt.errMsg,
					)
					return
				}
				if tt.errMsg != "" &&
					!contains(validateErr.Error(), tt.errMsg) {
					t.Errorf(
						"validateVaultPath() error = %q, want error containing %q",
						validateErr.Error(),
						tt.errMsg,
					)
				}
			} else if validateErr != nil {
				t.Errorf("validateVaultPath() unexpected error = %v", validateErr)
			}
		})
	}
}

//nolint:nestif // test function with nested error checking logic
func TestConfig_validateLogLevel(t *testing.T) {
	tests := []struct {
		name     string
		logLevel string
		wantErr  bool
		expected string // expected normalized value
	}{
		{
			name:     "valid lowercase",
			logLevel: "info",
			wantErr:  false,
			expected: "info",
		},
		{
			name:     "valid uppercase - should normalize",
			logLevel: "DEBUG",
			wantErr:  false,
			expected: "debug",
		},
		{
			name:     "valid mixed case - should normalize",
			logLevel: "WaRn",
			wantErr:  false,
			expected: "warn",
		},
		{
			name:     "invalid level",
			logLevel: "invalid",
			wantErr:  true,
		},
		{
			name:     "empty level",
			logLevel: "",
			wantErr:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := &Config{LogLevel: tt.logLevel}
			err := config.validateLogLevel()

			if tt.wantErr {
				if err == nil {
					t.Error("validateLogLevel() expected error, got nil")
				}
			} else {
				if err != nil {
					t.Errorf("validateLogLevel() unexpected error = %v", err)
				}
				if config.LogLevel != tt.expected {
					t.Errorf("validateLogLevel() normalized level = %q, want %q", config.LogLevel, tt.expected)
				}
			}
		})
	}
}

func TestConfig_ResolvePath(t *testing.T) {
	config := &Config{
		VaultPath: "/vault/root",
	}

	tests := []struct {
		name     string
		path     string
		expected string
	}{
		{
			name:     "absolute path unchanged",
			path:     "/absolute/path",
			expected: "/absolute/path",
		},
		{
			name:     "relative path resolved",
			path:     "relative/path",
			expected: "/vault/root/relative/path",
		},
		{
			name:     "empty path resolved to vault root",
			path:     "",
			expected: "/vault/root",
		},
		{
			name:     "dot path resolved to vault root",
			path:     ".",
			expected: "/vault/root",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := config.ResolvePath(tt.path)
			if result != tt.expected {
				t.Errorf(
					"ResolvePath(%q) = %q, want %q",
					tt.path,
					result,
					tt.expected,
				)
			}
		})
	}
}

// Helper function to check if a string contains a substring.
func contains(s, substr string) bool {
	return substr == "" ||
		(len(s) >= len(substr) && s[len(s)-len(substr):] == substr) ||
		(len(s) > len(substr) && s[:len(substr)] == substr) ||
		(len(s) > len(substr) && findSubstring(s, substr))
}

func findSubstring(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
