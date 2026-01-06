package persistence

import (
	"context"
	"errors"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"
)

type mockWriteStrategy struct {
	mock.Mock
}

type mockCacheWriter struct {
	mock.Mock
}

func (m *mockWriteStrategy) Execute(
	ctx context.Context,
	ops []PersistenceOperation,
	writers []spi.CacheWriterPort,
) error {
	args := m.Called(ctx, ops, writers)
	return args.Error(0)
}

func (m *mockWriteStrategy) Name() string {
	return "Mock"
}

func (m *mockWriteStrategy) Describe() string {
	return "Mock Strategy"
}

func (m *mockCacheWriter) Persist(
	ctx context.Context,
	note domain.Note,
	meta spi.CacheWriteMetadata,
) error {
	args := m.Called(ctx, note, meta)
	return args.Error(0)
}

func (m *mockCacheWriter) Delete(ctx context.Context, path string) error {
	args := m.Called(ctx, path)
	return args.Error(0)
}

func TestCacheTransaction(t *testing.T) {
	strategy := new(mockWriteStrategy)
	writer := new(mockCacheWriter)
	tx := NewCacheTransaction(strategy, writer)

	t.Run("AddWrite", func(t *testing.T) {
		note := domain.Note{Path: "test.md"}
		meta := spi.CacheWriteMetadata{}
		tx.AddWrite(note, meta)
		assert.Len(t, tx.operations, 1)
		assert.Equal(t, OpWrite, tx.operations[0].Type())
	})

	t.Run("AddDelete", func(t *testing.T) {
		tx.AddDelete("delete.md")
		assert.Len(t, tx.operations, 2)
		assert.Equal(t, OpDelete, tx.operations[1].Type())
	})

	t.Run("CommitSuccess", func(t *testing.T) {
		strategy.On("Execute", mock.Anything, tx.operations, tx.writers).
			Return(nil).
			Once()
		err := tx.Commit(context.Background())
		require.NoError(t, err)
		assert.Empty(t, tx.operations)
		strategy.AssertExpectations(t)
	})

	t.Run("CommitEmpty", func(t *testing.T) {
		err := tx.Commit(context.Background())
		require.NoError(t, err)
	})

	t.Run("CommitFailure", func(t *testing.T) {
		tx.AddDelete("fail.md")
		testErr := errors.New("strategy failed")
		strategy.On("Execute", mock.Anything, tx.operations, tx.writers).
			Return(testErr).
			Once()
		err := tx.Commit(context.Background())
		require.ErrorIs(t, err, testErr)
		strategy.AssertExpectations(t)
	})

	t.Run("Rollback", func(t *testing.T) {
		tx.AddWrite(domain.Note{Path: "rollback.md"}, spi.CacheWriteMetadata{})
		assert.NotEmpty(t, tx.operations)
		tx.Rollback()
		assert.Empty(t, tx.operations)
	})
}
