package schema

import (
	"context"
	"fmt"
	"strings"

	"github.com/JackMatanky/lithos/internal/domain"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
)

// SchemaExtender handles extends/excludes inheritance attribute processing.
// It operates at the adapter layer, resolving inheritance chains into
// flattened property sets ready for consumption.
//
// This component is part of the DDD architecture refactoring, splitting
// the original SchemaResolver into focused infrastructure components.
//
// Responsibilities:
//   - Handle extends/excludes inheritance attribute processing
//   - Topological sorting for inheritance chains
//   - Cycle detection with informative error paths
//   - Property merge semantics (complete override by name)
//
// Architecture Reference: docs/architecture/components.md#schemaextender.
type SchemaExtender struct{}

// NewSchemaExtender creates a new SchemaExtender instance.
// SchemaExtender has no dependencies and is pure inheritance logic.
func NewSchemaExtender() *SchemaExtender {
	return &SchemaExtender{}
}

// ExtendSchemas performs comprehensive inheritance resolution.
// It transforms schemas with inheritance into flattened resolved schemas.
//
// Resolution Steps:
//  1. Build dependency graph for inheritance chains
//  2. Detect circular dependencies (fail-fast)
//  3. Topological sort (parents resolve before children)
//  4. Resolve each schema in order (inheritance + excludes + overrides)
//
// Returns new schema copies with ResolvedProperties populated.
// Original schemas are never mutated.
//
// Context is used for cancellation during potentially long-running resolution.
func (e *SchemaExtender) ExtendSchemas(
	ctx context.Context,
	schemas []domain.Schema,
) ([]domain.Schema, error) {
	// Check for cancellation
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	// Build dependency graph
	graph := e.buildDependencyGraph(schemas)

	// Detect circular dependencies
	if err := e.detectCycles(schemas, graph); err != nil {
		return nil, err
	}

	// Topological sort
	sortedSchemas := e.topologicalSort(schemas, graph)

	// Resolve schemas in order
	resolved := make([]domain.Schema, 0, len(schemas))
	resolvedMap := make(map[string]domain.Schema)

	for _, schema := range sortedSchemas {
		// Check for cancellation
		if err := ctx.Err(); err != nil {
			return nil, err
		}

		resolvedSchema := e.resolveSchemaInheritance(schema, resolvedMap)
		resolved = append(resolved, resolvedSchema)
		resolvedMap[resolvedSchema.Name] = resolvedSchema
	}

	return resolved, nil
}

// buildDependencyGraph creates a map from schema name to parent name.
func (e *SchemaExtender) buildDependencyGraph(
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
func (e *SchemaExtender) detectCycles(
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
			if err := e.processParentInCycle(parent, name, path, recStack, visited, dfs); err != nil {
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
func (e *SchemaExtender) processParentInCycle(
	parent, name string,
	path []string,
	recStack, visited map[string]bool,
	dfs func(string, []string) error,
) error {
	if recStack[parent] {
		// Cycle detected - build cycle path
		cyclePath := strings.Join(append(path, parent), " → ")
		return lithosErr.NewSchemaErrorWithRemediation(
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
func (e *SchemaExtender) topologicalSort(
	schemas []domain.Schema,
	graph map[string]string,
) []domain.Schema {
	sorted := make([]domain.Schema, 0, len(schemas))
	visited := make(map[string]bool)
	schemaMap := e.buildSchemaMap(schemas)

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
func (e *SchemaExtender) buildSchemaMap(
	schemas []domain.Schema,
) map[string]domain.Schema {
	schemaMap := make(map[string]domain.Schema, len(schemas))
	for _, schema := range schemas {
		schemaMap[schema.Name] = schema
	}
	return schemaMap
}

// resolveSchemaInheritance resolves a single schema's inheritance.
func (e *SchemaExtender) resolveSchemaInheritance(
	schema domain.Schema,
	resolvedMap map[string]domain.Schema,
) domain.Schema {
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
	finalProps := e.resolveProperties(schema, parentProps)

	// Create resolved schema copy (preserve original fields)
	resolved := domain.Schema{
		Name:               schema.Name,
		Extends:            schema.Extends,
		Excludes:           schema.Excludes,
		Properties:         schema.Properties,
		ResolvedProperties: finalProps,
	}

	return resolved
}

// resolveProperties applies inheritance, excludes, and property overrides.
func (e *SchemaExtender) resolveProperties(
	schema domain.Schema,
	parentProps []domain.Property,
) []domain.Property {
	capacity := len(parentProps) + len(schema.Properties)
	resolved := make(map[string]domain.Property, capacity)
	order := make([]string, 0, capacity)

	excludeSet := buildExcludeSet(schema.Excludes)
	appendParentProperties(
		resolved,
		&order,
		parentProps,
		excludeSet,
	)

	// Add/override child properties
	for _, childProp := range schema.Properties {
		updateResolved(resolved, &order, childProp)
	}

	// Convert back to slice maintaining order
	final := make([]domain.Property, 0, len(order))
	for _, name := range order {
		if prop, exists := resolved[name]; exists {
			final = append(final, prop)
		}
	}

	return final
}

// appendParentProperties adds parent properties to resolved map and
// order, respecting exclusions.
// Checks resolved map for existence to maintain order.
func appendParentProperties(
	resolved map[string]domain.Property,
	order *[]string,
	parentProps []domain.Property,
	excludeSet map[string]struct{},
) {
	for _, prop := range parentProps {
		if _, excluded := excludeSet[prop.Name]; !excluded {
			if _, exists := resolved[prop.Name]; !exists {
				resolved[prop.Name] = prop
				*order = append(*order, prop.Name)
			}
		}
	}
}

// updateResolved adds or updates a child property in resolved map and
// order.
// Maintains order for properties that are added/updated.
func updateResolved(
	resolved map[string]domain.Property,
	order *[]string,
	childProp domain.Property,
) {
	name := childProp.Name
	if _, exists := resolved[name]; exists {
		// Property exists, update it
		resolved[name] = childProp
		// Order remains the same
	} else {
		// New property, add to end
		resolved[name] = childProp
		*order = append(*order, name)
	}
}

// buildExcludeSet creates a set for O(1) exclude lookups.
func buildExcludeSet(excludes []string) map[string]struct{} {
	excludeSet := make(map[string]struct{}, len(excludes))
	for _, name := range excludes {
		excludeSet[name] = struct{}{}
	}
	return excludeSet
}
