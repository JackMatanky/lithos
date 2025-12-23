package domain

import (
	"path/filepath"
	"sync"
)

// Default configuration values for Config.
// These provide sensible defaults for vault structure and operational settings.
const (
	defaultVaultPath        = "."
	defaultTemplatesDir     = "templates/"
	defaultSchemasDir       = "schemas/"
	defaultPropertyBankFile = "property_bank.json"
	defaultCacheDir         = ".lithos/cache/"
	defaultLogLevel         = "info"
	defaultFileClassKey     = "file_class"
)

// Singleton pattern implementation for Config using sync.Once.
var (
	configInstance *Config
	configOnce     sync.Once
	configMu       sync.RWMutex // Protects instance during testing
)

// Config represents application configuration as an immutable value object.
// It defines vault structure and operational settings loaded from lithos.json
// and environment variables. Two Config instances with identical values are
// equivalent.
//
// Reference: docs/architecture/data-models.md#config.
type Config struct {
	// VaultPath is the root directory of the vault. Default: ".".
	// All relative paths in config are resolved relative to this.
	// Must exist and be readable.
	VaultPath string `json:"vault_path"`

	// TemplatesDir is the path to the templates directory. Default:
	// "{VaultPath}/templates/".
	// Can be absolute or relative to VaultPath.
	// Must exist for lithos new and lithos find commands.
	TemplatesDir string `json:"templates_dir"`

	// SchemasDir is the path to the schemas directory. Default:
	// "{VaultPath}/schemas/".
	// Can be absolute or relative to VaultPath.
	// Must exist if schemas are used.
	SchemasDir string `json:"schemas_dir"`

	// PropertyBankFile is the filename of the property bank file within
	// SchemasDir.
	// Default: "property_bank.json".
	// Full path is {SchemasDir}/{PropertyBankFile}.
	// Optional—if missing, schemas cannot use $ref references.
	PropertyBankFile string `json:"property_bank_file"`

	// CacheDir is the path to the index cache directory.
	// Default: "{VaultPath}/.lithos/cache/".
	// Can be absolute or relative to VaultPath.
	// Created automatically if missing. Must be writable.
	CacheDir string `json:"cache_dir"`

	// LogLevel is the logging verbosity for zerolog.
	// Default: "info". Options: "debug", "info", "warn", "error".
	// Case-insensitive. Invalid values fall back to "info" with warning.
	LogLevel string `json:"log_level"`

	// FileClassKey is the frontmatter key used to identify file class/schema.
	// Default: "file_class". Supports user preferences like "fileClass",
	// "type", etc.
	// Used consistently across all storage adapters and query operations.
	FileClassKey string `json:"file_class_key" yaml:"file_class_key" mapstructure:"file_class_key"`
}

// ConfigBuilder provides a fluent API for building Config objects with
// validation.
type ConfigBuilder struct {
	config     Config
	validators []ConfigValidator
}

// ConfigValidator defines validation logic for config fields.
type ConfigValidator interface {
	Validate(config Config) error
}

// NewConfigBuilder creates a new ConfigBuilder with default values.
func NewConfigBuilder() *ConfigBuilder {
	return &ConfigBuilder{
		config: Config{
			VaultPath:        ".",
			TemplatesDir:     "templates/",
			SchemasDir:       "schemas/",
			PropertyBankFile: "property_bank.json",
			CacheDir:         ".lithos/cache/",
			LogLevel:         "info",
			FileClassKey:     "file_class",
		},
		validators: []ConfigValidator{},
	}
}

// WithVaultPath sets the vault path.
func (b *ConfigBuilder) WithVaultPath(path string) *ConfigBuilder {
	b.config.VaultPath = path
	return b
}

// WithTemplatesDir sets the templates directory.
func (b *ConfigBuilder) WithTemplatesDir(dir string) *ConfigBuilder {
	b.config.TemplatesDir = dir
	return b
}

// WithSchemasDir sets the schemas directory.
func (b *ConfigBuilder) WithSchemasDir(dir string) *ConfigBuilder {
	b.config.SchemasDir = dir
	return b
}

// WithPropertyBankFile sets the property bank file name.
func (b *ConfigBuilder) WithPropertyBankFile(file string) *ConfigBuilder {
	b.config.PropertyBankFile = file
	return b
}

// WithCacheDir sets the cache directory.
func (b *ConfigBuilder) WithCacheDir(dir string) *ConfigBuilder {
	b.config.CacheDir = dir
	return b
}

// WithLogLevel sets the log level.
func (b *ConfigBuilder) WithLogLevel(level string) *ConfigBuilder {
	b.config.LogLevel = level
	return b
}

// WithFileClassKey sets the file class key.
func (b *ConfigBuilder) WithFileClassKey(key string) *ConfigBuilder {
	b.config.FileClassKey = key
	return b
}

// WithValidator adds a validator to the builder.
func (b *ConfigBuilder) WithValidator(
	validator ConfigValidator,
) *ConfigBuilder {
	b.validators = append(b.validators, validator)
	return b
}

// Build creates the final Config object with validation.
func (b *ConfigBuilder) Build() (Config, error) {
	// Apply defaults for relative paths
	config := b.applyDefaults(b.config)

	// Run validations
	for _, validator := range b.validators {
		if err := validator.Validate(config); err != nil {
			return Config{}, err
		}
	}

	return config, nil
}

// applyDefaults applies default values for relative paths.
func (b *ConfigBuilder) applyDefaults(config Config) Config {
	// If paths are relative, make them relative to VaultPath
	if !filepath.IsAbs(config.TemplatesDir) {
		config.TemplatesDir = filepath.Join(
			config.VaultPath,
			config.TemplatesDir,
		)
	}
	if !filepath.IsAbs(config.SchemasDir) {
		config.SchemasDir = filepath.Join(config.VaultPath, config.SchemasDir)
	}
	if !filepath.IsAbs(config.CacheDir) {
		config.CacheDir = filepath.Join(config.VaultPath, config.CacheDir)
	}
	return config
}

// NewConfig creates a Config with sensible defaults applied for empty values.
// Use this constructor when you want automatic defaults for unspecified fields.
// The Config is immutable after creation.
//
// Defaults applied:
// - VaultPath: current working directory (".")
// - TemplatesDir: "{VaultPath}/templates/"
// - SchemasDir: "{VaultPath}/schemas/"
// - PropertyBankFile: "property_bank.json"
// - CacheDir: "{VaultPath}/.lithos/cache/"
// - LogLevel: "info"
// - FileClassKey: "file_class".
func NewConfig(
	vaultPath, templatesDir, schemasDir, propertyBankFile, cacheDir, logLevel, fileClassKey string,
) Config {
	// Apply defaults for empty values
	if vaultPath == "" {
		vaultPath = defaultVaultPath
	}
	if templatesDir == "" {
		templatesDir = filepath.Join(vaultPath, "templates")
	}
	if schemasDir == "" {
		schemasDir = filepath.Join(vaultPath, "schemas")
	}
	if propertyBankFile == "" {
		propertyBankFile = defaultPropertyBankFile
	}
	if cacheDir == "" {
		cacheDir = filepath.Join(vaultPath, ".lithos", "cache")
	}
	if logLevel == "" {
		logLevel = defaultLogLevel
	}
	if fileClassKey == "" {
		fileClassKey = defaultFileClassKey
	}

	return Config{
		VaultPath:        vaultPath,
		TemplatesDir:     templatesDir,
		SchemasDir:       schemasDir,
		PropertyBankFile: propertyBankFile,
		CacheDir:         cacheDir,
		LogLevel:         logLevel,
		FileClassKey:     fileClassKey,
	}
}

// DefaultConfig returns a Config with sensible default values.
// Use this constructor for quickstart scenarios where minimal configuration is
// needed.
// The Config is immutable after creation.
func DefaultConfig() Config {
	return Config{
		VaultPath:        defaultVaultPath,
		TemplatesDir:     defaultTemplatesDir,
		SchemasDir:       defaultSchemasDir,
		PropertyBankFile: defaultPropertyBankFile,
		CacheDir:         defaultCacheDir,
		LogLevel:         defaultLogLevel,
		FileClassKey:     defaultFileClassKey,
	}
}

// PropertyBankPath returns the full path to the property bank file by joining
// SchemasDir with PropertyBankFile.
func (c Config) PropertyBankPath() string {
	return filepath.Join(c.SchemasDir, c.PropertyBankFile)
}

// Instance returns the singleton Config instance.
// Thread-safe initialization guaranteed by sync.Once.
// On first call, creates default Config. Subsequent calls return same instance.
func Instance() *Config {
	configMu.RLock()
	if configInstance != nil {
		defer configMu.RUnlock()
		return configInstance
	}
	configMu.RUnlock()

	configOnce.Do(func() {
		configMu.Lock()
		defer configMu.Unlock()
		cfg := DefaultConfig()
		configInstance = &cfg
	})

	configMu.RLock()
	defer configMu.RUnlock()
	return configInstance
}

// SetInstanceForTesting allows setting a custom Config instance for testing.
// This enables test isolation without global state pollution.
// Should only be used in tests. Use ResetConfigForTesting() in test cleanup.
func SetInstanceForTesting(cfg *Config) {
	configMu.Lock()
	defer configMu.Unlock()

	configInstance = cfg
}

// ResetConfigForTesting resets the singleton instance for test isolation.
// Should be called in test cleanup (typically via defer).
func ResetConfigForTesting() {
	configMu.Lock()
	defer configMu.Unlock()

	configOnce = sync.Once{}
	configInstance = nil
}
