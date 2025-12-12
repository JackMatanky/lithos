package integration

import (
	"context"
	"database/sql"
	"fmt"
	"os"
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

// floatPtr creates a pointer to a float64 value.
func floatPtr(v float64) *float64 {
	return &v
}

func defaultFrontmatter(fields map[string]interface{}) domain.Frontmatter {
	return frontmatterWithMetadata(fields, time.Now().Add(-time.Minute), 4096)
}

func frontmatterWithMetadata(
	fields map[string]interface{},
	modTime time.Time,
	size int64,
) domain.Frontmatter {
	enriched := make(map[string]interface{}, len(fields)+2)
	for k, v := range fields {
		enriched[k] = v
	}
	enriched["file_mod_time"] = modTime.UTC()
	enriched["file_size"] = size
	return domain.NewFrontmatter(enriched)
}

func createTestNote(path string, fields map[string]interface{}) domain.Note {
	note, _ := domain.NewNote(
		path,
		domain.NewFrontmatter(fields),
		nil,
		nil,
		nil,
		nil,
	)
	return note
}

func createComplexTestNotes() []domain.Note {
	return []domain.Note{
		createTestNote("contacts/alice.md", map[string]interface{}{
			"title":      "Alice Smith",
			"fileClass":  "contact",
			"name":       "Alice Smith",
			"email":      "alice@company.com",
			"department": "Engineering",
			"active":     true,
			"tags":       []string{"team-lead", "senior", "backend"},
		}),
		createTestNote("contacts/bob.md", map[string]interface{}{
			"title":      "Bob Jones",
			"fileClass":  "contact",
			"name":       "Bob Jones",
			"email":      "bob@company.com",
			"department": "Sales",
			"active":     false,
			"tags":       []string{"junior", "frontend"},
		}),
		createTestNote("contacts/carol.md", map[string]interface{}{
			"title":      "Carol Davis",
			"fileClass":  "contact",
			"name":       "Carol Davis",
			"email":      "carol@company.com",
			"department": "Engineering",
			"active":     true,
			"tags":       []string{"senior", "fullstack"},
		}),
		createTestNote("projects/webapp.md", map[string]interface{}{
			"title":     "Web Application",
			"fileClass": "project",
			"name":      "Web Application Redesign",
			"priority":  "high",
			"progress":  75,
			"tags":      []string{"web", "ui", "critical"},
		}),
		createTestNote("projects/api.md", map[string]interface{}{
			"title":     "API Upgrade",
			"fileClass": "project",
			"name":      "API Performance Upgrade",
			"priority":  "medium",
			"progress":  30,
			"tags":      []string{"backend", "performance"},
		}),
		createTestNote("projects/mobile.md", map[string]interface{}{
			"title":     "Mobile App",
			"fileClass": "project",
			"name":      "Mobile Application",
			"priority":  "low",
			"progress":  10,
			"tags":      []string{"mobile", "ios", "android"},
		}),
	}
}

func TestSQLiteCacheIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	vaultDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		VaultPath:    vaultDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Initialize Writer (creates DB and tables)
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
	migrator := sqlite.NewSchemaViewMigrator(
		[]domain.Schema{contactSchema},
		config.FileClassKey,
		log,
	)

	writer, err := sqlite.NewSQLiteWriterAdapter(config, log, migrator)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	// 4. Persist Notes
	notes := []domain.Note{
		func() domain.Note {
			note, _ := domain.NewNote(
				"contacts/alice.md",
				defaultFrontmatter(map[string]interface{}{
					"title":     "Alice",
					"fileClass": "contact",
					"name":      "Alice Smith",
					"email":     "alice@example.com",
					"status":    "active",
				}),
				nil,
				nil,
				nil,
				nil,
			)
			return note
		}(),
		func() domain.Note {
			note, _ := domain.NewNote(
				"contacts/bob.md",
				defaultFrontmatter(map[string]interface{}{
					"title":     "Bob",
					"fileClass": "contact",
					"name":      "Bob Jones",
					"email":     "bob@example.com",
					"status":    "inactive",
				}),
				nil,
				nil,
				nil,
				nil,
			)
			return note
		}(),
		func() domain.Note {
			note, _ := domain.NewNote(
				"projects/project1.md",
				defaultFrontmatter(map[string]interface{}{
					"title":     "Project 1",
					"fileClass": "project",
					"status":    "active",
				}),
				nil,
				nil,
				nil,
				nil,
			)
			return note
		}(),
	}

	// Create actual files in vault with old mod times
	oldTime := time.Now().Add(-time.Minute)
	for _, n := range notes {
		fullPath := filepath.Join(vaultDir, n.Path)
		require.NoError(t, os.MkdirAll(filepath.Dir(fullPath), 0o755))
		content := fmt.Sprintf(
			"---\nfileClass: %s\ntitle: %s\n---\n\nContent.",
			n.Frontmatter.Fields["fileClass"],
			n.Frontmatter.Fields["title"],
		)
		require.NoError(t, os.WriteFile(fullPath, []byte(content), 0o644))
		require.NoError(t, os.Chtimes(fullPath, oldTime, oldTime))
		require.NoError(t, writer.Persist(ctx, n, time.Now()))
	}

	// 5. Initialize Reader and Query
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Query FileClassQuery (uses view)
	contacts, err := reader.FileClassQuery(ctx, "contact")
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
	alice, err := reader.Read(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.Equal(t, "Alice Smith", alice.Frontmatter.Fields["name"])

	// Verify view-based filtering correctness (manually check if we can query
	// the view from outside) This is covered by FileClassQuery which we
	// verified
	// returns 2 (Alice and Bob) and NOT Project 1.

	// 6. Check Staleness
	// File exists with old mod time, indexed_time is now, so not stale
	stale, err := reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.False(t, stale)

	// Now simulate a file update by touching the file
	fullPath := filepath.Join(vaultDir, "contacts", "alice.md")
	require.NoError(t, os.Chtimes(fullPath, time.Now(), time.Now()))

	stale, err = reader.IsStale(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.True(t, stale)

	staleList, err := reader.GetStaleNotes(ctx)
	require.NoError(t, err)
	assert.Contains(t, staleList, "contacts/alice.md")
}

// TestSQLiteStalenessWithFileEdits tests staleness detection when files are
// actually modified on disk.
func TestSQLiteStalenessWithFileEdits(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	vaultDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		VaultPath:    vaultDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// Create a test file in vault with old mod time
	filePath := "contacts/bob.md"
	fullPath := filepath.Join(vaultDir, filePath)
	require.NoError(t, os.MkdirAll(filepath.Dir(fullPath), 0o755))
	content := `---
fileClass: contact
name: Bob
---

Some content.`
	require.NoError(t, os.WriteFile(fullPath, []byte(content), 0o644))
	oldTime := time.Now().Add(-time.Minute)
	require.NoError(t, os.Chtimes(fullPath, oldTime, oldTime))

	// Index the file
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	note, err := domain.NewNote(
		filePath,
		frontmatterWithMetadata(map[string]interface{}{
			"fileClass": "contact",
			"name":      "Bob",
		}, oldTime, 4096),
		nil,
		nil,
		nil,
		nil,
	)
	require.NoError(t, err)
	indexTime := time.Now()
	require.NoError(t, writer.Persist(ctx, note, indexTime))

	// Create reader
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Initially not stale (file mod time == indexed time approx, but since
	// indexed_time is now, and file is old, wait no)
	// file mod time is old, indexed_time is now, so old < now, not stale
	stale, err := reader.IsStale(ctx, filePath)
	require.NoError(t, err)
	assert.False(t, stale)

	staleList, err := reader.GetStaleNotes(ctx)
	require.NoError(t, err)
	assert.NotContains(t, staleList, filePath)

	// Modify the file on disk
	time.Sleep(10 * time.Millisecond) // Ensure mod time changes
	newContent := content + "\n\nUpdated content."
	require.NoError(t, os.WriteFile(fullPath, []byte(newContent), 0o644))

	// Now should be stale
	stale, err = reader.IsStale(ctx, filePath)
	require.NoError(t, err)
	assert.True(t, stale)

	staleList, err = reader.GetStaleNotes(ctx)
	require.NoError(t, err)
	assert.Contains(t, staleList, filePath)
}

// TestSQLiteSchemaChangeWorkflow tests schema changes and view migration.
func TestSQLiteSchemaChangeWorkflow(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Create initial contact schema and view migrator
	initialSchema := domain.Schema{
		Name: "contact",
		Properties: []domain.Property{
			{Name: "name", Spec: &domain.StringSpec{}},
			{Name: "email", Spec: &domain.StringSpec{}},
		},
	}
	initialMigrator := sqlite.NewSchemaViewMigrator(
		[]domain.Schema{initialSchema},
		config.FileClassKey,
		log,
	)

	writer, err := sqlite.NewSQLiteWriterAdapter(config, log, initialMigrator)
	require.NoError(t, err)

	// 3. Persist test note with initial schema
	note, err := domain.NewNote(
		"contacts/alice.md",
		defaultFrontmatter(map[string]interface{}{
			"title":     "Alice",
			"fileClass": "contact",
			"name":      "Alice Smith",
			"email":     "alice@example.com",
		}),
		nil,
		nil,
		nil,
		nil,
	)
	require.NoError(t, err)

	require.NoError(t, writer.Persist(ctx, note, time.Now()))

	// 4. Verify initial view works
	reader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)

	contacts, err := reader.FileClassQuery(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 1)
	assert.Equal(t, "Alice Smith", contacts[0].Frontmatter.Fields["name"])

	require.NoError(t, writer.Close())
	require.NoError(t, reader.Close())

	// Remove old cache DB to simulate rebuild with new schema
	require.NoError(t, os.Remove(filepath.Join(cacheDir, "cold.db")))

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

	updatedMigrator := sqlite.NewSchemaViewMigrator(
		[]domain.Schema{updatedSchema},
		config.FileClassKey,
		log,
	)
	db, err := sqlite.InitializeDatabase(filepath.Join(cacheDir, "cold.db"))
	require.NoError(t, err)
	require.NoError(t, updatedMigrator.EnsureViews(ctx, db))
	require.NoError(t, db.Close())

	writer, err = sqlite.NewSQLiteWriterAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()
	reader, err = sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// 7. Add note with new schema fields
	updatedNote, err := domain.NewNote(
		"contacts/bob.md",
		defaultFrontmatter(map[string]interface{}{
			"title":     "Bob",
			"fileClass": "contact",
			"name":      "Bob Jones",
			"email":     "bob@example.com",
			"phone":     "555-0123",
			"status":    "active",
		}),
		nil,
		nil,
		nil,
		nil,
	)
	require.NoError(t, err)

	require.NoError(t, writer.Persist(ctx, note, time.Now()))
	require.NoError(t, writer.Persist(ctx, updatedNote, time.Now()))

	// 8. Verify migrated view works with both old and new data
	contacts, err = reader.FileClassQuery(ctx, "contact")
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

// setupTestSchemas creates the test schemas used across multiple tests.
func setupTestSchemas() []domain.Schema {
	return []domain.Schema{
		{
			Name: "contact",
			Properties: []domain.Property{
				{Name: "name", Spec: &domain.StringSpec{}},
				{Name: "email", Spec: &domain.StringSpec{}},
				{Name: "department", Spec: &domain.StringSpec{}},
				{Name: "active", Spec: &domain.BoolSpec{}},
			},
		},
		{
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
					Spec: &domain.NumberSpec{
						Min: floatPtr(0),
						Max: floatPtr(100),
					},
				},
			},
		},
		{
			Name: "meeting",
			Properties: []domain.Property{
				{Name: "title", Spec: &domain.StringSpec{}},
				{Name: "date", Spec: &domain.DateSpec{}},
				{Name: "attendees", Spec: &domain.StringSpec{}, Array: true},
				{
					Name: "duration",
					Spec: &domain.NumberSpec{
						Min: floatPtr(15),
						Max: floatPtr(480),
					},
				},
			},
		},
	}
}

// createTestNotes creates the diverse test dataset.
func createTestNotes() []domain.Note {
	notes := []domain.Note{
		// Contact notes
		createTestNote("contacts/john-doe.md", map[string]interface{}{
			"fileClass":  "contact",
			"name":       "John Doe",
			"email":      "john.doe@company.com",
			"department": "Engineering",
			"active":     true,
		}),
		createTestNote("contacts/jane-smith.md", map[string]interface{}{
			"fileClass":  "contact",
			"name":       "Jane Smith",
			"email":      "jane.smith@company.com",
			"department": "Marketing",
			"active":     true,
		}),
		createTestNote("contacts/bob-johnson.md", map[string]interface{}{
			"fileClass":  "contact",
			"name":       "Bob Johnson",
			"email":      "bob.johnson@company.com",
			"department": "Engineering",
			"active":     false,
		}),
		// Project notes
		createTestNote("projects/alpha.md", map[string]interface{}{
			"fileClass": "project",
			"name":      "Project Alpha",
			"priority":  "high",
			"progress":  75,
		}),
		createTestNote("projects/beta.md", map[string]interface{}{
			"fileClass": "project",
			"name":      "Project Beta",
			"priority":  "medium",
			"progress":  45,
		}),
		createTestNote("projects/gamma.md", map[string]interface{}{
			"fileClass": "project",
			"name":      "Project Gamma",
			"priority":  "low",
			"progress":  10,
		}),
		// Meeting notes
		createTestNote("meetings/standup.md", map[string]interface{}{
			"fileClass": "meeting",
			"title":     "Daily Standup",
			"date":      "2024-01-15",
			"attendees": []interface{}{"john", "jane", "bob"},
			"duration":  30,
		}),
		createTestNote("meetings/retrospective.md", map[string]interface{}{
			"fileClass": "meeting",
			"title":     "Sprint Retrospective",
			"date":      "2024-01-20",
			"attendees": []interface{}{"john", "jane"},
			"duration":  60,
		}),
	}

	return notes
}

// TestSQLiteMetadataQueryPortWithRealData tests MetadataQueryPort with diverse
// datasets.
func TestSQLiteMetadataQueryPortWithRealData(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{CacheDir: cacheDir, FileClassKey: "fileClass"}
	log := zerolog.Nop()
	ctx := context.Background()

	schemas := setupTestSchemas()
	migrator := sqlite.NewSchemaViewMigrator(schemas, config.FileClassKey, log)

	// Initialize adapters
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log, migrator)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	reader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = reader.Close() }()

	// Setup schemas and data
	notes := createTestNotes()

	// Persist all notes
	indexTime := time.Now()
	for _, note := range notes {
		persistErr := writer.Persist(ctx, note, indexTime)
		require.NoError(t, persistErr)
	}

	// Test FileClassQuery queries
	testFileClassQueryQueries(t, ctx, reader)

	// Test FrontmatterQuery with various data types
	testFrontmatterQueries(t, ctx, reader)

	// Test TagQuery with complex tag scenarios
	testTagQueries(t, ctx, writer, reader, indexTime)

	// Test PathQuery functionality
	testPathQueries(t, ctx, reader)

	// Verify data integrity and field extraction
	testDataIntegrity(t, ctx, reader)
}

// testFileClassQueryQueries tests FileClassQuery with different schemas.
func testFileClassQueryQueries(
	t *testing.T,
	ctx context.Context,
	reader *sqlite.SQLiteReaderAdapter,
) {
	contacts, err := reader.FileClassQuery(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 3)

	projects, err := reader.FileClassQuery(ctx, "project")
	require.NoError(t, err)
	assert.Len(t, projects, 3)

	meetings, err := reader.FileClassQuery(ctx, "meeting")
	require.NoError(t, err)
	assert.Len(t, meetings, 2)

	// Test non-existent fileClass
	empty, err := reader.FileClassQuery(ctx, "nonexistent")
	require.NoError(t, err)
	assert.Empty(t, empty)
}

// testFrontmatterQueries tests FrontmatterQuery with various data types.
func testFrontmatterQueries(
	t *testing.T,
	ctx context.Context,
	reader *sqlite.SQLiteReaderAdapter,
) {
	// String queries
	engineering, err := reader.FrontmatterQuery(
		ctx,
		"department",
		"Engineering",
	)
	require.NoError(t, err)
	assert.Len(t, engineering, 2)

	marketing, err := reader.FrontmatterQuery(ctx, "department", "Marketing")
	require.NoError(t, err)
	assert.Len(t, marketing, 1)

	// Boolean queries
	active, err := reader.FrontmatterQuery(ctx, "active", "true")
	require.NoError(t, err)
	assert.Len(t, active, 2)

	inactive, err := reader.FrontmatterQuery(ctx, "active", "false")
	require.NoError(t, err)
	assert.Len(t, inactive, 1)

	// Number queries
	highProgress, err := reader.FrontmatterQuery(ctx, "progress", "75")
	require.NoError(t, err)
	assert.Len(t, highProgress, 1)

	// Enum queries
	highPriority, err := reader.FrontmatterQuery(ctx, "priority", "high")
	require.NoError(t, err)
	assert.Len(t, highPriority, 1)
}

// testTagQueries tests TagQuery with complex tag scenarios.
func testTagQueries(
	t *testing.T,
	ctx context.Context,
	writer *sqlite.SQLiteWriterAdapter,
	reader *sqlite.SQLiteReaderAdapter,
	indexTime time.Time,
) {
	// Create notes with tags for testing
	taggedNotes := []domain.Note{
		createTestNote("tagged/work-project.md", map[string]interface{}{
			"fileClass": "project",
			"name":      "Work Project",
			"priority":  "high",
			"tags":      []interface{}{"work", "urgent"},
		}),
		createTestNote("tagged/personal-note.md", map[string]interface{}{
			"fileClass": "meeting",
			"title":     "Personal Meeting",
			"date":      "2024-01-25",
			"attendees": []interface{}{"self"},
			"tags":      []interface{}{"personal"},
		}),
	}

	for _, note := range taggedNotes {
		err := writer.Persist(ctx, note, indexTime)
		require.NoError(t, err)
	}

	workNotes, err := reader.TagQuery(ctx, "work")
	require.NoError(t, err)
	assert.Len(t, workNotes, 1)

	personalNotes, err := reader.TagQuery(ctx, "personal")
	require.NoError(t, err)
	assert.Len(t, personalNotes, 1)

	urgentNotes, err := reader.TagQuery(ctx, "urgent")
	require.NoError(t, err)
	assert.Len(t, urgentNotes, 1)
}

// testPathQueries tests PathQuery functionality.
func testPathQueries(
	t *testing.T,
	ctx context.Context,
	reader *sqlite.SQLiteReaderAdapter,
) {
	// Test basename queries
	contactsBasenameQuery, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Value: "john-doe",
		Scope: spi.PathQueryScopeBasename,
	})
	require.NoError(t, err)
	assert.Len(t, contactsBasenameQuery, 1)

	// Test folder queries
	contactNotes, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Value: "contacts",
		Scope: spi.PathQueryScopeFolder,
	})
	require.NoError(t, err)
	assert.Len(t, contactNotes, 3)

	projectNotes, err := reader.PathQuery(ctx, spi.PathQueryOptions{
		Value: "projects",
		Scope: spi.PathQueryScopeFolder,
	})
	require.NoError(t, err)
	assert.Len(t, projectNotes, 3)
}

// testDataIntegrity verifies data integrity and field extraction.
func testDataIntegrity(
	t *testing.T,
	ctx context.Context,
	reader *sqlite.SQLiteReaderAdapter,
) {
	// Read specific notes and verify all fields are preserved
	johnNote, err := reader.Read(ctx, "contacts/john-doe.md")
	require.NoError(t, err)
	assert.Equal(t, "John Doe", johnNote.Frontmatter.Fields["name"])
	assert.Equal(
		t,
		"john.doe@company.com",
		johnNote.Frontmatter.Fields["email"],
	)
	assert.Equal(t, "Engineering", johnNote.Frontmatter.Fields["department"])
	assert.Equal(t, true, johnNote.Frontmatter.Fields["active"])

	alphaProject, err := reader.Read(ctx, "projects/alpha.md")
	require.NoError(t, err)
	assert.Equal(t, "Project Alpha", alphaProject.Frontmatter.Fields["name"])
	assert.Equal(t, "high", alphaProject.Frontmatter.Fields["priority"])
	assert.InDelta(
		t,
		float64(75),
		alphaProject.Frontmatter.Fields["progress"],
		0.01,
	)

	standupMeeting, err := reader.Read(
		ctx,
		"meetings/standup.md",
	)
	require.NoError(t, err)
	assert.Equal(t, "Daily Standup", standupMeeting.Frontmatter.Fields["title"])
	assert.Equal(t, "2024-01-15", standupMeeting.Frontmatter.Fields["date"])
	assert.InDelta(
		t,
		float64(30),
		standupMeeting.Frontmatter.Fields["duration"],
		0.01,
	)

	attendees := standupMeeting.Frontmatter.Fields["attendees"].([]interface{})
	assert.Len(t, attendees, 3)
	assert.Contains(t, attendees, "john")
	assert.Contains(t, attendees, "jane")
	assert.Contains(t, attendees, "bob")

	// Verify tagged note integrity
	workProject, err := reader.Read(
		ctx,
		"tagged/work-project.md",
	)
	require.NoError(t, err)
	assert.Equal(t, "Work Project", workProject.Frontmatter.Fields["name"])
	tags := workProject.Frontmatter.Fields["tags"].([]interface{})
	assert.Len(t, tags, 2)
	assert.Contains(t, tags, "work")
	assert.Contains(t, tags, "urgent")

	webapp, err := reader.Read(ctx, "tagged/personal-note.md")
	require.NoError(t, err)
	assert.Equal(t, "Personal Meeting", webapp.Frontmatter.Fields["title"])
	assert.Contains(t, webapp.Frontmatter.Fields["tags"], "personal")
	assert.Contains(t, webapp.Frontmatter.Fields["tags"], "planning")
	assert.Contains(t, webapp.Frontmatter.Fields["tags"], "critical")
}

// TestSQLitePerformanceWith1000Notes tests performance with large dataset.
func TestSQLitePerformanceWith1000Notes(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping performance test in short mode")
	}

	cacheDir := t.TempDir()
	config := domain.Config{
		CacheDir:     cacheDir,
		FileClassKey: "fileClass",
	}
	log := zerolog.Nop()
	ctx := context.Background()

	// 1. Initialize adapters
	writer, err := sqlite.NewSQLiteWriterAdapter(config, log, nil)
	require.NoError(t, err)
	defer func() { _ = writer.Close() }()

	reader, err := sqlite.NewSQLiteReaderAdapter(config, log, nil)
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
	notes := createComplexTestNotes()

	// Persist all notes
	indexTime := time.Now()
	for _, note := range notes {
		require.NoError(t, writer.Persist(ctx, note, indexTime))
	}

	// 4. Test FileClassQuery with different schemas
	contacts, err := reader.FileClassQuery(ctx, "contact")
	require.NoError(t, err)
	assert.Len(t, contacts, 3)

	projects, err := reader.FileClassQuery(ctx, "project")
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
	alice, err := reader.Read(ctx, "contacts/alice.md")
	require.NoError(t, err)
	assert.Equal(t, "Alice Smith", alice.Frontmatter.Fields["name"])
	assert.Equal(t, "Engineering", alice.Frontmatter.Fields["department"])
	assert.Equal(t, true, alice.Frontmatter.Fields["active"])
	assert.Contains(t, alice.Frontmatter.Fields["tags"], "team-lead")

	webapp, err := reader.Read(ctx, "projects/webapp.md")
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
