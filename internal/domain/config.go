package domain

// Default configuration values for Config.
// These provide sensible defaults for vault structure and operational settings.
const (
	defaultVaultPath        = "."
	defaultTemplatesDir     = "templates/"
	defaultSchemasDir       = "schemas/"
	defaultPropertyBankFile = "property_bank.json"
	defaultCacheDir         = ".lithos/cache/"
	defaultLogLevel         = "info"
)

// Config represents application configuration as an immutable value object.
// It defines vault structure and operational settings loaded from lithos.json
// and environment variables. Two Config instances with identical values are
// equivalent.
type Config struct {
	// VaultPath is the root directory of the vault. Default: ".".
	// All relative paths in config are resolved relative to this.
	// Must exist and be readable.
	VaultPath string

	// TemplatesDir is the path to the templates directory. Default:
	// "templates/".
	// Can be absolute or relative to VaultPath.
	// Must exist for lithos new and lithos find commands.
	TemplatesDir string

	// SchemasDir is the path to the schemas directory. Default: "schemas/".
	// Can be absolute or relative to VaultPath.
	// Must exist if schemas are used.
	SchemasDir string

	// PropertyBankFile is the filename of the property bank file within
	// SchemasDir.
	// Default: "property_bank.json".
	// Full path is {SchemasDir}/{PropertyBankFile}.
	// Optional—if missing, schemas cannot use $ref references.
	PropertyBankFile string

	// CacheDir is the path to the index cache directory. Default:
	// ".lithos/cache/".
	// Can be absolute or relative to VaultPath.
	// Created automatically if missing. Must be writable.
	CacheDir string

	// LogLevel is the logging verbosity for zerolog. Default: "info".
	// One of: "debug", "info", "warn", "error".
	// Case-insensitive. Invalid values fall back to "info" with warning.
	LogLevel string
}

// NewConfig creates a Config with all fields explicitly set.
// Use this constructor when you have all configuration values available.
// The Config is immutable after creation.
func NewConfig(
	vaultPath, templatesDir, schemasDir, propertyBankFile, cacheDir, logLevel string,
) Config {
	return Config{
		VaultPath:        vaultPath,
		TemplatesDir:     templatesDir,
		SchemasDir:       schemasDir,
		PropertyBankFile: propertyBankFile,
		CacheDir:         cacheDir,
		LogLevel:         logLevel,
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
	}
}
