package schema

import (
	"context"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	lithoserrors "github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaResolver resolves inheritance and $ref substitution for schemas.
// It transforms schemas with inheritance chains into flattened, resolved
// schemas with complete property sets ready for validation and consumption.
//
// SchemaResolver has no external dependencies and is instantiated internally
// by SchemaEngine. It assumes all input schemas have passed validation via
// SchemaValidator.
//
// Architecture Reference: docs/architecture/components.md#schemaresolver
// Requirements: FR5 (Schema Loading), FR9 (Schema Registry) from
// docs/prd/requirements.md
//
// Resolution Algorithm:
//  1. Build dependency graph mapping each schema to its Extends parent
//  2. Detect circular inheritance chains using depth-first search
//  3. Perform topological sort to order schemas (parents resolve before
//     children)
//  4. For each schema in order:
//     - Get parent's resolved properties (or empty if root schema)
//     - Apply Excludes (remove properties by name)
//     - Merge child properties (override parent properties with same name)
//     - Substitute $ref references with PropertyBank definitions
//     - Store as ResolvedProperties in new Schema copy
//
// Property Override Semantics:
//   - If child Property.Name matches parent Property.Name, child completely
//     replaces parent
//   - This is explicit override, not attribute-level merge
//   - Properties not overridden or excluded are inherited from parent
//
// Immutability:
//   - Original schemas are never mutated
//   - Returns new schema copies with ResolvedProperties populated
//   - Original Extends/Excludes/Properties remain unchanged
//
// Distinction from SchemaValidator:
//   - SchemaValidator: Ensures schemas are structurally valid and references
//     exist
//   - SchemaResolver: Performs inheritance resolution and $ref substitution.
type SchemaResolver struct{}

// NewSchemaResolver creates a new SchemaResolver instance.
// SchemaResolver has no dependencies and is pure domain logic.
func NewSchemaResolver() *SchemaResolver {
	return &SchemaResolver{}
}

// Resolve performs comprehensive inheritance resolution and $ref substitution.
// It transforms schemas with inheritance into flattened resolved schemas.
//
// Resolution Steps:
//  1. Build dependency graph for inheritance chains
//  2. Detect circular dependencies (fail-fast)
//  3. Topological sort (parents resolve before children)
//  4. Resolve each schema in order (inheritance + $ref substitution)
//
// Returns new schema copies with ResolvedProperties populated.
// Original schemas are never mutated.
//
// Context is used for cancellation during potentially long-running resolution.
func (r *SchemaResolver) Resolve(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) ([]domain.Schema, error) {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	// Build dependency graph
	graph := r.buildDependencyGraph(schemas)

	// Detect circular dependencies
	if err := r.detectCycles(schemas, graph); err != nil {
		return nil, err
	}

	// Topological sort
	sortedSchemas := r.topologicalSort(schemas, graph)

	// Resolve schemas in order
	resolved := make([]domain.Schema, 0, len(schemas))
	resolvedMap := make(map[string]domain.Schema)

	for _, schema := range sortedSchemas {
		// Check for cancellation
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		resolvedSchema, err := r.resolveSchema(ctx, schema, resolvedMap, bank)
		if err != nil {
			return nil, err
		}

		resolved = append(resolved, resolvedSchema)
		resolvedMap[resolvedSchema.Name] = resolvedSchema
	}

	return resolved, nil
}

// buildDependencyGraph creates a map from schema name to parent name.
func (r *SchemaResolver) buildDependencyGraph(
	schemas []domain.Schema,
) map[string]string {
	graph := make(map[string]string)
	for _, schema := range schemas {
		if schema.Extends != "" {
			graph[schema.Name] = schema.Extends
		}
	}
	return graph
}

// detectCycles uses depth-first search to detect circular inheritance chains.
func (r *SchemaResolver) detectCycles(
	schemas []domain.Schema,
	graph map[string]string,
) error {
	visited := make(map[string]bool)
	recStack := make(map[string]bool) // Recursion stack for current path

	var dfs func(name string, path []string) error
	dfs = func(name string, path []string) error {
		visited[name] = true
		recStack[name] = true
		path = append(path, name)

		if parent, hasParent := graph[name]; hasParent {
			if err := r.processParentInCycle(parent, name, path, recStack, visited, dfs); err != nil {
				return err
			}
		}

		recStack[name] = false
		return nil
	}

	for _, schema := range schemas {
		if !visited[schema.Name] {
			if err := dfs(schema.Name, []string{}); err != nil {
				return err
			}
		}
	}

	return nil
}

// processParentInCycle handles parent processing during cycle detection.
func (r *SchemaResolver) processParentInCycle(
	parent, name string,
	path []string,
	recStack, visited map[string]bool,
	dfs func(string, []string) error,
) error {
	if recStack[parent] {
		// Cycle detected - build cycle path
		cyclePath := strings.Join(append(path, parent), " → ")
		return lithoserrors.NewSchemaErrorWithRemediation(
			fmt.Sprintf("circular inheritance: %s", cyclePath),
			name,
			"remove circular dependency by breaking inheritance chain",
			nil,
		)
	}
	if !visited[parent] {
		return dfs(parent, path)
	}
	return nil
}

// topologicalSort orders schemas so parents resolve before children.
func (r *SchemaResolver) topologicalSort(
	schemas []domain.Schema,
	graph map[string]string,
) []domain.Schema {
	sorted := make([]domain.Schema, 0, len(schemas))
	visited := make(map[string]bool)
	schemaMap := r.buildSchemaMap(schemas)

	var visit func(name string)
	visit = func(name string) {
		if visited[name] {
			return
		}

		visited[name] = true

		// Visit parent first (if exists)
		if parent, hasParent := graph[name]; hasParent {
			visit(parent)
		}

		// Add current schema after parent
		if schema, exists := schemaMap[name]; exists {
			sorted = append(sorted, schema)
		}
	}

	for _, schema := range schemas {
		visit(schema.Name)
	}

	return sorted
}

// buildSchemaMap creates a lookup map from schema names to schemas.
func (r *SchemaResolver) buildSchemaMap(
	schemas []domain.Schema,
) map[string]domain.Schema {
	schemaMap := make(map[string]domain.Schema, len(schemas))
	for _, schema := range schemas {
		schemaMap[schema.Name] = schema
	}
	return schemaMap
}

// resolveSchema resolves a single schema's inheritance and $ref substitution.
func (r *SchemaResolver) resolveSchema(
	ctx context.Context,
	schema domain.Schema,
	resolvedMap map[string]domain.Schema,
	bank domain.PropertyBank,
) (domain.Schema, error) {
	// Get parent's resolved properties
	var parentProps []domain.Property
	if schema.Extends != "" {
		if parent, exists := resolvedMap[schema.Extends]; exists {
			parentProps = parent.ResolvedProperties
		}
		// If parent doesn't exist in resolvedMap, it means no parent properties
		// (this should not happen if topological sort worked correctly)
	}

	// Resolve properties (inheritance + excludes + overrides)
	resolvedProps := r.resolveProperties(schema, parentProps)

	// Substitute $ref references
	finalProps, err := r.substituteRefs(ctx, resolvedProps, bank, schema.Name)
	if err != nil {
		return domain.Schema{}, err
	}

	// Create resolved schema copy (preserve original fields)
	resolved := domain.Schema{
		Name:               schema.Name,
		Extends:            schema.Extends,
		Excludes:           schema.Excludes,
		Properties:         schema.Properties,
		ResolvedProperties: finalProps,
	}

	return resolved, nil
}

// resolveProperties applies inheritance, excludes, and property overrides.
func (r *SchemaResolver) resolveProperties(
	schema domain.Schema,
	parentProps []domain.Property,
) []domain.Property {
	// Start with parent properties
	resolved := make(
		[]domain.Property,
		0,
		len(parentProps)+len(schema.Properties),
	)

	// Apply Excludes
	excludeSet := make(map[string]bool)
	for _, name := range schema.Excludes {
		excludeSet[name] = true
	}

	// Add non-excluded parent properties
	for _, prop := range parentProps {
		if !excludeSet[prop.Name] {
			resolved = append(resolved, prop)
		}
	}

	// Merge child properties (override by name)
	for _, childProp := range schema.Properties {
		// Remove parent property with same name
		resolved = r.removeProperty(resolved, childProp.Name)
		// Add child property
		resolved = append(resolved, childProp)
	}

	return resolved
}

// removeProperty removes a property by name from the property list.
func (r *SchemaResolver) removeProperty(
	props []domain.Property,
	name string,
) []domain.Property {
	filtered := make([]domain.Property, 0, len(props))
	for _, prop := range props {
		if prop.Name != name {
			filtered = append(filtered, prop)
		}
	}
	return filtered
}

// substituteRefs replaces $ref references with PropertyBank definitions.
func (r *SchemaResolver) substituteRefs(
	ctx context.Context,
	props []domain.Property,
	bank domain.PropertyBank,
	schemaName string,
) ([]domain.Property, error) {
	substituted := make([]domain.Property, 0, len(props))

	for _, prop := range props {
		// Check for cancellation
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		if prop.Ref != "" {
			resolved, err := r.resolveRefProperty(prop, bank, schemaName)
			if err != nil {
				return nil, err
			}
			substituted = append(substituted, resolved)
		} else {
			// Property without $ref - keep as is
			substituted = append(substituted, prop)
		}
	}

	return substituted, nil
}

// resolveRefProperty resolves a single property's $ref reference.
func (r *SchemaResolver) resolveRefProperty(
	prop domain.Property,
	bank domain.PropertyBank,
	schemaName string,
) (domain.Property, error) {
	refProp, exists := bank.Properties[prop.Ref]
	if !exists {
		return domain.Property{}, lithoserrors.NewSchemaErrorWithRemediation(
			fmt.Sprintf(
				"property %s: $ref '%s' not found in property bank",
				prop.Name,
				prop.Ref,
			),
			schemaName,
			fmt.Sprintf(
				"add property '%s' to property bank or fix reference",
				prop.Ref,
			),
			nil,
		)
	}

	// Create new property with substituted definition
	// Keep original name but use ref's spec
	return domain.Property{
		Name:     prop.Name,
		Required: prop.Required,
		Array:    prop.Array,
		Spec:     refProp.Spec,
		Ref:      "", // Clear ref after substitution
	}, nil
}
