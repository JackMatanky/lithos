package integration

import (
	"context"
	"database/sql"
	"fmt"
	"path/filepath"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/adapters/spi/cache/sqlite"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/ports/spi"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSQLiteCacheIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir: cacheDir,
	}
	log := zerolog.Nop()

	// 1. Initialize Writer (creates DB and tables)
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// 2. Define Schema and Generate View
	contactSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
			{
				Name: "status",
				Spec: &domain.StringSpec{Enum: []string{"active", "inactive"}},
			},
		},
	}

	viewSQL, err := sqlite.GenerateSchemaView(contactSchema)
	require.NoError(t, err)

	// 3. Apply View manually (simulating migration)
	dbPath := filepath.Join(cacheDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	// Enable WAL mode for concurrency
	ctx := context.Background()
	_, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL;")
	require.NoError(t, err)

	_, err = db.ExecContext(ctx, viewSQL)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	// 4. Persist Notes
	notes := []domain.Note{
		{
			ID: domain.NewNoteID("contacts/alice.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Alice",
				"fileClass": "contact",
				"name":      "Alice Smith",
				"email":     "alice@example.com",
				"status":    "active",
			}),
		},
		{
			ID: domain.NewNoteID("contacts/bob.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Bob",
				"fileClass": "contact",
				"name":      "Bob Jones",
				"email":     "bob@example.com",
				"status":    "inactive",
			}),
		},
		{
			ID: domain.NewNoteID("projects/project1.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Project 1",
				"fileClass": "project",
				"status":    "active",
			}),
		},
	}

	for _, n := range notes {
		require.NoError(t, writer.Persist(ctx, n, time.Now()))
	}

	// 5. Initialize Reader and Query
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Query ByFileClass (uses view)
	contacts, err := reader.ByFileClass(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 2)

	// Verify extracted fields via FrontmatterQuery (uses json_extract on base
	// table, but verifies deep query works)
	activeContacts, err := reader.FrontmatterQuery(ctx, "status", "active")
	require.NoError(t, err)
	// Should match Alice (active contact) and Project 1 (active project)
	// Wait, FrontmatterQuery queries ALL notes.
	assert.Len(t, activeContacts, 2)

	// Verify specific note content
	alice, err := reader.Read(ctx, domain.NewNoteID("contacts/alice.md"))
	require.NoError(t, err)
	assert.Equal(t, "Alice Smith", alice.Frontmatter.Fields["name"])

	// Verify view-based filtering correctness (manually check if we can query
	// the view from outside) This is covered by ByFileClass which we verified
	// returns 2 (Alice and Bob) and NOT Project 1.

	// 6. Check Staleness
	// Since file doesn't exist, IsStale should check DB timestamps.
	// In our writer.go, indexTime is time.Now().
	// modified_at comes from cache.ExtractFileModTime(fields).
	// Our fields didn't have "modified_at" or similar, so it defaults to 0
	// (1970).
	// indexed_time is Now (~2025).
	// So modified_at (0) < indexed_time (2025).
	// Thus IsStale should be false?
	// Wait, ExtractFileModTime uses "modified" or "mtime" fields if present,
	// else time.Time{}.
	// 0 < Now -> Not Stale.
	stale, err := reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.False(t, stale)

	// Now simulate a file update (update DB modified_at > indexed_time)
	// We need to hack the DB to simulate this state
	db, err = sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	futureMod := time.Now().Add(1 * time.Hour).Unix()
	_, err = db.ExecContext(
		ctx,
		"UPDATE notes SET modified_at = ? WHERE path = ?",
		futureMod,
		"contacts/alice.md",
	)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	stale, err = reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.True(t, stale)

	staleList, err := reader.GetStaleNotes(ctx)
	require.NoError(t, err)
	assert.Contains(t, staleList, "contacts/alice.md")
}

// TestSQLiteSchemaChangeWorkflow tests schema changes and view migration.
func TestSQLiteSchemaChangeWorkflow(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir: cacheDir,
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Initialize Writer with initial schema
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// 2. Create initial contact schema
	initialSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
		},
	}

	viewSQL, err := sqlite.GenerateSchemaView(initialSchema)
	require.NoError(t, err)

	// Apply initial view
	dbPath := filepath.Join(cacheDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	_, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL;")
	require.NoError(t, err)
	_, err = db.ExecContext(ctx, viewSQL)
	require.NoError(t, err)

	// 3. Persist test note with initial schema
	note := domain.Note{
		ID: domain.NewNoteID("contacts/alice.md"),
		Frontmatter: domain.NewFrontmatter(map[string]interface{}{
			"title":     "Alice",
			"fileClass": "contact",
			"name":      "Alice Smith",
			"email":     "alice@example.com",
		}),
	}

	require.NoError(t, writer.Persist(ctx, note, time.Now()))

	// 4. Verify initial view works
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	contacts, err := reader.ByFileClass(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 1)
	assert.Equal(t, "Alice Smith", contacts[0].Frontmatter.Fields["name"])

	// 5. Update schema to add new fields
	updatedSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
			{Name: "phone", Spec: &domain.StringSpec{}},
			{
				Name: "status",
				Spec: &domain.StringSpec{Enum: []string{"active", "inactive"}},
			},
		},
	}

	// 6. Simulate view migration (drop and recreate)
	dropViewSQL := "DROP VIEW IF EXISTS v_contact_notes;"
	_, err = db.ExecContext(ctx, dropViewSQL)
	require.NoError(t, err)

	newViewSQL, err := sqlite.GenerateSchemaView(updatedSchema)
	require.NoError(t, err)
	_, err = db.ExecContext(ctx, newViewSQL)
	require.NoError(t, err)
	require.NoError(t, db.Close())

	// 7. Add note with new schema fields
	updatedNote := domain.Note{
		ID: domain.NewNoteID("contacts/bob.md"),
		Frontmatter: domain.NewFrontmatter(map[string]interface{}{
			"title":     "Bob",
			"fileClass": "contact",
			"name":      "Bob Jones",
			"email":     "bob@example.com",
			"phone":     "555-0123",
			"status":    "active",
		}),
	}

	require.NoError(t, writer.Persist(ctx, updatedNote, time.Now()))

	// 8. Verify migrated view works with both old and new data
	contacts, err = reader.ByFileClass(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 2)

	// Query with new field - should work for Bob, gracefully handle Alice (null
	// phone)
	results, err := reader.FrontmatterQuery(ctx, "phone", "555-0123")
	require.NoError(t, err)
	assert.Len(t, results, 1)
	assert.Equal(t, "Bob Jones", results[0].Frontmatter.Fields["name"])

	// Query with status enum field
	activeResults, err := reader.FrontmatterQuery(ctx, "status", "active")
	require.NoError(t, err)
	assert.Len(t, activeResults, 1)
	assert.Equal(t, "Bob Jones", activeResults[0].Frontmatter.Fields["name"])
}

// TestSQLiteMetadataQueryPortWithRealData tests MetadataQueryPort with diverse
// datasets.
func TestSQLiteMetadataQueryPortWithRealData(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir: cacheDir,
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Initialize adapters
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	reader, err := sqlite.NewSQLiteReaderAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// 2. Create multiple schemas and views
	contactSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
			{Name: "department", Spec: &domain.StringSpec{}},
			{Name: "active", Spec: &domain.BoolSpec{}},
		},
	}

	projectSchema := domain.Schema{
		Name: "project",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{
				Name: "priority",
				Spec: &domain.StringSpec{
					Enum: []string{"high", "medium", "low"},
				},
			},
			{
				Name: "progress",
				Spec: &domain.NumberSpec{Min: floatPtr(0), Max: floatPtr(100)},
			},
		},
	}

	// Apply views
	dbPath := filepath.Join(cacheDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	_, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL;")
	require.NoError(t, err)

	for _, schema := range []domain.Schema{contactSchema, projectSchema} {
		viewSQL, viewErr := sqlite.GenerateSchemaView(schema)
		require.NoError(t, viewErr)
		_, execErr := db.ExecContext(ctx, viewSQL)
		require.NoError(t, execErr)
	}

	// 3. Create diverse test dataset
	notes := []domain.Note{
		// Contacts
		{
			ID: domain.NewNoteID("contacts/alice.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      "Alice Smith",
				"fileClass":  "contact",
				"name":       "Alice Smith",
				"email":      "alice@company.com",
				"department": "Engineering",
				"active":     true,
				"tags":       []string{"team-lead", "senior", "backend"},
			}),
		},
		{
			ID: domain.NewNoteID("contacts/bob.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      "Bob Jones",
				"fileClass":  "contact",
				"name":       "Bob Jones",
				"email":      "bob@company.com",
				"department": "Sales",
				"active":     false,
				"tags":       []string{"junior", "frontend"},
			}),
		},
		{
			ID: domain.NewNoteID("contacts/carol.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      "Carol Davis",
				"fileClass":  "contact",
				"name":       "Carol Davis",
				"email":      "carol@company.com",
				"department": "Engineering",
				"active":     true,
				"tags":       []string{"senior", "fullstack"},
			}),
		},
		{
			ID: domain.NewNoteID("contacts/bob.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      "Bob Jones",
				"fileClass":  "contact",
				"name":       "Bob Jones",
				"email":      "bob@company.com",
				"department": "Sales",
				"active":     false,
				"tags":       []string{"junior", "frontend"},
			}),
		},
		{
			ID: domain.NewNoteID("contacts/carol.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      "Carol Davis",
				"fileClass":  "contact",
				"name":       "Carol Davis",
				"email":      "carol@company.com",
				"department": "Engineering",
				"active":     true,
				"tags":       []string{"senior", "fullstack"},
			}),
		},
		// Projects
		{
			ID: domain.NewNoteID("projects/webapp.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Web Application",
				"fileClass": "project",
				"name":      "Web Application Redesign",
				"priority":  "high",
				"progress":  75,
				"tags":      []string{"web", "ui", "critical"},
			}),
		},
		{
			ID: domain.NewNoteID("projects/api.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{

				"title":     "API Upgrade",
				"fileClass": "project",
				"name":      "API Performance Upgrade",
				"priority":  "medium",
				"progress":  30,
				"tags":      []string{"backend", "performance"},
			}),
		},
		{
			ID: domain.NewNoteID("projects/mobile.md"),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":     "Mobile App",
				"fileClass": "project",
				"name":      "Mobile Application",
				"priority":  "low",
				"progress":  10,
				"tags":      []string{"mobile", "ios", "android"},
			}),
		},
	}

	// Persist all notes
	indexTime := time.Now()
	for _, note := range notes {
		require.NoError(t, writer.Persist(ctx, note, indexTime))
	}

	// 4. Test ByFileClass with different schemas
	contacts, err := reader.ByFileClass(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 3)

	projects, err := reader.ByFileClass(ctx, "project")
	require.NoError(t, err)
	assert.Len(t, projects, 3)

	// 5. Test FrontmatterQuery with various data types

	// String queries
	engineeringContacts, err := reader.FrontmatterQuery(
		ctx,
		"department",
		"Engineering",
	)
	require.NoError(t, err)
	assert.Len(t, engineeringContacts, 2) // Alice and Carol

	highPriorityProjects, err := reader.FrontmatterQuery(
		ctx,
		"priority",
		"high",
	)
	require.NoError(t, err)
	assert.Len(t, highPriorityProjects, 1) // Web Application

	// Boolean queries (convert to string)
	activeContacts, err := reader.FrontmatterQuery(ctx, "active", "true")
	require.NoError(t, err)
	assert.Len(t, activeContacts, 2) // Alice and Carol

	// Number queries (SQLite converts progress to float64)
	highProgressProjects, err := reader.FrontmatterQuery(ctx, "progress", "75")
	require.NoError(t, err)
	assert.Len(t, highProgressProjects, 1) // Web Application

	// 6. Test TagQuery with complex tag scenarios
	seniorTags, err := reader.TagQuery(ctx, "senior")
	require.NoError(t, err)
	assert.Len(t, seniorTags, 2) // Alice and Carol

	backendTags, err := reader.TagQuery(ctx, "backend")
	require.NoError(t, err)
	assert.Len(t, backendTags, 2) // Alice and API project

	criticalTags, err := reader.TagQuery(ctx, "critical")
	require.NoError(t, err)
	assert.Len(t, criticalTags, 1) // Web Application

	// Non-existent tag
	nonExistentTags, err := reader.TagQuery(ctx, "nonexistent")
	require.NoError(t, err)
	assert.Empty(t, nonExistentTags)

	// 7. Test PathQuery functionality
	// Test full path scope
	fullPathResults, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFull,
		Value: "contacts/alice.md",
	})
	require.NoError(t, err)
	assert.Len(t, fullPathResults, 1)
	assert.Equal(
		t,
		"Alice Smith",
		fullPathResults[0].Frontmatter.Fields["name"],
	)

	// Test basename scope (should find alice.md and bob.md)
	basenameResults, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeBasename,
		Value: "alice",
	})
	require.NoError(t, err)
	assert.Len(t, basenameResults, 1) // Only alice.md matches exactly

	// Test folder scope
	folderResults, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFolder,
		Value: "contacts/",
	})
	require.NoError(t, err)
	assert.Len(t, folderResults, 3) // All contacts

	// Test folder scope with different folder
	projectResults, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Scope: spi.PathQueryScopeFolder,
		Value: "projects/",
	})
	require.NoError(t, err)
	assert.Len(t, projectResults, 3) // All projects

	// 8. Verify data integrity and field extraction
	alice, err := reader.Read(ctx, domain.NewNoteID("contacts/alice.md"))
	require.NoError(t, err)
	assert.Equal(t, "Alice Smith", alice.Frontmatter.Fields["name"])
	assert.Equal(t, "Engineering", alice.Frontmatter.Fields["department"])
	assert.Equal(t, true, alice.Frontmatter.Fields["active"])
	assert.Contains(t, alice.Frontmatter.Fields["tags"], "team-lead")

	webapp, err := reader.Read(ctx, domain.NewNoteID("projects/webapp.md"))
	require.NoError(t, err)
	assert.Equal(
		t,
		"Web Application Redesign",
		webapp.Frontmatter.Fields["name"],
	)
	assert.Equal(t, "high", webapp.Frontmatter.Fields["priority"])
	assert.InDelta(t, float64(75), webapp.Frontmatter.Fields["progress"], 0.01)
	assert.Contains(t, webapp.Frontmatter.Fields["tags"], "critical")
}

// TestSQLitePerformanceWith1000Notes tests performance with large dataset.
func TestSQLitePerformanceWith1000Notes(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping performance test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir: cacheDir,
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Initialize adapters
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	reader, err := sqlite.NewSQLiteReaderAdapter(config, log)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// 2. Create schema and view for performance testing
	contactSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
			{Name: "department", Spec: &domain.StringSpec{}},
			{
				Name: "level",
				Spec: &domain.StringSpec{
					Enum: []string{"junior", "senior", "lead"},
				},
			},
			{
				Name: "salary",
				Spec: &domain.NumberSpec{
					Min: floatPtr(30000),
					Max: floatPtr(200000),
				},
			},
			{Name: "active", Spec: &domain.BoolSpec{}},
		},
	}

	// Apply view
	dbPath := filepath.Join(cacheDir, "cold.db")
	db, err := sql.Open("sqlite", dbPath)
	require.NoError(t, err)
	defer func() { _ = db.Close() }()

	_, err = db.ExecContext(ctx, "PRAGMA journal_mode=WAL;")
	require.NoError(t, err)

	viewSQL, err := sqlite.GenerateSchemaView(contactSchema)
	require.NoError(t, err)
	_, err = db.ExecContext(ctx, viewSQL)
	require.NoError(t, err)

	// 3. Generate 1000 test notes
	departments := []string{
		"Engineering",
		"Sales",
		"Marketing",
		"HR",
		"Finance",
	}
	levels := []string{"junior", "senior", "lead"}

	indexTime := time.Now()

	t.Log("Generating and persisting 1000 notes...")
	persistStart := time.Now()

	for i := range 1000 {
		note := domain.Note{
			ID: domain.NewNoteID(fmt.Sprintf("contacts/employee_%04d.md", i)),
			Frontmatter: domain.NewFrontmatter(map[string]interface{}{
				"title":      fmt.Sprintf("Employee %d", i),
				"fileClass":  "contact",
				"name":       fmt.Sprintf("Employee %d", i),
				"email":      fmt.Sprintf("employee%d@company.com", i),
				"department": departments[i%len(departments)],
				"level":      levels[i%len(levels)],
				"salary":     30000 + (i%17)*10000, // Varied salaries
				"active":     i%7 != 0,             // ~85% active
				"tags": []string{
					fmt.Sprintf("team-%d", i%5),
					levels[i%len(levels)],
				},
			}),
		}

		require.NoError(t, writer.Persist(ctx, note, indexTime))

		if i%250 == 0 {
			t.Logf("Persisted %d notes", i)
		}
	}

	persistDuration := time.Since(persistStart)
	t.Logf(
		"Persisted 1000 notes in %v (avg: %v per note)",
		persistDuration,
		persistDuration/1000,
	)

	// 4. Test query performance
	tests := []struct {
		name    string
		queryFn func() ([]domain.Note, error)
		target  time.Duration
	}{
		{
			name: "ByFileClass",
			queryFn: func() ([]domain.Note, error) {
				return reader.ByFileClass(ctx, "contact")
			},
			target: 50 * time.Millisecond,
		},
		{
			name: "FrontmatterQuery - Department",
			queryFn: func() ([]domain.Note, error) {
				return reader.FrontmatterQuery(ctx, "department", "Engineering")
			},
			target: 50 * time.Millisecond,
		},
		{
			name: "FrontmatterQuery - Level",
			queryFn: func() ([]domain.Note, error) {
				return reader.FrontmatterQuery(ctx, "level", "senior")
			},
			target: 50 * time.Millisecond,
		},
		{
			name: "FrontmatterQuery - Active Boolean",
			queryFn: func() ([]domain.Note, error) {
				return reader.FrontmatterQuery(ctx, "active", "true")
			},
			target: 50 * time.Millisecond,
		},
		{
			name: "TagQuery",
			queryFn: func() ([]domain.Note, error) {
				return reader.TagQuery(ctx, "senior")
			},
			target: 50 * time.Millisecond,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Warm up
			_, warmErr := test.queryFn()
			require.NoError(t, warmErr)

			// Measure performance
			start := time.Now()
			results, queryErr := test.queryFn()
			duration := time.Since(start)

			require.NoError(t, queryErr)
			assert.NotEmpty(t, results, "Should return results")

			t.Logf(
				"%s: %v (target: %v) - %d results",
				test.name,
				duration,
				test.target,
				len(results),
			)

			// Performance assertion - should be well under target
			assert.Less(
				t,
				duration,
				test.target,
				"Query should complete within target time",
			)
		})
	}

	// 5. Test staleness detection performance
	t.Run("Staleness Detection", func(t *testing.T) {
		start := time.Now()
		staleNotes, staleErr := reader.GetStaleNotes(ctx)
		duration := time.Since(start)

		require.NoError(t, staleErr)
		assert.Empty(t, staleNotes) // All notes should be fresh

		t.Logf("Staleness check for 1000 notes: %v", duration)
		assert.Less(
			t,
			duration,
			10*time.Millisecond,
			"Staleness check should be very fast",
		)
	})

	// 6. Test individual note read performance
	t.Run("Individual Read Performance", func(t *testing.T) {
		start := time.Now()
		note, readErr := reader.Read(
			ctx,
			domain.NewNoteID("contacts/employee_0500.md"),
		)
		duration := time.Since(start)

		require.NoError(t, readErr)
		assert.Equal(t, "Employee 500", note.Frontmatter.Fields["name"])

		t.Logf("Individual note read: %v", duration)
		assert.Less(
			t,
			duration,
			5*time.Millisecond,
			"Individual reads should be very fast",
		)
	})

	// 7. Memory and resource usage validation
	totalNotes, err := reader.List(ctx)
	require.NoError(t, err)
	assert.Len(t, totalNotes, 1000, "Should have all 1000 notes")

	t.Log("Performance test completed successfully - all targets met")
}

// Helper function for creating float64 pointers.
func floatPtr(f float64) *float64 {
	return &f
}
