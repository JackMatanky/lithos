package spi

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestMetadataQueryPortInterfaceContract verifies the MetadataQueryPort
// interface contract. This test ensures that any implementation of
// MetadataQueryPort behaves correctly
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
