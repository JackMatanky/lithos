package config

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// Config represents application configuration loaded from lithos.yaml and
// environment variables. This is an SPI adapter model that defines vault
// structure and operational settings.
//
// Architecture Layer: SPI Adapter (Configuration)
// Rationale: Configuration is pure infrastructure wiring - filesystem paths,
// log levels, directory locations.
// Domain never needs to know about these concerns.
type Config struct {
	// VaultPath is the root directory of the vault. Default: current working
	// directory. All relative paths in config are resolved relative to this.
	// Must exist and be readable.
	VaultPath string `yaml:"vaultPath" json:"vaultPath"`

	// TemplatesDir is the path to templates directory. Default:
	// {VaultPath}/templates. Can be absolute or relative to VaultPath. Must
	// exist for `lithos new` and `lithos find` commands.
	TemplatesDir string `yaml:"templatesDir" json:"templatesDir"`

	// SchemasDir is the path to schemas directory. Default:
	// {VaultPath}/schemas.
	// Can be absolute or relative to VaultPath. Must exist if schemas are used.
	SchemasDir string `yaml:"schemasDir" json:"schemasDir"`

	// CacheDir is the path to index cache. Default: {VaultPath}/.lithos/cache.
	// Can be absolute or relative to VaultPath. Created automatically if
	// missing. Must be writable.
	CacheDir string `yaml:"cacheDir" json:"cacheDir"`

	// LogLevel is the logging verbosity for zerolog. One of: "debug", "info",
	// "warn", "error". Default: "info". Case-insensitive. Invalid values fall
	// back to "info" with warning.
	LogLevel string `yaml:"logLevel" json:"logLevel"`
}

// NewConfig creates a new Config with sensible defaults based on the vault
// path.
// VaultPath defaults to current working directory if empty.
func NewConfig(vaultPath string) *Config {
	if vaultPath == "" {
		cwd, err := os.Getwd()
		if err != nil {
			// Fallback to current directory
			vaultPath = "."
		} else {
			vaultPath = cwd
		}
	}

	// Convert to absolute path
	absVaultPath, err := filepath.Abs(vaultPath)
	if err != nil {
		// Fallback to original path
		absVaultPath = vaultPath
	}

	return &Config{
		VaultPath:    absVaultPath,
		TemplatesDir: filepath.Join(absVaultPath, "templates"),
		SchemasDir:   filepath.Join(absVaultPath, "schemas"),
		CacheDir:     filepath.Join(absVaultPath, ".lithos", "cache"),
		LogLevel:     "info",
	}
}

// Validate ensures the configuration is valid and vault path exists and is
// readable.
// This is called at application startup to fail fast on invalid configurations.
func (c *Config) Validate() error {
	// Validate VaultPath exists and is readable
	if err := c.validateVaultPath(); err != nil {
		return fmt.Errorf("vault path validation failed: %w", err)
	}

	// Validate LogLevel is one of the allowed values
	if err := c.validateLogLevel(); err != nil {
		return fmt.Errorf("log level validation failed: %w", err)
	}

	return nil
}

// ResolvePath resolves a path relative to VaultPath if it's not absolute.
func (c *Config) ResolvePath(path string) string {
	if filepath.IsAbs(path) {
		return path
	}
	return filepath.Join(c.VaultPath, path)
}

// validateVaultPath checks that VaultPath exists and is a readable directory.
func (c *Config) validateVaultPath() error {
	if c.VaultPath == "" {
		return fmt.Errorf("vault path cannot be empty")
	}

	info, err := os.Stat(c.VaultPath)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("vault path does not exist: %s", c.VaultPath)
		}
		return fmt.Errorf("cannot access vault path %s: %w", c.VaultPath, err)
	}

	if !info.IsDir() {
		return fmt.Errorf("vault path is not a directory: %s", c.VaultPath)
	}

	// Test readability by attempting to read the directory
	_, err = os.ReadDir(c.VaultPath)
	if err != nil {
		return fmt.Errorf(
			"vault path is not readable: %s: %w",
			c.VaultPath,
			err,
		)
	}

	return nil
}

// validateLogLevel ensures LogLevel is one of the allowed values.
func (c *Config) validateLogLevel() error {
	allowedLevels := []string{"debug", "info", "warn", "error"}
	normalizedLevel := strings.ToLower(c.LogLevel)

	for _, level := range allowedLevels {
		if normalizedLevel == level {
			// Update the config with normalized value
			c.LogLevel = normalizedLevel
			return nil
		}
	}

	return fmt.Errorf(
		"invalid log level %q, must be one of: %v",
		c.LogLevel,
		allowedLevels,
	)
}
