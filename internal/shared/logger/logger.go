// Package logger provides structured logging with zerolog.
//
// This package implements centralized logging for the Lithos application,
// providing consistent structured output with automatic TTY detection for
// human-readable vs machine-readable formats. It includes context enrichment
// methods and automatic filtering of sensitive data fields.
package logger

import (
	"bytes"
	"context"
	"io"
	"os"
	"strings"

	"github.com/rs/zerolog"
	"golang.org/x/term"
)

var (
	// Log is the global logger instance for the application.
	// It should be initialized once during application startup.
	Log zerolog.Logger

	sensitiveMarkers = [][]byte{
		[]byte("\"password\":"),
		[]byte("\"token\":"),
		[]byte("\"apiKey\":"),
	}
)

// sensitiveWriter wraps an io.Writer to redact sensitive fields from JSON
// output.
type sensitiveWriter struct {
	writer io.Writer
}

// Logger defines the interface for structured logging operations.
//
//nolint:interfacebloat // Comprehensive logging interface by design
type Logger interface {
	// WithContext returns a logger with context information.
	WithContext(ctx context.Context) Logger

	// WithField adds a field to the logger.
	WithField(key string, value interface{}) Logger

	// WithFields adds multiple fields to the logger.
	WithFields(fields map[string]interface{}) Logger

	// WithError adds an error to the logger.
	WithError(err error) Logger

	// Debug logs a debug message.
	Debug(msg string)

	// Info logs an info message.
	Info(msg string)

	// Warn logs a warning message.
	Warn(msg string)

	// Error logs an error message.
	Error(msg string)

	// Fatal logs a fatal message and exits.
	Fatal(msg string)

	// LogOperation logs an operation with structured context.
	LogOperation(operation string, params map[string]interface{}, err error)
}

// ZerologAdapter adapts zerolog to the Logger interface.
type ZerologAdapter struct {
	logger zerolog.Logger
}

// NewZerologAdapter creates a new zerolog adapter.
func NewZerologAdapter(logger zerolog.Logger) *ZerologAdapter {
	return &ZerologAdapter{logger: logger}
}

// WithContext returns a logger with context information.
func (z *ZerologAdapter) WithContext(ctx context.Context) Logger {
	return &ZerologAdapter{logger: z.logger.With().Caller().Logger()}
}

// WithField adds a field to the logger.
func (z *ZerologAdapter) WithField(key string, value interface{}) Logger {
	return &ZerologAdapter{
		logger: z.logger.With().Interface(key, value).Logger(),
	}
}

// WithFields adds multiple fields to the logger.
func (z *ZerologAdapter) WithFields(fields map[string]interface{}) Logger {
	event := z.logger.With()
	for k, v := range fields {
		event = event.Interface(k, v)
	}
	return &ZerologAdapter{logger: event.Logger()}
}

// WithError adds an error to the logger.
func (z *ZerologAdapter) WithError(err error) Logger {
	return &ZerologAdapter{logger: z.logger.With().Err(err).Logger()}
}

// Debug logs a debug message.
func (z *ZerologAdapter) Debug(msg string) {
	z.logger.Debug().Msg(msg)
}

// Info logs an info message.
func (z *ZerologAdapter) Info(msg string) {
	z.logger.Info().Msg(msg)
}

// Warn logs a warning message.
func (z *ZerologAdapter) Warn(msg string) {
	z.logger.Warn().Msg(msg)
}

// Error logs an error message.
func (z *ZerologAdapter) Error(msg string) {
	z.logger.Error().Msg(msg)
}

// Fatal logs a fatal message and exits.
func (z *ZerologAdapter) Fatal(msg string) {
	z.logger.Fatal().Msg(msg)
}

// LogOperation logs an operation with structured context.
func (z *ZerologAdapter) LogOperation(
	operation string,
	params map[string]interface{},
	err error,
) {
	event := z.logger.Info().Str("operation", operation)
	for k, v := range params {
		event = event.Interface(k, v)
	}
	if err != nil {
		event = event.Err(err)
	}
	event.Msg("operation completed")
}

// New creates a configured zerolog logger with the specified output and level.
// It automatically detects TTY terminals for pretty-print output vs JSON for
// pipes/files.
// Supported levels: debug, info, warn/warning, error (case-insensitive).
// Invalid levels default to info.
func New(output io.Writer, level string) zerolog.Logger {
	logLevel := parseLevel(level)
	output = configureTTY(output)

	return createLogger(output, logLevel)
}

// createLogger initializes a zerolog logger with the specified output and
// level.
func createLogger(output io.Writer, level zerolog.Level) zerolog.Logger {
	// Wrap output with sensitive data redaction
	sensitiveOutput := &sensitiveWriter{writer: output}

	return zerolog.New(sensitiveOutput).
		Level(level).
		With().
		Timestamp().
		Logger()
}

// configureTTY detects if the output is a TTY and configures appropriate
// formatting.
func configureTTY(output io.Writer) io.Writer {
	// Detect if output is a TTY for pretty-print vs JSON format
	if f, ok := output.(*os.File); ok && term.IsTerminal(int(f.Fd())) {
		// Pretty-print for terminals
		//nolint:exhaustruct // Using minimal config for readability; zerolog
		// provides sensible defaults
		return zerolog.ConsoleWriter{
			Out:        output,
			TimeFormat: "15:04:05",
		}
	}
	// else: JSON format for pipes/files
	return output
}

// parseLevel converts a string level to zerolog.Level.
// Supports debug, info, warn/warning, error (case-insensitive).
// Invalid levels default to InfoLevel.
func parseLevel(level string) zerolog.Level {
	switch strings.ToLower(level) {
	case "debug":
		return zerolog.DebugLevel
	case "info":
		return zerolog.InfoLevel
	case "warn", "warning":
		return zerolog.WarnLevel
	case "error":
		return zerolog.ErrorLevel
	default:
		return zerolog.InfoLevel // Fallback
	}
}

// WithComponent adds a component field to the logger context.
func WithComponent(logger zerolog.Logger, component string) zerolog.Logger {
	return logger.With().Str("component", component).Logger()
}

// WithOperation adds an operation field to the logger context.
func WithOperation(logger zerolog.Logger, operation string) zerolog.Logger {
	return logger.With().Str("operation", operation).Logger()
}

// WithCorrelationID adds a correlation_id field to the logger context.
func WithCorrelationID(logger zerolog.Logger, id string) zerolog.Logger {
	return logger.With().Str("correlation_id", id).Logger()
}

// NewTest creates a logger that discards all output, useful for testing.
func NewTest() zerolog.Logger {
	return zerolog.New(io.Discard).Level(zerolog.Disabled)
}

// Write redacts sensitive fields from JSON log output before writing.
func (w *sensitiveWriter) Write(p []byte) (n int, err error) {
	needsRedaction := false
	for _, marker := range sensitiveMarkers {
		if bytes.Contains(p, marker) {
			needsRedaction = true
			break
		}
	}

	if !needsRedaction {
		return w.writer.Write(p)
	}

	line := redactSensitiveFields(string(p))
	return w.writer.Write([]byte(line))
}

// redactSensitiveFields redacts all known sensitive fields from a JSON log
// line.
func redactSensitiveFields(jsonStr string) string {
	// Redact known sensitive fields
	jsonStr = redactField(jsonStr, "password")
	jsonStr = redactField(jsonStr, "token")
	jsonStr = redactField(jsonStr, "apiKey")
	return jsonStr
}

// redactField redacts a specific field value in JSON string by replacing it
// with "[REDACTED]".
func redactField(jsonStr, field string) string {
	// Find the field pattern "field":
	fieldStart := `"` + field + `":`
	startIdx := strings.Index(jsonStr, fieldStart)
	if startIdx == -1 {
		return jsonStr
	}

	// Find the value start (after the colon and opening quote)
	valueStart := startIdx + len(fieldStart)
	if valueStart >= len(jsonStr) || jsonStr[valueStart] != '"' {
		return jsonStr
	}
	valueStart++ // Skip the opening quote

	// Find the value end (closing quote)
	valueEnd := strings.Index(jsonStr[valueStart:], `"`)
	if valueEnd == -1 {
		return jsonStr
	}
	valueEnd += valueStart

	// Replace the value with [REDACTED]
	return jsonStr[:valueStart] + "[REDACTED]" + jsonStr[valueEnd:]
}
