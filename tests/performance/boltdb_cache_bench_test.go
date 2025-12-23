package performance

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
)

func BenchmarkBoltDBCache(b *testing.B) {
	cacheDir := b.TempDir()
	config := domain.Config{CacheDir: cacheDir, FileClassKey: "fileClass"}
	log := zerolog.Nop()

	db, openErr := boltdb.Open(config)
	if openErr != nil {
		b.Fatal(openErr)
	}
	defer func() { _ = db.Close() }()

	// Pre-populate data
	{
		writer, writeErr := boltdb.NewBoltDBCacheWriter(config, log, db)
		if writeErr != nil {
			b.Fatal(writeErr)
		}
		ctx := context.Background()

		note, _ := domain.NewNote(
			"bench/note.md",
			domain.NewFrontmatter(map[string]interface{}{
				"title":     "Benchmark Note",
				"fileClass": "bench",
			}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)
		metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
		if persistErr := writer.Persist(ctx, note, metadata); persistErr != nil {
			b.Fatal(persistErr)
		}
	}

	b.Run("Read", func(b *testing.B) {
		reader, readErr := boltdb.NewBoltDBCacheReadAdapter(config, log, db)
		if readErr != nil {
			b.Fatal(readErr)
		}

		ctx := context.Background()
		path := "bench/note.md"
		b.ResetTimer()
		for range b.N {
			_, err := reader.Read(ctx, path)
			if err != nil {
				b.Fatal(err)
			}
		}
	})

	b.Run("Write", func(b *testing.B) {
		writer, writeErr := boltdb.NewBoltDBCacheWriter(config, log, db)
		if writeErr != nil {
			b.Fatal(writeErr)
		}

		ctx := context.Background()
		note, _ := domain.NewNote(
			"bench/note.md",
			domain.NewFrontmatter(map[string]interface{}{
				"title":     "Benchmark Note",
				"fileClass": "bench",
			}),
			[]domain.Link{},
			[]domain.Heading{},
			[]string{},
			[]domain.TaskItem{},
		)

		b.ResetTimer()
		for range b.N {
			metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
			if persistErr := writer.Persist(ctx, note, metadata); persistErr != nil {
				b.Fatal(persistErr)
			}
		}
	})

	b.Run("WriteUnique", func(b *testing.B) {
		writer, writeErr := boltdb.NewBoltDBCacheWriter(config, log, db)
		if writeErr != nil {
			b.Fatal(writeErr)
		}

		ctx := context.Background()

		b.ResetTimer()
		for i := range b.N {
			b.StopTimer()
			n, _ := domain.NewNote(
				fmt.Sprintf("bench/note_%d.md", i),
				domain.NewFrontmatter(map[string]interface{}{
					"title":     "Benchmark Note",
					"fileClass": "bench",
				}),
				[]domain.Link{},
				[]domain.Heading{},
				[]string{},
				[]domain.TaskItem{},
			)
			b.StartTimer()

			metadata := spi.CacheWriteMetadata{IndexTime: time.Now()}
			if persistErr := writer.Persist(ctx, n, metadata); persistErr != nil {
				b.Fatal(persistErr)
			}
		}
	})
}
