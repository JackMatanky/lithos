// Package logger provides unit tests for the shared logger package.
package logger

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"

	"github.com/rs/zerolog"
)

func TestWithComponent(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	componentLogger := WithComponent("test.component")
	componentLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["component"] != "test.component" {
		t.Errorf(
			"Expected component 'test.component', got %v",
			logEntry["component"],
		)
	}

	if logEntry["message"] != "test message" {
		t.Errorf("Expected message 'test message', got %v", logEntry["message"])
	}
}

func TestWithOperation(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	opLogger := WithOperation("test.operation")
	opLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["operation"] != "test.operation" {
		t.Errorf(
			"Expected operation 'test.operation', got %v",
			logEntry["operation"],
		)
	}
}

func TestWithCorrelationID(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	corrLogger := WithCorrelationID("test-correlation-id")
	corrLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["correlation_id"] != "test-correlation-id" {
		t.Errorf(
			"Expected correlation_id 'test-correlation-id', got %v",
			logEntry["correlation_id"],
		)
	}
}

func TestWithCommand(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	cmdLogger := WithCommand("new")
	cmdLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["command"] != "new" {
		t.Errorf("Expected command 'new', got %v", logEntry["command"])
	}
}

func TestWithTemplateID(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	tmplLogger := WithTemplateID("note.md")
	tmplLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["template_id"] != "note.md" {
		t.Errorf(
			"Expected template_id 'note.md', got %v",
			logEntry["template_id"],
		)
	}
}

func TestWithFilePath(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	fileLogger := WithFilePath("notes/test.md")
	fileLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["file_path"] != "notes/test.md" {
		t.Errorf(
			"Expected file_path 'notes/test.md', got %v",
			logEntry["file_path"],
		)
	}
}

func TestLogLevels(t *testing.T) {
	// Save original level
	originalLevel := zerolog.GlobalLevel()

	// Test Info level (default)
	zerolog.SetGlobalLevel(zerolog.InfoLevel)

	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	Log.Debug().Msg("debug message") // Should not appear
	Log.Info().Msg("info message")   // Should appear
	Log.Warn().Msg("warn message")   // Should appear
	Log.Error().Msg("error message") // Should appear

	output := buf.String()
	if strings.Contains(output, "debug message") {
		t.Error("Debug message should not appear at Info level")
	}
	if !strings.Contains(output, "info message") {
		t.Error("Info message should appear at Info level")
	}
	if !strings.Contains(output, "warn message") {
		t.Error("Warn message should appear at Info level")
	}
	if !strings.Contains(output, "error message") {
		t.Error("Error message should appear at Info level")
	}

	// Restore original level
	zerolog.SetGlobalLevel(originalLevel)
}

func TestJSONOutput(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	Log.Info().Msg("test message")

	// Should be valid JSON
	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Log output is not valid JSON: %v", err)
	}

	if logEntry["level"] != "info" {
		t.Errorf("Expected level 'info', got %v", logEntry["level"])
	}

	if logEntry["message"] != "test message" {
		t.Errorf("Expected message 'test message', got %v", logEntry["message"])
	}

	// Should have timestamp
	if _, exists := logEntry["time"]; !exists {
		t.Error("Log entry should have timestamp field")
	}
}

func TestParseLevel(t *testing.T) {
	// Test that zerolog can parse level strings correctly
	tests := []struct {
		input    string
		expected zerolog.Level
		hasError bool
	}{
		{"debug", zerolog.DebugLevel, false},
		{"info", zerolog.InfoLevel, false},
		{"warn", zerolog.WarnLevel, false},
		{"error", zerolog.ErrorLevel, false},
		{"invalid", zerolog.InfoLevel, true}, // should error
	}

	for _, test := range tests {
		level, err := zerolog.ParseLevel(test.input)
		if test.hasError && err == nil {
			t.Errorf("Expected error for input %s, but got none", test.input)
		}
		if !test.hasError && err != nil {
			t.Errorf("Unexpected error for input %s: %v", test.input, err)
		}
		if !test.hasError && level != test.expected {
			t.Errorf(
				"For input %s, expected level %v, got %v",
				test.input,
				test.expected,
				level,
			)
		}
	}
}

func TestMultipleContextFields(t *testing.T) {
	var buf bytes.Buffer
	logger := zerolog.New(&buf).With().Timestamp().Logger()
	Log = logger

	contextLogger := WithComponent("test.component").
		With().Str("correlation_id", "test-id").
		Logger()

	contextLogger.Info().Msg("test message")

	var logEntry map[string]interface{}
	if err := json.Unmarshal(buf.Bytes(), &logEntry); err != nil {
		t.Fatalf("Failed to unmarshal log entry: %v", err)
	}

	if logEntry["component"] != "test.component" {
		t.Errorf(
			"Expected component 'test.component', got %v",
			logEntry["component"],
		)
	}

	if logEntry["correlation_id"] != "test-id" {
		t.Errorf(
			"Expected correlation_id 'test-id', got %v",
			logEntry["correlation_id"],
		)
	}
}
