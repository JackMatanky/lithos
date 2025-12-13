package query

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

// mockMetadataQueryPort implements spi.MetadataQueryPort for testing.
type mockMetadataQueryPort struct {
	fileClassQueryFn   func(ctx context.Context, fileClass string) ([]domain.Note, error)
	basenameQueryFn    func(ctx context.Context, basename string) ([]domain.Note, error)
	aliasQueryFn       func(ctx context.Context, alias string) ([]domain.Note, error)
	pathQueryFn        func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error)
	frontmatterQueryFn func(ctx context.Context, field, value string) ([]domain.Note, error)
	tagQueryFn         func(ctx context.Context, tag string) ([]domain.Note, error)
}

func (m *mockMetadataQueryPort) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	if m.fileClassQueryFn != nil {
		return m.fileClassQueryFn(ctx, fileClass)
	}
	return nil, nil
}

func (m *mockMetadataQueryPort) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	if m.basenameQueryFn != nil {
		return m.basenameQueryFn(ctx, basename)
	}
	return nil, nil
}

func (m *mockMetadataQueryPort) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	if m.aliasQueryFn != nil {
		return m.aliasQueryFn(ctx, alias)
	}
	return nil, nil
}

func (m *mockMetadataQueryPort) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	if m.pathQueryFn != nil {
		return m.pathQueryFn(ctx, opts)
	}
	return nil, nil
}

func (m *mockMetadataQueryPort) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	if m.frontmatterQueryFn != nil {
		return m.frontmatterQueryFn(ctx, field, value)
	}
	return nil, nil
}

func (m *mockMetadataQueryPort) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	if m.tagQueryFn != nil {
		return m.tagQueryFn(ctx, tag)
	}
	return nil, nil
}

// TestHybridStorageRouter_NewHybridStorageRouter verifies router construction.
func TestHybridStorageRouter_NewHybridStorageRouter(t *testing.T) {
	t.Parallel()

	boltPort := &mockMetadataQueryPort{}
	sqlitePort := &mockMetadataQueryPort{}

	router := NewHybridStorageRouter(boltPort, sqlitePort)

	if router == nil {
		t.Fatal("Expected router to be created")
	}
	if router.boltQuery != boltPort {
		t.Error("Expected boltQuery to be set")
	}
	if router.sqliteQuery != sqlitePort {
		t.Error("Expected sqliteQuery to be set")
	}
}

// TestHybridStorageRouter_RouteMetadataQuery_HotPath verifies hot path routing.
func TestHybridStorageRouter_RouteMetadataQuery_HotPath(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}
	sqlitePort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			t.Error("SQLite should not be called when BoltDB succeeds")
			return nil, nil
		},
	}

	router := NewHybridStorageRouter(boltPort, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	if err != nil {
		t.Fatalf("Expected no error, got: %v", err)
	}
	if len(notes) != 1 {
		t.Fatalf("Expected 1 note, got %d", len(notes))
	}
	if notes[0].Path != expectedNote.Path {
		t.Errorf(
			"Expected note ID %s, got %s",
			expectedNote.Path,
			notes[0].Path,
		)
	}
}

// TestHybridStorageRouter_RouteMetadataQuery_DeepPathFallback verifies fallback
// to SQLite.
func TestHybridStorageRouter_RouteMetadataQuery_DeepPathFallback(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return nil, errors.New("bolt error")
		},
	}
	sqlitePort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewHybridStorageRouter(boltPort, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	if err != nil {
		t.Fatalf("Expected no error, got: %v", err)
	}
	if len(notes) != 1 {
		t.Fatalf("Expected 1 note, got %d", len(notes))
	}
	if notes[0].Path != expectedNote.Path {
		t.Errorf(
			"Expected note ID %s, got %s",
			expectedNote.Path,
			notes[0].Path,
		)
	}
}

// TestHybridStorageRouter_RouteMetadataQuery_NoBackends verifies behavior with
// no backends.
func TestHybridStorageRouter_RouteMetadataQuery_NoBackends(t *testing.T) {
	t.Parallel()

	router := NewHybridStorageRouter(nil, nil)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	if err != nil {
		t.Errorf("Expected no error, got: %v", err)
	}
	if notes != nil {
		t.Errorf("Expected nil notes, got %v", notes)
	}
}

// TestHybridStorageRouter_RouteMetadataQuery_OnlyBolt verifies routing with
// only BoltDB.
func TestHybridStorageRouter_RouteMetadataQuery_OnlyBolt(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	boltPort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewHybridStorageRouter(boltPort, nil)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	if err != nil {
		t.Fatalf("Expected no error, got: %v", err)
	}
	if len(notes) != 1 {
		t.Fatalf("Expected 1 note, got %d", len(notes))
	}
	if notes[0].Path != expectedNote.Path {
		t.Errorf(
			"Expected note ID %s, got %s",
			expectedNote.Path,
			notes[0].Path,
		)
	}
}

// TestHybridStorageRouter_RouteMetadataQuery_OnlySQLite verifies routing with
// only SQLite.
func TestHybridStorageRouter_RouteMetadataQuery_OnlySQLite(t *testing.T) {
	t.Parallel()

	expectedNote := domain.Note{
		Path: "test.md",
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{"title": "Test"},
		},
	}

	sqlitePort := &mockMetadataQueryPort{
		basenameQueryFn: func(_ context.Context, _ string) ([]domain.Note, error) {
			return []domain.Note{expectedNote}, nil
		},
	}

	router := NewHybridStorageRouter(nil, sqlitePort)

	notes, err := router.RouteMetadataQuery(
		context.Background(),
		func(port spi.MetadataQueryPort, ctx context.Context, param string) ([]domain.Note, error) {
			return port.BasenameQuery(ctx, param)
		},
		"test",
	)

	if err != nil {
		t.Fatalf("Expected no error, got: %v", err)
	}
	if len(notes) != 1 {
		t.Fatalf("Expected 1 note, got %d", len(notes))
	}
	if notes[0].Path != expectedNote.Path {
		t.Errorf(
			"Expected note ID %s, got %s",
			expectedNote.Path,
			notes[0].Path,
		)
	}
}

// TestHybridStorageRouter_GetSQLiteQuery verifies SQLite query port getter.
func TestHybridStorageRouter_GetSQLiteQuery(t *testing.T) {
	t.Parallel()

	sqlitePort := &mockMetadataQueryPort{}
	router := NewHybridStorageRouter(nil, sqlitePort)

	result := router.GetSQLiteQuery()

	if result != sqlitePort {
		t.Error("Expected GetSQLiteQuery to return sqlitePort")
	}
}

// TestHybridStorageRouter_GetBoltQuery verifies BoltDB query port getter.
func TestHybridStorageRouter_GetBoltQuery(t *testing.T) {
	t.Parallel()

	boltPort := &mockMetadataQueryPort{}
	router := NewHybridStorageRouter(boltPort, nil)

	result := router.GetBoltQuery()

	if result != boltPort {
		t.Error("Expected GetBoltQuery to return boltPort")
	}
}

// TestHybridStorageRouter_HasHotPath verifies hot path availability check.
func TestHybridStorageRouter_HasHotPath(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name     string
		boltPort spi.MetadataQueryPort
		want     bool
	}{
		{
			name:     "with bolt port",
			boltPort: &mockMetadataQueryPort{},
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
			t.Parallel()
			router := NewHybridStorageRouter(tt.boltPort, nil)
			if got := router.HasHotPath(); got != tt.want {
				t.Errorf("HasHotPath() = %v, want %v", got, tt.want)
			}
		})
	}
}

// TestHybridStorageRouter_HasDeepPath verifies deep path availability check.
func TestHybridStorageRouter_HasDeepPath(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name       string
		sqlitePort spi.MetadataQueryPort
		want       bool
	}{
		{
			name:       "with sqlite port",
			sqlitePort: &mockMetadataQueryPort{},
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
			t.Parallel()
			router := NewHybridStorageRouter(nil, tt.sqlitePort)
			if got := router.HasDeepPath(); got != tt.want {
				t.Errorf("HasDeepPath() = %v, want %v", got, tt.want)
			}
		})
	}
}
