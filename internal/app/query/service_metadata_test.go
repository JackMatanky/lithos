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

func (noopCacheReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	return domain.Note{}, fmt.Errorf("not implemented")
}

func (noopCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	return nil, nil
}

func newTestQueryService(
	t *testing.T,
	metadata spi.MetadataQueryPort,
) *QueryService {
	t.Helper()
	logger := zerolog.New(zerolog.NewTestWriter(t))
	reader := noopCacheReader{}
	return NewQueryService(
		metadata,
		reader,
		reader,
		domain.DefaultConfig(),
		logger,
	)
}

func TestQueryService_ByFileClassFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	qs := newTestQueryService(t, nil)

	meetingNote := domain.Note{
		ID: domain.NoteID("notes/meeting.md"),
		Frontmatter: domain.Frontmatter{
			FileClass: "meeting",
			Fields: map[string]interface{}{
				"file_class": "meeting",
			},
		},
	}

	qs.byFileClass["meeting"] = []domain.Note{meetingNote}

	results, err := qs.ByFileClass(ctx, "meeting")
	require.NoError(t, err)
	assert.Equal(t, []domain.Note{meetingNote}, results)
}

func TestQueryService_ByAliasFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	qs := newTestQueryService(t, nil)

	note := domain.Note{
		ID: domain.NoteID("notes/project.md"),
		Frontmatter: domain.Frontmatter{
			Fields: map[string]interface{}{
				"aliases": []interface{}{"project-alpha", "alpha"},
			},
		},
	}

	qs.byPath = map[string]domain.Note{
		"notes/project.md": note,
	}

	results, err := qs.ByAlias(ctx, "project-alpha")
	require.NoError(t, err)
	assert.Equal(t, []domain.Note{note}, results)
}

func TestQueryService_ByBasenameFallsBackToPathQuery(t *testing.T) {
	ctx := context.Background()
	qs := newTestQueryService(t, nil)

	note := domain.Note{ID: domain.NoteID("notes/project.md")}
	qs.byBasename["project"] = []domain.Note{note}

	results, err := qs.ByBasename(ctx, "project")
	require.NoError(t, err)
	assert.Equal(t, []domain.Note{note}, results)
}

func TestQueryService_PathQueryFolderFallback(t *testing.T) {
	ctx := context.Background()
	qs := newTestQueryService(t, nil)

	qs.byPath = map[string]domain.Note{
		"projects/alpha/note.md": {ID: domain.NoteID("projects/alpha/note.md")},
		"projects/beta/note.md":  {ID: domain.NoteID("projects/beta/note.md")},
	}

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFolder,
		Value: "projects/alpha/",
	})
	require.NoError(t, err)
	require.Len(t, results, 1)
	assert.Equal(t, domain.NoteID("projects/alpha/note.md"), results[0].ID)
}

func TestQueryService_UsesMetadataPortWhenConfigured(t *testing.T) {
	ctx := context.Background()
	mock := spi.NewMockMetadataQueryPort()
	expected := []domain.Note{{ID: domain.NoteID("foo.md")}}
	mock.SetPathQueryResult(expected, nil)

	qs := newTestQueryService(t, mock)

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFull,
		Value: "foo.md",
	})
	require.NoError(t, err)
	assert.Equal(t, expected, results)
	assert.Equal(t, 1, mock.PathQueryCallCount)
}
