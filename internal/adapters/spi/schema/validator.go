package schema

import (
	"context"
	"errors"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
)

// SchemaValidator orchestrates validation of schemas and property bank.
// It performs both model-level validation (structural integrity) and
// cross-schema validation (inheritance and reference integrity).
//
// SchemaValidator is instantiated internally by SchemaEngine and has no
// external dependencies. It orchestrates validation but does not perform
// inheritance resolution - that's handled by SchemaResolver.
//
// Architecture Reference: docs/architecture/components.md#schemavalidator
// Requirements: FR5 (Schema Loading), FR7 (Schema Registry) from
// docs/prd/requirements.md
//
// Validation Process:
//  1. Model validation: Calls schema.Validate() on each schema
//  2. Cross-schema validation: Checks Extends references, duplicates, $ref
//     validity
//  3. Error aggregation: Combines all errors using errors.Join()
//
// Distinction from SchemaResolver:
//   - SchemaValidator: Ensures schemas are structurally valid and references
//     exist
//   - SchemaResolver: Performs inheritance resolution and $ref substitution
type SchemaValidator struct{}

// NewSchemaValidator creates a new SchemaValidator instance.
// SchemaValidator has no dependencies and is pure domain logic.
func NewSchemaValidator() *SchemaValidator {
	return &SchemaValidator{}
}

// ValidateAll performs comprehensive validation of schemas and property bank.
// It orchestrates both model-level and cross-schema validation, aggregating
// all errors for comprehensive reporting.
//
// Validation Steps:
//  1. Model validation: Calls schema.Validate() for each schema
//  2. Cross-schema validation: Checks Extends, duplicates, $ref references
//  3. Error aggregation: Combines all errors using errors.Join()
//
// Returns nil if all validation passes.
// Returns aggregated error if any validation fails.
//
// Context is used for cancellation during potentially long-running validation.
func (v *SchemaValidator) ValidateAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	var errs []error

	// 1. Orchestrate model-level validation
	modelErrs := v.validateModels(ctx, schemas)
	errs = append(errs, modelErrs...)

	// 2. Cross-schema validation
	crossErrs := v.validateCrossSchema(ctx, schemas)
	errs = append(errs, crossErrs...)

	// 3. Aggregate all errors
	if len(errs) > 0 {
		return errors.Join(errs...)
	}

	return nil
}

// validateModels orchestrates model-level validation by calling
// schema.Validate() on each schema and wrapping errors with schema context.
func (v *SchemaValidator) validateModels(
	ctx context.Context,
	schemas []domain.Schema,
) []error {
	var errs []error

	for _, schema := range schemas {
		// Check for cancellation
		if err := ctx.Err(); err != nil {
			return []error{err}
		}

		if err := schema.Validate(ctx); err != nil {
			errs = append(errs, fmt.Errorf("schema %s: %w", schema.Name, err))
		}
	}

	return errs
}

// validateCrossSchema performs cross-schema validation including Extends
// references, duplicate names, and $ref validity.
func (v *SchemaValidator) validateCrossSchema(
	ctx context.Context,
	schemas []domain.Schema,
) []error {
	var errs []error

	// Build schema map for reference checking
	schemaMap := v.buildSchemaMap(schemas)

	// Check Extends references
	extendsErrs := v.validateExtendsReferences(schemas, schemaMap)
	errs = append(errs, extendsErrs...)

	// Check for unique schema names
	uniqueErrs := v.validateUniqueSchemaNames(schemas)
	errs = append(errs, uniqueErrs...)

	// Check property references (basic validation)
	refErrs := v.validatePropertyRefs(ctx, schemas)
	errs = append(errs, refErrs...)

	return errs
}

// buildSchemaMap creates a lookup map from schema names to schemas.
func (v *SchemaValidator) buildSchemaMap(
	schemas []domain.Schema,
) map[string]domain.Schema {
	schemaMap := make(map[string]domain.Schema, len(schemas))
	for _, schema := range schemas {
		schemaMap[schema.Name] = schema
	}
	return schemaMap
}

// validateExtendsReferences checks that all Extends references point to
// existing schemas.
func (v *SchemaValidator) validateExtendsReferences(
	schemas []domain.Schema,
	schemaMap map[string]domain.Schema,
) []error {
	var errs []error

	for _, schema := range schemas {
		if schema.Extends != "" {
			if _, exists := schemaMap[schema.Extends]; !exists {
				errs = append(errs, fmt.Errorf(
					"schema %s extends non-existent schema %s",
					schema.Name, schema.Extends,
				))
			}
		}
	}

	return errs
}

// validateUniqueSchemaNames checks for duplicate schema names across all
// schemas.
func (v *SchemaValidator) validateUniqueSchemaNames(
	schemas []domain.Schema,
) []error {
	seen := make(map[string]bool)
	var errs []error

	for _, schema := range schemas {
		if seen[schema.Name] {
			errs = append(errs, fmt.Errorf(
				"duplicate schema name: %s", schema.Name,
			))
		}
		seen[schema.Name] = true
	}

	return errs
}

// validatePropertyRefs checks that property references in schemas are valid.
// Since $ref resolution now happens in the DTO layer and all properties are
// concrete Property entities created via NewProperty (which validates them),
// this method is now a no-op for future extensibility.
func (v *SchemaValidator) validatePropertyRefs(
	ctx context.Context,
	schemas []domain.Schema,
) []error {
	// All property validation now happens during Property construction
	// in the DTO layer. No additional validation needed here.
	return nil
}
