package schema

import "github.com/JackMatanky/lithos/internal/ports/spi"

// Validator is a placeholder for the upcoming schema validation service.
// Story 2.5 wires the dependency so future stories can build validation logic
// without revisiting adapter wiring.
type Validator struct {
	registry spi.SchemaRegistryPort
}

// NewValidator constructs a validator with injected registry access.
func NewValidator(registry spi.SchemaRegistryPort) *Validator {
	return &Validator{registry: registry}
}
