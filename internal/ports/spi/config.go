// Package spi provides Service Provider Interface (SPI) port definitions.
// These interfaces define contracts that adapters must implement to provide
// external service capabilities to the domain layer.
//
// SPI ports follow hexagonal architecture principles, allowing the domain
// to remain independent of infrastructure concerns while defining clear
// contracts for external dependencies.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// ConfigPort defines the contract for loading application configuration.
// Implementations must load configuration from various sources with proper
// precedence and validation.
//
// The port follows hexagonal architecture principles, allowing domain services
// to request configuration without knowing the implementation details (file,
// environment variables, CLI flags, etc.).
//
// Load method must handle:
//   - Configuration precedence (CLI flags > env vars > config file > defaults)
//   - Validation of critical paths (VaultPath must exist and be directory)
//   - Graceful fallback for optional configuration
//   - Context cancellation for timeout handling
type ConfigPort interface {
	// Load retrieves and validates application configuration from all available
	// sources. Returns a fully resolved Config value object or an error if
	// critical configuration is invalid.
	//
	// The method must implement the following precedence order:
	// 1. CLI flags (highest priority, reserved for future implementation)
	// 2. Environment variables (LITHOS_* prefix)
	// 3. Config file (lithos.json found via upward directory search)
	// 4. Default values (lowest priority)
	//
	// Parameters:
	//   - ctx: Context for cancellation and timeout handling
	//
	// Returns:
	//   - Config: Fully resolved configuration value object
	//   - error: Non-nil if critical configuration validation fails
	//
	// Error conditions:
	//   - VaultPath does not exist: returns descriptive error
	//   - VaultPath is not a directory: returns descriptive error
	//   - Context canceled: returns context.Canceled error
	// - Config file parse errors: logged but not returned (fallback to
	// defaults)
	Load(ctx context.Context) (domain.Config, error)
}
