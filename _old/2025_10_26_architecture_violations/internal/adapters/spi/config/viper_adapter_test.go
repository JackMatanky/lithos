package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/spf13/viper"
)

const (
	configContent = `vaultPath: .
templatesDir: templates
schemasDir: schemas
cacheDir: .lithos/cache
logLevel: info
`
	testLogLevelInfo  = "info"
	testVaultPath     = "/tmp/vault"
	testLogLevelDebug = "debug"
)

func getTestCases(configFile string) []struct {
	name    string
	wantErr bool
	setup   func() error
	cleanup func() error
} {
	return []struct {
		name    string
		wantErr bool
		setup   func() error
		cleanup func() error
	}{
		{
			name:    "successful creation with config file",
			wantErr: false,
			setup:   func() error { return nil },
			cleanup: func() error { return nil },
		},
		{
			name:    "successful creation without config file",
			wantErr: false,
			setup: func() error {
				return os.Remove(configFile)
			},
			cleanup: func() error {
				// Recreate config file for other tests
				return os.WriteFile(configFile, []byte(configContent), 0o600)
			},
		},
	}
}

func runTestCase(t *testing.T, tt struct {
	name    string
	wantErr bool
	setup   func() error
	cleanup func() error
}) {
	if tt.setup != nil {
		if setupErr := tt.setup(); setupErr != nil {
			t.Fatalf("Setup failed: %v", setupErr)
		}
	}

	adapter, adapterErr := NewConfigViperAdapter()

	if tt.wantErr {
		if adapterErr == nil {
			t.Error("NewConfigViperAdapter() expected error, got nil")
		}
		return
	}

	if adapterErr != nil {
		t.Errorf(
			"NewConfigViperAdapter() unexpected error = %v",
			adapterErr,
		)
		return
	}

	validateAdapterAndConfig(t, adapter)

	if tt.cleanup != nil {
		if cleanupErr := tt.cleanup(); cleanupErr != nil {
			t.Errorf("Cleanup failed: %v", cleanupErr)
		}
	}
}

func validateAdapterAndConfig(t *testing.T, adapter *ConfigViperAdapter) {
	if adapter == nil {
		t.Error("NewConfigViperAdapter() returned nil adapter")
		return
	}

	config := adapter.Config()
	if config == nil {
		t.Error("Config() returned nil")
		return
	}

	// Verify basic configuration
	if config.VaultPath == "" {
		t.Error("VaultPath should not be empty")
	}
	if config.LogLevel == "" {
		t.Error("LogLevel should not be empty")
	}

	// Verify VaultPath is absolute
	if !filepath.IsAbs(config.VaultPath) {
		t.Errorf(
			"VaultPath should be absolute, got %q",
			config.VaultPath,
		)
	}
}

func setupTestEnvironment(
	t *testing.T,
) (tempDir, configFile string, cleanup func()) {
	t.Helper()

	var err error
	var originalDir string

	tempDir, err = os.MkdirTemp("", "lithos-viper-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}

	configFile = filepath.Join(tempDir, "lithos.yaml")
	if writeErr := os.WriteFile(configFile, []byte(configContent), 0o600); writeErr != nil {
		t.Fatalf("Failed to write config file: %v", writeErr)
	}

	originalDir, err = os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}

	if chdirErr := os.Chdir(tempDir); chdirErr != nil {
		t.Fatalf("Failed to change to temp directory: %v", chdirErr)
	}

	cleanup = func() {
		_ = os.Chdir(originalDir)
		_ = os.RemoveAll(tempDir)
	}

	return tempDir, configFile, cleanup
}

func TestNewConfigViperAdapter(t *testing.T) {
	_, configFile, cleanup := setupTestEnvironment(t)
	defer cleanup()

	tests := getTestCases(configFile)

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			runTestCase(t, tt)
		})
	}
}

func TestConfigViperAdapter_Config(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-viper-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Change to temp directory for test
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer func() { _ = os.Chdir(originalDir) }()

	if chdirErr := os.Chdir(tempDir); chdirErr != nil {
		t.Fatalf("Failed to change to temp directory: %v", chdirErr)
	}

	adapter, err := NewConfigViperAdapter()
	if err != nil {
		t.Fatalf("Failed to create adapter: %v", err)
	}

	config1 := adapter.Config()
	config2 := adapter.Config()

	// Should return the same cached config instance
	if config1 != config2 {
		t.Error("Config() should return the same cached instance")
	}

	// Verify config has expected structure
	if config1.VaultPath == "" {
		t.Error("VaultPath should not be empty")
	}
	if config1.TemplatesDir == "" {
		t.Error("TemplatesDir should not be empty")
	}
	if config1.SchemasDir == "" {
		t.Error("SchemasDir should not be empty")
	}
	if config1.CacheDir == "" {
		t.Error("CacheDir should not be empty")
	}
	if config1.LogLevel == "" {
		t.Error("LogLevel should not be empty")
	}
}

func TestSetDefaults(t *testing.T) {
	v := viper.New()

	if err := setDefaults(v); err != nil {
		t.Errorf("setDefaults() unexpected error = %v", err)
		return
	}

	// Verify defaults are set
	if v.GetString("vaultPath") == "" {
		t.Error("vaultPath default should not be empty")
	}
	if v.GetString("templatesDir") == "" {
		t.Error("templatesDir default should not be empty")
	}
	if v.GetString("schemasDir") == "" {
		t.Error("schemasDir default should not be empty")
	}
	if v.GetString("cacheDir") == "" {
		t.Error("cacheDir default should not be empty")
	}
	if v.GetString("logLevel") != testLogLevelInfo {
		t.Errorf(
			"logLevel default = %q, want %q",
			v.GetString("logLevel"),
			"info",
		)
	}

	// Verify that VaultPath is absolute
	vaultPath := v.GetString("vaultPath")
	if !filepath.IsAbs(vaultPath) {
		t.Errorf("default vaultPath should be absolute, got %q", vaultPath)
	}
}

//nolint:gocognit // test function with multiple test cases
func TestBuildConfigFromViper(t *testing.T) {
	tests := []struct {
		name     string
		setup    func(*viper.Viper)
		wantErr  bool
		validate func(*testing.T, *Config)
	}{
		{
			name: "absolute paths",
			setup: func(v *viper.Viper) {
				v.Set("vaultPath", testVaultPath)
				v.Set("templatesDir", "/tmp/templates")
				v.Set("schemasDir", "/tmp/schemas")
				v.Set("cacheDir", "/tmp/cache")
				v.Set("logLevel", testLogLevelDebug)
			},
			wantErr: false,
			validate: func(t *testing.T, config *Config) {
				if config.VaultPath != testVaultPath {
					t.Errorf(
						"VaultPath = %q, want %q",
						config.VaultPath,
						"/tmp/vault",
					)
				}
				if config.TemplatesDir != "/tmp/templates" {
					t.Errorf(
						"TemplatesDir = %q, want %q",
						config.TemplatesDir,
						"/tmp/templates",
					)
				}
				if config.SchemasDir != "/tmp/schemas" {
					t.Errorf(
						"SchemasDir = %q, want %q",
						config.SchemasDir,
						"/tmp/schemas",
					)
				}
				if config.CacheDir != "/tmp/cache" {
					t.Errorf(
						"CacheDir = %q, want %q",
						config.CacheDir,
						"/tmp/cache",
					)
				}
				if config.LogLevel != testLogLevelDebug {
					t.Errorf(
						"LogLevel = %q, want %q",
						config.LogLevel,
						testLogLevelDebug,
					)
				}
			},
		},
		{
			name: "relative paths resolved to vault path",
			setup: func(v *viper.Viper) {
				v.Set("vaultPath", testVaultPath)
				v.Set("templatesDir", "templates")
				v.Set("schemasDir", "schemas")
				v.Set("cacheDir", ".lithos/cache")
				v.Set("logLevel", "info")
			},
			wantErr: false,
			validate: func(t *testing.T, config *Config) {
				if config.VaultPath != testVaultPath {
					t.Errorf(
						"VaultPath = %q, want %q",
						config.VaultPath,
						"/tmp/vault",
					)
				}
				if config.TemplatesDir != "/tmp/vault/templates" {
					t.Errorf(
						"TemplatesDir = %q, want %q",
						config.TemplatesDir,
						"/tmp/vault/templates",
					)
				}
				if config.SchemasDir != "/tmp/vault/schemas" {
					t.Errorf(
						"SchemasDir = %q, want %q",
						config.SchemasDir,
						"/tmp/vault/schemas",
					)
				}
				if config.CacheDir != "/tmp/vault/.lithos/cache" {
					t.Errorf(
						"CacheDir = %q, want %q",
						config.CacheDir,
						"/tmp/vault/.lithos/cache",
					)
				}
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			v := viper.New()
			tt.setup(v)

			config, err := buildConfigFromViper(v)

			if tt.wantErr {
				if err == nil {
					t.Error("buildConfigFromViper() expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Errorf("buildConfigFromViper() unexpected error = %v", err)
				return
			}

			if config == nil {
				t.Error("buildConfigFromViper() returned nil config")
				return
			}

			if tt.validate != nil {
				tt.validate(t, config)
			}
		})
	}
}

func TestNewConfigViperAdapter_InvalidConfig(t *testing.T) {
	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-viper-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Create an invalid config file (invalid YAML)
	invalidConfig := `vaultPath: /nonexistent
logLevel: invalid_level
this is not valid YAML syntax [[[
`
	configFile := filepath.Join(tempDir, "lithos.yaml")
	if writeErr := os.WriteFile(configFile, []byte(invalidConfig), 0o600); writeErr != nil {
		t.Fatalf("Failed to write config file: %v", writeErr)
	}

	// Change to temp directory for test
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer func() { _ = os.Chdir(originalDir) }()

	if chdirErr := os.Chdir(tempDir); chdirErr != nil {
		t.Fatalf("Failed to change to temp directory: %v", chdirErr)
	}

	_, err = NewConfigViperAdapter()
	if err == nil {
		t.Error(
			"NewConfigViperAdapter() expected error for invalid config file, got nil",
		)
	}
}

func TestConfigViperAdapter_WithEnvironmentVariables(t *testing.T) {
	// Set environment variables for testing
	originalVaultPath := os.Getenv("LITHOS_VAULTPATH")
	originalLogLevel := os.Getenv("LITHOS_LOGLEVEL")

	defer func() {
		// Restore original environment
		if originalVaultPath != "" {
			_ = os.Setenv("LITHOS_VAULTPATH", originalVaultPath)
			_ = os.Unsetenv("LITHOS_VAULTPATH")
			_ = os.Setenv("LITHOS_LOGLEVEL", originalLogLevel)
			_ = os.Unsetenv("LITHOS_LOGLEVEL")
		}
	}()

	// Create a temporary directory for testing
	tempDir, err := os.MkdirTemp("", "lithos-viper-test")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}
	defer func() { _ = os.RemoveAll(tempDir) }()

	// Set environment variables (viper converts camelCase keys to UPPERCASE)
	_ = os.Setenv("LITHOS_VAULTPATH", tempDir)
	_ = os.Setenv("LITHOS_LOGLEVEL", testLogLevelDebug)

	// Change to a different directory to ensure env vars are used
	originalDir, err := os.Getwd()
	if err != nil {
		t.Fatalf("Failed to get current directory: %v", err)
	}
	defer func() { _ = os.Chdir(originalDir) }()

	// Change to temp directory
	if chdirErr := os.Chdir(tempDir); chdirErr != nil {
		t.Fatalf("Failed to change to temp directory: %v", chdirErr)
	}

	adapter, err := NewConfigViperAdapter()
	if err != nil {
		t.Fatalf("NewConfigViperAdapter() unexpected error = %v", err)
	}

	config := adapter.Config()

	// Environment variables should override defaults
	expectedVaultPath, _ := filepath.Abs(tempDir)
	if config.VaultPath != expectedVaultPath {
		t.Errorf(
			"VaultPath = %q, want %q (from env var)",
			config.VaultPath,
			expectedVaultPath,
		)
	}
	if config.LogLevel != testLogLevelDebug {
		t.Errorf(
			"LogLevel = %q, want %q",
			config.LogLevel,
			testLogLevelDebug,
		)
	}
}
