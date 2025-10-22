// Package schema provides filesystem-based implementations of schema loading
// ports for hexagonal architecture.
//
// This file contains the InheritanceResolver which implements the builder
// pattern for resolving schema inheritance chains with cycle detection and
// property merging according to AC requirements.
package schema

import (
	"context"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	sharederrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// InheritanceResolver implements the builder pattern for resolving schema
// inheritance chains. It processes schemas by building dependency graphs,
// detecting cycles, and resolving properties in topological order.
//
// The resolver follows these steps as per AC 2.6.1:
// 1. Load all schemas
// 2. Build dependency graph
// 3. Detect cycles
// 4. Resolve in topological order.
type InheritanceResolver struct {
	// lookup maps schema names to their definitions for fast access
	lookup map[string]domain.Schema
	// cache stores resolved schemas to avoid recomputation
	cache map[string]domain.Schema
}

// NewInheritanceResolver creates a new inheritance resolver with the provided
// schemas. All schemas are validated during construction.
//
// Returns error if any schema fails validation or if there are duplicate names.
func NewInheritanceResolver(
	schemas []domain.Schema,
) (*InheritanceResolver, error) {
	lookup := make(map[string]domain.Schema, len(schemas))
	cache := make(map[string]domain.Schema, len(schemas))

	// Build lookup table and validate all schemas
	for _, schema := range schemas {
		if err := schema.Validate(); err != nil {
			return nil, sharederrors.NewSchemaError(
				schema.Name,
				fmt.Sprintf("schema validation failed: %v", err),
				err,
			)
		}

		// Check for duplicate schema names
		if _, exists := lookup[schema.Name]; exists {
			return nil, sharederrors.NewSchemaError(
				schema.Name,
				"duplicate schema name found",
				nil,
			)
		}

		lookup[schema.Name] = schema
	}

	return &InheritanceResolver{
		lookup: lookup,
		cache:  cache,
	}, nil
}

// ResolveAll resolves inheritance for all schemas and returns a map of
// resolved schemas. This implements topological ordering as required by
// AC 2.6.1.
//
// Returns error if circular dependencies are detected or if resolution fails.
func (r *InheritanceResolver) ResolveAll(
	ctx context.Context,
) (map[string]domain.Schema, error) {
	// Process all schemas in the lookup table
	for name := range r.lookup {
		if _, err := r.resolveByName(ctx, name, nil); err != nil {
			return nil, err
		}
	}

	return r.cloneCache(), nil
}

// resolveByName resolves a single schema by name, handling inheritance chains
// and cycle detection. The stack parameter tracks the current resolution path
// for cycle detection as per AC 2.6.4.
func (r *InheritanceResolver) resolveByName(
	ctx context.Context,
	name string,
	stack []string,
) (domain.Schema, error) {
	// Check for context cancellation
	if err := ctx.Err(); err != nil {
		return domain.Schema{}, err
	}

	// Return cached result if already resolved
	if schema, exists := r.cache[name]; exists {
		return schema, nil
	}

	// Cycle detection: check if name is already in resolution stack
	if r.containsName(stack, name) {
		return domain.Schema{}, sharederrors.NewSchemaError(
			name,
			fmt.Sprintf(
				"cyclic inheritance detected: %s",
				r.formatCycle(stack, name),
			),
			nil,
		)
	}

	// Get schema from lookup table
	schema, exists := r.lookup[name]
	if !exists {
		return domain.Schema{}, sharederrors.NewSchemaError(
			name,
			"schema not found during resolution",
			nil,
		)
	}

	// Create a copy to avoid modifying the original
	schemaCopy := schema
	resolvedSchema, err := r.buildResolvedSchema(ctx, &schemaCopy, stack)
	if err != nil {
		return domain.Schema{}, err
	}

	// Cache the resolved schema
	r.cache[name] = resolvedSchema
	return resolvedSchema, nil
}

// buildResolvedSchema constructs a resolved schema with inheritance applied.
// This implements the core property resolution logic per AC 2.6.2 and 2.6.3.
func (r *InheritanceResolver) buildResolvedSchema(
	ctx context.Context,
	schema *domain.Schema,
	stack []string,
) (domain.Schema, error) {
	// Add current schema to stack for cycle detection
	currentStack := append(append([]string{}, stack...), schema.Name)

	// Resolve properties with inheritance
	resolvedProps, err := r.resolveProperties(ctx, schema, currentStack)
	if err != nil {
		return domain.Schema{}, err
	}

	// Create resolved schema with immutable properties (AC 2.6.5)
	resolvedSchema := *schema
	resolvedSchema.SetResolvedProperties(resolvedProps)
	return resolvedSchema, nil
}

// resolveProperties implements property resolution with inheritance as per
// AC 2.6.2: parent properties → apply Excludes → merge child properties.
func (r *InheritanceResolver) resolveProperties(
	ctx context.Context,
	schema *domain.Schema,
	stack []string,
) ([]domain.Property, error) {
	result := make([]domain.Property, 0, len(schema.Properties))

	// Step 1: Get parent properties if schema extends another
	parentName := strings.TrimSpace(schema.Extends)
	if parentName != "" {
		parentProps, err := r.resolveParentProperties(ctx, parentName, stack)
		if err != nil {
			return nil, err
		}
		result = append(result, parentProps...)
	}

	// Step 2: Merge child properties (overrides parent properties by name)
	result = r.mergeProperties(result, schema.Properties)

	// Step 3: Apply excludes to remove unwanted inherited properties
	return r.applyExcludes(result, schema.Excludes), nil
}

// resolveParentProperties gets the resolved properties from a parent schema.
// This ensures proper handling of multi-level inheritance chains per AC 2.6.3.
func (r *InheritanceResolver) resolveParentProperties(
	ctx context.Context,
	parentName string,
	stack []string,
) ([]domain.Property, error) {
	// Resolve parent schema recursively
	parent, err := r.resolveByName(ctx, parentName, stack)
	if err != nil {
		return nil, err
	}

	// Clone properties to ensure immutability
	return r.cloneProperties(parent.GetResolvedProperties()), nil
}

// mergeProperties overlays override properties onto base properties.
// Child properties with the same name replace parent properties,
// implementing proper override priority per AC 2.6.3.
func (r *InheritanceResolver) mergeProperties(
	base []domain.Property,
	overrides []domain.Property,
) []domain.Property {
	result := make([]domain.Property, 0, len(base)+len(overrides))
	index := make(map[string]int, len(base)+len(overrides))

	// Add all base properties first
	for i, prop := range base {
		result = append(result, prop)
		index[prop.Name] = i
	}

	// Add or override with child properties
	for _, prop := range overrides {
		if idx, exists := index[prop.Name]; exists {
			// Override existing property
			result[idx] = prop
		} else {
			// Add new property
			index[prop.Name] = len(result)
			result = append(result, prop)
		}
	}

	return result
}

// applyExcludes removes properties whose names match the excludes list.
// This implements subtractive inheritance as per AC 2.6.2.
func (r *InheritanceResolver) applyExcludes(
	props []domain.Property,
	excludes []string,
) []domain.Property {
	if len(excludes) == 0 {
		return props
	}

	excludeSet := r.buildExcludeSet(excludes)
	if len(excludeSet) == 0 {
		return props
	}

	return r.filterExcludedProperties(props, excludeSet)
}

// buildExcludeSet creates a set of property names to exclude.
func (r *InheritanceResolver) buildExcludeSet(
	excludes []string,
) map[string]struct{} {
	set := make(map[string]struct{}, len(excludes))
	for _, name := range excludes {
		trimmed := strings.TrimSpace(name)
		if trimmed != "" {
			set[trimmed] = struct{}{}
		}
	}
	return set
}

// filterExcludedProperties removes properties that are in the exclude set.
func (r *InheritanceResolver) filterExcludedProperties(
	props []domain.Property,
	excludeSet map[string]struct{},
) []domain.Property {
	filtered := make([]domain.Property, 0, len(props))
	for _, prop := range props {
		if _, skip := excludeSet[prop.Name]; !skip {
			filtered = append(filtered, prop)
		}
	}
	return filtered
}

// cloneCache returns a copy of the resolved schemas cache.
func (r *InheritanceResolver) cloneCache() map[string]domain.Schema {
	result := make(map[string]domain.Schema, len(r.cache))
	for name, schema := range r.cache {
		result[name] = schema
	}
	return result
}

// cloneProperties creates a copy of the provided properties slice.
// This ensures immutability as required by AC 2.6.5.
func (r *InheritanceResolver) cloneProperties(
	props []domain.Property,
) []domain.Property {
	cloned := make([]domain.Property, len(props))
	copy(cloned, props)
	return cloned
}

// containsName checks if the provided name exists in the resolution stack.
// Used for cycle detection per AC 2.6.4.
func (r *InheritanceResolver) containsName(stack []string, name string) bool {
	for _, existing := range stack {
		if existing == name {
			return true
		}
	}
	return false
}

// formatCycle formats the inheritance cycle path for error messages.
// Provides clear error messages as required by AC 2.6.4.
func (r *InheritanceResolver) formatCycle(stack []string, name string) string {
	cycle := make([]string, len(stack)+1)
	copy(cycle, stack)
	cycle[len(stack)] = name
	return strings.Join(cycle, " -> ")
}
