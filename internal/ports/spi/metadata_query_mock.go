// MockMetadataQueryPort provides mock implementations of SPI ports for testing.
package spi

import (
	"context"

	"github.com/JackMatanky/lithos/internal/domain"
)

// Ensure MockMetadataQueryPort implements MetadataQueryPort interface.
var _ MetadataQueryPort = (*MockMetadataQueryPort)(nil)

// MockMetadataQueryPort provides a mock implementation of MetadataQueryPort for
// testing. It allows configuring mock responses for each query method and
// tracks call counts
// for assertion purposes.
//
// Usage:
//
//	mock := NewMockMetadataQueryPort()
//	mock.SetByBasenameResult([]domain.Note{note1, note2}, nil)
//	mock.SetByAliasResult([]domain.Note{note3}, nil)
//
//	// Use in tests
//	service := NewQueryService(mock, ...)
//	notes, err := service.FindByBasename("test")
//
//	// Assert calls
//	assert.Equal(t, 1, mock.ByBasenameCallCount)
type MockMetadataQueryPort struct {
	// Function fields for method delegation
	ByBasenameFunc  func(ctx context.Context, basename string) ([]domain.Note, error)
	ByAliasFunc     func(ctx context.Context, alias string) ([]domain.Note, error)
	ByFileClassFunc func(ctx context.Context, fileClass string) ([]domain.Note, error)
	PathQueryFunc   func(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error)

	// Call tracking for assertions
	ByBasenameCallCount  int
	ByAliasCallCount     int
	ByFileClassCallCount int
	PathQueryCallCount   int

	// Last call arguments for detailed assertions
	LastByBasenameArg  string
	LastByAliasArg     string
	LastByFileClassArg string
	LastPathQueryOpts  PathQueryOptions
}

// NewMockMetadataQueryPort creates a new MockMetadataQueryPort with default
// behavior.
// By default, all methods return empty slices and nil errors.
// Configure specific behavior using the Set*Result methods.
func NewMockMetadataQueryPort() *MockMetadataQueryPort {
	return &MockMetadataQueryPort{
		ByBasenameFunc: func(ctx context.Context, basename string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		ByAliasFunc: func(ctx context.Context, alias string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		ByFileClassFunc: func(ctx context.Context, fileClass string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		PathQueryFunc: func(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		ByBasenameCallCount:  0,
		ByAliasCallCount:     0,
		ByFileClassCallCount: 0,
		PathQueryCallCount:   0,
		LastByBasenameArg:    "",
		LastByAliasArg:       "",
		LastByFileClassArg:   "",
		LastPathQueryOpts:    PathQueryOptions{Value: "", Scope: ""},
	}
}

// SetByBasenameResult configures the mock to return the specified result for
// ByBasename calls.
func (m *MockMetadataQueryPort) SetByBasenameResult(
	notes []domain.Note,
	err error,
) {
	m.ByBasenameFunc = func(ctx context.Context, basename string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetByAliasResult configures the mock to return the specified result for
// ByAlias calls.
func (m *MockMetadataQueryPort) SetByAliasResult(
	notes []domain.Note,
	err error,
) {
	m.ByAliasFunc = func(ctx context.Context, alias string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetByFileClassResult configures the mock to return the specified result for
// ByFileClass calls.
func (m *MockMetadataQueryPort) SetByFileClassResult(
	notes []domain.Note,
	err error,
) {
	m.ByFileClassFunc = func(ctx context.Context, fileClass string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetPathQueryResult configures the mock to return the specified result for
// PathQuery calls.
func (m *MockMetadataQueryPort) SetPathQueryResult(
	notes []domain.Note,
	err error,
) {
	m.PathQueryFunc = func(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error) {
		return notes, err
	}
}

// ByBasename implements MetadataQueryPort.ByBasename with mock behavior.
func (m *MockMetadataQueryPort) ByBasename(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	m.ByBasenameCallCount++
	m.LastByBasenameArg = basename
	return m.ByBasenameFunc(ctx, basename)
}

// ByAlias implements MetadataQueryPort.ByAlias with mock behavior.
func (m *MockMetadataQueryPort) ByAlias(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	m.ByAliasCallCount++
	m.LastByAliasArg = alias
	return m.ByAliasFunc(ctx, alias)
}

// ByFileClass implements MetadataQueryPort.ByFileClass with mock behavior.
func (m *MockMetadataQueryPort) ByFileClass(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	m.ByFileClassCallCount++
	m.LastByFileClassArg = fileClass
	return m.ByFileClassFunc(ctx, fileClass)
}

// PathQuery implements MetadataQueryPort.PathQuery with mock behavior.
func (m *MockMetadataQueryPort) PathQuery(
	ctx context.Context,
	opts PathQueryOptions,
) ([]domain.Note, error) {
	m.PathQueryCallCount++
	m.LastPathQueryOpts = opts
	return m.PathQueryFunc(ctx, opts)
}

// Reset resets all call tracking counters and last arguments.
// Useful for testing multiple scenarios in the same test.
func (m *MockMetadataQueryPort) Reset() {
	m.ByBasenameCallCount = 0
	m.ByAliasCallCount = 0
	m.ByFileClassCallCount = 0
	m.PathQueryCallCount = 0
	m.LastByBasenameArg = ""
	m.LastByAliasArg = ""
	m.LastByFileClassArg = ""
	m.LastPathQueryOpts = PathQueryOptions{Value: "", Scope: ""}
}
