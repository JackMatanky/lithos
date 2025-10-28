package domain

import (
	"testing"
)

// TestNewConfig tests that NewConfig constructs a Config with all fields.
func TestNewConfig(t *testing.T) {
	tests := []struct {
		name             string
		vaultPath        string
		templatesDir     string
		schemasDir       string
		propertyBankFile string
		cacheDir         string
		logLevel         string
	}{
		{
			name:             "default values",
			vaultPath:        ".",
			templatesDir:     "templates/",
			schemasDir:       "schemas/",
			propertyBankFile: "property_bank.json",
			cacheDir:         ".lithos/cache/",
			logLevel:         "info",
		},
		{
			name:             "custom values",
			vaultPath:        "/home/user/vault",
			templatesDir:     "custom/templates/",
			schemasDir:       "custom/schemas/",
			propertyBankFile: "custom_bank.json",
			cacheDir:         "/tmp/cache/",
			logLevel:         "debug",
		},
		{
			name:             "empty strings",
			vaultPath:        "",
			templatesDir:     "",
			schemasDir:       "",
			propertyBankFile: "",
			cacheDir:         "",
			logLevel:         "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			config := NewConfig(
				tt.vaultPath,
				tt.templatesDir,
				tt.schemasDir,
				tt.propertyBankFile,
				tt.cacheDir,
				tt.logLevel,
			)

			if config.VaultPath != tt.vaultPath {
				t.Errorf(
					"expected VaultPath %q, got %q",
					tt.vaultPath,
					config.VaultPath,
				)
			}
			if config.TemplatesDir != tt.templatesDir {
				t.Errorf(
					"expected TemplatesDir %q, got %q",
					tt.templatesDir,
					config.TemplatesDir,
				)
			}
			if config.SchemasDir != tt.schemasDir {
				t.Errorf(
					"expected SchemasDir %q, got %q",
					tt.schemasDir,
					config.SchemasDir,
				)
			}
			if config.PropertyBankFile != tt.propertyBankFile {
				t.Errorf(
					"expected PropertyBankFile %q, got %q",
					tt.propertyBankFile,
					config.PropertyBankFile,
				)
			}
			if config.CacheDir != tt.cacheDir {
				t.Errorf(
					"expected CacheDir %q, got %q",
					tt.cacheDir,
					config.CacheDir,
				)
			}
			if config.LogLevel != tt.logLevel {
				t.Errorf(
					"expected LogLevel %q, got %q",
					tt.logLevel,
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
