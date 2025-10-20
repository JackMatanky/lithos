package spi

import "github.com/JackMatanky/lithos/internal/adapters/spi/config"

// ConfigPort defines the interface for configuration management adapters.
// This is an SPI (Service Provider Interface) port for driven adapters that
// provide configuration services to the domain.
//
// Follows hexagonal architecture patterns where ports define contracts
// and adapters implement the actual infrastructure concerns.
type ConfigPort interface {
	// Config returns the current application configuration.
	// The configuration is loaded at application startup and cached.
	// This method should not perform I/O operations - configuration
	// loading happens during adapter initialization.
	Config() *config.Config
}
