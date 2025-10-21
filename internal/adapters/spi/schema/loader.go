// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains the main SchemaLoaderAdapter structure and coordination
// logic.
package schema

import (
	"context"
	"fmt"
	"path/filepath"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaLoaderAdapter implements SchemaLoaderPort for filesystem-based schema
// loading.
// Handles JSON parsing, PropertySpec discriminator logic, $ref resolution,
// and domain object creation.
//
// Architecture: SPI Adapter implementing SchemaLoaderPort
// Dependencies: FileSystemPort for file operations, ConfigPort for paths.
type SchemaLoaderAdapter struct {
	fs     spi.FileSystemPort
	config spi.ConfigPort
	refMap map[string]string // Tracks $ref mappings for property resolution
}

/* ---------------------------------------------------------- */
/*                         Constructor                        */
/* ---------------------------------------------------------- */

// NewSchemaLoaderAdapter creates a new SchemaLoaderAdapter with dependency
// injection.
func NewSchemaLoaderAdapter(
	fs spi.FileSystemPort,
	config spi.ConfigPort,
) *SchemaLoaderAdapter {
	return &SchemaLoaderAdapter{
		fs:     fs,
		config: config,
		refMap: make(map[string]string),
	}
}

/* ---------------------------------------------------------- */
/*               SchemaLoaderPort Implementation              */
/* ---------------------------------------------------------- */

// LoadSchemas implements SchemaLoaderPort.LoadSchemas.
func (s *SchemaLoaderAdapter) LoadSchemas(
	ctx context.Context,
) ([]domain.Schema, error) {
	schemasDir, err := s.getSchemasDirectory()
	if err != nil {
		return nil, err
	}

	var schemas []domain.Schema
	walkFn := s.createSchemaWalkFunction(ctx, schemasDir, &schemas)

	if walkErr := s.fs.Walk(schemasDir, walkFn); walkErr != nil {
		return nil, errors.NewFileSystemError("walk", schemasDir, walkErr)
	}

	return schemas, nil
}

// LoadPropertyBank implements SchemaLoaderPort.LoadPropertyBank.
func (s *SchemaLoaderAdapter) LoadPropertyBank(
	ctx context.Context,
) (*domain.PropertyBank, error) {
	propertiesDir, err := s.getPropertiesDirectory()
	if err != nil {
		return nil, err
	}

	propertyFiles, err := s.loadAllPropertyFiles(propertiesDir)
	if err != nil {
		return nil, err
	}

	bank := s.createPropertyBank(propertiesDir)

	if popErr := s.populatePropertyBank(propertyFiles, &bank); popErr != nil {
		return nil, popErr
	}

	return &bank, nil
}

/* ---------------------------------------------------------- */
/*                 Schema Loading Coordination                */
/* ---------------------------------------------------------- */

// getSchemasDirectory gets and validates the schemas directory path.
func (s *SchemaLoaderAdapter) getSchemasDirectory() (string, error) {
	cfg := s.config.Config()
	schemasDir := cfg.SchemasDir

	if err := s.validateDirectoryPath(schemasDir); err != nil {
		return "", errors.NewSchemaError(
			"schemas",
			fmt.Sprintf("invalid schemas directory: %v", err),
		)
	}

	return schemasDir, nil
}

// createSchemaWalkFunction creates a walk function for processing schema files.
func (s *SchemaLoaderAdapter) createSchemaWalkFunction(
	ctx context.Context,
	schemasDir string,
	schemas *[]domain.Schema,
) spi.WalkFunc {
	return func(path string, isDir bool) error {
		return s.processSchemaFileInWalk(ctx, path, isDir, schemasDir, schemas)
	}
}

// processSchemaFileInWalk processes a single file during schema directory walk.
func (s *SchemaLoaderAdapter) processSchemaFileInWalk(
	ctx context.Context,
	path string,
	isDir bool,
	schemasDir string,
	schemas *[]domain.Schema,
) error {
	if s.shouldSkipFile(path, isDir, schemasDir) {
		return nil
	}

	return s.loadAndAppendSchema(ctx, path, schemas)
}

// shouldSkipFile determines if a file should be skipped during walk.
func (s *SchemaLoaderAdapter) shouldSkipFile(
	path string,
	isDir bool,
	schemasDir string,
) bool {
	if isDir {
		return true
	}

	return !s.isValidSchemaFile(path, schemasDir)
}

// isValidSchemaFile checks if a file is a valid schema file to process.
func (s *SchemaLoaderAdapter) isValidSchemaFile(path, schemasDir string) bool {
	return s.isFileSecure(path, schemasDir) &&
		s.isJSONFile(path) &&
		s.isNotPropertyBankFile(path)
}

// loadAndAppendSchema loads a schema and appends it to the collection.
func (s *SchemaLoaderAdapter) loadAndAppendSchema(
	ctx context.Context,
	path string,
	schemas *[]domain.Schema,
) error {
	schema, err := s.loadSingleSchema(ctx, path)
	if err != nil {
		return s.wrapSchemaLoadError(path, err)
	}

	*schemas = append(*schemas, schema)
	return nil
}

// wrapSchemaLoadError wraps schema loading errors with context.
func (s *SchemaLoaderAdapter) wrapSchemaLoadError(
	path string,
	err error,
) error {
	return fmt.Errorf("failed to load schema from %s: %w", path, err)
}

/* ---------------------------------------------------------- */
/*             Property Bank Loading Coordination             */
/* ---------------------------------------------------------- */

// createPropertyBank creates a new property bank instance.
func (s *SchemaLoaderAdapter) createPropertyBank(
	propertiesDir string,
) domain.PropertyBank {
	return domain.NewPropertyBank(propertiesDir)
}

// populatePropertyBank resolves references and populates the property bank.
func (s *SchemaLoaderAdapter) populatePropertyBank(
	propertyFiles map[string]propertyBankDTO,
	bank *domain.PropertyBank,
) error {
	return s.resolvePropertyReferences(propertyFiles, bank)
}

// getPropertiesDirectory gets and validates the properties directory path.
func (s *SchemaLoaderAdapter) getPropertiesDirectory() (string, error) {
	cfg := s.config.Config()
	propertiesDir := filepath.Join(cfg.SchemasDir, "properties")

	if err := s.validateDirectoryPath(propertiesDir); err != nil {
		return "", errors.NewSchemaError(
			"properties",
			fmt.Sprintf("invalid properties directory: %v", err),
		)
	}

	return propertiesDir, nil
}

// loadAllPropertyFiles loads all property bank files from the properties
// directory.
func (s *SchemaLoaderAdapter) loadAllPropertyFiles(
	propertiesDir string,
) (map[string]propertyBankDTO, error) {
	propertyFiles := make(map[string]propertyBankDTO)

	walkFn := s.createPropertyBankWalkFunction(propertiesDir, propertyFiles)

	if err := s.fs.Walk(propertiesDir, walkFn); err != nil {
		return nil, errors.NewFileSystemError("walk", propertiesDir, err)
	}

	return propertyFiles, nil
}

// createPropertyBankWalkFunction creates a walk function for processing
// property bank files.
func (s *SchemaLoaderAdapter) createPropertyBankWalkFunction(
	propertiesDir string,
	propertyFiles map[string]propertyBankDTO,
) spi.WalkFunc {
	return func(path string, isDir bool) error {
		return s.processPropertyBankFileInWalk(
			path,
			isDir,
			propertiesDir,
			propertyFiles,
		)
	}
}

// processPropertyBankFileInWalk processes a single property bank file during
// walk.
func (s *SchemaLoaderAdapter) processPropertyBankFileInWalk(
	path string,
	isDir bool,
	propertiesDir string,
	propertyFiles map[string]propertyBankDTO,
) error {
	if s.shouldSkipPropertyBankFile(path, isDir) {
		return nil
	}

	if err := s.validatePropertyBankFilePath(path, propertiesDir); err != nil {
		return err
	}

	return s.loadAndStorePropertyBank(path, propertyFiles)
}

// shouldSkipPropertyBankFile determines if a property bank file should be
// skipped.
func (s *SchemaLoaderAdapter) shouldSkipPropertyBankFile(
	path string,
	isDir bool,
) bool {
	return isDir || !s.isJSONFile(path)
}

// validatePropertyBankFilePath validates the property bank file path.
func (s *SchemaLoaderAdapter) validatePropertyBankFilePath(
	path,
	propertiesDir string,
) error {
	if err := s.validateFilePath(path, propertiesDir); err != nil {
		return errors.NewSchemaError(
			path,
			fmt.Sprintf("security validation failed: %v", err),
		)
	}
	return nil
}

// loadAndStorePropertyBank loads a property bank file and stores it.
func (s *SchemaLoaderAdapter) loadAndStorePropertyBank(
	path string,
	propertyFiles map[string]propertyBankDTO,
) error {
	bankDTO, err := s.loadPropertyBankFile(path)
	if err != nil {
		return s.wrapPropertyBankLoadError(path, err)
	}

	propertyFiles[path] = bankDTO
	return nil
}

// wrapPropertyBankLoadError wraps property bank loading errors with context.
func (s *SchemaLoaderAdapter) wrapPropertyBankLoadError(
	path string,
	err error,
) error {
	return fmt.Errorf(
		"failed to load property bank from %s: %w",
		path,
		err,
	)
}

/* ---------------------------------------------------------- */
/*                File Operation Helper Methods               */
/* ---------------------------------------------------------- */

// readAndValidateFile reads a file and validates its size.
func (s *SchemaLoaderAdapter) readAndValidateFile(path string) ([]byte, error) {
	data, err := s.readFileData(path)
	if err != nil {
		return nil, err
	}

	if sizeErr := s.validateFileSize(path, data); sizeErr != nil {
		return nil, sizeErr
	}

	return data, nil
}

// readFileData reads file data from the filesystem.
func (s *SchemaLoaderAdapter) readFileData(path string) ([]byte, error) {
	data, err := s.fs.ReadFile(path)
	if err != nil {
		return nil, errors.NewFileSystemError("read", path, err)
	}
	return data, nil
}

// resolveAbsolutePath resolves a path to its absolute form.
func (s *SchemaLoaderAdapter) resolveAbsolutePath(path string) (string, error) {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return "", fmt.Errorf("failed to resolve absolute path: %w", err)
	}
	return absPath, nil
}

// loadSingleSchema loads and parses a single schema file.
func (s *SchemaLoaderAdapter) loadSingleSchema(
	_ context.Context,
	path string,
) (domain.Schema, error) {
	data, err := s.readAndValidateFile(path)
	if err != nil {
		return domain.Schema{}, err
	}

	dto, err := s.parseSchemaJSON(path, data)
	if err != nil {
		return domain.Schema{}, err
	}

	properties, err := s.convertDTOPropertiesToDomain(dto)
	if err != nil {
		return domain.Schema{}, err
	}

	return s.createAndValidateSchema(dto, properties)
}

// loadPropertyBankFile loads and parses a single property bank file.
func (s *SchemaLoaderAdapter) loadPropertyBankFile(
	path string,
) (propertyBankDTO, error) {
	data, err := s.readAndValidateFile(path)
	if err != nil {
		return propertyBankDTO{}, err
	}

	dto, err := s.parsePropertyBankJSON(path, data)
	if err != nil {
		return propertyBankDTO{}, err
	}

	return dto, nil
}
