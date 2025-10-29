package domain

import (
	"encoding/json"
	"testing"
)

// TestNewConfig tests that NewConfig applies defaults correctly.
func TestNewConfig(t *testing.T) {
	tests := []struct {
		name                     string
		inputVaultPath           string
		inputTemplatesDir        string
		inputSchemasDir          string
		inputPropertyBankFile    string
		inputCacheDir            string
		inputLogLevel            string
		expectedVaultPath        string
		expectedTemplatesDir     string
		expectedSchemasDir       string
		expectedPropertyBankFile string
		expectedCacheDir         string
		expectedLogLevel         string
	}{
		{
			name:                     "all defaults applied",
			inputVaultPath:           "",
			inputTemplatesDir:        "",
			inputSchemasDir:          "",
			inputPropertyBankFile:    "",
			inputCacheDir:            "",
			inputLogLevel:            "",
			expectedVaultPath:        ".",
			expectedTemplatesDir:     "templates",
			expectedSchemasDir:       "schemas",
			expectedPropertyBankFile: "property_bank.json",
			expectedCacheDir:         ".lithos/cache",
			expectedLogLevel:         "info",
		},
		{
			name:                     "partial defaults applied",
			inputVaultPath:           "/custom/vault",
			inputTemplatesDir:        "",
			inputSchemasDir:          "",
			inputPropertyBankFile:    "custom.json",
			inputCacheDir:            "",
			inputLogLevel:            "debug",
			expectedVaultPath:        "/custom/vault",
			expectedTemplatesDir:     "/custom/vault/templates",
			expectedSchemasDir:       "/custom/vault/schemas",
			expectedPropertyBankFile: "custom.json",
			expectedCacheDir:         "/custom/vault/.lithos/cache",
			expectedLogLevel:         "debug",
		},
		{
			name:                     "no defaults needed",
			inputVaultPath:           "/home/user/vault",
			inputTemplatesDir:        "custom/templates/",
			inputSchemasDir:          "custom/schemas/",
			inputPropertyBankFile:    "custom_bank.json",
			inputCacheDir:            "/tmp/cache/",
			inputLogLevel:            "debug",
			expectedVaultPath:        "/home/user/vault",
			expectedTemplatesDir:     "custom/templates/",
			expectedSchemasDir:       "custom/schemas/",
			expectedPropertyBankFile: "custom_bank.json",
			expectedCacheDir:         "/tmp/cache/",
			expectedLogLevel:         "debug",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := NewConfig(
				tt.inputVaultPath,
				tt.inputTemplatesDir,
				tt.inputSchemasDir,
				tt.inputPropertyBankFile,
				tt.inputCacheDir,
				tt.inputLogLevel,
			)

			if config.VaultPath != tt.expectedVaultPath {
				t.Errorf(
					"expected VaultPath %q, got %q",
					tt.expectedVaultPath,
					config.VaultPath,
				)
			}
			if config.TemplatesDir != tt.expectedTemplatesDir {
				t.Errorf(
					"expected TemplatesDir %q, got %q",
					tt.expectedTemplatesDir,
					config.TemplatesDir,
				)
			}
			if config.SchemasDir != tt.expectedSchemasDir {
				t.Errorf(
					"expected SchemasDir %q, got %q",
					tt.expectedSchemasDir,
					config.SchemasDir,
				)
			}
			if config.PropertyBankFile != tt.expectedPropertyBankFile {
				t.Errorf(
					"expected PropertyBankFile %q, got %q",
					tt.expectedPropertyBankFile,
					config.PropertyBankFile,
				)
			}
			if config.CacheDir != tt.expectedCacheDir {
				t.Errorf(
					"expected CacheDir %q, got %q",
					tt.expectedCacheDir,
					config.CacheDir,
				)
			}
			if config.LogLevel != tt.expectedLogLevel {
				t.Errorf(
					"expected LogLevel %q, got %q",
					tt.expectedLogLevel,
					config.LogLevel,
				)
			}
		})
	}
}

// TestDefaultConfig tests that DefaultConfig returns Config with correct
// defaults.
func TestDefaultConfig(t *testing.T) {
	config := DefaultConfig()

	if config.VaultPath != defaultVaultPath {
		t.Errorf(
			"expected VaultPath %q, got %q",
			defaultVaultPath,
			config.VaultPath,
		)
	}
	if config.TemplatesDir != defaultTemplatesDir {
		t.Errorf(
			"expected TemplatesDir %q, got %q",
			defaultTemplatesDir,
			config.TemplatesDir,
		)
	}
	if config.SchemasDir != defaultSchemasDir {
		t.Errorf(
			"expected SchemasDir %q, got %q",
			defaultSchemasDir,
			config.SchemasDir,
		)
	}
	if config.PropertyBankFile != defaultPropertyBankFile {
		t.Errorf(
			"expected PropertyBankFile %q, got %q",
			defaultPropertyBankFile,
			config.PropertyBankFile,
		)
	}
	if config.CacheDir != defaultCacheDir {
		t.Errorf(
			"expected CacheDir %q, got %q",
			defaultCacheDir,
			config.CacheDir,
		)
	}
	if config.LogLevel != defaultLogLevel {
		t.Errorf(
			"expected LogLevel %q, got %q",
			defaultLogLevel,
			config.LogLevel,
		)
	}
}

// TestPropertyBankPath tests that PropertyBankPath returns correct joined path.
func TestPropertyBankPath(t *testing.T) {
	tests := []struct {
		name             string
		schemasDir       string
		propertyBankFile string
		expected         string
	}{
		{
			name:             "default paths",
			schemasDir:       "schemas",
			propertyBankFile: "property_bank.json",
			expected:         "schemas/property_bank.json",
		},
		{
			name:             "custom paths",
			schemasDir:       "/custom/schemas/",
			propertyBankFile: "custom_bank.json",
			expected:         "/custom/schemas/custom_bank.json",
		},
		{
			name:             "absolute paths",
			schemasDir:       "/absolute/path/schemas",
			propertyBankFile: "bank.json",
			expected:         "/absolute/path/schemas/bank.json",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := Config{
				SchemasDir:       tt.schemasDir,
				PropertyBankFile: tt.propertyBankFile,
			}
			result := config.PropertyBankPath()
			if result != tt.expected {
				t.Errorf(
					"expected PropertyBankPath %q, got %q",
					tt.expected,
					result,
				)
			}
		})
	}
}

// TestConfigJSONMarshaling tests JSON marshaling and unmarshaling round-trip.
func TestConfigJSONMarshaling(t *testing.T) {
	original := NewConfig(
		"/vault/path",
		"templates/",
		"schemas/",
		"bank.json",
		"/cache/dir",
		"debug",
	)

	// Marshal to JSON
	jsonData, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("failed to marshal config: %v", err)
	}

	// Unmarshal back to Config
	var unmarshaled Config
	err = json.Unmarshal(jsonData, &unmarshaled)
	if err != nil {
		t.Fatalf("failed to unmarshal config: %v", err)
	}

	// Verify round-trip preserves all values
	if original != unmarshaled {
		t.Errorf(
			"JSON round-trip failed: original=%+v, unmarshaled=%+v",
			original,
			unmarshaled,
		)
	}
}

// TestConfigImmutability tests that Config is a value object - immutable and
// equality-based on field values.
func TestConfigImmutability(t *testing.T) {
	// Test that two configs with identical values are equal
	config1 := NewConfig(
		".",
		"templates/",
		"schemas/",
		"property_bank.json",
		".lithos/cache/",
		"info",
	)
	config2 := NewConfig(
		".",
		"templates/",
		"schemas/",
		"property_bank.json",
		".lithos/cache/",
		"info",
	)

	if config1 != config2 {
		t.Error("configs with identical values should be equal")
	}

	// Test that modifying one doesn't affect equality comparison
	// (since they're passed by value, this is inherent in Go)
	config3 := config1 // copy
	if config1 != config3 {
		t.Error("copied config should equal original")
	}
}
