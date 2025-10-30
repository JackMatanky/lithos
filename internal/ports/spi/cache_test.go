package spi

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
)

// MockCacheWriter is a mock implementation of CacheWriterPort for testing.
type MockCacheWriter struct {
	PersistFunc func(ctx context.Context, note domain.Note) error
	DeleteFunc  func(ctx context.Context, id domain.NoteID) error
}

// MockCacheReader is a mock implementation of CacheReaderPort for testing.
type MockCacheReader struct {
	ReadFunc func(ctx context.Context, id domain.NoteID) (domain.Note, error)
	ListFunc func(ctx context.Context) ([]domain.Note, error)
}

// Persist delegates to PersistFunc if set, otherwise returns nil.
func (m *MockCacheWriter) Persist(ctx context.Context, note domain.Note) error {
	if m.PersistFunc != nil {
		return m.PersistFunc(ctx, note)
	}
	return nil
}

// Delete delegates to DeleteFunc if set, otherwise returns nil.
func (m *MockCacheWriter) Delete(ctx context.Context, id domain.NoteID) error {
	if m.DeleteFunc != nil {
		return m.DeleteFunc(ctx, id)
	}
	return nil
}

// Read delegates to ReadFunc if set, otherwise returns empty Note and nil
// error.
func (m *MockCacheReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	if m.ReadFunc != nil {
		return m.ReadFunc(ctx, id)
	}
	return domain.Note{}, nil
}

// List delegates to ListFunc if set, otherwise returns empty slice and nil
// error.
func (m *MockCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	if m.ListFunc != nil {
		return m.ListFunc(ctx)
	}
	return []domain.Note{}, nil
}

// TestCacheWriterPortInterfaceCompliance verifies that MockCacheWriter
// satisfies CacheWriterPort interface.
func TestCacheWriterPortInterfaceCompliance(t *testing.T) {
	// This will fail to compile until CacheWriterPort interface is defined
	var _ CacheWriterPort = (*MockCacheWriter)(nil)
}

// TestCacheWriterPortPersistSignature verifies Persist method signature.
func TestCacheWriterPortPersistSignature(t *testing.T) {
	mock := &MockCacheWriter{}
	ctx := context.Background()
	note := domain.Note{} // This will fail until Note type is defined

	// This will fail to compile until Persist method signature matches
	err := mock.Persist(ctx, note)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
}

// TestCacheWriterPortDeleteSignature verifies Delete method signature.
func TestCacheWriterPortDeleteSignature(t *testing.T) {
	mock := &MockCacheWriter{}
	ctx := context.Background()
	id := domain.NoteID("test") // This will fail until NoteID type is defined

	// This will fail to compile until Delete method signature matches
	err := mock.Delete(ctx, id)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
}

// TestCacheReaderPortInterfaceCompliance verifies that MockCacheReader
// satisfies CacheReaderPort interface.
func TestCacheReaderPortInterfaceCompliance(t *testing.T) {
	// This will fail to compile until CacheReaderPort interface is defined
	var _ CacheReaderPort = (*MockCacheReader)(nil)
}

// TestCacheReaderPortReadSignature verifies Read method signature.
func TestCacheReaderPortReadSignature(t *testing.T) {
	mock := &MockCacheReader{}
	ctx := context.Background()
	id := domain.NoteID("test")

	// This will fail to compile until Read method signature matches
	note, err := mock.Read(ctx, id)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if note.ID != "" {
		t.Errorf("Expected empty note, got: %v", note)
	}
}

// TestCacheReaderPortListSignature verifies List method signature.
func TestCacheReaderPortListSignature(t *testing.T) {
	mock := &MockCacheReader{}
	ctx := context.Background()

	// This will fail to compile until List method signature matches
	notes, err := mock.List(ctx)
	if err != nil {
		t.Errorf("Unexpected error: %v", err)
	}
	if len(notes) != 0 {
		t.Errorf("Expected empty slice, got: %v", notes)
	}
}
