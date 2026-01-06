package query

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockQueryBackend implements QueryBackend for testing.
type mockQueryBackend struct {
	fileClassQueryFn   func(ctx context.Context, fileClass string) ([]domain.Note, error)
	basenameQueryFn    func(ctx context.Context, basename string) ([]domain.Note, error)
	aliasQueryFn       func(ctx context.Context, alias string) ([]domain.Note, error)
	pathQueryFn        func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error)
	frontmatterQueryFn func(ctx context.Context, field, value string) ([]domain.Note, error)
	tagQueryFn         func(ctx context.Context, tag string) ([]domain.Note, error)
	readFn             func(ctx context.Context, path string) (domain.Note, error)
	listFn             func(ctx context.Context) ([]domain.Note, error)
}

func (m *mockQueryBackend) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	if m.fileClassQueryFn != nil {
		return m.fileClassQueryFn(ctx, fileClass)
	}
	return nil, nil
}

func (m *mockQueryBackend) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	if m.basenameQueryFn != nil {
		return m.basenameQueryFn(ctx, basename)
	}
	return nil, nil
}

func (m *mockQueryBackend) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	if m.aliasQueryFn != nil {
		return m.aliasQueryFn(ctx, alias)
	}
	return nil, nil
}

func (m *mockQueryBackend) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	if m.pathQueryFn != nil {
		return m.pathQueryFn(ctx, opts)
	}
	return nil, nil
}

func (m *mockQueryBackend) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	if m.frontmatterQueryFn != nil {
		return m.frontmatterQueryFn(ctx, field, value)
	}
	return nil, nil
}

func (m *mockQueryBackend) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	if m.tagQueryFn != nil {
		return m.tagQueryFn(ctx, tag)
	}
	return nil, nil
}

func (m *mockQueryBackend) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if m.readFn != nil {
		return m.readFn(ctx, path)
	}
	return domain.Note{}, nil
}

func (m *mockQueryBackend) List(ctx context.Context) ([]domain.Note, error) {
	if m.listFn != nil {
		return m.listFn(ctx)
	}
	return nil, nil
}

// TestStorageRouter_NewStorageRouter verifies router construction.
func TestStorageRouter_NewStorageRouter(t *testing.T) {
	t.Parallel()

	boltPort := &mockQueryBackend{}
	sqlitePort := &mockQueryBackend{}

	router := NewStorageRouter(boltPort, sqlitePort)

	require.NotNil(t, router)
	assert.Equal(t, boltPort, router.bolt)
	assert.Equal(t, sqlitePort, router.sqlite)
}

// TestStorageRouter_RouteMetadataQuery_HotPath verifies hot path routing.
func TestStorageRouter_RouteMetadataQuery_HotPath(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}
	sqlitePort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			t.Error("SQLite should not be called when BoltDB succeeds")
			return nil, nil
		},
	}

	router := NewStorageRouter(boltPort, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	require.NoError(t, err)
	require.Len(t, notes, 1)
	assert.Equal(t, expectedNote.Path, notes[0].Path)
}

// TestStorageRouter_RouteMetadataQuery_DeepPathFallback verifies fallback
// to SQLite.
func TestStorageRouter_RouteMetadataQuery_DeepPathFallback(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return nil, errors.New("bolt error")
		},
	}
	sqlitePort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewStorageRouter(boltPort, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	require.NoError(t, err)
	require.Len(t, notes, 1)
	assert.Equal(t, expectedNote.Path, notes[0].Path)
}

// TestStorageRouter_RouteMetadataQuery_NoBackends verifies behavior with
// no backends.
func TestStorageRouter_RouteMetadataQuery_NoBackends(t *testing.T) {
	t.Parallel()

	router := NewStorageRouter(nil, nil)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	require.NoError(t, err)
	assert.Nil(t, notes)
}

// TestStorageRouter_RouteMetadataQuery_OnlyBolt verifies routing with
// only BoltDB.
func TestStorageRouter_RouteMetadataQuery_OnlyBolt(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewStorageRouter(boltPort, nil)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	require.NoError(t, err)
	require.Len(t, notes, 1)
	assert.Equal(t, expectedNote.Path, notes[0].Path)
}

// TestStorageRouter_RouteMetadataQuery_OnlySQLite verifies routing with
// only SQLite.
func TestStorageRouter_RouteMetadataQuery_OnlySQLite(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	sqlitePort := &mockQueryBackend{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewStorageRouter(nil, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	require.NoError(t, err)
	require.Len(t, notes, 1)
	assert.Equal(t, expectedNote.Path, notes[0].Path)
}

// TestStorageRouter_Read verifies single-note read with fallback.
func TestStorageRouter_Read(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{Path: "test.md"}

	t.Run("HotPathSuccess", func(t *testing.T) {
		bolt := &mockQueryBackend{
			readFn: func(_ context.Context, _ string) (domain.Note, error) {
				return expectedNote, nil
			},
		}
		router := NewStorageRouter(bolt, nil)
		note, err := router.Read(context.Background(), "test.md")
		require.NoError(t, err)
		assert.Equal(t, expectedNote.Path, note.Path)
	})

	t.Run("DeepPathFallback", func(t *testing.T) {
		bolt := &mockQueryBackend{
			readFn: func(_ context.Context, _ string) (domain.Note, error) {
				return domain.Note{}, errors.New("bolt error")
			},
		}
		sqlite := &mockQueryBackend{
			readFn: func(_ context.Context, _ string) (domain.Note, error) {
				return expectedNote, nil
			},
		}
		router := NewStorageRouter(bolt, sqlite)
		note, err := router.Read(context.Background(), "test.md")
		require.NoError(t, err)
		assert.Equal(t, expectedNote.Path, note.Path)
	})
}

// TestStorageRouter_GetSQLiteQuery verifies SQLite query port getter.
func TestStorageRouter_GetSQLiteQuery(t *testing.T) {
	t.Parallel()

	sqlitePort := &mockQueryBackend{}
	router := NewStorageRouter(nil, sqlitePort)

	result := router.GetSQLiteQuery()

	assert.Equal(t, sqlitePort, result)
}

// TestStorageRouter_GetBoltQuery verifies BoltDB query port getter.
func TestStorageRouter_GetBoltQuery(t *testing.T) {
	t.Parallel()

	boltPort := &mockQueryBackend{}
	router := NewStorageRouter(boltPort, nil)

	result := router.GetBoltQuery()

	assert.Equal(t, boltPort, result)
}

// TestStorageRouter_HasHotPath verifies hot path availability check.
func TestStorageRouter_HasHotPath(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name     string
		boltPort QueryBackend
		want     bool
	}{
		{
			name:     "with bolt port",
			boltPort: &mockQueryBackend{},
			want:     true,
		},
		{
			name:     "without bolt port",
			boltPort: nil,
			want:     false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := NewStorageRouter(tt.boltPort, nil)
			assert.Equal(t, tt.want, router.HasHotPath())
		})
	}
}

// TestStorageRouter_HasDeepPath verifies deep path availability check.
func TestStorageRouter_HasDeepPath(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name       string
		sqlitePort QueryBackend
		want       bool
	}{
		{
			name:       "with sqlite port",
			sqlitePort: &mockQueryBackend{},
			want:       true,
		},
		{
			name:       "without sqlite port",
			sqlitePort: nil,
			want:       false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := NewStorageRouter(nil, tt.sqlitePort)
			assert.Equal(t, tt.want, router.HasDeepPath())
		})
	}
}
