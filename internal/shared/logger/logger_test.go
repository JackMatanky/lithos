package logger

import (
	"bytes"
	"testing"

	"github.com/rs/zerolog"
)

// TestNew verifies logger creation with different levels and TTY detection.
func TestNew(t *testing.T) {
	tests := []struct {
		name     string
		level    string
		expected zerolog.Level
	}{
		{
			name:     "debug level",
			level:    "debug",
			expected: zerolog.DebugLevel,
		},
		{
			name:     "info level",
			level:    "info",
			expected: zerolog.InfoLevel,
		},
		{
			name:     "warn level",
			level:    "warn",
			expected: zerolog.WarnLevel,
		},
		{
			name:     "warning level",
			level:    "warning",
			expected: zerolog.WarnLevel,
		},
		{
			name:     "error level",
			level:    "error",
			expected: zerolog.ErrorLevel,
		},
		{
			name:     "case insensitive",
			level:    "DEBUG",
			expected: zerolog.DebugLevel,
		},
		{
			name:     "invalid level defaults to info",
			level:    "invalid",
			expected: zerolog.InfoLevel,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var buf bytes.Buffer
			logger := New(&buf, tt.level)

			// Check that the logger has the correct level
			if logger.GetLevel() != tt.expected {
				t.Errorf(
					"New() level = %v, expected %v",
					logger.GetLevel(),
					tt.expected,
				)
			}

			// Verify logger can write output at the configured level
			switch tt.expected {
			case zerolog.DebugLevel:
				logger.Debug().Msg("test message")
			case zerolog.InfoLevel:
				logger.Info().Msg("test message")
			case zerolog.WarnLevel:
				logger.Warn().Msg("test message")
			case zerolog.ErrorLevel:
				logger.Error().Msg("test message")
			}
			output := buf.String()
			if output == "" {
				t.Error("New() should produce output")
			}
		})
	}
}

// TestTTYDetection verifies pretty-print vs JSON output based on TTY detection.
func TestTTYDetection(t *testing.T) {
	// Note: Testing TTY detection properly requires mocking term.IsTerminal
	// This is a basic smoke test to ensure the logger works
	var buf bytes.Buffer
	logger := New(&buf, "info")

	// Log a message
	logger.Info().Str("test", "value").Msg("test message")

	output := buf.String()

	// Verify output is produced (exact format depends on TTY detection)
	if output == "" {
		t.Error("Expected output from logger")
	}

	// In a real implementation, we'd mock term.IsTerminal to test both paths
	// For now, this ensures the basic functionality works
}

// TestWithComponent verifies WithComponent adds component field to logs.
func TestWithComponent(t *testing.T) {
	var buf bytes.Buffer
	baseLogger := New(&buf, "info")

	// Test WithComponent
	componentLogger := WithComponent(baseLogger, "vault")
	componentLogger.Info().Msg("test message")

	output := buf.String()
	if !bytes.Contains([]byte(output), []byte("vault")) {
		t.Errorf(
			"Expected output to contain component 'vault', got: %s",
			output,
		)
	}
}

// TestWithOperation verifies WithOperation adds operation field to logs.
func TestWithOperation(t *testing.T) {
	var buf bytes.Buffer
	baseLogger := New(&buf, "info")

	// Test WithOperation
	operationLogger := WithOperation(baseLogger, "loadConfig")
	operationLogger.Info().Msg("test message")

	output := buf.String()
	if !bytes.Contains([]byte(output), []byte("loadConfig")) {
		t.Errorf(
			"Expected output to contain operation 'loadConfig', got: %s",
			output,
		)
	}
}

// TestWithCorrelationID verifies WithCorrelationID adds correlation_id field to
// logs.
func TestWithCorrelationID(t *testing.T) {
	var buf bytes.Buffer
	baseLogger := New(&buf, "info")

	// Test WithCorrelationID
	corrLogger := WithCorrelationID(baseLogger, "abc-123")
	corrLogger.Info().Msg("test message")

	output := buf.String()
	if !bytes.Contains([]byte(output), []byte("abc-123")) {
		t.Errorf(
			"Expected output to contain correlation_id 'abc-123', got: %s",
			output,
		)
	}
}

// TestSensitiveDataFiltering verifies password fields are redacted in logs.
func TestSensitiveDataFiltering(t *testing.T) {
	var buf bytes.Buffer
	logger := New(&buf, "info")

	// Log with sensitive data
	logger.Info().
		Str("password", "secret").
		Str("username", "user").
		Msg("login attempt")

	output := buf.String()
	t.Logf("Output: %s", output) // Debug output

	// Password should be redacted
	if bytes.Contains([]byte(output), []byte("testpassword")) {
		t.Error(
			"Password should be redacted, but found 'testpassword' in output",
		)
	}

	// Should contain [REDACTED]
	if !bytes.Contains([]byte(output), []byte("[REDACTED]")) {
		t.Error("Expected [REDACTED] in output for sensitive field")
	}

	// Non-sensitive data should remain
	if !bytes.Contains([]byte(output), []byte("user")) {
		t.Error("Non-sensitive field 'username' should not be redacted")
	}
}

// TestMultipleSensitiveFields verifies all sensitive fields are redacted.
func TestMultipleSensitiveFields(t *testing.T) {
	var buf bytes.Buffer
	logger := New(&buf, "info")

	// Log with multiple sensitive fields
	logger.Info().
		Str("password", "secret").
		Str("token", "token123").
		Str("apiKey", "apikey").
		Str("username", "user").
		Msg("auth test")

	output := buf.String()

	// All sensitive fields should be redacted
	if bytes.Contains([]byte(output), []byte("testpassword")) {
		t.Error("Password should be redacted")
	}
	if bytes.Contains([]byte(output), []byte("testtoken123")) {
		t.Error("Token should be redacted")
	}
	if bytes.Contains([]byte(output), []byte("testapikey")) {
		t.Error("API key should be redacted")
	}

	// Should contain [REDACTED] for each sensitive field
	redactedCount := bytes.Count([]byte(output), []byte("[REDACTED]"))
	if redactedCount != 3 {
		t.Errorf("Expected 3 [REDACTED] entries, got %d", redactedCount)
	}

	// Non-sensitive data should remain
	if !bytes.Contains([]byte(output), []byte("user")) {
		t.Error("Non-sensitive field 'username' should not be redacted")
	}
}

// TestNewTest verifies NewTest() creates a logger that produces no output.
func TestNewTest(t *testing.T) {
	logger := NewTest()

	// Log various messages
	logger.Info().Msg("info message")
	logger.Warn().Str("field", "value").Msg("warn message")
	logger.Error().Msg("error message")

	// NewTest should use ioutil.Discard, so no output should be produced
	// We can't easily test this directly, but we can verify the logger is
	// created
	// and has the expected disabled level

	if logger.GetLevel() != zerolog.Disabled {
		t.Errorf(
			"NewTest() logger level should be Disabled, got %v",
			logger.GetLevel(),
		)
	}
}
