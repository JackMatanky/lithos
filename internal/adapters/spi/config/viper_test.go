package config

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// resolvePath resolves symlinks for consistent path comparison on macOS.
func resolvePath(path string) string {
	resolved, err := filepath.EvalSymlinks(path)
	if err != nil {
		return path
	}
	return resolved
}

// TestNewViperAdapter tests the ViperAdapter constructor.
func TestNewViperAdapter(t *testing.T) {
	log := logger.New(os.Stdout, "debug")
	adapter := NewViperAdapter(log)

	assert.NotNil(t, adapter)
	assert.NotNil(t, adapter.logger)
}

// TestViperAdapter_Load_DefaultsOnly tests loading configuration with only
// defaults.
func TestViperAdapter_Load_DefaultsOnly(t *testing.T) {
	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	cfg, err := adapter.Load(ctx)

	require.NoError(t, err)
	assert.Equal(t, domain.DefaultConfig(), cfg)
}

// TestViperAdapter_Load_ConfigFileOverride tests loading with config file
// override.
func TestViperAdapter_Load_ConfigFileOverride(t *testing.T) {
	// Create temporary directory structure
	tempDir := t.TempDir()
	configDir := filepath.Join(tempDir, "config")
	require.NoError(t, os.MkdirAll(configDir, 0o750))

	// Create lithos.json in config directory
	configFile := filepath.Join(configDir, "lithos.json")
	configContent := `{
		"vault_path": "` + configDir + `",
		"templates_dir": "custom-templates",
		"schemas_dir": "custom-schemas",
		"property_bank_file": "custom-bank.json",
		"cache_dir": "custom-cache",
		"log_level": "debug"
	}`
	require.NoError(t, os.WriteFile(configFile, []byte(configContent), 0o600))

	// Change to config directory (where lithos.json is located)
	originalWd, _ := os.Getwd()
	require.NoError(t, os.Chdir(configDir))
	defer func() {
		_ = os.Chdir(originalWd)
	}()

	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	cfg, err := adapter.Load(ctx)

	require.NoError(t, err)
	assert.Equal(
		t,
		configDir,
		cfg.VaultPath,
	) // Should be the vault_path from config file
	assert.Equal(t, "custom-templates", cfg.TemplatesDir)
	assert.Equal(t, "custom-schemas", cfg.SchemasDir)
	assert.Equal(t, "custom-bank.json", cfg.PropertyBankFile)
	assert.Equal(t, "custom-cache", cfg.CacheDir)
	assert.Equal(t, "debug", cfg.LogLevel)
}

// TestViperAdapter_Load_EnvironmentOverride tests loading with environment
// variable override.
func TestViperAdapter_Load_EnvironmentOverride(t *testing.T) {
	// Create a temporary directory for vault path
	tempDir := t.TempDir()

	// Set environment variables
	envVars := map[string]string{
		"LITHOS_VAULT_PATH":         tempDir,
		"LITHOS_TEMPLATES_DIR":      "env-templates",
		"LITHOS_SCHEMAS_DIR":        "env-schemas",
		"LITHOS_PROPERTY_BANK_FILE": "env-bank.json",
		"LITHOS_CACHE_DIR":          "env-cache",
		"LITHOS_LOG_LEVEL":          "warn",
	}

	// Set env vars and defer cleanup
	var cleanups []func()
	for key, value := range envVars {
		oldValue := os.Getenv(key)
		require.NoError(t, os.Setenv(key, value))
		k, v := key, oldValue // Capture loop variables
		cleanups = append(cleanups, func() {
			if v == "" {
				_ = os.Unsetenv(k)
			} else {
				_ = os.Setenv(k, v)
			}
		})
	}
	defer func() {
		for _, cleanup := range cleanups {
			cleanup()
		}
	}()

	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	cfg, err := adapter.Load(ctx)

	require.NoError(t, err)
	assert.Equal(t, tempDir, cfg.VaultPath)
	assert.Equal(t, "env-templates", cfg.TemplatesDir)
	assert.Equal(t, "env-schemas", cfg.SchemasDir)
	assert.Equal(t, "env-bank.json", cfg.PropertyBankFile)
	assert.Equal(t, "env-cache", cfg.CacheDir)
	assert.Equal(t, "warn", cfg.LogLevel)
}

// TestViperAdapter_Load_Precedence tests configuration loading precedence
// order.
func TestViperAdapter_Load_Precedence(t *testing.T) {
	// Create temp directory with config file
	tempDir := t.TempDir()
	configDir := filepath.Join(tempDir, "config")
	require.NoError(t, os.MkdirAll(configDir, 0o750))

	configFile := filepath.Join(configDir, "lithos.json")
	configContent := `{
		"templates_dir": "file-templates",
		"log_level": "info"
	}`
	require.NoError(t, os.WriteFile(configFile, []byte(configContent), 0o600))

	// Change to config directory
	originalWd, _ := os.Getwd()
	require.NoError(t, os.Chdir(configDir))
	defer func() {
		_ = os.Chdir(originalWd)
	}()

	// Set environment variable that should override config file
	require.NoError(t, os.Setenv("LITHOS_TEMPLATES_DIR", "env-templates"))
	defer func() { _ = os.Unsetenv("LITHOS_TEMPLATES_DIR") }()

	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	cfg, err := adapter.Load(ctx)

	require.NoError(t, err)
	assert.Equal(
		t,
		resolvePath(configDir),
		resolvePath(cfg.VaultPath),
	) // From config file location
	assert.Equal(
		t,
		"env-templates",
		cfg.TemplatesDir,
	) // Env overrides config file
	assert.Equal(
		t,
		"info",
		cfg.LogLevel,
	) // From config file (not overridden)
}

// TestViperAdapter_Load_InvalidVaultPath tests loading with invalid vault path.
func TestViperAdapter_Load_InvalidVaultPath(t *testing.T) {
	// Set environment variable to non-existent path
	require.NoError(t, os.Setenv("LITHOS_VAULT_PATH", "/nonexistent/path"))
	defer func() { _ = os.Unsetenv("LITHOS_VAULT_PATH") }()

	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	_, err := adapter.Load(ctx)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "vault path validation failed")
	assert.Contains(t, err.Error(), "vault path does not exist")
}

// TestViperAdapter_Load_VaultPathIsFile tests loading when vault path is a
// file.
func TestViperAdapter_Load_VaultPathIsFile(t *testing.T) {
	// Create a temporary file
	tempFile := filepath.Join(t.TempDir(), "not-a-directory")
	require.NoError(t, os.WriteFile(tempFile, []byte("test"), 0o600))

	// Set environment variable to point to the file
	require.NoError(t, os.Setenv("LITHOS_VAULT_PATH", tempFile))
	defer func() { _ = os.Unsetenv("LITHOS_VAULT_PATH") }()

	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	ctx := context.Background()
	_, err := adapter.Load(ctx)

	require.Error(t, err)
	assert.Contains(t, err.Error(), "vault path validation failed")
	assert.Contains(t, err.Error(), "vault path is not a directory")
}

// TestViperAdapter_searchConfigFile tests the config file search functionality.
func TestViperAdapter_searchConfigFile(t *testing.T) {
	t.Run("config in current directory", func(t *testing.T) {
		tempDir := t.TempDir()
		configFile := filepath.Join(tempDir, "lithos.json")
		require.NoError(t, os.WriteFile(configFile, []byte("{}"), 0o600))

		originalWd, _ := os.Getwd()
		require.NoError(t, os.Chdir(tempDir))
		defer func() {
			_ = os.Chdir(originalWd)
		}()

		log := zerolog.New(os.Stdout)
		adapter := NewViperAdapter(log)

		path, err := adapter.searchConfigFile()
		require.NoError(t, err)
		assert.Equal(t, resolvePath(tempDir), resolvePath(path))
	})

	t.Run("config in parent directory", func(t *testing.T) {
		tempDir := t.TempDir()
		parentDir := filepath.Join(tempDir, "parent")
		childDir := filepath.Join(parentDir, "child")

		require.NoError(t, os.MkdirAll(childDir, 0o750))

		configFile := filepath.Join(parentDir, "lithos.json")
		require.NoError(t, os.WriteFile(configFile, []byte("{}"), 0o600))

		originalWd, _ := os.Getwd()
		require.NoError(t, os.Chdir(childDir))
		defer func() {
			_ = os.Chdir(originalWd)
		}()

		log := zerolog.New(os.Stdout)
		adapter := NewViperAdapter(log)

		path, err := adapter.searchConfigFile()
		require.NoError(t, err)
		assert.Equal(t, resolvePath(parentDir), resolvePath(path))
	})

	t.Run("no config file found", func(t *testing.T) {
		tempDir := t.TempDir()

		originalWd, _ := os.Getwd()
		require.NoError(t, os.Chdir(tempDir))
		defer func() {
			_ = os.Chdir(originalWd)
		}()

		log := zerolog.New(os.Stdout)
		adapter := NewViperAdapter(log)

		_, err := adapter.searchConfigFile()
		require.Error(t, err)
		assert.Equal(t, os.ErrNotExist, err)
	})

	t.Run("stops at first config found", func(t *testing.T) {
		tempDir := t.TempDir()
		parentDir := filepath.Join(tempDir, "parent")
		childDir := filepath.Join(parentDir, "child")

		require.NoError(t, os.MkdirAll(childDir, 0o750))

		// Create config in both parent and child
		parentConfig := filepath.Join(parentDir, "lithos.json")
		childConfig := filepath.Join(childDir, "lithos.json")
		require.NoError(t, os.WriteFile(parentConfig, []byte("{}"), 0o600))
		require.NoError(t, os.WriteFile(childConfig, []byte("{}"), 0o600))

		originalWd, _ := os.Getwd()
		require.NoError(t, os.Chdir(childDir))
		defer func() {
			_ = os.Chdir(originalWd)
		}()

		log := zerolog.New(os.Stdout)
		adapter := NewViperAdapter(log)

		path, err := adapter.searchConfigFile()
		require.NoError(t, err)
		assert.Equal(
			t,
			resolvePath(childDir),
			resolvePath(path),
		) // Should find the one in child first
	})
}

// TestViperAdapter_validateVaultPath tests vault path validation.
func TestViperAdapter_validateVaultPath(t *testing.T) {
	log := zerolog.New(os.Stdout)
	adapter := NewViperAdapter(log)

	t.Run("valid directory", func(t *testing.T) {
		tempDir := t.TempDir()
		err := adapter.validateVaultPath(tempDir)
		assert.NoError(t, err)
	})

	t.Run("non-existent path", func(t *testing.T) {
		nonExistent := "/this/path/does/not/exist"
		err := adapter.validateVaultPath(nonExistent)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "vault path does not exist")
	})

	t.Run("path is file", func(t *testing.T) {
		tempFile := filepath.Join(t.TempDir(), "file.txt")
		require.NoError(t, os.WriteFile(tempFile, []byte("test"), 0o600))

		err := adapter.validateVaultPath(tempFile)
		require.Error(t, err)
		assert.Contains(t, err.Error(), "vault path is not a directory")
	})
}
