// Package api provides primary (driving) port interfaces for Lithos.
// These interfaces define the contracts that domain services expose to
// application adapters (CLI, TUI, LSP). They follow hexagonal architecture
// principles where domain defines the interfaces and adapters implement them.
//
// Primary ports represent use cases - the ways users interact with the system.
// They use callback patterns where domain injects itself into adapters.
// These interfaces define the contracts that domain services expose to
// application adapters (CLI, TUI, LSP). They follow hexagonal architecture
// principles where domain defines the interfaces and adapters implement them.
//
// Primary ports represent use cases - the ways users interact with the system.
// They use callback patterns where domain injects itself into adapters.
package api

import (
	"context"
)

// CLIPort defines the contract for CLI framework integration.
// This primary port is implemented by CLI adapters (CobraCLIAdapter) and
// defines how the domain expects to interact with command-line interfaces.
//
// The hexagonal callback pattern: Domain calls CLIPort.Start() with itself
// as the CommandPort handler. The CLI adapter receives control, sets up
// commands, parses user input, and delegates business logic back to domain
// through the CommandPort interface.
//
// Architecture Pattern:
// ```
// CLIComander (Domain)
//
//	└─> Calls CLIPort.Start(itself as CommandPort)
//	    └─> CobraCLIAdapter receives control
//	        └─> Sets up Cobra commands
//	        └─> Parses user input
//	        └─> Calls back to CommandPort.NewNote/IndexVault/FindTemplates
//	            └─> CommandOrchestrator orchestrates domain services
//	            └─> Returns result to CLI adapter
//	        └─> Formats and displays output
//
// ```
//
// Why This Design:
//   - Decouples CLI framework from domain: CLIComander never imports
//     Cobra
//   - Enables multiple adapters: TUI/LSP can implement CLIPort without
//     affecting domain
//   - Testable: Mock CLIPort to test CommandOrchestrator without CLI framework
//   - Inversion of Control: Domain starts the application and delegates command
//     parsing to adapter
//
// Reference: docs/architecture/components.md#api-port-interfaces - CLIPort
// (v0.6.4).
type CLIPort interface {
	// Start begins the CLI event loop and command processing.
	// The CLI adapter receives the CommandPort handler (typically
	// CLIComander) and uses it to delegate business logic execution.
	//
	// The adapter is responsible for:
	// - Setting up command definitions and flags
	// - Parsing command-line arguments
	// - Handling user input validation
	// - Formatting and displaying results
	// - Error handling and exit codes
	//
	// The handler is responsible for:
	// - Executing business logic
	// - Orchestrating domain services
	// - Returning domain objects/errors
	//
	// Parameters:
	//   - ctx: Context for cancellation and timeout control
	//   - handler: Domain service implementing CommandPort for business logic
	//
	// Returns:
	//   - error: Any startup or execution errors from the CLI framework
	Start(ctx context.Context, handler CommandPort) error
}
