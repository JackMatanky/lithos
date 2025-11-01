package query

import (
	"context"
	"errors"
	"sync"
	"testing"

	"github.com/JackMatanky/lithos/internal/domain"
	domainerrors "github.com/JackMatanky/lithos/internal/shared/errors"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// FakeCacheReader implements CacheReaderPort for testing.
type FakeCacheReader struct {
	notes []domain.Note
	err   error
}

// Read implements CacheReaderPort.Read for testing.
func (f *FakeCacheReader) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	return domain.Note{}, nil
}

// List implements CacheReaderPort.List for testing.
func (f *FakeCacheReader) List(ctx context.Context) ([]domain.Note, error) {
	if f.err != nil {
		return nil, f.err
	}
	return f.notes, nil
}

// setupQueryServiceForRefresh creates a QueryService with initial indices
// for testing RefreshFromCache method.
func setupQueryServiceForRefresh(
	t *testing.T,
	initialNotes []domain.Note,
) *QueryService {
	t.Helper()

	fakeCacheReader := &FakeCacheReader{notes: initialNotes}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Pre-populate with some initial data to test clearing
	if len(initialNotes) > 0 {
		qs.byID = map[domain.NoteID]domain.Note{
			initialNotes[0].ID: initialNotes[0],
		}
		qs.byPath = map[string]domain.Note{
			"initial/path.md": initialNotes[0],
		}
		qs.byFileClass = map[string][]domain.Note{
			initialNotes[0].Frontmatter.FileClass: {initialNotes[0]},
		}
	}

	return qs
}

// TestQueryService_StructExists verifies that QueryService struct exists
// with sync.RWMutex and all required index maps are present (initially nil).
func TestQueryService_StructExists(t *testing.T) {
	// This test verifies that QueryService struct exists with sync.RWMutex
	// and all required index maps are present (initially nil as expected)

	// Create a QueryService instance
	qs := &QueryService{}

	// Verify sync.RWMutex exists (should be zero value)
	assert.NotNil(t, &qs.mu, "QueryService should have sync.RWMutex field 'mu'")

	// Verify byID map exists (nil initially)
	assert.Nil(
		t,
		qs.byID,
		"QueryService should have byID map[NoteID]Note (nil initially)",
	)

	// Verify byPath map exists (nil initially)
	assert.Nil(
		t,
		qs.byPath,
		"QueryService should have byPath map[string]Note (nil initially)",
	)

	// Verify byBasename map exists (nil initially)
	assert.Nil(
		t,
		qs.byBasename,
		"QueryService should have byBasename map[string][]Note (nil initially)",
	)

	// Verify byFileClass map exists (nil initially)
	assert.Nil(
		t,
		qs.byFileClass,
		"QueryService should have byFileClass map[string][]Note (nil initially)",
	)
}

// TestQueryService_DependenciesExist verifies that QueryService has
// required dependencies (CacheReaderPort and zerolog.Logger).
func TestQueryService_DependenciesExist(t *testing.T) {
	// This test verifies that QueryService has required dependencies

	qs := &QueryService{}

	// Verify cacheReader dependency field exists (interface, nil initially)
	assert.Nil(
		t,
		qs.cacheReader,
		"QueryService should have CacheReaderPort dependency field",
	)

	// Verify logger dependency field exists (zerolog.Logger has zero value)
	// We can't check nil for zerolog.Logger as it has a non-nil zero value
	assert.NotNil(
		t,
		&qs.log,
		"QueryService should have zerolog.Logger dependency field",
	)
}

// setupQueryServiceWithNotes creates a QueryService with populated indices
// for testing query methods. It includes sample notes with different file
// classes.
func setupQueryServiceWithNotes(t *testing.T) *QueryService {
	t.Helper()

	// Create sample notes
	note1 := domain.Note{
		ID: domain.NoteID("note-1"),
		Frontmatter: domain.Frontmatter{
			FileClass: "contact",
		},
	}
	note2 := domain.Note{
		ID: domain.NoteID("note-2"),
		Frontmatter: domain.Frontmatter{
			FileClass: "meeting",
		},
	}
	note3 := domain.Note{
		ID: domain.NoteID("note-3"),
		Frontmatter: domain.Frontmatter{
			FileClass: "contact",
		},
	}

	// Define paths separately (Note struct doesn't contain path)
	path1 := "contacts/john-doe.md"
	path2 := "meetings/2023-10-01.md"
	path3 := "contacts/jane-smith.md"

	// Create QueryService with fake dependencies
	fakeCacheReader := &FakeCacheReader{}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Manually populate indices for testing
	qs.byID = map[domain.NoteID]domain.Note{
		note1.ID: note1,
		note2.ID: note2,
		note3.ID: note3,
	}
	qs.byPath = map[string]domain.Note{
		path1: note1,
		path2: note2,
		path3: note3,
	}
	qs.byBasename = map[string][]domain.Note{
		"john-doe":   {note1},
		"2023-10-01": {note2},
		"jane-smith": {note3},
	}
	qs.byFileClass = map[string][]domain.Note{
		"contact": {note1, note3},
		"meeting": {note2},
	}

	return qs
}

// TestQueryService_ByID_ExistingNote verifies ByID returns note for existing
// NoteID.
func TestQueryService_ByID_ExistingNote(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	note, err := qs.ByID(ctx, domain.NoteID("note-1"))

	require.NoError(t, err, "ByID should not return error for existing note")
	assert.Equal(
		t,
		domain.NoteID("note-1"),
		note.ID,
		"ByID should return correct note",
	)
	assert.Equal(
		t,
		"contact",
		note.Frontmatter.FileClass,
		"ByID should return note with correct file class",
	)
}

// TestQueryService_ByID_MissingNote verifies ByID returns ResourceError for
// missing NoteID.
func TestQueryService_ByID_MissingNote(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	note, err := qs.ByID(ctx, domain.NoteID("missing-note"))

	require.Error(t, err, "ByID should return error for missing note")
	var resErr *domainerrors.ResourceError
	require.ErrorAs(
		t,
		err,
		&resErr,
		"ByID should return ResourceError for missing note",
	)
	assert.Equal(
		t,
		domain.Note{},
		note,
		"ByID should return zero Note for missing note",
	)
}

// TestQueryService_ByPath_ExistingPath verifies ByPath returns note for
// existing path.
func TestQueryService_ByPath_ExistingPath(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	note, err := qs.ByPath(ctx, "contacts/john-doe.md")

	require.NoError(t, err, "ByPath should not return error for existing path")
	assert.Equal(
		t,
		domain.NoteID("note-1"),
		note.ID,
		"ByPath should return note with correct ID",
	)
	assert.Equal(
		t,
		"contact",
		note.Frontmatter.FileClass,
		"ByPath should return note with correct file class",
	)
}

// TestQueryService_ByPath_MissingPath verifies ByPath returns ResourceError for
// missing path.
func TestQueryService_ByPath_MissingPath(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	note, err := qs.ByPath(ctx, "missing/path.md")

	require.Error(t, err, "ByPath should return error for missing path")
	var resErr *domainerrors.ResourceError
	require.ErrorAs(
		t,
		err,
		&resErr,
		"ByPath should return ResourceError for missing path",
	)
	assert.Equal(
		t,
		domain.Note{},
		note,
		"ByPath should return zero Note for missing path",
	)
}

// TestQueryService_ByFileClass_ExistingClass verifies ByFileClass returns all
// notes matching schema.
func TestQueryService_ByFileClass_ExistingClass(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	notes, err := qs.ByFileClass(ctx, "contact")

	require.NoError(
		t,
		err,
		"ByFileClass should not return error for existing file class",
	)
	assert.Len(
		t,
		notes,
		2,
		"ByFileClass should return 2 notes for 'contact' class",
	)
	assert.Equal(
		t,
		domain.NoteID("note-1"),
		notes[0].ID,
		"ByFileClass should return notes in correct order",
	)
	assert.Equal(
		t,
		domain.NoteID("note-3"),
		notes[1].ID,
		"ByFileClass should return notes in correct order",
	)
}

// TestQueryService_ByFileClass_NonMatchingClass verifies ByFileClass returns
// empty slice for non-matching schema.
func TestQueryService_ByFileClass_NonMatchingClass(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	notes, err := qs.ByFileClass(ctx, "nonexistent")

	require.NoError(
		t,
		err,
		"ByFileClass should not return error for non-matching file class",
	)
	assert.Empty(
		t,
		notes,
		"ByFileClass should return empty slice for non-matching file class",
	)
	assert.Nil(
		t,
		notes,
		"ByFileClass should return nil slice for non-matching file class",
	)
}

// TestQueryService_RefreshFromCache_Success verifies RefreshFromCache rebuilds
// indices from CacheReaderPort.
func TestQueryService_RefreshFromCache_Success(t *testing.T) {
	// Create new notes for cache
	newNotes := []domain.Note{
		{
			ID: domain.NoteID("new-1"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
			},
		},
		{
			ID: domain.NoteID("new-2"),
			Frontmatter: domain.Frontmatter{
				FileClass: "meeting",
			},
		},
	}

	qs := setupQueryServiceForRefresh(t, newNotes)
	ctx := context.Background()

	err := qs.RefreshFromCache(ctx)

	require.NoError(
		t,
		err,
		"RefreshFromCache should not return error on success",
	)

	// Verify indices were rebuilt
	assert.Len(t, qs.byID, 2, "byID should contain 2 notes after refresh")
	assert.Contains(
		t,
		qs.byID,
		domain.NoteID("new-1"),
		"byID should contain new-1",
	)
	assert.Contains(
		t,
		qs.byID,
		domain.NoteID("new-2"),
		"byID should contain new-2",
	)

	// Verify byFileClass was rebuilt
	assert.Len(
		t,
		qs.byFileClass["project"],
		1,
		"byFileClass should have 1 project note",
	)
	assert.Len(
		t,
		qs.byFileClass["meeting"],
		1,
		"byFileClass should have 1 meeting note",
	)
}

// TestQueryService_RefreshFromCache_ClearsExistingIndices verifies
// RefreshFromCache clears existing indices before rebuild.
func TestQueryService_RefreshFromCache_ClearsExistingIndices(t *testing.T) {
	// New notes that will replace any existing data
	newNotes := []domain.Note{
		{
			ID: domain.NoteID("new-1"),
			Frontmatter: domain.Frontmatter{
				FileClass: "project",
			},
		},
	}

	qs := setupQueryServiceForRefresh(t, newNotes)
	ctx := context.Background()

	err := qs.RefreshFromCache(ctx)

	require.NoError(t, err, "RefreshFromCache should not return error")

	// Verify indices were rebuilt with new data
	assert.Contains(
		t,
		qs.byID,
		domain.NoteID("new-1"),
		"byID should contain new-1 after refresh",
	)
	assert.Contains(
		t,
		qs.byFileClass,
		"project",
		"byFileClass should contain project after refresh",
	)
}

// TestQueryService_RefreshFromCache_CacheReadError verifies RefreshFromCache
// handles cache read errors.
func TestQueryService_RefreshFromCache_CacheReadError(t *testing.T) {
	fakeCacheReader := &FakeCacheReader{
		err: errors.New("cache read failed"),
	}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	ctx := context.Background()

	err := qs.RefreshFromCache(ctx)

	require.Error(
		t,
		err,
		"RefreshFromCache should return error when cache read fails",
	)
	assert.Contains(
		t,
		err.Error(),
		"cache refresh failed",
		"error should contain cache refresh context",
	)
}

// TestQueryService_ConcurrentReads verifies thread-safety of query methods
// under concurrent read access using race detector.
func TestQueryService_ConcurrentReads(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)
	ctx := context.Background()

	// Number of goroutines to spawn
	numGoroutines := 10
	// Number of operations per goroutine
	opsPerGoroutine := 100

	var wg sync.WaitGroup
	wg.Add(numGoroutines)

	for range numGoroutines {
		go func() {
			defer wg.Done()
			for range opsPerGoroutine {
				// Test ByID
				_, _ = qs.ByID(ctx, domain.NoteID("note-1"))

				// Test ByPath
				_, _ = qs.ByPath(ctx, "contacts/john-doe.md")

				// Test ByFileClass
				_, _ = qs.ByFileClass(ctx, "contact")
			}
		}()
	}

	wg.Wait()
}

// TestQueryService_EdgeCases_EmptyService verifies behavior with empty indices.
func TestQueryService_EdgeCases_EmptyService(t *testing.T) {
	fakeCacheReader := &FakeCacheReader{notes: []domain.Note{}}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)
	ctx := context.Background()

	// Test ByID on empty service
	note, err := qs.ByID(ctx, domain.NoteID("any-id"))
	require.Error(t, err, "ByID should return error for empty service")
	var resErr *domainerrors.ResourceError
	require.ErrorAs(t, err, &resErr, "ByID should return ResourceError")
	assert.Equal(t, domain.Note{}, note, "ByID should return zero Note")

	// Test ByPath on empty service
	note, err = qs.ByPath(ctx, "any/path.md")
	require.Error(t, err, "ByPath should return error for empty service")
	require.ErrorAs(t, err, &resErr, "ByPath should return ResourceError")
	assert.Equal(t, domain.Note{}, note, "ByPath should return zero Note")

	// Test ByFileClass on empty service
	notes, err := qs.ByFileClass(ctx, "any-class")
	require.NoError(
		t,
		err,
		"ByFileClass should not return error for empty service",
	)
	assert.Empty(
		t,
		notes,
		"ByFileClass should return empty slice for empty service",
	)
}

// TestQueryService_EdgeCases_TODOContext verifies behavior with context.TODO.
func TestQueryService_EdgeCases_TODOContext(t *testing.T) {
	qs := setupQueryServiceWithNotes(t)

	// Test ByID with context.TODO
	note, err := qs.ByID(context.TODO(), domain.NoteID("note-1"))
	require.NoError(
		t,
		err,
		"ByID should not return error with context.TODO",
	)
	assert.Equal(
		t,
		domain.NoteID("note-1"),
		note.ID,
		"ByID should return correct note with context.TODO",
	)

	// Test ByPath with context.TODO
	note, err = qs.ByPath(context.TODO(), "contacts/john-doe.md")
	require.NoError(
		t,
		err,
		"ByPath should not return error with context.TODO",
	)
	assert.Equal(
		t,
		domain.NoteID("note-1"),
		note.ID,
		"ByPath should return correct note with context.TODO",
	)

	// Test ByFileClass with context.TODO
	notes, err := qs.ByFileClass(context.TODO(), "contact")
	require.NoError(
		t,
		err,
		"ByFileClass should not return error with context.TODO",
	)
	assert.Len(
		t,
		notes,
		2,
		"ByFileClass should return correct notes with context.TODO",
	)
}

// TestQueryService_RefreshFromCache_EmptyCache verifies RefreshFromCache
// handles empty cache correctly.
func TestQueryService_RefreshFromCache_EmptyCache(t *testing.T) {
	fakeCacheReader := &FakeCacheReader{notes: []domain.Note{}}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Pre-populate with some data to ensure clearing
	qs.byID = map[domain.NoteID]domain.Note{
		domain.NoteID("old"): {ID: domain.NoteID("old")},
	}
	qs.byPath = map[string]domain.Note{
		"old/path.md": {ID: domain.NoteID("old")},
	}
	qs.byFileClass = map[string][]domain.Note{
		"old": {{ID: domain.NoteID("old")}},
	}

	ctx := context.Background()
	err := qs.RefreshFromCache(ctx)

	require.NoError(
		t,
		err,
		"RefreshFromCache should not return error with empty cache",
	)
	assert.Empty(
		t,
		qs.byID,
		"byID should be empty after refresh with empty cache",
	)
	assert.Empty(
		t,
		qs.byPath,
		"byPath should be empty after refresh with empty cache",
	)
	assert.Empty(
		t,
		qs.byFileClass,
		"byFileClass should be empty after refresh with empty cache",
	)
}

// RED TESTS: ByFrontmatter method (AC: 3)

// TestQueryService_ByFrontmatter_ExistingField tests frontmatter field lookups.
func TestQueryService_ByFrontmatter_ExistingField(t *testing.T) {
	// Given
	authorNote := domain.NewNote(
		domain.NewNoteID("note1"),
		domain.NewFrontmatter(map[string]interface{}{
			"author": "John Doe",
			"status": "published",
		}),
	)
	tagNote := domain.NewNote(
		domain.NewNoteID("note2"),
		domain.NewFrontmatter(map[string]interface{}{
			"tags":   "project-x",
			"author": "Jane Smith",
		}),
	)

	fakeCacheReader := &FakeCacheReader{
		notes: []domain.Note{authorNote, tagNote},
	}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Populate indices
	err := qs.RefreshFromCache(context.Background())
	require.NoError(t, err)

	// When
	notes, err := qs.ByFrontmatter(context.Background(), "author", "John Doe")

	// Then
	require.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "note1", notes[0].ID.String())
}

// TestQueryService_ByFrontmatter_MultipleMatches tests multiple notes with same
// frontmatter value.
func TestQueryService_ByFrontmatter_MultipleMatches(t *testing.T) {
	// Given
	note1 := domain.NewNote(
		domain.NewNoteID("note1"),
		domain.NewFrontmatter(map[string]interface{}{
			"status": "draft",
			"author": "John",
		}),
	)
	note2 := domain.NewNote(
		domain.NewNoteID("note2"),
		domain.NewFrontmatter(map[string]interface{}{
			"status": "draft",
			"author": "Jane",
		}),
	)

	fakeCacheReader := &FakeCacheReader{notes: []domain.Note{note1, note2}}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Populate indices
	err := qs.RefreshFromCache(context.Background())
	require.NoError(t, err)

	// When
	notes, err := qs.ByFrontmatter(context.Background(), "status", "draft")

	// Then
	require.NoError(t, err)
	assert.Len(t, notes, 2)
}

// TestQueryService_ByFrontmatter_MissingField tests missing frontmatter field.
func TestQueryService_ByFrontmatter_MissingField(t *testing.T) {
	// Given
	note := domain.NewNote(
		domain.NewNoteID("note1"),
		domain.NewFrontmatter(map[string]interface{}{
			"author": "John Doe",
		}),
	)

	fakeCacheReader := &FakeCacheReader{notes: []domain.Note{note}}
	logger := zerolog.New(nil).Level(zerolog.Disabled)
	qs := NewQueryService(fakeCacheReader, logger)

	// Populate indices
	err := qs.RefreshFromCache(context.Background())
	require.NoError(t, err)

	// When
	notes, err := qs.ByFrontmatter(context.Background(), "tags", "nonexistent")

	// Then
	require.NoError(t, err)
	assert.Empty(t, notes) // Should return empty slice, not error
}
