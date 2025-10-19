# Logger Package

The `logger` package provides centralized structured logging for the Lithos application using the [zerolog](https://github.com/rs/zerolog) library.

## Features

- **Structured Logging**: JSON output for machine readability, pretty-print for human readability
- **Context-Aware**: Add correlation IDs, component names, operations, and custom fields
- **TTY Detection**: Automatically switches to pretty-print output in terminals
- **Level Filtering**: Configurable log levels (Debug, Info, Warn, Error)
- **Sensitive Data Protection**: Designed to prevent logging of sensitive information

## Quick Start

```go
import "github.com/jack/lithos/internal/shared/logger"

// Basic logging
logger.Log.Info().Msg("Application started")

// With context
logger.WithComponent("vault.indexer").
    WithCorrelationID("0192d1b8-5c5c-7b8f-9c5c-8f5c7b8f9c5c").
    WithOperation("scan").
    Info().Msg("Vault indexing completed")
```

## API Reference

### Global Logger

- `Log zerolog.Logger`: The global logger instance configured for the application

### Context Methods

- `WithComponent(component string) zerolog.Logger`: Add component context (e.g., "template.engine")
- `WithOperation(operation string) zerolog.Logger`: Add operation context (e.g., "scan", "render")
- `WithCorrelationID(id string) zerolog.Logger`: Add correlation ID for request tracing
- `WithCommand(command string) zerolog.Logger`: Add CLI command context (e.g., "new", "find")
- `WithTemplateID(id string) zerolog.Logger`: Add template ID (optional)
- `WithFilePath(path string) zerolog.Logger`: Add file path (optional)

## Usage Examples

### Basic Logging

```go
// Info level
logger.Log.Info().Msg("Template processing completed")

// Warning with context
logger.WithComponent("file.validator").Warn().Msg("Invalid file format detected")

// Error with detailed context
logger.WithComponent("vault.indexer").
    WithFilePath("notes/invalid.md").
    Error().Msg("Failed to parse frontmatter")
```

### Request Tracing

```go
correlationID := "0192d1b8-5c5c-7b8f-9c5c-8f5c7b8f9c5c"

logger.WithComponent("api.handler").
    WithCorrelationID(correlationID).
    WithOperation("create-note").
    Info().Msg("Processing note creation request")

// Later in the same request
logger.WithComponent("template.engine").
    WithCorrelationID(correlationID).
    WithTemplateID("note.md").
    Debug().Msg("Rendering template")
```

### CLI Command Logging

```go
logger.WithCommand("new").
    WithComponent("cli.orchestrator").
    Info().Msg("Starting new command execution")

logger.WithCommand("index").
    WithComponent("vault.indexer").
    WithOperation("scan").
    Info().Msg("Scanning vault for changes")
```

## Output Formats

### JSON Format (Default, Machine-Readable)

```json
{
  "level": "info",
  "correlation_id": "0192d1b8-5c5c-7b8f-9c5c-8f5c7b8f9c5c",
  "component": "template.engine",
  "command": "new",
  "template_id": "note.md",
  "time": "2025-01-12T10:30:45.123456789Z",
  "message": "Template processing completed successfully"
}
```

### Pretty-Print Format (TTY Detected)

```
10:30:45 INF Template processing completed successfully component=template.engine command=new correlation_id=0192d1b8-5c5c-7b8f-9c5c-8f5c7b8f9c5c template_id=note.md
```

## Configuration

### Log Level

Set the `LOG_LEVEL` environment variable to control minimum log level:

```bash
export LOG_LEVEL=debug  # Show all logs
export LOG_LEVEL=info   # Default: show info, warn, error
export LOG_LEVEL=warn   # Show only warnings and errors
export LOG_LEVEL=error  # Show only errors
```

Valid levels: `debug`, `info`, `warn`, `error`

### Output Mode

- **JSON**: Default for files, logs, and non-interactive environments
- **Pretty-print**: Automatic when TTY is detected (interactive terminals)

## Best Practices

### Context Fields

Always include relevant context fields:

- `component`: Which part of the system (e.g., "vault.indexer", "template.engine")
- `correlation_id`: For request tracing across components
- `command`: CLI command being executed
- `operation`: Specific operation within a component

### Log Levels

- **Debug**: Development diagnostics, verbose operation details
- **Info**: Normal command summaries, successful operations
- **Warn**: Validation failures, partial successes, recoverable issues
- **Error**: Unexpected faults, data corruption, unrecoverable errors

### Sensitive Data

**NEVER** log:
- User prompts or responses
- Template content being rendered
- File contents
- Authentication credentials
- Personal identifiable information

Only log metadata like file paths, template IDs, and operation status.

### Performance

- Logging is designed to be high-performance with zero-allocation JSON encoding
- Use appropriate log levels to avoid performance impact in production
- Context fields are efficiently added without string concatenation

## Integration Examples

### With Error Handling

```go
if err := processTemplate(templateID); err != nil {
    logger.WithComponent("template.engine").
        WithTemplateID(templateID).
        WithCorrelationID(correlationID).
        Error().Err(err).Msg("Template processing failed")
    return err
}

logger.WithComponent("template.engine").
    WithTemplateID(templateID).
    WithCorrelationID(correlationID).
    Info().Msg("Template processed successfully")
```

### With Business Logic

```go
func (s *NoteService) CreateNote(ctx context.Context, req *CreateNoteRequest) (*Note, error) {
    log := logger.WithComponent("note.service").
        WithCorrelationID(req.CorrelationID).
        WithOperation("create")

    log.Info().Msg("Creating new note")

    // Business logic here...

    log.WithTemplateID(req.TemplateID).
        Info().Msg("Note created successfully")

    return note, nil
}
```

## Testing

The package includes comprehensive unit tests covering:

- Context field addition
- Log level filtering
- JSON vs pretty-print output
- Multiple context fields combination

Run tests with:

```bash
go test ./internal/shared/logger/
```

## Architecture Notes

- **Shared Package**: Used across all application layers
- **No Dependencies**: Only depends on zerolog and standard library
- **Thread-Safe**: Global logger is safe for concurrent use
- **Initialization**: Automatically configured on package import
