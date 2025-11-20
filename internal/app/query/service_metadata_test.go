package query

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type noopCacheReader struct{}
type mockReader struct {
	notes []domain.Note
}
type metadataReaderAdapter struct {
	noopCacheReader
	spi.MetadataQueryPort
}

func (noopCacheReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	return domain.Note{}, fmt.Errorf("not implemented")
}

func (noopCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	return nil, nil
}

func (m *mockReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	for _, n := range m.notes {
		if n.ID == id {
			return n, nil
		}
	}
	return domain.Note{}, fmt.Errorf("not found")
}

func (m *mockReader) List(ctx context.Context) ([]domain.Note, error) {
	return m.notes, nil
}

func newTestQueryService(
	t *testing.T,
	metadata spi.MetadataQueryPort,
	notes []domain.Note,
) *QueryService {
	t.Helper()
	logger := zerolog.New(zerolog.NewTestWriter(t))

	var reader spi.CacheReaderPort
	if notes != nil {
		reader = &mockReader{notes: notes}
	} else {
		reader = noopCacheReader{}
	}

	var boltReader = reader
	if metadata != nil {
		boltReader = &metadataReaderAdapter{
			noopCacheReader:   noopCacheReader{},
			MetadataQueryPort: metadata,
		}
	}

	return NewQueryService(
		boltReader,
		reader, // sqliteReader
		domain.DefaultConfig(),
		logger,
	)
}

func TestQueryService_ByFileClassFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	meetingNote := domain.Note{
		ID: domain.NoteID("notes/meeting.md"),
		Frontmatter: domain.Frontmatter{
			FileClass: "meeting",
			Fields: map[string]interface{}{
				"file_class": "meeting",
			},
		},
	}

	qs := newTestQueryService(t, nil, []domain.Note{meetingNote})

	results, err := qs.ByFileClass(ctx, "meeting")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_ByAliasFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	note := domain.Note{
		ID: domain.NoteID("notes/project.md"),
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{
				"aliases": []interface{}{"project-alpha", "alpha"},
			},
		},
	}
	qs := newTestQueryService(t, nil, []domain.Note{note})

	results, err := qs.ByAlias(ctx, "project-alpha")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_ByBasenameFallsBackToPathQuery(t *testing.T) {
	ctx := context.Background()
	note := domain.Note{ID: domain.NoteID("notes/project.md")}
	qs := newTestQueryService(t, nil, []domain.Note{note})

	results, err := qs.ByBasename(ctx, "project")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_PathQueryFolderFallback(t *testing.T) {
	ctx := context.Background()
	notes := []domain.Note{
		{ID: domain.NoteID("projects/alpha/note.md")},
		{ID: domain.NoteID("projects/beta/note.md")},
	}
	qs := newTestQueryService(t, nil, notes)

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFolder,
		Value: "projects/alpha/",
	})
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_UsesMetadataPortWhenConfigured(t *testing.T) {
	ctx := context.Background()
	mock := spi.NewMockMetadataQueryPort()
	expected := []domain.Note{{ID: domain.NoteID("foo.md")}}
	mock.SetPathQueryResult(expected, nil)

	qs := newTestQueryService(t, mock, nil)

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFull,
		Value: "foo.md",
	})
	require.NoError(t, err)
	assert.Equal(t, expected, results)
	assert.Equal(t, 1, mock.PathQueryCallCount)
}
