package performance

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/boltdb"
	"github.com/JackMatanky/lithos/internal/domain"
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
		if persistErr := writer.Persist(ctx, note, time.Now()); persistErr != nil {
			b.Fatal(persistErr)
		}
	}

	b.Run("Read", func(b *testing.B) {
		reader, readErr := boltdb.NewBoltDBCacheReadAdapter(config, log, db)
		if readErr != nil {
			b.Fatal(readErr)
		}

		ctx := context.Background()
		id := domain.NewNoteID("bench/note.md")
		b.ResetTimer()
		for range b.N {
			_, err := reader.Read(ctx, id)
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
			if persistErr := writer.Persist(ctx, note, time.Now()); persistErr != nil {
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

			if persistErr := writer.Persist(ctx, n, time.Now()); persistErr != nil {
				b.Fatal(persistErr)
			}
		}
	})
}
