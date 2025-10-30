// Package spi_test provides documentation examples for the SPI ports.
// These tests serve as executable documentation for the SchemaPort interface.
package spi_test

import (
	"context"
	"fmt"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

type exampleSchemaPort struct {
	err error
}

// Load satisfies the spi.SchemaPort interface for example usage.
func (p exampleSchemaPort) Load(
	context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	if p.err != nil {
		return nil, domain.PropertyBank{}, p.err
	}

	schemas := []domain.Schema{{Name: "base-note"}}
	bank := domain.PropertyBank{
		Properties: map[string]domain.Property{
			"title": {
				Name: "title",
				Spec: domain.StringSpec{},
			},
		},
	}

	return schemas, bank, nil
}

// ExampleSchemaPort demonstrates the SchemaPort interface usage.
// This example shows how a SchemaPort implementation loads schemas and property
// bank.
func ExampleSchemaPort() {
	var port spi.SchemaPort = exampleSchemaPort{}

	// Load all schemas and property bank from storage
	schemas, propertyBank, err := port.Load(
		context.Background(),
	)
	if err != nil {
		fmt.Printf("Failed to load schemas: %v\n", err)
		return
	}

	fmt.Printf("Loaded %d schemas and property bank with %d properties\n",
		len(schemas), len(propertyBank.Properties))

	// Schemas contain raw definitions (no inheritance resolution)
	for _, schema := range schemas {
		fmt.Printf("Schema: %s\n", schema.Name)
		if schema.Extends != "" {
			fmt.Printf("  Extends: %s\n", schema.Extends)
		}
	}
}

// ExampleSchemaPort_Load_success shows successful loading of schemas and
// property bank.
func ExampleSchemaPort_Load_success() {
	var port spi.SchemaPort = exampleSchemaPort{}

	schemas, propertyBank, err := port.Load(
		context.Background(),
	)
	if err != nil {
		panic(err) // In real usage, handle error appropriately
	}

	// Verify we got schemas and property bank
	_ = schemas      // Raw schema definitions
	_ = propertyBank // Shared property definitions for $ref resolution
}

// ExampleSchemaPort_Load_errors demonstrates error handling in SchemaPort
// implementations.
func ExampleSchemaPort_Load_errors() {
	var port spi.SchemaPort = exampleSchemaPort{err: fmt.Errorf("load failed")}

	_, _, err := port.Load(
		context.Background(),
	)
	if err != nil {
		// Handle specific error types:
		// - ResourceError: Missing files or access issues
		// - SchemaError: Malformed JSON or duplicate names
		fmt.Printf("Schema loading failed: %v\n", err)
	}
}

// ExampleSchemaPort_Load_rawSchemas shows that Load returns raw schemas without
// resolution.
func ExampleSchemaPort_Load_rawSchemas() {
	var port spi.SchemaPort = exampleSchemaPort{}

	schemas, _, err := port.Load(
		context.Background(),
	)
	if err != nil {
		panic(err)
	}

	// Schemas contain Extends/Excludes/Properties exactly as defined in storage
	for _, schema := range schemas {
		if schema.Extends != "" {
			fmt.Printf("Schema %s extends %s\n", schema.Name, schema.Extends)
		}
		// Extends/Excludes/Properties are preserved as-is
		// SchemaResolver handles inheritance resolution separately
	}
}

// ExampleSchemaPort_Load_propertyBank demonstrates property bank loading.
func ExampleSchemaPort_Load_propertyBank() {
	var port spi.SchemaPort = exampleSchemaPort{}

	_, propertyBank, err := port.Load(
		context.Background(),
	)
	if err != nil {
		panic(err)
	}

	// Property bank contains shared property definitions
	// Used by SchemaResolver for $ref substitution
	for name, prop := range propertyBank.Properties {
		fmt.Printf("Property %s: %s\n", name, prop.Spec.Type())
	}
}
