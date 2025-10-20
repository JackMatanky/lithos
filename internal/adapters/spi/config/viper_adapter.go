package config

import (
	"errors"
	"fmt"
	"path/filepath"

	"github.com/spf13/viper"
)

// ConfigViperAdapter implements ConfigPort using github.com/spf13/viper for
// configuration management. This is an SPI adapter that handles infrastructure
// concerns like YAML file loading,
// environment variable overrides, and directory traversal search.
//
// Architecture Layer: SPI Adapter (Configuration)
// Rationale: Viper integration is pure infrastructure - domain never needs to
// know about file I/O,
// YAML parsing, or environment variable precedence.
type ConfigViperAdapter struct {
	config *Config
	viper  *viper.Viper
}

// NewConfigViperAdapter creates a new ConfigViperAdapter.
// It searches for lithos.yaml starting from the current directory and walking
// up the directory tree. Configuration precedence (highest to lowest): CLI
// flags > Environment variables > Config file > Defaults.
func NewConfigViperAdapter() (*ConfigViperAdapter, error) {
	v := createAndConfigureViper()

	var err error
	if err = bindEnvironmentVariables(v); err != nil {
		return nil, fmt.Errorf("failed to bind environment variables: %w", err)
	}

	if err = setDefaults(v); err != nil {
		return nil, fmt.Errorf("failed to set default values: %w", err)
	}

	if err = readConfigFile(v); err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	config, err := buildConfigFromViper(v)
	if err != nil {
		return nil, fmt.Errorf("failed to build config: %w", err)
	}

	if err = config.Validate(); err != nil {
		return nil, fmt.Errorf("configuration validation failed: %w", err)
	}

	return &ConfigViperAdapter{
		config: config,
		viper:  v,
	}, nil
}

// Config returns the current application configuration.
// The configuration is loaded at application startup and cached.
func (a *ConfigViperAdapter) Config() *Config {
	return a.config
}

// setDefaults configures default values for all configuration options.
func setDefaults(v *viper.Viper) error {
	// Get current working directory for VaultPath default
	cwd, err := filepath.Abs(".")
	if err != nil {
		return fmt.Errorf("failed to get current working directory: %w", err)
	}

	// Set default values
	v.SetDefault("vaultPath", cwd)
	v.SetDefault("templatesDir", filepath.Join(cwd, "templates"))
	v.SetDefault("schemasDir", filepath.Join(cwd, "schemas"))
	v.SetDefault("cacheDir", filepath.Join(cwd, ".lithos", "cache"))
	v.SetDefault("logLevel", "info")

	return nil
}

// buildConfigFromViper creates a Config struct from viper values.
func buildConfigFromViper(v *viper.Viper) (*Config, error) {
	vaultPath, err := getVaultPath(v)
	if err != nil {
		return nil, err
	}

	config := &Config{
		VaultPath:    vaultPath,
		LogLevel:     v.GetString("logLevel"),
		TemplatesDir: "",
		SchemasDir:   "",
		CacheDir:     "",
	}

	config.TemplatesDir = resolvePath(v.GetString("templatesDir"), vaultPath)
	config.SchemasDir = resolvePath(v.GetString("schemasDir"), vaultPath)
	config.CacheDir = resolvePath(v.GetString("cacheDir"), vaultPath)

	return config, nil
}

// createAndConfigureViper creates and configures a new viper instance.
func createAndConfigureViper() *viper.Viper {
	v := viper.New()

	// Configure viper for lithos.yaml
	v.SetConfigName("lithos")
	v.SetConfigType("yaml")

	// Search for config file in current directory and parent directories
	v.AddConfigPath(".")
	v.AddConfigPath("$HOME")

	// Set environment variable prefix and automatic binding
	v.SetEnvPrefix("LITHOS")
	v.AutomaticEnv()

	return v
}

// bindEnvironmentVariables binds all required environment variables.
func bindEnvironmentVariables(v *viper.Viper) error {
	envVars := []string{
		"vaultPath",
		"templatesDir",
		"schemasDir",
		"cacheDir",
		"logLevel",
	}

	for _, envVar := range envVars {
		if err := v.BindEnv(envVar); err != nil {
			return fmt.Errorf("failed to bind %s env var: %w", envVar, err)
		}
	}

	return nil
}

// readConfigFile attempts to read the config file, treating file not found as
// acceptable.
func readConfigFile(v *viper.Viper) error {
	if err := v.ReadInConfig(); err != nil {
		// Only return error if it's not a "file not found" error
		var configFileNotFoundError viper.ConfigFileNotFoundError
		if !errors.As(err, &configFileNotFoundError) {
			return fmt.Errorf("failed to read config file: %w", err)
		}
		// Config file not found is acceptable - we'll use defaults and
		// environment variables
	}
	return nil
}

// getVaultPath gets the vault path from viper and ensures it's absolute.
func getVaultPath(v *viper.Viper) (string, error) {
	vaultPath := v.GetString("vaultPath")
	absVaultPath, err := filepath.Abs(vaultPath)
	if err != nil {
		return "", fmt.Errorf("failed to make vault path absolute: %w", err)
	}
	return absVaultPath, nil
}

// resolvePath resolves a path relative to vault path if it's not absolute.
func resolvePath(path, vaultPath string) string {
	if filepath.IsAbs(path) {
		return path
	}
	return filepath.Join(vaultPath, path)
}
