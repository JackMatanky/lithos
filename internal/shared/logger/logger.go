// Package logger provides centralized structured logging for Lithos.
// This package wraps zerolog to provide consistent log formatting across
// components with JSON and pretty-print output modes. Includes context-aware
// logging with correlation IDs, component tracking, and data filtering.
package logger

import (
	"fmt"
	"os"

	"github.com/rs/zerolog"
	"golang.org/x/term"
)

// Logger wraps zerolog.Logger to provide our custom logging interface
type Logger struct {
	zerolog.Logger
}

// Log is the global logger instance used throughout the application.
// Configured for JSON output by default with TTY detection for pretty-print.
var Log Logger

// configureZerolog sets up global zerolog configuration
func configureZerolog() {
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnixMs
	zerolog.CallerMarshalFunc = func(pc uintptr, file string, line int) string {
		short := file
		for i := len(file) - 1; i > 0; i-- {
			if file[i] == '/' {
				short = file[i+1:]
				break
			}
		}
		return fmt.Sprintf("%s:%d", short, line)
	}
}

// setupLogLevel configures the global log level from environment variables
func setupLogLevel() {
	level := zerolog.InfoLevel
	if envLevel := os.Getenv("LOG_LEVEL"); envLevel != "" {
		if parsedLevel, err := zerolog.ParseLevel(envLevel); err == nil {
			level = parsedLevel
		}
	}
	zerolog.SetGlobalLevel(level)
}

// createLogger creates and configures the logger based on TTY detection
func createLogger() Logger {
	var zl zerolog.Logger
	if term.IsTerminal(int(os.Stdout.Fd())) {
		// Pretty-print for human readability in terminals
		//nolint:exhaustruct // Using defaults for ConsoleWriter is appropriate
		zl = zerolog.New(zerolog.ConsoleWriter{Out: os.Stdout}).
			With().
			Timestamp().
			Caller().
			Logger()
	} else {
		// JSON output for machine readability (logs, files, etc.)
		zl = zerolog.New(os.Stdout).
			With().
			Timestamp().
			Caller().
			Logger()
	}
	return Logger{Logger: zl}
}

func init() {
	configureZerolog()
	setupLogLevel()
	Log = createLogger()
}

// WithComponent adds a component field to the logger context.
// This helps identify which part of the application generated the log entry.
func WithComponent(component string) Logger {
	return Logger{Logger: Log.With().Str("component", component).Logger()}
}

// WithOperation adds an operation field to the logger context.
// This identifies the specific operation being performed.
func WithOperation(operation string) Logger {
	return Logger{Logger: Log.With().Str("operation", operation).Logger()}
}

// WithCorrelationID adds a correlation_id field to the logger context.
// This enables tracing requests across multiple components and operations.
func WithCorrelationID(id string) Logger {
	return Logger{Logger: Log.With().Str("correlation_id", id).Logger()}
}

// WithCommand adds a command field to the logger context.
// This identifies the CLI command being executed (e.g., "new", "find",
// "index").
func WithCommand(command string) Logger {
	return Logger{Logger: Log.With().Str("command", command).Logger()}
}

// WithTemplateID adds a template_id field to the logger context.
// This identifies the template being processed (optional field).
func WithTemplateID(id string) Logger {
	return Logger{Logger: Log.With().Str("template_id", id).Logger()}
}

// WithFilePath adds a file_path field to the logger context.
// This identifies the file being processed (optional field).
func WithFilePath(path string) Logger {
	return Logger{Logger: Log.With().Str("file_path", path).Logger()}
}
