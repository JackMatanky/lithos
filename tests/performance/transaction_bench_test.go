package performance

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/persistence"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
)

type benchCacheWriter struct {
	// Simulate some work
}

func (b *benchCacheWriter) Persist(
	ctx context.Context,
	note domain.Note,
	meta spi.CacheWriteMetadata,
) error {
	// Simulate I/O work
	time.Sleep(100 * time.Microsecond)
	return nil
}

func (b *benchCacheWriter) Delete(ctx context.Context, path string) error {
	// Simulate I/O work
	time.Sleep(100 * time.Microsecond)
	return nil
}

func BenchmarkCacheTransaction_Strategies(b *testing.B) {
	writer1 := &benchCacheWriter{}
	writer2 := &benchCacheWriter{}
	writers := []spi.CacheWriterPort{writer1, writer2}

	// Prepare operations
	const numNotes = 100
	notes := make([]domain.Note, numNotes)
	for i := range numNotes {
		notes[i] = domain.Note{Path: fmt.Sprintf("note-%d.md", i)}
	}
	meta := spi.CacheWriteMetadata{}

	ctx := context.Background()

	b.Run("Sequential", func(b *testing.B) {
		strategy := &persistence.SequentialWriter{}
		tx := persistence.NewCacheTransaction(strategy, writers...)
		b.ResetTimer()
		for range b.N {
			for i := range notes {
				tx.AddWrite(notes[i], meta)
			}
			if err := tx.Commit(ctx); err != nil {
				b.Fatal(err)
			}
		}
	})

	b.Run("Parallel", func(b *testing.B) {
		strategy := &persistence.ParallelWriter{}
		tx := persistence.NewCacheTransaction(strategy, writers...)
		b.ResetTimer()
		for range b.N {
			for i := range notes {
				tx.AddWrite(notes[i], meta)
			}
			if err := tx.Commit(ctx); err != nil {
				b.Fatal(err)
			}
		}
	})
}
