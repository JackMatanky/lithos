package performance

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/query"
	"github.com/JackMatanky/lithos/internal/domain"
	lithosErr "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
)

// queryServiceBench wraps a prepared QueryService instance for benchmarks.
type queryServiceBench struct {
	qs *query.QueryService
}

// benchCacheReader provides an in-memory CacheReader implementation.
type benchCacheReader struct {
	notes []domain.Note
}

// benchQueryReader simulates a query adapter with in-memory indices.
type benchQueryReader struct {
	notes            []domain.Note
	pathIndex        map[string]domain.Note
	fileClassIndex   map[string][]domain.Note
	frontmatterIndex map[string]map[interface{}][]domain.Note
}

// BenchmarkQueryService_Performance exercises mixed workloads (hot / deep /
// ID).
func BenchmarkQueryService_Performance(b *testing.B) {
	notes := generateNotes(
		100,
		[]string{"contact", "meeting", "project", "task", "idea"},
	)
	bench := newQueryServiceBench(b, notes) // complexity reduced via helpers
	ctx := context.Background()
	b.Run(
		"BoltDB_HotPath_ByFileClass",
		func(b *testing.B) { runBoltHotPath(b, bench, ctx) },
	)
	b.Run(
		"SQLite_Complex_ByFrontmatter",
		func(b *testing.B) { runSQLiteFrontmatter(b, bench, ctx) },
	)
	b.Run(
		"EndToEnd_MixedWorkload",
		func(b *testing.B) { runMixedWorkload(b, bench, ctx, notes) },
	)
}

// BenchmarkQueryService_BoltDBPerformance isolates hot path cache lookups.
func BenchmarkQueryService_BoltDBPerformance(b *testing.B) {
	notes := generateNotes(1000, nil)
	bench := newQueryServiceBench(b, notes) // isolates BoltDB hot path queries
	ctx := context.Background()
	b.Run("ByPath", func(b *testing.B) {
		b.ResetTimer()
		size := len(notes)
		for i := range b.N {
			path := fmt.Sprintf("note-%d.md", i%size)
			if _, err := bench.qs.ByPath(ctx, path); err != nil {
				b.Fatal(err)
			}
		}
		assertDuration(b, time.Millisecond)
	})
	b.Run("ByFileClass", func(b *testing.B) {
		b.ResetTimer()
		for i := range b.N {
			class := fmt.Sprintf("class-%d", i%10)
			if _, err := bench.qs.ByFileClass(ctx, class); err != nil {
				b.Fatal(err)
			}
		}
		assertDuration(b, time.Millisecond)
	})
}

// BenchmarkQueryService_SQLitePerformance isolates frontmatter filtering path.
func BenchmarkQueryService_SQLitePerformance(b *testing.B) {
	notes := generateAuthorNotes(1000)
	bench := newQueryServiceBench(b, notes) // isolates deep frontmatter queries
	ctx := context.Background()
	b.Run("ByFrontmatter", func(b *testing.B) {
		b.ResetTimer()
		for i := range b.N {
			author := fmt.Sprintf("author-%d", i%50)
			if _, err := bench.qs.ByFrontmatter(ctx, "author", author); err != nil {
				b.Fatal(err)
			}
		}
		assertDuration(b, 50*time.Millisecond)
	})
}

// BenchmarkQueryService_EndToEndTemplateRendering runs a representative
// template workflow.
func BenchmarkQueryService_EndToEndTemplateRendering(b *testing.B) {
	notes := generateTemplateNotes(500)
	bench := newQueryServiceBench(b, notes) // benchmarks a template workflow
	ctx := context.Background()
	size := len(notes)
	b.Run("TemplateWorkflow", func(b *testing.B) {
		b.ResetTimer()
		classes := []string{"contact", "project", "meeting", "task", "note"}
		clen := len(classes)
		for i := range b.N {
			if _, err := bench.qs.ByPath(ctx, fmt.Sprintf("templates/note-%d.md", i%size)); err != nil {
				b.Fatal(err)
			}
			if _, err := bench.qs.ByFileClass(ctx, classes[i%clen]); err != nil {
				b.Fatal(err)
			}
			if _, err := bench.qs.ByFrontmatter(ctx, "author", fmt.Sprintf("author-%d", i%25)); err != nil {
				b.Fatal(err)
			}
			if _, err := bench.qs.ByID(
				ctx, domain.NoteID(fmt.Sprintf("template-note-%d.md", i%size)),
			); err != nil {
				b.Fatal(err)
			}
		}
		assertDuration(b, 100*time.Millisecond)
	})
}

// runBoltHotPath benchmarks repeated file class lookups (hot path).
func runBoltHotPath(
	b *testing.B,
	bench queryServiceBench,
	ctx context.Context,
) {
	b.ResetTimer()
	for range b.N {
		if _, err := bench.qs.ByFileClass(ctx, "contact"); err != nil {
			b.Fatal(err)
		}
	}
	assertDuration(b, time.Millisecond)
}

// Read returns a note by ID from the cache reader.
func (r *benchCacheReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	for _, note := range r.notes {
		if note.ID == id {
			return note, nil
		}
	}
	return domain.Note{}, lithosErr.NewResourceError(
		"note",
		"read",
		id.String(),
		fmt.Errorf("not found"),
	)
}

// List returns all notes from the cache reader.
func (r *benchCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	return append([]domain.Note(nil), r.notes...), nil
}

// Read looks up a note by ID using the path index.
func (r *benchQueryReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	return r.getByPathString(string(id))
}

// List returns all notes known to the query reader.
func (r *benchQueryReader) List(ctx context.Context) ([]domain.Note, error) {
	return append([]domain.Note(nil), r.notes...), nil
}

// GetByPath returns a note by vault-relative path.
func (r *benchQueryReader) GetByPath(
	ctx context.Context,
	path string,
) (domain.Note, error) {
	return r.getByPathString(path)
}

// GetByFileClass returns notes belonging to a file class.
func (r *benchQueryReader) GetByFileClass(ctx context.Context, fileClass string,
	config domain.Config) ([]domain.Note, error) {
	return append([]domain.Note(nil), r.fileClassIndex[fileClass]...), nil
}

// QueryByFrontmatter returns notes matching a frontmatter key/value pair.
func (r *benchQueryReader) QueryByFrontmatter(ctx context.Context, key string,
	value interface{}) ([]domain.Note, error) {
	return append([]domain.Note(nil), r.frontmatterIndex[key][value]...), nil
}

// getByPathString performs a path lookup in the in-memory index.
func (r *benchQueryReader) getByPathString(path string) (domain.Note, error) {
	note, ok := r.pathIndex[path]
	if !ok {
		return domain.Note{}, lithosErr.ErrNotFound
	}
	return note, nil
}

// assertDuration validates average iteration duration against a max threshold.
func assertDuration(b *testing.B, maxDur time.Duration) {
	avg := b.Elapsed() / time.Duration(b.N)
	if avg > maxDur {
		b.Errorf("average duration %v exceeded target %v", avg, maxDur)
	}
}

// runSQLiteFrontmatter benchmarks repeated frontmatter lookups (deep path).
func runSQLiteFrontmatter(
	b *testing.B,
	bench queryServiceBench,
	ctx context.Context,
) {
	b.ResetTimer()
	for range b.N {
		if _, err := bench.qs.ByFrontmatter(ctx, "priority", 2); err != nil {
			b.Fatal(err)
		}
	}
	assertDuration(b, 50*time.Millisecond)
}

// runMixedWorkload benchmarks a mixed hot/deep workload plus ID lookups.
func runMixedWorkload(
	b *testing.B,
	bench queryServiceBench,
	ctx context.Context,
	notes []domain.Note,
) {
	b.ResetTimer()
	size := len(notes)
	for i := range b.N {
		if _, err := bench.qs.ByFileClass(ctx, "project"); err != nil {
			b.Fatal(err)
		}
		if _, err := bench.qs.ByFrontmatter(ctx, "priority", i%5); err != nil {
			b.Fatal(err)
		}
		if _, err := bench.qs.ByID(ctx, domain.NoteID(fmt.Sprintf("note-%d.md", i%size))); err != nil {
			b.Fatal(err)
		}
	}
	assertDuration(b, 100*time.Millisecond)
}

// newQueryServiceBench constructs a benchmark harness with in-memory adapters.
func newQueryServiceBench(b *testing.B, notes []domain.Note) queryServiceBench {
	b.Helper()
	config := domain.DefaultConfig()
	sqliteReader := &benchCacheReader{notes: notes}
	boltReader := newBenchQueryReader(notes, config)
	logger := zerolog.New(zerolog.NewTestWriter(b))
	qs := query.NewQueryService(boltReader, sqliteReader, config, logger)
	if err := qs.RefreshFromCache(context.Background()); err != nil {
		b.Fatalf("RefreshFromCache failed: %v", err)
	}
	return queryServiceBench{qs: qs}
}

// generateNotes produces notes with optional file classes.
func generateNotes(count int, classes []string) []domain.Note {
	if len(classes) == 0 {
		classes = []string{"class-0", "class-1", "class-2"}
	}
	notes := make([]domain.Note, 0, count)
	for i := range count {
		class := classes[i%len(classes)]
		notes = append(notes, domain.NewNote(
			domain.NewNoteID(fmt.Sprintf("note-%d.md", i)),
			domain.NewFrontmatter(
				map[string]any{"file_class": class, "priority": i % 5},
			),
		))
	}
	return notes
}

// generateAuthorNotes produces notes with author and file_class fields.
func generateAuthorNotes(count int) []domain.Note {
	notes := make([]domain.Note, 0, count)
	for i := range count {
		notes = append(notes, domain.NewNote(
			domain.NewNoteID(fmt.Sprintf("note-%d.md", i)),
			domain.NewFrontmatter(map[string]any{
				"author":     fmt.Sprintf("author-%d", i%50),
				"priority":   i % 5,
				"file_class": fmt.Sprintf("class-%d", i%10),
			}),
		))
	}
	return notes
}

// generateTemplateNotes produces template notes with varied file classes.
func generateTemplateNotes(count int) []domain.Note {
	notes := make([]domain.Note, 0, count)
	classes := []string{"contact", "project", "meeting", "task", "note"}
	clen := len(classes)
	for i := range count {
		notes = append(notes, domain.NewNote(
			domain.NewNoteID(fmt.Sprintf("templates/note-%d.md", i)),
			domain.NewFrontmatter(map[string]any{
				"file_class": classes[i%clen],
				"author":     fmt.Sprintf("author-%d", i%25),
				"priority":   i % 3,
			}),
		))
	}
	return notes
}

// newBenchQueryReader constructs an in-memory query reader with indices.
func newBenchQueryReader(
	notes []domain.Note,
	config domain.Config,
) *benchQueryReader {
	r := &benchQueryReader{
		notes:            append([]domain.Note(nil), notes...),
		pathIndex:        make(map[string]domain.Note),
		fileClassIndex:   make(map[string][]domain.Note),
		frontmatterIndex: make(map[string]map[interface{}][]domain.Note),
	}
	for _, note := range notes {
		r.pathIndex[string(note.ID)] = note
		if class, ok := note.Frontmatter.Fields[config.FileClassKey]; ok {
			if str, ok2 := class.(string); ok2 {
				r.fileClassIndex[str] = append(r.fileClassIndex[str], note)
			}
		}
		for field, value := range note.Frontmatter.Fields {
			if r.frontmatterIndex[field] == nil {
				r.frontmatterIndex[field] = make(map[interface{}][]domain.Note)
			}
			r.frontmatterIndex[field][value] = append(
				r.frontmatterIndex[field][value],
				note,
			)
		}
	}
	return r
}
