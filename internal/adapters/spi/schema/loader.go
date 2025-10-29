// Package schema provides filesystem-based schema loading adapters.
// This package implements the SchemaPort interface using filesystem operations
// to load schemas and property bank definitions from configured directories.
package schema

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// SchemaLoaderAdapter implements SchemaPort by loading schemas and property
// bank
// from filesystem. It scans the configured SchemasDir for JSON files, loads the
// property bank first, then loads all schema definitions with duplicate
// detection.
//
// The adapter follows hexagonal architecture by implementing the SPI port
// contract
// while encapsulating filesystem operations and JSON parsing logic.
type SchemaLoaderAdapter struct {
	// config holds the application configuration for schema paths
	config *domain.Config
	// log provides structured logging for loading operations
	log *zerolog.Logger
}

// NewSchemaLoaderAdapter creates a new SchemaLoaderAdapter with configuration
// and logger. The adapter is initialized with the provided config and logger
// for dependency injection.
//
// Parameters:
//   - config: Application configuration containing schema directory paths
//   - log: Logger for debug/info/error messages during loading operations
//
// Returns a fully initialized SchemaLoaderAdapter ready for Load() operations.
func NewSchemaLoaderAdapter(
	config *domain.Config,
	log *zerolog.Logger,
) *SchemaLoaderAdapter {
	return &SchemaLoaderAdapter{
		config: config,
		log:    log,
	}
}

// Load retrieves all schemas and the property bank from the configured
// filesystem paths. It implements the SchemaPort interface by scanning the
// SchemasDir for JSON files, loading the property bank first, then loading all
// schema definitions with duplicate detection.
//
// The method follows the loading order specified in the architecture:
// 1. Load property bank first (required for $ref resolution)
// 2. Scan schemas directory for *.json files
// 3. Load and unmarshal each schema
// 4. Detect duplicate schema names
// 5. Return raw schemas + property bank
//
// Parameters:
//   - ctx: Context for cancellation and timeout handling during file operations
//
// Returns:
// - []domain.Schema: Raw schema definitions (no inheritance resolution applied)
//   - domain.PropertyBank: Shared property definitions for $ref resolution
//   - error: Loading, parsing, or validation errors with descriptive messages
//
// Error conditions:
//   - Missing property bank file: ResourceError with remediation hint
//   - Malformed JSON: SchemaError with file path and syntax details
//   - Duplicate schema names: SchemaError with list of duplicates
//   - File system access errors: Wrapped with context
//
// The returned schemas contain Extends/Excludes/Properties exactly as defined
// in the JSON files. SchemaResolver handles inheritance resolution separately.
func (a *SchemaLoaderAdapter) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	// 1. Load property bank first
	bankPath := a.config.PropertyBankPath()
	a.log.Debug().Str("path", bankPath).Msg("loading property bank")

	bank, err := a.loadPropertyBank(bankPath)
	if err != nil {
		return nil, domain.PropertyBank{}, fmt.Errorf(
			"failed to load property bank: %w",
			err,
		)
	}

	// 2. Scan and load schemas
	a.log.Debug().
		Str("dir", a.config.SchemasDir).
		Msg("scanning schemas directory")

	schemas, err := a.loadSchemas(a.config.SchemasDir)
	if err != nil {
		return nil, domain.PropertyBank{}, fmt.Errorf(
			"failed to load schemas: %w",
			err,
		)
	}

	// 3. Detect duplicates
	if err := a.checkDuplicates(schemas); err != nil {
		return nil, domain.PropertyBank{}, err
	}

	a.log.Debug().Int("count", len(schemas)).Msg("schema loading complete")

	return schemas, bank, nil
}

// loadPropertyBank loads and parses the property bank JSON file.
// It manually parses the JSON structure to maintain separation of concerns.
//
// Parameters:
//   - path: Full path to the property bank JSON file
//
// Returns:
// - domain.PropertyBank: Parsed property bank with properties converted from
// JSON
//   - error: ResourceError for missing files, SchemaError for malformed JSON
//
// The method preserves unknown JSON fields as required by FR6.
func (a *SchemaLoaderAdapter) loadPropertyBank(
	path string,
) (domain.PropertyBank, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return domain.PropertyBank{}, errors.NewResourceError(
				"property bank",
				"load",
				path,
				err,
			)
		}
		return domain.PropertyBank{}, err
	}

	// Parse property bank JSON manually to avoid domain layer serialization
	type propertyBankAlias struct {
		Properties map[string]interface{} `json:"properties"`
	}

	var alias propertyBankAlias
	if err := json.Unmarshal(data, &alias); err != nil {
		return domain.PropertyBank{}, errors.NewSchemaError(
			"malformed property bank JSON",
			path,
			err,
		)
	}

	// Convert properties map to domain.Property map
	properties := make(map[string]domain.Property)
	for propID, propValue := range alias.Properties {
		propData, err := json.Marshal(propValue)
		if err != nil {
			return domain.PropertyBank{}, errors.NewSchemaError(
				fmt.Sprintf("failed to marshal property %s", propID),
				path,
				err,
			)
		}

		prop, err := a.parsePropertyWithName("", propData)
		if err != nil {
			return domain.PropertyBank{}, errors.NewSchemaError(
				fmt.Sprintf("failed to parse property %s", propID),
				path,
				err,
			)
		}

		properties[propID] = prop
	}

	bank := domain.PropertyBank{
		Properties: properties,
	}

	return bank, nil
}

// loadSchemas scans the schemas directory and loads all JSON schema files.
// It filters out the property bank file and detects duplicate schema names.
// Only loads files directly in the specified directory, not in subdirectories.
//
// Parameters:
//   - dir: Path to the schemas directory
//
// Returns:
//   - []domain.Schema: List of loaded schemas with preserved unknown fields
//   - error: File system errors or duplicate name errors
//
// The method preserves unknown JSON fields as required by FR6.
func (a *SchemaLoaderAdapter) loadSchemas(dir string) ([]domain.Schema, error) {
	var schemas []domain.Schema

	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue // Skip subdirectories
		}

		filename := entry.Name()

		// Skip property bank file
		if filename == a.config.PropertyBankFile {
			continue
		}

		// Only process .json files
		if filepath.Ext(filename) != ".json" {
			continue
		}

		path := filepath.Join(dir, filename)
		schema, err := a.loadSchema(path)
		if err != nil {
			return nil, err
		}

		schemas = append(schemas, schema)
	}

	return schemas, nil
}

// loadSchema loads and parses a single schema JSON file.
// It manually parses the JSON structure to maintain separation of concerns.
//
// Parameters:
//   - path: Full path to the schema JSON file
//
// Returns:
//   - domain.Schema: Parsed schema with properties converted from JSON
//   - error: File system or JSON parsing errors
//
// The method preserves unknown JSON fields as required by FR6.
func (a *SchemaLoaderAdapter) loadSchema(path string) (domain.Schema, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return domain.Schema{}, err
	}

	// Parse schema JSON manually to avoid domain layer serialization
	type schemaAlias struct {
		Name       string                 `json:"name"`
		Extends    string                 `json:"extends,omitempty"`
		Excludes   []string               `json:"excludes,omitempty"`
		Properties map[string]interface{} `json:"properties"`
	}

	var alias schemaAlias
	if err := json.Unmarshal(data, &alias); err != nil {
		return domain.Schema{}, errors.NewSchemaError(
			"malformed schema JSON",
			path,
			err,
		)
	}

	// Convert properties map to domain.Property slice
	properties := make([]domain.Property, 0, len(alias.Properties))
	for propName, propValue := range alias.Properties {
		propData, err := json.Marshal(propValue)
		if err != nil {
			return domain.Schema{}, errors.NewSchemaError(
				fmt.Sprintf("failed to marshal property %s", propName),
				path,
				err,
			)
		}

		prop, err := a.parsePropertyWithName(propName, propData)
		if err != nil {
			return domain.Schema{}, errors.NewSchemaError(
				fmt.Sprintf("failed to parse property %s", propName),
				path,
				err,
			)
		}

		properties = append(properties, prop)
	}

	schema := domain.Schema{
		Name:       alias.Name,
		Extends:    alias.Extends,
		Excludes:   alias.Excludes,
		Properties: properties,
	}

	return schema, nil
}

// parsePropertyWithName unmarshals a property from JSON bytes with the property
// name.
// It handles both $ref references and full property definitions.
//
// Parameters:
//   - name: Property name from the schema properties map key
//   - data: JSON bytes representing a property definition
//
// Returns:
//   - domain.Property: Parsed property object with name set
//   - error: Parsing error if JSON is malformed or invalid
//
// This function decouples JSON parsing from the domain model,
// keeping the domain layer free of serialization concerns.
func (a *SchemaLoaderAdapter) parsePropertyWithName(
	name string,
	data []byte,
) (domain.Property, error) {
	// First, try to unmarshal as a $ref
	var refMap map[string]string
	if err := json.Unmarshal(data, &refMap); err == nil {
		if ref, ok := refMap["$ref"]; ok {
			return domain.Property{
				Name: name,
				Ref:  ref,
			}, nil
		}
	}

	// Not a $ref, unmarshal as full property definition
	type propertyAlias struct {
		Required bool            `json:"required"`
		Array    bool            `json:"array"`
		Ref      string          `json:"$ref,omitempty"`
		Spec     json.RawMessage `json:"spec,omitempty"`
	}

	var alias propertyAlias
	if err := json.Unmarshal(data, &alias); err != nil {
		return domain.Property{}, err
	}

	property := domain.Property{
		Name:     name,
		Required: alias.Required,
		Array:    alias.Array,
		Ref:      alias.Ref,
	}

	// If it's a $ref, spec should be empty
	if property.Ref != "" {
		if alias.Spec != nil {
			return domain.Property{}, fmt.Errorf(
				"property cannot have both $ref and spec",
			)
		}
		return property, nil
	}

	// Unmarshal the spec based on type
	if alias.Spec == nil {
		return domain.Property{}, fmt.Errorf(
			"property spec cannot be nil when not using $ref",
		)
	}

	var specMap map[string]interface{}
	if err := json.Unmarshal(alias.Spec, &specMap); err != nil {
		return domain.Property{}, err
	}

	typeVal, ok := specMap["type"].(string)
	if !ok {
		return domain.Property{}, fmt.Errorf(
			"property spec must have a 'type' field",
		)
	}

	var spec domain.PropertySpec
	switch domain.PropertyType(typeVal) {
	case domain.PropertyTypeString:
		spec = &domain.StringSpec{}
	case domain.PropertyTypeNumber:
		spec = &domain.NumberSpec{}
	case domain.PropertyTypeBool:
		spec = &domain.BoolSpec{}
	case domain.PropertyTypeDate:
		spec = &domain.DateSpec{}
	case domain.PropertyTypeFile:
		spec = &domain.FileSpec{}
	default:
		return domain.Property{}, fmt.Errorf(
			"unknown property type: %s",
			typeVal,
		)
	}

	if err := json.Unmarshal(alias.Spec, spec); err != nil {
		return domain.Property{}, err
	}

	property.Spec = spec
	return property, nil
}

// checkDuplicates validates that all schema names are unique.
// It returns an error if any duplicates are found.
//
// Parameters:
//   - schemas: List of loaded schemas to check
//
// Returns:
//   - error: SchemaError with list of duplicate names if found, nil otherwise
//
// The error message includes all duplicate names for comprehensive reporting.
func (a *SchemaLoaderAdapter) checkDuplicates(schemas []domain.Schema) error {
	nameCount := make(map[string]int)
	var duplicates []string

	for _, schema := range schemas {
		nameCount[schema.Name]++
		if nameCount[schema.Name] == 2 {
			duplicates = append(duplicates, schema.Name)
		}
	}

	if len(duplicates) > 0 {
		return errors.NewSchemaError(
			fmt.Sprintf("duplicate schema names found: %v", duplicates),
			"",
			nil,
		)
	}

	return nil
}
