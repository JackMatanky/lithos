package spi

import (
	"context"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// MockCacheWriter is a mock implementation of CacheWriterPort for testing.
type MockCacheWriter struct {
	PersistFunc func(ctx context.Context, note domain.Note, indexTime time.Time) error
	DeleteFunc  func(ctx context.Context, id domain.NoteID) error
}

// MockCacheReader is a mock implementation of CacheReaderPort for testing.
type MockCacheReader struct {
	ReadFunc func(ctx context.Context, id domain.NoteID) (domain.Note, error)
	ListFunc func(ctx context.Context) ([]domain.Note, error)
}

// Persist delegates to PersistFunc if set, otherwise returns nil.
func (m *MockCacheWriter) Persist(
	ctx context.Context,
	note domain.Note,
	indexTime time.Time,
) error {
	if m.PersistFunc != nil {
		return m.PersistFunc(ctx, note, indexTime)
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
//
// logical grouping.
//
//nolint:decorder // Test file - type defined after some test functions for
type MockMetadataQueryPort struct {
	// Function fields for method delegation
	ByBasenameFunc       func(ctx context.Context, basename string) ([]domain.Note, error)
	ByAliasFunc          func(ctx context.Context, alias string) ([]domain.Note, error)
	ByFileClassFunc      func(ctx context.Context, fileClass string) ([]domain.Note, error)
	PathQueryFunc        func(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error)
	TagQueryFunc         func(ctx context.Context, tag string) ([]domain.Note, error)
	FrontmatterQueryFunc func(ctx context.Context, field, value string) ([]domain.Note, error)

	// Call tracking for assertions
	ByBasenameCallCount       int
	ByAliasCallCount          int
	ByFileClassCallCount      int
	PathQueryCallCount        int
	TagQueryCallCount         int
	FrontmatterQueryCallCount int

	// Last call arguments for detailed assertions
	LastByBasenameArg            string
	LastByAliasArg               string
	LastByFileClassArg           string
	LastPathQueryOpts            PathQueryOptions
	LastTagQueryArg              string
	LastFrontmatterQueryArgField string
	LastFrontmatterQueryArgValue string
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
		TagQueryFunc: func(ctx context.Context, tag string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		FrontmatterQueryFunc: func(ctx context.Context, field, value string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		ByBasenameCallCount:          0,
		ByAliasCallCount:             0,
		ByFileClassCallCount:         0,
		PathQueryCallCount:           0,
		TagQueryCallCount:            0,
		FrontmatterQueryCallCount:    0,
		LastByBasenameArg:            "",
		LastByAliasArg:               "",
		LastByFileClassArg:           "",
		LastPathQueryOpts:            PathQueryOptions{Value: "", Scope: ""},
		LastTagQueryArg:              "",
		LastFrontmatterQueryArgField: "",
		LastFrontmatterQueryArgValue: "",
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

// SetTagQueryResult configures the mock to return the specified result for
// TagQuery calls.
func (m *MockMetadataQueryPort) SetTagQueryResult(
	notes []domain.Note,
	err error,
) {
	m.TagQueryFunc = func(ctx context.Context, tag string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetFrontmatterQueryResult configures the mock to return the specified result
// for
// FrontmatterQuery calls.
func (m *MockMetadataQueryPort) SetFrontmatterQueryResult(
	notes []domain.Note,
	err error,
) {
	m.FrontmatterQueryFunc = func(ctx context.Context, field, value string) ([]domain.Note, error) {
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

// TagQuery implements MetadataQueryPort.TagQuery with mock behavior.
func (m *MockMetadataQueryPort) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	m.TagQueryCallCount++
	m.LastTagQueryArg = tag
	return m.TagQueryFunc(ctx, tag)
}

// FrontmatterQuery implements MetadataQueryPort.FrontmatterQuery with mock
// behavior.
func (m *MockMetadataQueryPort) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	m.FrontmatterQueryCallCount++
	m.LastFrontmatterQueryArgField = field
	m.LastFrontmatterQueryArgValue = value
	return m.FrontmatterQueryFunc(ctx, field, value)
}

// Reset resets all call tracking counters and last arguments.
// Useful for testing multiple scenarios in the same test.
func (m *MockMetadataQueryPort) Reset() {
	m.ByBasenameCallCount = 0
	m.ByAliasCallCount = 0
	m.ByFileClassCallCount = 0
	m.PathQueryCallCount = 0
	m.TagQueryCallCount = 0
	m.FrontmatterQueryCallCount = 0
	m.LastByBasenameArg = ""
	m.LastByAliasArg = ""
	m.LastByFileClassArg = ""
	m.LastPathQueryOpts = PathQueryOptions{Value: "", Scope: ""}
	m.LastTagQueryArg = ""
	m.LastFrontmatterQueryArgField = ""
	m.LastFrontmatterQueryArgValue = ""
}

// according to the interface specification.
func TestMetadataQueryPortInterfaceContract(t *testing.T) {
	ctx := context.Background()
	mock := NewMockMetadataQueryPort()

	// Test ByBasename method signature and behavior
	t.Run("ByBasename method contract", func(t *testing.T) {
		// Configure mock to return test data
		testNotes := []domain.Note{
			{
				ID: "test1.md",
				Frontmatter: domain.NewFrontmatter(
					map[string]any{"title": "Test 1"},
				),
			},
			{
				ID: "test2.md",
				Frontmatter: domain.NewFrontmatter(
					map[string]any{"title": "Test 2"},
				),
			},
		}
		mock.SetByBasenameResult(testNotes, nil)

		// Call method
		result, err := mock.ByBasename(ctx, "test")

		// Verify contract
		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.ByBasenameCallCount)
		assert.Equal(t, "test", mock.LastByBasenameArg)
	})

	// Test ByAlias method signature and behavior
	t.Run("ByAlias method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			{
				ID: "note1.md",
				Frontmatter: domain.NewFrontmatter(
					map[string]any{"title": "Note 1"},
				),
			},
		}
		mock.SetByAliasResult(testNotes, nil)

		result, err := mock.ByAlias(ctx, "project-alpha")

		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.ByAliasCallCount)
		assert.Equal(t, "project-alpha", mock.LastByAliasArg)
	})

	// Test ByFileClass method signature and behavior
	t.Run("ByFileClass method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			{
				ID: "meeting1.md",
				Frontmatter: domain.NewFrontmatter(
					map[string]any{"fileClass": "meeting"},
				),
			},
			{
				ID: "meeting2.md",
				Frontmatter: domain.NewFrontmatter(
					map[string]any{"fileClass": "meeting"},
				),
			},
		}
		mock.SetByFileClassResult(testNotes, nil)

		result, err := mock.ByFileClass(ctx, "meeting")

		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.ByFileClassCallCount)
		assert.Equal(t, "meeting", mock.LastByFileClassArg)
	})

	// Test PathQuery method contract
	t.Run("PathQuery method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			{ID: "notes/project/foo.md"},
			{ID: "notes/project/bar.md"},
		}
		opts := PathQueryOptions{
			Scope: PathQueryScopeFolder,
			Value: "notes/project/",
		}
		mock.SetPathQueryResult(testNotes, nil)

		result, err := mock.PathQuery(ctx, opts)

		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.PathQueryCallCount)
		assert.Equal(t, opts, mock.LastPathQueryOpts)
	})

	// Test empty results behavior
	t.Run("empty results return empty slice not nil", func(t *testing.T) {
		mock.Reset()
		// Reset mock to default behavior (empty results)
		mock.SetByBasenameResult([]domain.Note{}, nil)
		mock.SetByAliasResult([]domain.Note{}, nil)
		mock.SetByFileClassResult([]domain.Note{}, nil)
		mock.SetPathQueryResult([]domain.Note{}, nil)

		// ByBasename with no matches
		result, err := mock.ByBasename(ctx, "nonexistent")
		require.NoError(t, err)
		assert.NotNil(t, result) // Should be empty slice, not nil
		assert.Empty(t, result)

		// ByAlias with no matches
		result, err = mock.ByAlias(ctx, "nonexistent")
		require.NoError(t, err)
		assert.NotNil(t, result)
		assert.Empty(t, result)

		// ByFileClass with no matches
		result, err = mock.ByFileClass(ctx, "nonexistent")
		require.NoError(t, err)
		assert.NotNil(t, result)
		assert.Empty(t, result)

		// PathQuery with no matches
		result, err = mock.PathQuery(
			ctx,
			PathQueryOptions{Value: "notes/missing.md"},
		)
		require.NoError(t, err)
		assert.NotNil(t, result)
		assert.Empty(t, result)
	})

	// Test context cancellation handling
	t.Run("context cancellation propagation", func(t *testing.T) {
		cancelledCtx, cancel := context.WithCancel(ctx)
		cancel() // Cancel immediately

		mock.SetByBasenameResult(nil, context.Canceled)

		_, err := mock.ByBasename(cancelledCtx, "test")
		assert.Equal(t, context.Canceled, err)
	})

	// Test call tracking reset
	t.Run("call tracking reset functionality", func(t *testing.T) {
		mock.ByBasenameCallCount = 5
		mock.LastByBasenameArg = "old"

		mock.Reset()

		assert.Equal(t, 0, mock.ByBasenameCallCount)
		assert.Empty(t, mock.LastByBasenameArg)
	})
}

// TestMockMetadataQueryPortDefaultBehavior verifies the default behavior of the
// mock.
func TestMockMetadataQueryPortDefaultBehavior(t *testing.T) {
	ctx := context.Background()
	mock := NewMockMetadataQueryPort()

	// Default behavior should return empty slices and no errors
	result, err := mock.ByBasename(ctx, "any")
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)

	result, err = mock.ByAlias(ctx, "any")
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)

	result, err = mock.ByFileClass(ctx, "any")
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)

	result, err = mock.PathQuery(ctx, PathQueryOptions{Value: "any"})
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)
}

// TestMockMetadataQueryPortImplementsInterface verifies the mock implements the
// interface.
func TestMockMetadataQueryPortImplementsInterface(t *testing.T) {
	var _ MetadataQueryPort = (*MockMetadataQueryPort)(nil)
	var port MetadataQueryPort = NewMockMetadataQueryPort()
	assert.NotNil(t, port)
}

func TestPathQueryOptionsValidate(t *testing.T) {
	t.Run("defaults to full path scope", func(t *testing.T) {
		opts, err := (PathQueryOptions{Value: "notes/foo.md"}).Validate()
		require.NoError(t, err)
		assert.Equal(t, PathQueryScopeFull, opts.Scope)
		assert.Equal(t, "notes/foo.md", opts.Value)
	})

	t.Run("rejects empty value", func(t *testing.T) {
		_, err := (PathQueryOptions{}).Validate()
		require.Error(t, err)
	})
}
