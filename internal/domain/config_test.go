package domain

import (
	"encoding/json"
	"fmt"
	"sync"
	"testing"
)

const (
	testCustomVaultPath = "/custom/vault"
)

// testConfigValidator implements ConfigValidator for testing.
type testConfigValidator struct {
	validateFunc func(Config) error
}

func (v testConfigValidator) Validate(c Config) error {
	return v.validateFunc(c)
}

// TestNewConfig tests that NewConfig applies defaults correctly with
// multiple test cases.
func TestNewConfig(t *testing.T) {
	tests := []struct {
		name                     string
		inputVaultPath           string
		inputTemplatesDir        string
		inputSchemasDir          string
		inputPropertyBankFile    string
		inputCacheDir            string
		inputLogLevel            string
		inputFileClassKey        string
		expectedVaultPath        string
		expectedTemplatesDir     string
		expectedSchemasDir       string
		expectedPropertyBankFile string
		expectedCacheDir         string
		expectedLogLevel         string
		expectedFileClassKey     string
	}{
		{
			name:                     "all defaults applied",
			inputVaultPath:           "",
			inputTemplatesDir:        "",
			inputSchemasDir:          "",
			inputPropertyBankFile:    "",
			inputCacheDir:            "",
			inputLogLevel:            "",
			inputFileClassKey:        "",
			expectedVaultPath:        ".",
			expectedTemplatesDir:     "templates",
			expectedSchemasDir:       "schemas",
			expectedPropertyBankFile: "property_bank.json",
			expectedCacheDir:         ".lithos/cache",
			expectedLogLevel:         "info",
			expectedFileClassKey:     "file_class",
		},
		{
			name:                     "partial defaults applied",
			inputVaultPath:           "/custom/vault",
			inputTemplatesDir:        "",
			inputSchemasDir:          "",
			inputPropertyBankFile:    "custom.json",
			inputCacheDir:            "",
			inputLogLevel:            "debug",
			inputFileClassKey:        "",
			expectedVaultPath:        "/custom/vault",
			expectedTemplatesDir:     "/custom/vault/templates",
			expectedSchemasDir:       "/custom/vault/schemas",
			expectedPropertyBankFile: "custom.json",
			expectedCacheDir:         "/custom/vault/.lithos/cache",
			expectedLogLevel:         "debug",
			expectedFileClassKey:     "file_class",
		},
		{
			name:                     "no defaults needed",
			inputVaultPath:           "/home/user/vault",
			inputTemplatesDir:        "custom/templates/",
			inputSchemasDir:          "custom/schemas/",
			inputPropertyBankFile:    "custom_bank.json",
			inputCacheDir:            "/tmp/cache/",
			inputLogLevel:            "debug",
			inputFileClassKey:        "",
			expectedVaultPath:        "/home/user/vault",
			expectedTemplatesDir:     "custom/templates/",
			expectedSchemasDir:       "custom/schemas/",
			expectedPropertyBankFile: "custom_bank.json",
			expectedCacheDir:         "/tmp/cache/",
			expectedLogLevel:         "debug",
			expectedFileClassKey:     "file_class",
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
				tt.inputFileClassKey,
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
			if config.FileClassKey != tt.expectedFileClassKey {
				t.Errorf(
					"expected FileClassKey %q, got %q",
					tt.expectedFileClassKey,
					config.FileClassKey,
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
	if config.FileClassKey != defaultFileClassKey {
		t.Errorf(
			"expected FileClassKey %q, got %q",
			defaultFileClassKey,
			config.FileClassKey,
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
		"file_class",
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

// TestConfigJSONUnmarshalFileClassKey ensures snake_case keys populate
// FileClassKey.
func TestConfigJSONUnmarshalFileClassKey(t *testing.T) {
	payload := []byte(`{"file_class_key":"type"}`)
	var cfg Config
	if err := json.Unmarshal(payload, &cfg); err != nil {
		t.Fatalf("failed to unmarshal config: %v", err)
	}
	if cfg.FileClassKey != "type" {
		t.Fatalf(
			"expected file_class_key to be 'type', got %q",
			cfg.FileClassKey,
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
		"file_class",
	)
	config2 := NewConfig(
		".",
		"templates/",
		"schemas/",
		"property_bank.json",
		".lithos/cache/",
		"info",
		"file_class",
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

// TestConfigInstance_SingletonBehavior tests that Instance() returns the same
// instance on multiple calls (singleton pattern with sync.Once).
func TestConfigInstance_SingletonBehavior(t *testing.T) {
	// Clear any existing instance for clean test
	ResetConfigForTesting()
	defer ResetConfigForTesting()

	// First call to Instance()
	instance1 := Instance()
	if instance1 == nil {
		t.Fatal("Instance() returned nil")
	}

	// Second call to Instance()
	instance2 := Instance()
	if instance2 == nil {
		t.Fatal("Instance() returned nil on second call")
	}

	// Both calls should return the exact same pointer
	if instance1 != instance2 {
		t.Error("Instance() should return same instance on multiple calls")
	}
}

// TestConfigInstance_ThreadSafe tests that Instance() is thread-safe using
// sync.Once.
func TestConfigInstance_ThreadSafe(t *testing.T) {
	// Clear any existing instance for clean test
	ResetConfigForTesting()
	defer ResetConfigForTesting()

	const goroutines = 100
	instances := make([]*Config, goroutines)

	// Use WaitGroup to coordinate goroutines
	var wg sync.WaitGroup
	wg.Add(goroutines)

	// Launch many goroutines simultaneously calling Instance()
	for i := range goroutines {
		go func(index int) {
			defer wg.Done()
			instances[index] = Instance()
		}(i)
	}

	// Wait for all goroutines to complete
	wg.Wait()

	// Verify all goroutines got the same instance
	firstInstance := instances[0]
	if firstInstance == nil {
		t.Fatal("First instance is nil")
	}

	for i := 1; i < goroutines; i++ {
		if instances[i] != firstInstance {
			t.Errorf(
				"Goroutine %d got different instance: expected %p, got %p",
				i,
				firstInstance,
				instances[i],
			)
		}
	}
}

// TestSetInstanceForTesting_TestIsolation tests that SetInstanceForTesting()
// allows test isolation without global state pollution.
func TestSetInstanceForTesting_TestIsolation(t *testing.T) {
	// Clear any existing instance for clean test
	ResetConfigForTesting()
	defer ResetConfigForTesting()

	// Create custom config for testing
	customConfig := NewConfig(
		testCustomVaultPath,
		"/custom/templates",
		"/custom/schemas",
		"custom_bank.json",
		"/custom/cache",
		"debug",
		"custom_file_class",
	)

	// Set custom instance for testing
	SetInstanceForTesting(&customConfig)

	// Verify Instance() returns the custom config
	instance := Instance()
	if instance == nil {
		t.Fatal("Instance() returned nil after SetInstanceForTesting()")
	}
	if instance.VaultPath != testCustomVaultPath {
		t.Errorf(
			"Expected VaultPath '/custom/vault', got %q",
			instance.VaultPath,
		)
	}
	if instance.FileClassKey != "custom_file_class" {
		t.Errorf(
			"Expected FileClassKey 'custom_file_class', got %q",
			instance.FileClassKey,
		)
	}

	// Reset should clear the custom instance
	ResetConfigForTesting()

	// After reset, Instance() should create new default instance
	newInstance := Instance()
	if newInstance == nil {
		t.Fatal("Instance() returned nil after reset")
	}
	// Should not be the same pointer as custom config
	if newInstance == instance {
		t.Error(
			"Instance() should return different instance after ResetConfigForTesting()",
		)
	}
}

// TestConfigBuilder tests the fluent ConfigBuilder API.
func TestConfigBuilder(t *testing.T) {
	// Test building with all methods
	config, err := NewConfigBuilder().
		WithVaultPath("/custom/vault").
		WithTemplatesDir("custom/templates").
		WithSchemasDir("custom/schemas").
		WithPropertyBankFile("custom_bank.json").
		WithCacheDir("/tmp/cache").
		WithLogLevel("debug").
		WithFileClassKey("file_class").
		Build()

	if err != nil {
		t.Fatalf("Build() failed: %v", err)
	}

	if config.VaultPath != "/custom/vault" {
		t.Errorf("Expected VaultPath '/custom/vault', got %q", config.VaultPath)
	}
	if config.TemplatesDir != "/custom/vault/custom/templates" {
		t.Errorf(
			"Expected TemplatesDir '/custom/vault/custom/templates', got %q",
			config.TemplatesDir,
		)
	}
	if config.SchemasDir != "/custom/vault/custom/schemas" {
		t.Errorf(
			"Expected SchemasDir '/custom/vault/custom/schemas', got %q",
			config.SchemasDir,
		)
	}
	if config.PropertyBankFile != "custom_bank.json" {
		t.Errorf(
			"Expected PropertyBankFile 'custom_bank.json', got %q",
			config.PropertyBankFile,
		)
	}
	if config.CacheDir != "/tmp/cache" {
		t.Errorf("Expected CacheDir '/tmp/cache', got %q", config.CacheDir)
	}
	if config.LogLevel != "debug" {
		t.Errorf("Expected LogLevel 'debug', got %q", config.LogLevel)
	}
	if config.FileClassKey != defaultFileClassKey {
		t.Errorf(
			"Expected FileClassKey %q, got %q",
			defaultFileClassKey,
			config.FileClassKey,
		)
	}
}

// TestConfigBuilderDefaults tests that ConfigBuilder starts with correct
// defaults.
func TestConfigBuilderDefaults(t *testing.T) {
	builder := NewConfigBuilder()
	config, err := builder.Build()
	if err != nil {
		t.Fatalf("Build() failed: %v", err)
	}

	if config.VaultPath != "." {
		t.Errorf("Expected default VaultPath '.', got %q", config.VaultPath)
	}
	if config.TemplatesDir != "templates" {
		t.Errorf(
			"Expected default TemplatesDir 'templates', got %q",
			config.TemplatesDir,
		)
	}
	if config.SchemasDir != "schemas" {
		t.Errorf(
			"Expected default SchemasDir 'schemas', got %q",
			config.SchemasDir,
		)
	}
	if config.PropertyBankFile != "property_bank.json" {
		t.Errorf(
			"Expected default PropertyBankFile 'property_bank.json', got %q",
			config.PropertyBankFile,
		)
	}
	if config.CacheDir != ".lithos/cache" {
		t.Errorf(
			"Expected default CacheDir '.lithos/cache', got %q",
			config.CacheDir,
		)
	}
	if config.LogLevel != "info" {
		t.Errorf("Expected default LogLevel 'info', got %q", config.LogLevel)
	}
	if config.FileClassKey != "file_class" {
		t.Errorf(
			"Expected default FileClassKey 'file_class', got %q",
			config.FileClassKey,
		)
	}
}

// TestConfigBuilderWithValidator tests ConfigBuilder with a validator.
func TestConfigBuilderWithValidator(t *testing.T) {
	validator := testConfigValidator{
		validateFunc: func(c Config) error {
			if c.VaultPath == "" {
				return fmt.Errorf("vault path required")
			}
			return nil
		},
	}

	config, err := NewConfigBuilder().
		WithVaultPath("/test").
		WithValidator(validator).
		Build()

	if err != nil {
		t.Fatalf("Build() failed: %v", err)
	}

	if config.VaultPath != "/test" {
		t.Errorf("Expected VaultPath '/test', got %q", config.VaultPath)
	}
}

// TestConfigBuilderValidationFailure tests ConfigBuilder with failing
// validator.
func TestConfigBuilderValidationFailure(t *testing.T) {
	validator := testConfigValidator{
		validateFunc: func(c Config) error {
			return fmt.Errorf("validation failed")
		},
	}

	_, err := NewConfigBuilder().
		WithVaultPath("/test").
		WithValidator(validator).
		Build()

	if err == nil {
		t.Error("Expected Build() to fail with validator error")
	}
}
