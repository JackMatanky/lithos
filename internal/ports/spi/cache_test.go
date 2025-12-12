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
	DeleteFunc  func(ctx context.Context, path string) error
}

// MockCacheReader is a mock implementation of CacheReaderPort for testing.
type MockCacheReader struct {
	ReadFunc func(ctx context.Context, path string) (domain.Note, error)
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
func (m *MockCacheWriter) Delete(ctx context.Context, path string) error {
	if m.DeleteFunc != nil {
		return m.DeleteFunc(ctx, path)
	}
	return nil
}

// Read delegates to ReadFunc if set, otherwise returns empty Note and nil
// error.
func (m *MockCacheReader) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if m.ReadFunc != nil {
		return m.ReadFunc(ctx, path)
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
//	mock.SetBasenameQueryResult([]domain.Note{note1, note2}, nil)
//	mock.SetAliasQueryResult([]domain.Note{note3}, nil)
//
//	// Use in tests
//	service := NewQueryService(mock, ...)
//	notes, err := service.FindBasenameQuery("test")
//
//	// Assert calls
//	assert.Equal(t, 1, mock.BasenameQueryCallCount)
//
// logical grouping.
//
//nolint:decorder // Test file - type defined after some test functions for
type MockMetadataQueryPort struct {
	// Function fields for method delegation
	BasenameQueryFunc    func(ctx context.Context, basename string) ([]domain.Note, error)
	AliasQueryFunc       func(ctx context.Context, alias string) ([]domain.Note, error)
	FileClassQueryFunc   func(ctx context.Context, fileClass string) ([]domain.Note, error)
	PathQueryFunc        func(ctx context.Context, opts PathQueryOptions) ([]domain.Note, error)
	TagQueryFunc         func(ctx context.Context, tag string) ([]domain.Note, error)
	FrontmatterQueryFunc func(ctx context.Context, field, value string) ([]domain.Note, error)

	// Call tracking for assertions
	BasenameQueryCallCount    int
	AliasQueryCallCount       int
	FileClassQueryCallCount   int
	PathQueryCallCount        int
	TagQueryCallCount         int
	FrontmatterQueryCallCount int

	// Last call arguments for detailed assertions
	LastBasenameQueryArg         string
	LastAliasQueryArg            string
	LastFileClassQueryArg        string
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
		BasenameQueryFunc: func(ctx context.Context, basename string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		AliasQueryFunc: func(ctx context.Context, alias string) ([]domain.Note, error) {
			return []domain.Note{}, nil
		},
		FileClassQueryFunc: func(ctx context.Context, fileClass string) ([]domain.Note, error) {
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
		BasenameQueryCallCount:       0,
		AliasQueryCallCount:          0,
		FileClassQueryCallCount:      0,
		PathQueryCallCount:           0,
		TagQueryCallCount:            0,
		FrontmatterQueryCallCount:    0,
		LastBasenameQueryArg:         "",
		LastAliasQueryArg:            "",
		LastFileClassQueryArg:        "",
		LastPathQueryOpts:            PathQueryOptions{Value: "", Scope: ""},
		LastTagQueryArg:              "",
		LastFrontmatterQueryArgField: "",
		LastFrontmatterQueryArgValue: "",
	}
}

// SetBasenameQueryResult configures the mock to return the specified result for
// BasenameQuery calls.
func (m *MockMetadataQueryPort) SetBasenameQueryResult(
	notes []domain.Note,
	err error,
) {
	m.BasenameQueryFunc = func(ctx context.Context, basename string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetAliasQueryResult configures the mock to return the specified result for
// AliasQuery calls.
func (m *MockMetadataQueryPort) SetAliasQueryResult(
	notes []domain.Note,
	err error,
) {
	m.AliasQueryFunc = func(ctx context.Context, alias string) ([]domain.Note, error) {
		return notes, err
	}
}

// SetFileClassQueryResult configures the mock to return the specified result
// for
// FileClassQuery calls.
func (m *MockMetadataQueryPort) SetFileClassQueryResult(
	notes []domain.Note,
	err error,
) {
	m.FileClassQueryFunc = func(ctx context.Context, fileClass string) ([]domain.Note, error) {
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

// BasenameQuery implements MetadataQueryPort.BasenameQuery with mock behavior.
func (m *MockMetadataQueryPort) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	m.BasenameQueryCallCount++
	m.LastBasenameQueryArg = basename
	return m.BasenameQueryFunc(ctx, basename)
}

// AliasQuery implements MetadataQueryPort.AliasQuery with mock behavior.
func (m *MockMetadataQueryPort) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	m.AliasQueryCallCount++
	m.LastAliasQueryArg = alias
	return m.AliasQueryFunc(ctx, alias)
}

// FileClassQuery implements MetadataQueryPort.FileClassQuery with mock
// behavior.
func (m *MockMetadataQueryPort) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	m.FileClassQueryCallCount++
	m.LastFileClassQueryArg = fileClass
	return m.FileClassQueryFunc(ctx, fileClass)
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
	m.BasenameQueryCallCount = 0
	m.AliasQueryCallCount = 0
	m.FileClassQueryCallCount = 0
	m.PathQueryCallCount = 0
	m.TagQueryCallCount = 0
	m.FrontmatterQueryCallCount = 0
	m.LastBasenameQueryArg = ""
	m.LastAliasQueryArg = ""
	m.LastFileClassQueryArg = ""
	m.LastPathQueryOpts = PathQueryOptions{Value: "", Scope: ""}
	m.LastTagQueryArg = ""
	m.LastFrontmatterQueryArgField = ""
	m.LastFrontmatterQueryArgValue = ""
}

// according to the interface specification.
func TestMetadataQueryPortInterfaceContract(t *testing.T) {
	ctx := context.Background()
	mock := NewMockMetadataQueryPort()

	// Test BasenameQuery method signature and behavior
	t.Run("BasenameQuery method contract", func(t *testing.T) {
		// Configure mock to return test data
		testNotes := []domain.Note{
			func() domain.Note {
				note, _ := domain.NewNote("test1.md", domain.NewFrontmatter(
					map[string]any{"title": "Test 1"},
				), []domain.Link{}, []domain.Heading{}, []string{}, []domain.TaskItem{})
				return note
			}(),
			func() domain.Note {
				note, _ := domain.NewNote("test2.md", domain.NewFrontmatter(
					map[string]any{"title": "Test 2"},
				), []domain.Link{}, []domain.Heading{}, []string{}, []domain.TaskItem{})
				return note
			}(),
		}
		mock.SetBasenameQueryResult(testNotes, nil)

		// Call method
		result, err := mock.BasenameQuery(ctx, "test")

		// Verify contract
		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.BasenameQueryCallCount)
		assert.Equal(t, "test", mock.LastBasenameQueryArg)
	})

	// Test AliasQuery method signature and behavior
	t.Run("AliasQuery method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			func() domain.Note {
				note, _ := domain.NewNote("note1.md", domain.NewFrontmatter(
					map[string]any{"title": "Note 1"},
				), []domain.Link{}, []domain.Heading{}, []string{}, []domain.TaskItem{})
				return note
			}(),
		}
		mock.SetAliasQueryResult(testNotes, nil)

		result, err := mock.AliasQuery(ctx, "project-alpha")

		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.AliasQueryCallCount)
		assert.Equal(t, "project-alpha", mock.LastAliasQueryArg)
	})

	// Test FileClassQuery method signature and behavior
	t.Run("FileClassQuery method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			func() domain.Note {
				note, _ := domain.NewNote("meeting1.md", domain.NewFrontmatter(
					map[string]any{"fileClass": "meeting"},
				), []domain.Link{}, []domain.Heading{}, []string{}, []domain.TaskItem{})
				return note
			}(),
			func() domain.Note {
				note, _ := domain.NewNote("meeting2.md", domain.NewFrontmatter(
					map[string]any{"fileClass": "meeting"},
				), []domain.Link{}, []domain.Heading{}, []string{}, []domain.TaskItem{})
				return note
			}(),
		}
		mock.SetFileClassQueryResult(testNotes, nil)

		result, err := mock.FileClassQuery(ctx, "meeting")

		require.NoError(t, err)
		assert.Equal(t, testNotes, result)
		assert.Equal(t, 1, mock.FileClassQueryCallCount)
		assert.Equal(t, "meeting", mock.LastFileClassQueryArg)
	})

	// Test PathQuery method contract
	t.Run("PathQuery method contract", func(t *testing.T) {
		testNotes := []domain.Note{
			func() domain.Note {
				note, _ := domain.NewNote(
					"notes/project/foo.md",
					domain.NewFrontmatter(map[string]any{}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
			func() domain.Note {
				note, _ := domain.NewNote(
					"notes/project/bar.md",
					domain.NewFrontmatter(map[string]any{}),
					[]domain.Link{},
					[]domain.Heading{},
					[]string{},
					[]domain.TaskItem{},
				)
				return note
			}(),
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
		mock.SetBasenameQueryResult([]domain.Note{}, nil)
		mock.SetAliasQueryResult([]domain.Note{}, nil)
		mock.SetFileClassQueryResult([]domain.Note{}, nil)
		mock.SetPathQueryResult([]domain.Note{}, nil)

		// BasenameQuery with no matches
		result, err := mock.BasenameQuery(ctx, "nonexistent")
		require.NoError(t, err)
		assert.NotNil(t, result) // Should be empty slice, not nil
		assert.Empty(t, result)

		// AliasQuery with no matches
		result, err = mock.AliasQuery(ctx, "nonexistent")
		require.NoError(t, err)
		assert.NotNil(t, result)
		assert.Empty(t, result)

		// FileClassQuery with no matches
		result, err = mock.FileClassQuery(ctx, "nonexistent")
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

		mock.SetBasenameQueryResult(nil, context.Canceled)

		_, err := mock.BasenameQuery(cancelledCtx, "test")
		assert.Equal(t, context.Canceled, err)
	})

	// Test call tracking reset
	t.Run("call tracking reset functionality", func(t *testing.T) {
		mock.BasenameQueryCallCount = 5
		mock.LastBasenameQueryArg = "old"

		mock.Reset()

		assert.Equal(t, 0, mock.BasenameQueryCallCount)
		assert.Empty(t, mock.LastBasenameQueryArg)
	})
}

// TestMockMetadataQueryPortDefaultBehavior verifies the default behavior of the
// mock.
func TestMockMetadataQueryPortDefaultBehavior(t *testing.T) {
	ctx := context.Background()
	mock := NewMockMetadataQueryPort()

	// Default behavior should return empty slices and no errors
	result, err := mock.BasenameQuery(ctx, "any")
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)

	result, err = mock.AliasQuery(ctx, "any")
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Empty(t, result)

	result, err = mock.FileClassQuery(ctx, "any")
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
