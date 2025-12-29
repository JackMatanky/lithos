package query

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/JackMatanky/lithos/tests/utils"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type noopCacheReader struct{}
type mockReader struct {
	notes []domain.Note
}

func (noopCacheReader) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	return domain.Note{}, fmt.Errorf("not implemented")
}

func (noopCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	return nil, nil
}

func (m *mockReader) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	for i := range m.notes {
		if m.notes[i].Path == path {
			return m.notes[i], nil
		}
	}
	return domain.Note{}, fmt.Errorf("not found")
}

func (m *mockReader) List(ctx context.Context) ([]domain.Note, error) {
	return m.notes, nil
}

func newTestQueryService(
	t *testing.T,
	boltReader spi.CacheReaderPort,
	sqliteReader spi.CacheReaderPort,
) *QueryService {
	t.Helper()
	logger := zerolog.New(zerolog.NewTestWriter(t))

	return NewQueryService(
		boltReader,
		sqliteReader,
		domain.DefaultConfig(),
		logger,
		nil,
	)
}

func TestQueryService_FileClassQueryFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	meetingNote, _ := domain.NewNote(
		"notes/meeting.md",
		domain.NewFrontmatter(map[string]interface{}{
			"file_class": "meeting",
		}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	reader := &mockReader{notes: []domain.Note{meetingNote}}
	qs := newTestQueryService(t, reader, reader)

	results, err := qs.FileClassQuery(ctx, "meeting")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_AliasQueryFallbackWithoutMetadata(t *testing.T) {
	ctx := context.Background()
	note, _ := domain.NewNote(
		"notes/project.md",
		domain.NewFrontmatter(map[string]interface{}{
			"aliases": []interface{}{"project-alpha", "alpha"},
		}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	reader := &mockReader{notes: []domain.Note{note}}
	qs := newTestQueryService(t, reader, reader)

	results, err := qs.AliasQuery(ctx, "project-alpha")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_BasenameQueryFallsBackToPathQuery(t *testing.T) {
	ctx := context.Background()
	note, _ := domain.NewNote(
		"notes/project.md",
		domain.NewFrontmatter(map[string]interface{}{}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)
	reader := &mockReader{notes: []domain.Note{note}}
	qs := newTestQueryService(t, reader, reader)

	results, err := qs.BasenameQuery(ctx, "project")
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_PathQueryFolderFallback(t *testing.T) {
	ctx := context.Background()
	notes := []domain.Note{
		func() domain.Note {
			n, _ := domain.NewNote(
				"projects/alpha/note.md",
				domain.NewFrontmatter(map[string]interface{}{}),
				[]domain.Link{},
				[]domain.Heading{},
				[]string{},
				[]domain.TaskItem{},
			)
			return n
		}(),
		func() domain.Note {
			n, _ := domain.NewNote(
				"projects/beta/note.md",
				domain.NewFrontmatter(map[string]interface{}{}),
				[]domain.Link{},
				[]domain.Heading{},
				[]string{},
				[]domain.TaskItem{},
			)
			return n
		}(),
	}
	reader := &mockReader{notes: notes}
	qs := newTestQueryService(t, reader, reader)

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFolder,
		Value: "projects/alpha/",
	})
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestQueryService_FrontmatterQuery_TypeNormalization(t *testing.T) {
	ctx := context.Background()
	mock := utils.NewMockMetadataQueryPort()
	expected := []domain.Note{func() domain.Note {
		n, _ := domain.NewNote(
			"foo.md",
			domain.NewFrontmatter(map[string]interface{}{"priority": 2}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		return n
	}()}
	mock.SetFrontmatterQueryResult(expected, nil)

	qs := newTestQueryService(t, noopCacheReader{}, mock) // bolt, sqlite

	// Test with int
	results, err := qs.FrontmatterQuery(ctx, "priority", 2)
	require.NoError(t, err)
	assert.Equal(t, expected, results)
	assert.Equal(t, 1, mock.FrontmatterQueryCallCount)

	mock.Reset()
	mock.SetFrontmatterQueryResult(expected, nil)

	// Test with float
	results, err = qs.FrontmatterQuery(ctx, "priority", 2.0)
	require.NoError(t, err)
	assert.Equal(t, expected, results)
	assert.Equal(t, 1, mock.FrontmatterQueryCallCount)
}

func TestQueryService_UsesMetadataPortWhenConfigured(t *testing.T) {
	ctx := context.Background()
	mock := utils.NewMockMetadataQueryPort()
	expected := []domain.Note{func() domain.Note {
		n, _ := domain.NewNote(
			"foo.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		return n
	}()}
	mock.SetPathQueryResult(expected, nil)

	qs := newTestQueryService(t, mock, noopCacheReader{})

	results, err := qs.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFull,
		Value: "foo.md",
	})
	require.NoError(t, err)
	assert.Equal(t, expected, results)
	assert.Equal(t, 1, mock.PathQueryCallCount)
}
