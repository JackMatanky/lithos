package query_test

import (
	"context"
	"testing"

	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- Manual Mocks ---

type MockHybridReader struct {
	// CacheReaderPort methods
	ReadFunc func(ctx context.Context, path string) (domain.Note, error)
	ListFunc func(ctx context.Context) ([]domain.Note, error)

	// MetadataQueryPort methods
	BasenameQueryFunc    func(ctx context.Context, basename string) ([]domain.Note, error)
	AliasQueryFunc       func(ctx context.Context, alias string) ([]domain.Note, error)
	FileClassQueryFunc   func(ctx context.Context, fileClass string) ([]domain.Note, error)
	PathQueryFunc        func(ctx context.Context, opts spi.PathQueryOptions) ([]domain.Note, error)
	TagQueryFunc         func(ctx context.Context, tag string) ([]domain.Note, error)
	FrontmatterQueryFunc func(ctx context.Context, field, value string) ([]domain.Note, error)
}

func (m *MockHybridReader) Read(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	if m.ReadFunc != nil {
		return m.ReadFunc(ctx, path)
	}
	return domain.Note{}, nil
}

func (m *MockHybridReader) List(ctx context.Context) ([]domain.Note, error) {
	if m.ListFunc != nil {
		return m.ListFunc(ctx)
	}
	return nil, nil
}

func (m *MockHybridReader) BasenameQuery(
	ctx context.Context,
	basename string,
) ([]domain.Note, error) {
	if m.BasenameQueryFunc != nil {
		return m.BasenameQueryFunc(ctx, basename)
	}
	return nil, nil
}

func (m *MockHybridReader) AliasQuery(
	ctx context.Context,
	alias string,
) ([]domain.Note, error) {
	if m.AliasQueryFunc != nil {
		return m.AliasQueryFunc(ctx, alias)
	}
	return nil, nil
}

func (m *MockHybridReader) FileClassQuery(
	ctx context.Context,
	fileClass string,
) ([]domain.Note, error) {
	if m.FileClassQueryFunc != nil {
		return m.FileClassQueryFunc(ctx, fileClass)
	}
	return nil, nil
}

func (m *MockHybridReader) PathQuery(
	ctx context.Context,
	opts spi.PathQueryOptions,
) ([]domain.Note, error) {
	if m.PathQueryFunc != nil {
		return m.PathQueryFunc(ctx, opts)
	}
	return nil, nil
}

func (m *MockHybridReader) TagQuery(
	ctx context.Context,
	tag string,
) ([]domain.Note, error) {
	if m.TagQueryFunc != nil {
		return m.TagQueryFunc(ctx, tag)
	}
	return nil, nil
}

func (m *MockHybridReader) FrontmatterQuery(
	ctx context.Context,
	field, value string,
) ([]domain.Note, error) {
	if m.FrontmatterQueryFunc != nil {
		return m.FrontmatterQueryFunc(ctx, field, value)
	}
	return nil, nil
}

// --- Tests ---

func TestQueryService_Constructor_DualReaders(t *testing.T) {
	boltReader := &MockHybridReader{}
	sqliteReader := &MockHybridReader{}
	logger := zerolog.New(nil)
	config := domain.Config{}

	qs := query.NewQueryService(boltReader, sqliteReader, config, logger, nil)
	require.NotNil(t, qs)
}

func TestQueryService_Routing_HotPath_PathQuery(t *testing.T) {
	boltReader := &MockHybridReader{}
	sqliteReader := &MockHybridReader{}
	logger := zerolog.New(nil)
	config := domain.Config{}

	path := "notes/test.md"
	note, _ := domain.NewNote(
		path,
		domain.NewFrontmatter(map[string]interface{}{}),
		[]domain.Link{},
		[]domain.Heading{},
		[]string{},
		[]domain.TaskItem{},
	)

	// Expectation: PathQuery calls boltReader.Read
	calledBolt := false
	boltReader.ReadFunc = func(ctx context.Context, p string) (domain.Note, error) {
		calledBolt = true
		assert.Equal(t, path, p)
		return note, nil
	}

	// SQLite should NOT be called
	sqliteReader.ReadFunc = func(ctx context.Context, p string) (domain.Note, error) {
		assert.Fail(t, "SQLite should not be called for PathQuery")
		return domain.Note{}, nil
	}

	qs := query.NewQueryService(boltReader, sqliteReader, config, logger, nil)
	result, err := qs.PathQuerySingle(context.Background(), path)

	require.NoError(t, err)
	assert.Equal(t, note, result)
	assert.True(t, calledBolt, "BoltDB should have been called")
}

func TestQueryService_Routing_HotPath_BasenameQuery(t *testing.T) {
	boltReader := &MockHybridReader{}
	sqliteReader := &MockHybridReader{}
	logger := zerolog.New(nil)
	config := domain.Config{}

	basename := "test"
	notes := []domain.Note{func() domain.Note {
		n, _ := domain.NewNote(
			"notes/test.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		return n
	}()}

	// Expectation: BasenameQuery calls boltReader.BasenameQuery
	calledBolt := false
	boltReader.BasenameQueryFunc = func(ctx context.Context, bn string) ([]domain.Note, error) {
		calledBolt = true
		assert.Equal(t, basename, bn)
		return notes, nil
	}

	sqliteReader.BasenameQueryFunc = func(ctx context.Context, bn string) ([]domain.Note, error) {
		assert.Fail(t, "SQLite should not be called for BasenameQuery")
		return nil, nil
	}

	qs := query.NewQueryService(boltReader, sqliteReader, config, logger, nil)
	result, err := qs.BasenameQuery(context.Background(), basename)

	require.NoError(t, err)
	assert.Equal(t, notes, result)
	assert.True(t, calledBolt, "BoltDB should have been called")
}

func TestQueryService_Routing_HotPath_AliasQuery(t *testing.T) {
	boltReader := &MockHybridReader{}
	sqliteReader := &MockHybridReader{}
	logger := zerolog.New(nil)
	config := domain.Config{}

	alias := "my-alias"
	notes := []domain.Note{func() domain.Note {
		n, _ := domain.NewNote(
			"notes/alias.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		return n
	}()}

	calledBolt := false
	boltReader.AliasQueryFunc = func(ctx context.Context, a string) ([]domain.Note, error) {
		calledBolt = true
		assert.Equal(t, alias, a)
		return notes, nil
	}

	qs := query.NewQueryService(boltReader, sqliteReader, config, logger, nil)
	result, err := qs.AliasQuery(context.Background(), alias)

	require.NoError(t, err)
	assert.Equal(t, notes, result)
	assert.True(t, calledBolt, "BoltDB should have been called")
}

func TestQueryService_Routing_DeepPath_FrontmatterQuery(t *testing.T) {
	boltReader := &MockHybridReader{}
	sqliteReader := &MockHybridReader{}
	logger := zerolog.New(nil)
	config := domain.Config{}

	field := "status"
	value := "active"
	notes := []domain.Note{func() domain.Note {
		n, _ := domain.NewNote(
			"notes/active.md",
			domain.NewFrontmatter(map[string]interface{}{}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		return n
	}()}

	// Expectation: FrontmatterQuery calls sqliteReader.FrontmatterQuery
	calledSqlite := false
	sqliteReader.FrontmatterQueryFunc = func(ctx context.Context, f, v string) ([]domain.Note, error) {
		calledSqlite = true
		assert.Equal(t, field, f)
		assert.Equal(t, value, v)
		return notes, nil
	}

	boltReader.FrontmatterQueryFunc = func(ctx context.Context, f, v string) ([]domain.Note, error) {
		assert.Fail(t, "BoltDB should not be called for FrontmatterQuery")
		return nil, nil
	}

	qs := query.NewQueryService(boltReader, sqliteReader, config, logger, nil)
	result, err := qs.FrontmatterQuery(context.Background(), field, value)

	require.NoError(t, err)
	assert.Equal(t, notes, result)
	assert.True(t, calledSqlite, "SQLite should have been called")
}

func TestQueryService_Consistency_Validation(t *testing.T) {
	// Placeholder for consistency validation tests
}

func TestQueryService_Staleness_Check(t *testing.T) {
	// Placeholder for staleness check tests
}
