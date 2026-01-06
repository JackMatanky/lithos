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

func TestSequentialWriteStrategy(t *testing.T) {
	strategy := &SequentialWriteStrategy{}
	writer1 := new(mockCacheWriter)
	writer2 := new(mockCacheWriter)
	writers := []spi.CacheWriterPort{writer1, writer2}

	note := domain.Note{Path: "test.md"}
	meta := spi.CacheWriteMetadata{}
	ops := []PersistenceOperation{WriteOperation{Note: note, Metadata: meta}}

	t.Run("Success", func(t *testing.T) {
		writer1.On("Persist", mock.Anything, note, meta).Return(nil).Once()
		writer2.On("Persist", mock.Anything, note, meta).Return(nil).Once()

		err := strategy.Execute(context.Background(), ops, writers)
		require.NoError(t, err)
		writer1.AssertExpectations(t)
		writer2.AssertExpectations(t)
	})

	t.Run("FailureAtWriter1", func(t *testing.T) {
		testErr := errors.New("fail 1")
		writer1.On("Persist", mock.Anything, note, meta).Return(testErr).Once()

		err := strategy.Execute(context.Background(), ops, writers)
		require.ErrorIs(t, err, testErr)
		writer1.AssertExpectations(t)
	})
}

func TestParallelWriteStrategy(t *testing.T) {
	strategy := &ParallelWriteStrategy{}
	writer1 := new(mockCacheWriter)
	writer2 := new(mockCacheWriter)
	writers := []spi.CacheWriterPort{writer1, writer2}

	note := domain.Note{Path: "test.md"}
	meta := spi.CacheWriteMetadata{}
	ops := []PersistenceOperation{WriteOperation{Note: note, Metadata: meta}}

	t.Run("Success", func(t *testing.T) {
		writer1.On("Persist", mock.Anything, note, meta).Return(nil).Once()
		writer2.On("Persist", mock.Anything, note, meta).Return(nil).Once()

		err := strategy.Execute(context.Background(), ops, writers)
		require.NoError(t, err)
		writer1.AssertExpectations(t)
		writer2.AssertExpectations(t)
	})

	t.Run("FailureWithRollback", func(t *testing.T) {
		testErr := errors.New("fail 2")
		writer1.On("Persist", mock.Anything, note, meta).Return(nil).Once()
		writer1.On("Delete", mock.Anything, note.Path).
			Return(nil).
			Once()
			// Rollback
		writer2.On("Persist", mock.Anything, note, meta).Return(testErr).Once()

		err := strategy.Execute(context.Background(), ops, writers)
		require.Error(t, err)
		assert.Contains(t, err.Error(), testErr.Error())
		writer1.AssertExpectations(t)
		writer2.AssertExpectations(t)
	})
}
