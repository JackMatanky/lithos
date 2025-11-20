package performance

import (
	"context"
	"fmt"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/rs/zerolog"
)

func BenchmarkBoltDBCache(b *testing.B) {
	cacheDir := b.TempDir()
	config := domain.Config{CacheDir: cacheDir, FileClassKey: "fileClass"}
	log := zerolog.Nop()

	// Pre-populate data
	{
		writer, err := boltdb.NewBoltDBCacheWriter(config, log)
		if err != nil {
			b.Fatal(err)
		}
		ctx := context.Background()
		note := domain.Note{
			ID: domain.NewNoteID("bench/note.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "bench",
				Fields: map[string]interface{}{
					"title":     "Benchmark Note",
					"fileClass": "bench",
				},
			},
		}
		if persistErr := writer.Persist(ctx, note); persistErr != nil {
			_ = writer.Close()
			b.Fatal(persistErr)
		}
		_ = writer.Close() // Close to release lock
	}

	b.Run("Read", func(b *testing.B) {
		reader, err := boltdb.NewBoltDBCacheReadAdapter(config, log)
		if err != nil {
			b.Fatal(err)
		}
		defer func() { _ = reader.Close() }()

		ctx := context.Background()
		id := domain.NewNoteID("bench/note.md")
		b.ResetTimer()
		for range b.N {
			_, readErr := reader.Read(ctx, id)
			if readErr != nil {
				b.Fatal(readErr)
			}
		}
	})

	b.Run("Write", func(b *testing.B) {
		writer, err := boltdb.NewBoltDBCacheWriter(config, log)
		if err != nil {
			b.Fatal(err)
		}
		defer func() { _ = writer.Close() }()

		ctx := context.Background()
		note := domain.Note{
			ID: domain.NewNoteID("bench/note.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "bench",
				Fields: map[string]interface{}{
					"title":     "Benchmark Note",
					"fileClass": "bench",
				},
			},
		}

		b.ResetTimer()
		for range b.N {
			if persistErr := writer.Persist(ctx, note); persistErr != nil {
				b.Fatal(persistErr)
			}
		}
	})

	b.Run("WriteUnique", func(b *testing.B) {
		writer, err := boltdb.NewBoltDBCacheWriter(config, log)
		if err != nil {
			b.Fatal(err)
		}
		defer func() { _ = writer.Close() }()

		ctx := context.Background()
		note := domain.Note{
			ID: domain.NewNoteID("bench/note.md"),
			Frontmatter: domain.Frontmatter{
				FileClass: "bench",
				Fields: map[string]interface{}{
					"title":     "Benchmark Note",
					"fileClass": "bench",
				},
			},
		}

		b.ResetTimer()
		for i := range b.N {
			b.StopTimer()
			n := note
			n.ID = domain.NewNoteID(fmt.Sprintf("bench/note_%d.md", i))
			b.StartTimer()

			if persistErr := writer.Persist(ctx, n); persistErr != nil {
				b.Fatal(persistErr)
			}
		}
	})
}
