// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains validation functions including security validation
// for file paths, directory bounds checking, file size validation, and
// other safety checks for the schema loader.
package schema

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

/* ---------------------------------------------------------- */
/*                  Directory Path Validation                 */
/* ---------------------------------------------------------- */

// validateDirectoryPath validates a directory path for security.
func (s *SchemaLoaderAdapter) validateDirectoryPath(dirPath string) error {
	if err := s.checkPathTraversal(dirPath); err != nil {
		return err
	}

	absPath, err := s.resolveAbsolutePath(dirPath)
	if err != nil {
		return err
	}

	return s.checkVaultBounds(absPath)
}

// checkPathTraversal checks for path traversal attempts.
func (s *SchemaLoaderAdapter) checkPathTraversal(path string) error {
	if strings.Contains(path, "..") {
		return fmt.Errorf("path traversal attempt detected")
	}
	return nil
}

// checkVaultBounds ensures the path is within vault bounds.
func (s *SchemaLoaderAdapter) checkVaultBounds(absPath string) error {
	cfg := s.config.Config()
	vaultPath, err := filepath.Abs(cfg.VaultPath)
	if err != nil {
		return fmt.Errorf("failed to resolve vault path: %w", err)
	}

	if !strings.HasPrefix(absPath, vaultPath) {
		return fmt.Errorf("directory path outside vault bounds")
	}

	return nil
}

/* ---------------------------------------------------------- */
/*                    File Path Validation                    */
/* ---------------------------------------------------------- */

// validateFilePath validates a file path for security and compliance.
func (s *SchemaLoaderAdapter) validateFilePath(filePath, baseDir string) error {
	if err := s.checkPathTraversal(filePath); err != nil {
		return err
	}

	if err := s.checkFileExtension(filePath); err != nil {
		return err
	}

	return s.checkFileInBaseDirectory(filePath, baseDir)
}

// checkFileExtension validates that the file has a .json extension.
func (s *SchemaLoaderAdapter) checkFileExtension(filePath string) error {
	if !strings.HasSuffix(strings.ToLower(filePath), ".json") {
		return fmt.Errorf("invalid file extension: only .json files allowed")
	}
	return nil
}

// checkFileInBaseDirectory ensures the file is within the base directory.
func (s *SchemaLoaderAdapter) checkFileInBaseDirectory(
	filePath,
	baseDir string,
) error {
	absPath, err := filepath.Abs(filePath)
	if err != nil {
		return fmt.Errorf("failed to resolve file path: %w", err)
	}

	absBaseDir, err := filepath.Abs(baseDir)
	if err != nil {
		return fmt.Errorf("failed to resolve base directory: %w", err)
	}

	if !strings.HasPrefix(absPath, absBaseDir) {
		return fmt.Errorf("file path outside allowed directory bounds")
	}

	return nil
}

/* ---------------------------------------------------------- */
/*                   File Content Validation                  */
/* ---------------------------------------------------------- */

// validateFileSize validates that a file is within size limits.
func (s *SchemaLoaderAdapter) validateFileSize(path string, data []byte) error {
	const (
		maxFileSize = 10 * 1024 * 1024 // 10MB
		bytesToMB   = 1024 * 1024
	)

	if len(data) > maxFileSize {
		return errors.NewSchemaError(
			path,
			fmt.Sprintf(
				"file size (%.1fMB) exceeds maximum allowed size (10MB)",
				float64(len(data))/bytesToMB,
			),
		)
	}

	return nil
}

/* ---------------------------------------------------------- */
/*                     File Type Checking                     */
/* ---------------------------------------------------------- */

// isFileSecure checks if the file path is secure and within bounds.
func (s *SchemaLoaderAdapter) isFileSecure(path, schemasDir string) bool {
	// Security validation: ensure file path is within bounds
	if err := s.validateFilePath(path, schemasDir); err != nil {
		// Log error but don't fail - just skip the file
		return false
	}
	return true
}

// isJSONFile checks if the file has a JSON extension.
func (s *SchemaLoaderAdapter) isJSONFile(path string) bool {
	return strings.HasSuffix(strings.ToLower(path), ".json")
}

// isNotPropertyBankFile checks if the file is not a property bank file.
func (s *SchemaLoaderAdapter) isNotPropertyBankFile(path string) bool {
	return !strings.Contains(path, "properties/")
}

/* ---------------------------------------------------------- */
/*               Reference Resolution Validation              */
/* ---------------------------------------------------------- */

// checkCircularReferenceDepth validates that resolution depth hasn't exceeded
// limits.
func (s *SchemaLoaderAdapter) checkCircularReferenceDepth(
	name string,
	depth int,
) error {
	const maxDepth = 10
	if depth > maxDepth {
		return errors.NewSchemaError(
			"property_bank",
			fmt.Sprintf(
				"circular reference detected: max depth exceeded (10) for property %s",
				name,
			),
		)
	}
	return nil
}

// checkSelfReference validates that a property doesn't reference itself.
func (s *SchemaLoaderAdapter) checkSelfReference(name, refName string) error {
	if refName == name {
		return errors.NewSchemaError(
			"property_bank",
			fmt.Sprintf(
				"circular reference detected: property %s references itself",
				name,
			),
		)
	}
	return nil
}

/* ---------------------------------------------------------- */
/*                  Domain Object Validation                  */
/* ---------------------------------------------------------- */

// validateDomainSchema validates a domain schema object.
func (s *SchemaLoaderAdapter) validateDomainSchema(
	schema *domain.Schema,
	schemaName string,
) error {
	if err := schema.Validate(); err != nil {
		return errors.NewSchemaError(
			schemaName,
			fmt.Sprintf("schema validation failed: %v", err),
		)
	}
	return nil
}
