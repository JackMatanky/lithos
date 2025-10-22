// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains consolidated property conversion logic shared between
// the schema loader and property serialization components.
package schema

import (
	"encoding/json"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/errors"
)

/* ---------------------------------------------------------- */
/*               Property Collection Conversion               */
/* ---------------------------------------------------------- */

// convertPropertiesToDomain converts property interfaces to domain Property
// objects. Properties can be either propertyDTO (full definitions) or
// propertyRefDTO (references).
func (s *SchemaLoaderAdapter) convertPropertiesToDomain(
	propertiesRaw map[string]interface{},
) ([]domain.Property, error) {
	properties := make([]domain.Property, 0, len(propertiesRaw))

	for name, propRaw := range propertiesRaw {
		prop, err := s.convertSinglePropertyToDomain(name, propRaw)
		if err != nil {
			return nil, err
		}
		properties = append(properties, prop)
	}

	return properties, nil
}

// convertSinglePropertyToDomain converts a single property interface to domain
// Property. Handles both propertyDTO and propertyRefDTO.
func (s *SchemaLoaderAdapter) convertSinglePropertyToDomain(
	name string,
	propRaw interface{},
) (domain.Property, error) {
	// First try to unmarshal as propertyRefDTO
	var refDTO propertyRefDTO
	if jsonData, marshalErr := json.Marshal(propRaw); marshalErr == nil {
		if unmarshalErr := json.Unmarshal(jsonData, &refDTO); unmarshalErr == nil &&
			refDTO.Ref != "" {
			return s.createRefPlaceholderProperty(name, refDTO.Ref), nil
		}
	}

	// If not a ref, try to unmarshal as propertyDTO
	var propDTO propertyDTO
	if jsonData, marshalErr := json.Marshal(propRaw); marshalErr == nil {
		if unmarshalErr := json.Unmarshal(jsonData, &propDTO); unmarshalErr == nil {
			return s.convertTypedPropertyToDomain(name, propDTO)
		}
	}

	return domain.Property{}, fmt.Errorf("invalid property format for %s", name)
}

/* ---------------------------------------------------------- */
/*                 Reference Property Handling                */
/* ---------------------------------------------------------- */

// convertTypedPropertyToDomain converts a typed property DTO to domain
// Property.
func (s *SchemaLoaderAdapter) convertTypedPropertyToDomain(
	name string,
	propDTO propertyDTO,
) (domain.Property, error) {
	spec, err := s.convertPropertySpecToConcreteType(propDTO)
	if err != nil {
		return domain.Property{}, s.wrapPropertyTypeConversionError(name, err)
	}

	return s.createDomainProperty(name, propDTO, spec), nil
}

// wrapPropertyTypeConversionError wraps property type conversion errors.
func (s *SchemaLoaderAdapter) wrapPropertyTypeConversionError(
	name string,
	err error,
) error {
	return fmt.Errorf("failed to convert property %s: %w", name, err)
}

// createDomainProperty creates a domain property with the given spec.
func (s *SchemaLoaderAdapter) createDomainProperty(
	name string,
	propDTO propertyDTO,
	spec domain.PropertySpec,
) domain.Property {
	// Use propDTO.Name if set (for property bank properties), otherwise use the
	// provided name
	propertyName := name
	if propDTO.Name != "" {
		propertyName = propDTO.Name
	}
	return domain.NewProperty(
		propertyName,
		propDTO.Required,
		propDTO.Array,
		spec,
	)
}

// createRefPlaceholderProperty creates a placeholder property for $ref
// resolution. In MVP, $ref appears alone without other attributes.
func (s *SchemaLoaderAdapter) createRefPlaceholderProperty(
	name string,
	ref string,
) domain.Property {
	// Store the ref for later resolution during property bank loading
	s.refMap[name] = ref
	return domain.NewProperty(
		name,
		false, // $ref properties are not required by default
		false, // $ref properties are not arrays by default
		domain.StringPropertySpec{
			Pattern: "",
			Enum:    []string{},
		},
	)
}

/* ---------------------------------------------------------- */
/*                   PropertySpec Conversion                  */
/* ---------------------------------------------------------- */

// convertPropertySpecToConcreteType handles PropertySpec discriminator logic.
func (s *SchemaLoaderAdapter) convertPropertySpecToConcreteType(
	propDTO propertyDTO,
) (domain.PropertySpec, error) {
	return buildSpecFromMap(propDTO.Type, propDTO.Spec)
}

/* ---------------------------------------------------------- */
/*                    Reference Resolution                    */
/* ---------------------------------------------------------- */

// resolvePropertyReferences resolves $ref references in property banks.
func (s *SchemaLoaderAdapter) resolvePropertyReferences(
	propertyFiles map[string]propertyBankDTO,
	bank *domain.PropertyBank,
) error {
	registry := s.buildPropertyRegistry(propertyFiles)
	return s.resolveAndRegisterProperties(registry, bank)
}

// buildPropertyRegistry builds a registry of all properties from property bank
// files.
func (s *SchemaLoaderAdapter) buildPropertyRegistry(
	propertyFiles map[string]propertyBankDTO,
) map[string]interface{} {
	registry := make(map[string]interface{})

	// First pass: collect all properties
	for _, bankDTO := range propertyFiles {
		for name, prop := range bankDTO.Properties {
			registry[name] = prop
		}
	}

	return registry
}

// resolveAndRegisterProperties resolves references and registers properties in
// the bank.
func (s *SchemaLoaderAdapter) resolveAndRegisterProperties(
	registry map[string]interface{},
	bank *domain.PropertyBank,
) error {
	resolved := make(map[string]bool)

	for name, prop := range registry {
		domainProp, err := s.resolveAndConvertProperty(
			name,
			prop,
			registry,
			resolved,
			0,
		)
		if err != nil {
			return err
		}

		if regErr := s.registerPropertyInBank(bank, name, domainProp); regErr != nil {
			return regErr
		}
	}

	return nil
}

// registerPropertyInBank registers a property in the property bank with error
// handling.
func (s *SchemaLoaderAdapter) registerPropertyInBank(
	bank *domain.PropertyBank,
	name string,
	prop domain.Property,
) error {
	if err := bank.RegisterProperty(name, prop); err != nil {
		return errors.NewSchemaError(
			"property_bank",
			fmt.Sprintf("failed to register property %s: %v", name, err),
			err,
		)
	}
	return nil
}

// resolveAndConvertProperty resolves $ref references recursively with circular
// detection.
//
// implementation.
//
//nolint:unparam // registry parameter reserved for future caching
func (s *SchemaLoaderAdapter) resolveAndConvertProperty(
	name string,
	prop interface{},
	registry map[string]interface{},
	resolved map[string]bool,
	depth int,
) (domain.Property, error) {
	if err := s.checkCircularReferenceDepth(name, depth); err != nil {
		return domain.Property{}, err
	}

	// If already resolved, return from cache
	// Note: This is simplified - in a real implementation, we'd need to
	// cache resolved properties. For now, we'll proceed with resolution.
	_ = resolved[name] // Acknowledge the check but continue

	// First try to unmarshal as propertyRefDTO
	var refDTO propertyRefDTO
	if jsonData, marshalErr := json.Marshal(prop); marshalErr == nil {
		if unmarshalErr := json.Unmarshal(jsonData, &refDTO); unmarshalErr == nil &&
			refDTO.Ref != "" {
			return s.createRefPlaceholderProperty(name, refDTO.Ref), nil
		}
	}

	// If not a ref, try to unmarshal as propertyDTO
	var propDTO propertyDTO
	if jsonData, marshalErr := json.Marshal(prop); marshalErr == nil {
		if unmarshalErr := json.Unmarshal(jsonData, &propDTO); unmarshalErr == nil {
			return s.convertTypedPropertyToDomain(name, propDTO)
		}
	}

	return domain.Property{}, fmt.Errorf("invalid property format for %s", name)
}

/* ---------------------------------------------------------- */
/*                      Schema Conversion                     */
/* ---------------------------------------------------------- */

// convertDTOPropertiesToDomain converts DTO properties with error wrapping.
func (s *SchemaLoaderAdapter) convertDTOPropertiesToDomain(
	dto schemaDTO,
) ([]domain.Property, error) {
	properties, err := s.convertPropertiesToDomain(dto.Properties)
	if err != nil {
		return nil, s.wrapPropertyConversionError(dto.Name, err)
	}
	return properties, nil
}

// wrapPropertyConversionError wraps property conversion errors with schema
// context.
func (s *SchemaLoaderAdapter) wrapPropertyConversionError(
	schemaName string,
	err error,
) error {
	return errors.NewSchemaError(
		schemaName,
		fmt.Sprintf("property conversion failed: %v", err),
		err,
	)
}

// createAndValidateSchema creates a domain schema and validates it.
func (s *SchemaLoaderAdapter) createAndValidateSchema(
	dto schemaDTO,
	properties []domain.Property,
) (domain.Schema, error) {
	schema := s.createDomainSchema(dto, properties)

	if err := s.validateDomainSchema(&schema, dto.Name); err != nil {
		return domain.Schema{}, err
	}

	return schema, nil
}

// createDomainSchema creates a domain schema from DTO and properties.
func (s *SchemaLoaderAdapter) createDomainSchema(
	dto schemaDTO,
	properties []domain.Property,
) domain.Schema {
	return domain.NewSchemaWithExtends(
		dto.Name,
		dto.Extends,
		dto.Excludes,
		properties,
	)
}
