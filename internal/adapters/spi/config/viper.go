// Package config provides SPI adapter implementations for configuration
// loading.
//
// It implements the hexagonal architecture pattern by providing concrete
// implementations of the ConfigPort interface defined in the ports layer.
// The ViperAdapter specifically uses the Viper library for configuration
// management with support for JSON files, environment variables, and
// hierarchical configuration resolution.
package config

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/spf13/viper"
)

// Ensure ViperAdapter implements ConfigPort at compile time.
var _ spi.ConfigPort = (*ViperAdapter)(nil)

// ViperAdapter implements the ConfigPort interface using spf13/viper.
// It loads configuration from lithos.json files, environment variables,
// and CLI flags with proper precedence handling.
//
// The adapter follows hexagonal architecture by implementing the SPI port
// contract while encapsulating all infrastructure concerns (file I/O, env vars,
// viper library usage).
type ViperAdapter struct {
	// logger provides structured logging for configuration loading operations.
	// Used to log config file discovery, parsing errors, and validation issues.
	logger zerolog.Logger
}

// NewViperAdapter creates a new ViperAdapter with the provided logger.
// The logger is injected for dependency inversion, allowing the adapter to
// log configuration loading operations without depending on global state.
//
// Parameters:
//   - logger: Configured zerolog.Logger instance for structured logging
//
// Returns:
//   - *ViperAdapter: Initialized adapter ready for configuration loading
//
// The constructor follows the factory pattern and ensures the adapter
// is properly initialized with all required dependencies.
//
//nolint:gocritic // logger is required for dependency injection
func NewViperAdapter(
	logger zerolog.Logger,
) *ViperAdapter {
	return &ViperAdapter{
		logger: logger,
	}
}

// Load implements the ConfigPort interface.
// It loads configuration from multiple sources with the following precedence:
// 1. CLI flags (reserved for future implementation)
// 2. Environment variables (LITHOS_* prefix)
// 3. Config file (lithos.json found via upward directory search)
// 4. Default values (lowest priority)
//
// The method handles context cancellation and provides detailed error messages
// for configuration validation failures.
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling
//
// Returns:
//   - domain.Config: Fully resolved configuration value object
//   - error: Non-nil if critical configuration validation fails
func (a *ViperAdapter) Load(ctx context.Context) (domain.Config, error) {
	// Start with default configuration
	cfg := domain.DefaultConfig()

	// Check for context cancellation early
	select {
	case <-ctx.Done():
		return domain.Config{}, ctx.Err()
	default:
	}

	// Search for config file (upward from CWD)
	vaultPath, err := a.searchConfigFile()
	if err != nil {
		// No config file found - use defaults as-is
		a.logger.Debug().Msg("No lithos.json config file found, using defaults")
	} else {
		// Config file found - load it
		a.logger.Debug().Str("vault_path", vaultPath).Msg("Found lithos.json config file")

		if loadErr := a.loadConfigFile(vaultPath, &cfg); loadErr != nil {
			// Log error but continue with defaults
			a.logger.Warn().Err(loadErr).Msg("Failed to read config file, falling back to defaults")
		}
	}

	// Override with environment variables (highest precedence except CLI flags)
	a.loadEnvironmentVars(&cfg)

	// Validate VaultPath
	if validateErr := a.validateVaultPath(cfg.VaultPath); validateErr != nil {
		return domain.Config{}, fmt.Errorf(
			"vault path validation failed: %w",
			validateErr,
		)
	}

	a.logger.Info().
		Str("vault_path", cfg.VaultPath).
		Str("templates_dir", cfg.TemplatesDir).
		Str("schemas_dir", cfg.SchemasDir).
		Msg("Configuration loaded successfully")

	return cfg, nil
}

// loadConfigFile attempts to load configuration from a lithos.json file
// found in the specified vault path directory.
//
// Parameters:
//   - vaultPath: Directory containing the lithos.json config file
//   - cfg: Configuration object to update with file values
//
// Returns:
//   - error: Non-nil if config file reading fails
func (a *ViperAdapter) loadConfigFile(
	vaultPath string,
	cfg *domain.Config,
) error {
	v := viper.New()
	v.SetConfigName("lithos")
	v.SetConfigType("json")
	v.AddConfigPath(vaultPath)

	if readErr := v.ReadInConfig(); readErr != nil {
		return readErr
	}

	// Override defaults with config file values
	// Use vault_path from config file if specified, otherwise use config file
	// directory
	if val := v.GetString("vault_path"); val != "" {
		cfg.VaultPath = val
	} else {
		cfg.VaultPath = vaultPath
	}
	if val := v.GetString("templates_dir"); val != "" {
		cfg.TemplatesDir = val
	}
	if val := v.GetString("schemas_dir"); val != "" {
		cfg.SchemasDir = val
	}
	if val := v.GetString("property_bank_file"); val != "" {
		cfg.PropertyBankFile = val
	}
	if val := v.GetString("cache_dir"); val != "" {
		cfg.CacheDir = val
	}
	if val := v.GetString("log_level"); val != "" {
		cfg.LogLevel = val
	}

	return nil
}

// searchConfigFile searches for lithos.json upward from the current working
// directory. It starts from CWD and traverses parent directories until a
// lithos.json file is found
// or the filesystem root is reached.
//
// Returns:
//   - string: Directory path containing lithos.json (becomes VaultPath)
//   - error: os.ErrNotExist if no config file found
//
// The search stops at the first lithos.json found, making the containing
// directory
// the effective vault root.
func (a *ViperAdapter) searchConfigFile() (string, error) {
	dir, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf(
			"failed to get current working directory: %w",
			err,
		)
	}

	for {
		configPath := filepath.Join(dir, "lithos.json")
		if _, statErr := os.Stat(configPath); statErr == nil {
			// Found config file
			return dir, nil
		}

		// Move to parent directory
		parent := filepath.Dir(dir)
		if parent == dir {
			// Reached filesystem root
			return "", os.ErrNotExist
		}
		dir = parent
	}
}

// loadEnvironmentVars loads LITHOS_* environment variables and overrides
// config.
// This implements the highest precedence level (except CLI flags).
//
// Environment variable mappings:
//   - LITHOS_VAULT_PATH → VaultPath
//   - LITHOS_TEMPLATES_DIR → TemplatesDir
//   - LITHOS_SCHEMAS_DIR → SchemasDir
//   - LITHOS_PROPERTY_BANK_FILE → PropertyBankFile
//   - LITHOS_CACHE_DIR → CacheDir
//   - LITHOS_LOG_LEVEL → LogLevel
func (a *ViperAdapter) loadEnvironmentVars(cfg *domain.Config) {
	// Use viper for automatic env binding
	v := viper.New()
	v.SetEnvPrefix("LITHOS")
	v.AutomaticEnv()

	// Define environment variable mappings with field setters
	envMappings := map[string]struct {
		fieldName string
		setter    func(*domain.Config, string)
	}{
		"VAULT_PATH": {
			fieldName: "vault_path",
			setter:    func(c *domain.Config, val string) { c.VaultPath = val },
		},
		"TEMPLATES_DIR": {
			fieldName: "templates_dir",
			setter:    func(c *domain.Config, val string) { c.TemplatesDir = val },
		},
		"SCHEMAS_DIR": {
			fieldName: "schemas_dir",
			setter:    func(c *domain.Config, val string) { c.SchemasDir = val },
		},
		"PROPERTY_BANK_FILE": {
			fieldName: "property_bank_file",
			setter:    func(c *domain.Config, val string) { c.PropertyBankFile = val },
		},
		"CACHE_DIR": {
			fieldName: "cache_dir",
			setter:    func(c *domain.Config, val string) { c.CacheDir = val },
		},
		"LOG_LEVEL": {
			fieldName: "log_level",
			setter:    func(c *domain.Config, val string) { c.LogLevel = val },
		},
	}

	// Override config with env vars if present
	for envKey, mapping := range envMappings {
		if val := v.GetString(envKey); val != "" {
			mapping.setter(cfg, val)
			a.logger.Debug().
				Str(mapping.fieldName, val).
				Msgf("Overriding %s from environment", mapping.fieldName)
		}
	}
}

// validateVaultPath validates that the VaultPath exists and is a directory.
// This is critical validation - the application cannot function without a valid
// vault.
//
// Parameters:
//   - path: The vault path to validate
//
// Returns:
//   - error: nil if valid, descriptive error if invalid
//
// Validation checks:
//   - Path exists on filesystem
//   - Path is a directory (not a file)
func (a *ViperAdapter) validateVaultPath(path string) error {
	info, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("vault path does not exist: %s", path)
		}
		return fmt.Errorf("cannot access vault path: %w", err)
	}

	if !info.IsDir() {
		return fmt.Errorf("vault path is not a directory: %s", path)
	}

	return nil
}
