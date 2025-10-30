package spi

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

const testPath = "/test/path.md"

// mockVaultWriter is a mock implementation for interface compliance testing.
type mockVaultWriter struct{}

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
