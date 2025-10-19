package cli

import (
	"bytes"
	"os"
	"strings"
	"testing"
)

func TestCobraCLIAdapter_Execute_VersionCommand(t *testing.T) {
	adapter := NewCobraCLIAdapter()

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute version command
	exitCode := adapter.Execute([]string{"version"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains version
	if !strings.Contains(output, "Lithos version") {
		t.Errorf("Execute() output = %q, should contain version info", output)
	}
}

func TestCobraCLIAdapter_Execute_HelpCommand(t *testing.T) {
	adapter := NewCobraCLIAdapter()

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute help command
	exitCode := adapter.Execute([]string{"--help"})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains help info
	if !strings.Contains(output, "lithos") {
		t.Errorf("Execute() output = %q, should contain command info", output)
	}
}

func TestCobraCLIAdapter_Execute_InvalidCommand(t *testing.T) {
	adapter := NewCobraCLIAdapter()

	// Execute invalid command
	exitCode := adapter.Execute([]string{"invalid-command"})

	// Verify exit code is non-zero for invalid command
	if exitCode == 0 {
		t.Error("Execute() should return non-zero exit code for invalid " +
			"command")
	}
}

func TestCobraCLIAdapter_Execute_NoArgs(t *testing.T) {
	adapter := NewCobraCLIAdapter()

	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	// Execute with no args (should show help)
	exitCode := adapter.Execute([]string{})

	// Restore stdout
	_ = w.Close()
	os.Stdout = oldStdout

	// Read captured output
	var buf bytes.Buffer
	_, _ = buf.ReadFrom(r)
	output := buf.String()

	// Verify exit code
	if exitCode != 0 {
		t.Errorf("Execute() exit code = %v, want 0", exitCode)
	}

	// Verify output contains help info
	if !strings.Contains(output, "lithos") {
		t.Errorf("Execute() output = %q, should contain command info", output)
	}
}
