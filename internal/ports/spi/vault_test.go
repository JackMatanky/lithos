package spi

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/dto"
)

const testPath = "/test/path.md"

// mockVaultWriter is a mock implementation for interface compliance testing.
type mockVaultWriter struct{}

// mockVaultScanner is a mock implementation for VaultScannerPort interface
// compliance testing.
type mockVaultScanner struct{}

// mockVaultReader is a mock implementation for VaultReaderPort interface
// compliance testing.
type mockVaultReader struct{}

// ScanAll is a mock implementation of VaultScannerPort.ScanAll.
func (m *mockVaultScanner) ScanAll(
	ctx context.Context,
) ([]dto.VaultFile, error) {
	return []dto.VaultFile{}, nil
}

// ScanModified is a mock implementation of VaultScannerPort.ScanModified.
func (m *mockVaultScanner) ScanModified(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	return []dto.VaultFile{}, nil
}

// Read is a mock implementation of VaultReaderPort.Read.
func (m *mockVaultReader) Read(
	ctx context.Context,
	path string,
) (dto.VaultFile, error) {
	return dto.VaultFile{}, nil
}

// Persist is a mock implementation of VaultWriterPort.Persist.
func (m *mockVaultWriter) Persist(
	ctx context.Context,
	note domain.Note,
	path string,
) error {
	return nil
}

// Delete is a mock implementation of VaultWriterPort.Delete.
func (m *mockVaultWriter) Delete(ctx context.Context, path string) error {
	return nil
}

// WriteContent is a mock implementation of VaultWriterPort.WriteContent.
func (m *mockVaultWriter) WriteContent(
	ctx context.Context,
	path string,
	content []byte,
) error {
	return nil
}

// TestVaultWriterPortInterfaceCompliance verifies that VaultWriterPort
// interface exists and has the required methods with correct signatures.
func TestVaultWriterPortInterfaceCompliance(t *testing.T) {
	// This test will fail until VaultWriterPort interface is defined
	// It verifies the interface exists and has the required methods

	// We can't instantiate an interface, but we can check if it compiles
	// by attempting to assign a nil pointer to a variable of the interface type
	var _ VaultWriterPort = (*mockVaultWriter)(nil)
}

// TestVaultWriterPortPersistSignature verifies Persist method signature.
func TestVaultWriterPortPersistSignature(t *testing.T) {
	// This test verifies the Persist method has the correct signature
	// by attempting to call it on a mock implementation

	mock := &mockVaultWriter{}
	ctx := context.Background()
	note := domain.Note{} // Empty note for signature test

	// This should compile and run without error if signature is correct
	err := mock.Persist(ctx, note, testPath)
	if err != nil {
		t.Errorf("Persist should not return error in mock: %v", err)
	}
}

// TestVaultWriterPortDeleteSignature verifies Delete method signature.
func TestVaultWriterPortDeleteSignature(t *testing.T) {
	// This test verifies the Delete method has the correct signature
	// by attempting to call it on a mock implementation

	mock := &mockVaultWriter{}
	ctx := context.Background()

	// This should compile and run without error if signature is correct
	err := mock.Delete(ctx, testPath)
	if err != nil {
		t.Errorf("Delete should not return error in mock: %v", err)
	}
}

// TestVaultWriterPortWriteContentSignature verifies WriteContent method
// signature.
func TestVaultWriterPortWriteContentSignature(t *testing.T) {
	mock := &mockVaultWriter{}
	ctx := context.Background()

	err := mock.WriteContent(ctx, testPath, []byte("content"))
	if err != nil {
		t.Errorf("WriteContent should not return error in mock: %v", err)
	}
}

// TestVaultScannerPortInterfaceCompliance verifies that VaultScannerPort
// interface exists and has the required methods with correct signatures.
func TestVaultScannerPortInterfaceCompliance(t *testing.T) {
	// This test will fail until VaultScannerPort interface is defined
	// It verifies the interface exists and has the required methods

	// We can't instantiate an interface, but we can check if it compiles
	// by attempting to assign a nil pointer to a variable of the interface type
	var _ VaultScannerPort = (*mockVaultScanner)(nil)
}

// TestVaultScannerPortScanAllSignature verifies ScanAll method signature.
func TestVaultScannerPortScanAllSignature(t *testing.T) {
	// This test verifies the ScanAll method has the correct signature
	// by attempting to call it on a mock implementation

	mock := &mockVaultScanner{}
	ctx := context.Background()

	// This should compile and run without error if signature is correct
	files, err := mock.ScanAll(ctx)
	if err != nil {
		t.Errorf("ScanAll should not return error in mock: %v", err)
	}
	if files == nil {
		t.Errorf("ScanAll should return empty slice, not nil")
	}
}

// TestVaultScannerPortScanModifiedSignature verifies ScanModified method
// signature.
func TestVaultScannerPortScanModifiedSignature(t *testing.T) {
	// This test verifies the ScanModified method has the correct signature
	// by attempting to call it on a mock implementation

	mock := &mockVaultScanner{}
	ctx := context.Background()
	since := time.Now()

	// This should compile and run without error if signature is correct
	files, err := mock.ScanModified(ctx, since)
	if err != nil {
		t.Errorf("ScanModified should not return error in mock: %v", err)
	}
	if files == nil {
		t.Errorf("ScanModified should return empty slice, not nil")
	}
}

// TestVaultReaderPortInterfaceCompliance verifies that VaultReaderPort
// interface exists and has the required methods with correct signatures.
func TestVaultReaderPortInterfaceCompliance(t *testing.T) {
	// This test will fail until VaultReaderPort interface is defined
	// It verifies the interface exists and has the required methods

	// We can't instantiate an interface, but we can check if it compiles
	// by attempting to assign a nil pointer to a variable of the interface type
	var _ VaultReaderPort = (*mockVaultReader)(nil)
}

// TestVaultReaderPortReadSignature verifies Read method signature.
func TestVaultReaderPortReadSignature(t *testing.T) {
	// This test verifies the Read method has the correct signature
	// by attempting to call it on a mock implementation

	mock := &mockVaultReader{}
	ctx := context.Background()

	// This should compile and run without error if signature is correct
	file, err := mock.Read(ctx, testPath)
	if err != nil {
		t.Errorf("Read should not return error in mock: %v", err)
	}
	// file can be empty for signature test
	_ = file
}
