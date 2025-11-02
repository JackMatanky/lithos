package domain

import "path/filepath"

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
	FileClassKey string `yaml:"file_class_key" mapstructure:"file_class_key"`
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
