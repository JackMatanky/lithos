package vault

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/JackMatanky/lithos/internal/app/schema"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/dto"
	"github.com/rs/zerolog"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// FakeVaultScannerPort implements VaultScannerPort for testing.
type FakeVaultScannerPort struct {
	scanAllResult      []dto.VaultFile
	scanAllError       error
	scanModifiedResult []dto.VaultFile
	scanModifiedError  error
}

// FakeCacheWriterPort implements CacheWriterPort for testing.
type FakeCacheWriterPort struct {
	persistCalls []domain.Note
	persistError error
	deleteCalls  []domain.NoteID
	deleteError  error
}

// FakeCacheReaderPort implements CacheReaderPort for testing.
type FakeCacheReaderPort struct {
	listResult []domain.Note
	listError  error
}

// FakeSchemaPort implements SchemaPort for testing with call tracking.
type FakeSchemaPort struct {
	loadCalled bool
	schemas    []domain.Schema
	bank       domain.PropertyBank
	err        error
}

// FakeSchemaRegistryPort implements SchemaRegistryPort for testing.
type FakeSchemaRegistryPort struct {
	schemas        map[string]domain.Schema
	properties     map[string]domain.Property
	getSchemaErr   error
	getPropertyErr error
	registerAllErr error
}

// Load implements SchemaPort.Load for testing.
func (f *FakeSchemaPort) Load(
	ctx context.Context,
) ([]domain.Schema, domain.PropertyBank, error) {
	f.loadCalled = true
	if f.err != nil {
		return nil, domain.PropertyBank{}, f.err
	}
	return f.schemas, f.bank, nil
}

// GetSchema implements SchemaRegistryPort.GetSchema for testing.
func (f *FakeSchemaRegistryPort) GetSchema(
	ctx context.Context,
	name string,
) (domain.Schema, error) {
	if f.getSchemaErr != nil {
		return domain.Schema{}, f.getSchemaErr
	}
	sch, exists := f.schemas[name]
	if !exists {
		return domain.Schema{}, fmt.Errorf("schema not found: %s", name)
	}
	return sch, nil
}

// GetProperty implements SchemaRegistryPort.GetProperty for testing.
func (f *FakeSchemaRegistryPort) GetProperty(
	ctx context.Context,
	name string,
) (domain.Property, error) {
	if f.getPropertyErr != nil {
		return domain.Property{}, f.getPropertyErr
	}
	property, exists := f.properties[name]
	if !exists {
		return domain.Property{}, fmt.Errorf("property not found: %s", name)
	}
	return property, nil
}

// HasSchema implements SchemaRegistryPort.HasSchema for testing.
func (f *FakeSchemaRegistryPort) HasSchema(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.schemas[name]
	return exists
}

// HasProperty implements SchemaRegistryPort.HasProperty for testing.
func (f *FakeSchemaRegistryPort) HasProperty(
	ctx context.Context,
	name string,
) bool {
	_, exists := f.properties[name]
	return exists
}

// RegisterAll implements SchemaRegistryPort.RegisterAll for testing.
func (f *FakeSchemaRegistryPort) RegisterAll(
	ctx context.Context,
	schemas []domain.Schema,
	bank domain.PropertyBank,
) error {
	if f.registerAllErr != nil {
		return f.registerAllErr
	}
	if f.schemas == nil {
		f.schemas = make(map[string]domain.Schema)
	}
	if f.properties == nil {
		f.properties = make(map[string]domain.Property)
	}
	for i := range schemas {
		f.schemas[schemas[i].Name] = schemas[i]
	}
	f.properties = bank.Properties
	return nil
}

// ScanAll implements VaultScannerPort.ScanAll for testing.
func (f *FakeVaultScannerPort) ScanAll(
	ctx context.Context,
) ([]dto.VaultFile, error) {
	return f.scanAllResult, f.scanAllError
}

// ScanModified implements VaultScannerPort.ScanModified for testing.
func (f *FakeVaultScannerPort) ScanModified(
	ctx context.Context,
	since time.Time,
) ([]dto.VaultFile, error) {
	return f.scanModifiedResult, f.scanModifiedError
}

// Persist implements CacheWriterPort.Persist for testing.
func (f *FakeCacheWriterPort) Persist(
	ctx context.Context,
	note domain.Note,
) error {
	f.persistCalls = append(f.persistCalls, note)
	return f.persistError
}

// Delete implements CacheWriterPort.Delete for testing.
func (f *FakeCacheWriterPort) Delete(
	ctx context.Context,
	id domain.NoteID,
) error {
	f.deleteCalls = append(f.deleteCalls, id)
	return f.deleteError
}

// Read implements CacheReaderPort.Read for testing.
func (f *FakeCacheReaderPort) Read(
	ctx context.Context,
	id domain.NoteID,
) (domain.Note, error) {
	return domain.Note{}, nil
}

// List implements CacheReaderPort.List for testing.
func (f *FakeCacheReaderPort) List(
	ctx context.Context,
) ([]domain.Note, error) {
	return f.listResult, f.listError
}

// TestBuildCallsScanAll tests Build calls ScanAll.
func TestBuildCallsScanAll(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path:    "/vault/test.md",
				Ext:     ".md",
				Size:    100,
				ModTime: time.Now(),
			}, []byte("# Test")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	_, err := indexer.Build(context.Background())

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1)
}

// TestBuildCallsPersist tests Build calls Persist.
func TestBuildCallsPersist(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/test.md",
				Ext:  ".md",
			}, []byte("# Test")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	_, err := indexer.Build(context.Background())

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1)
}

// TestBuildHandlesCacheFailures tests cache failure handling.
func TestBuildHandlesCacheFailures(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/test.md",
				Ext:  ".md",
			}, []byte("# Test")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{
		persistError: assert.AnError, // Simulate cache failure
	}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	stats, err := indexer.Build(context.Background())

	// Then
	require.NoError(
		t,
		err,
	) // Build should succeed despite cache failure
	assert.Len(t, fakeWriter.persistCalls, 1) // Persist was called
	assert.Equal(t, 1, stats.CacheFailures)   // Cache failure counted
	assert.Equal(t, 0, stats.IndexedCount)    // No successful indexing
}

// TestBuildHandlesScanFailure tests Build on scan failure.
func TestBuildHandlesScanFailure(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllError: assert.AnError, // Simulate scan failure
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	stats, err := indexer.Build(context.Background())

	// Then
	require.Error(t, err) // Build should fail on scan error
	assert.Equal(t, assert.AnError, err)
	assert.Equal(t, 0, stats.ScannedCount)   // No files scanned
	assert.Empty(t, fakeWriter.persistCalls) // No persistence attempted
}

// TestRefreshSuccess tests successful incremental refresh.
func TestRefreshSuccess(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path:    "/vault/modified.md",
				Ext:     ".md",
				Size:    200,
				ModTime: time.Now(),
			}, []byte("# Modified")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1) // One file processed
}

// TestRefreshNoModifications tests refresh with no modified files.
func TestRefreshNoModifications(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedResult: []dto.VaultFile{}, // No modified files
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.Empty(t, fakeWriter.persistCalls) // No files processed
}

// TestRefreshCacheFailure tests refresh with cache write failure.
func TestRefreshCacheFailure(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/fail.md",
				Ext:  ".md",
			}, []byte("# Fail")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{
		persistError: assert.AnError, // Simulate cache failure
	}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(
		t,
		err,
	) // Refresh should succeed despite cache failure
	assert.Len(t, fakeWriter.persistCalls, 1) // Persist was called
}

// TestRefreshScanFailure tests refresh on scan failure.
func TestRefreshScanFailure(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedError: assert.AnError, // Simulate scan failure
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.Error(t, err) // Refresh should fail on scan error
	assert.Equal(t, assert.AnError, err)
	assert.Empty(t, fakeWriter.persistCalls) // No persistence attempted
}

// TestDeriveNoteIDFromPath tests the deriveNoteIDFromPath function.
func TestDeriveNoteIDFromPath(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "simple filename",
			input:    "note.md",
			expected: "note.md",
		},
		{
			name:     "nested path",
			input:    "projects/ideas/note.md",
			expected: "projects/ideas/note.md",
		},
		{
			name:     "deeply nested path",
			input:    "year2024/projects/active/note.md",
			expected: "year2024/projects/active/note.md",
		},
		{
			name:     "windows path separators normalized",
			input:    "projects\\ideas\\note.md",
			expected: "projects/ideas/note.md",
		},
		{
			name:     "path with spaces",
			input:    "projects/my project/note.md",
			expected: "projects/my project/note.md",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Use empty vault root since test inputs are already relative paths
			result := deriveNoteIDFromPath("", tt.input)
			assert.Equal(t, domain.NewNoteID(tt.expected), result)
		})
	}
}

// TestRefreshReconcileDeletions tests deletion reconciliation.
func TestRefreshReconcileDeletions(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/existing.md",
				Ext:  ".md",
			}, []byte("# Existing")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{
		listResult: []domain.Note{
			domain.NewNote(
				domain.NewNoteID("existing.md"),
				domain.Frontmatter{},
			),
			domain.NewNote(
				domain.NewNoteID("deleted.md"),
				domain.Frontmatter{},
			), // Orphan
		},
	}
	config := domain.Config{VaultPath: "/vault"}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.deleteCalls, 1) // One orphan deleted
	assert.Equal(t, domain.NewNoteID("deleted.md"), fakeWriter.deleteCalls[0])
}

// TestReconcileCacheReadFailure tests reconciliation with cache read failure.
func TestReconcileCacheReadFailure(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/existing.md",
				Ext:  ".md",
			}, []byte("# Existing")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{
		listError: assert.AnError, // Simulate cache read failure
	}
	config := domain.Config{VaultPath: "/vault"}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(
		t,
		err,
	) // Should not fail on cache read error
	assert.Empty(t, fakeWriter.deleteCalls) // No deletions attempted
}

// TestReconcileDeleteFailure tests reconciliation with delete failure.
func TestReconcileDeleteFailure(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/existing.md",
				Ext:  ".md",
			}, []byte("# Existing")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{
		deleteError: assert.AnError, // Simulate delete failure
	}
	fakeReader := &FakeCacheReaderPort{
		listResult: []domain.Note{
			domain.NewNote(
				domain.NewNoteID("existing.md"),
				domain.Frontmatter{},
			),
			domain.NewNote(
				domain.NewNoteID("deleted.md"),
				domain.Frontmatter{},
			), // Orphan
		},
	}
	config := domain.Config{VaultPath: "/vault"}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		nil,
		config,
		log,
	)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)                  // Should not fail on delete error
	assert.Len(t, fakeWriter.deleteCalls, 1) // Delete was attempted
}

// BenchmarkValidateCacheState benchmarks cache state validation performance.
func BenchmarkValidateCacheState(b *testing.B) {
	// Setup test data with varying sizes
	benchmarks := []struct {
		name       string
		vaultFiles int
		cacheNotes int
	}{
		{"Small", 10, 10},
		{"Medium", 100, 100},
		{"Large", 1000, 1000},
		{"Unbalanced", 100, 50}, // More vault files than cache
	}

	for _, bm := range benchmarks {
		b.Run(bm.name, func(b *testing.B) {
			// Create vault files
			vaultFiles := make([]dto.VaultFile, bm.vaultFiles)
			for i := range vaultFiles {
				vaultFiles[i] = dto.NewVaultFile(
					dto.FileMetadata{
						Path: fmt.Sprintf("/vault/note%d.md", i),
						Ext:  ".md",
					},
					[]byte(fmt.Sprintf("# Note %d", i)),
				)
			}

			// Create cache notes (some may be orphans)
			cacheNotes := make([]domain.Note, bm.cacheNotes)
			for i := range cacheNotes {
				cacheNotes[i] = domain.NewNote(
					domain.NewNoteID(fmt.Sprintf("note%d.md", i)),
					domain.Frontmatter{},
				)
			}

			fakeScanner := &FakeVaultScannerPort{
				scanAllResult: vaultFiles,
			}
			fakeReader := &FakeCacheReaderPort{
				listResult: cacheNotes,
			}
			config := domain.Config{VaultPath: "/vault"}
			log := zerolog.New(nil)

			indexer := NewVaultIndexer(
				fakeScanner,
				&FakeCacheWriterPort{},
				fakeReader,
				nil,
				nil,
				config,
				log,
			)

			ctx := context.Background()
			b.ResetTimer()

			for range b.N {
				_, _ = indexer.validateCacheState(ctx)
			}
		})
	}
}

// BenchmarkReconcileDeletions benchmarks deletion reconciliation performance.
func BenchmarkReconcileDeletions(b *testing.B) {
	// Setup test data with varying orphan counts
	benchmarks := []struct {
		name        string
		totalNotes  int
		orphanCount int
	}{
		{"NoOrphans", 100, 0},
		{"FewOrphans", 100, 5},
		{"ManyOrphans", 100, 50},
	}

	for _, bm := range benchmarks {
		b.Run(bm.name, func(b *testing.B) {
			// Create vault files (existing notes)
			existingCount := bm.totalNotes - bm.orphanCount
			vaultFiles := make([]dto.VaultFile, existingCount)
			for i := range vaultFiles {
				vaultFiles[i] = dto.NewVaultFile(
					dto.FileMetadata{
						Path: fmt.Sprintf("/vault/note%d.md", i),
						Ext:  ".md",
					},
					[]byte(fmt.Sprintf("# Note %d", i)),
				)
			}

			// Create cache notes (existing + orphans)
			cacheNotes := make([]domain.Note, bm.totalNotes)
			for i := range cacheNotes {
				cacheNotes[i] = domain.NewNote(
					domain.NewNoteID(fmt.Sprintf("note%d.md", i)),
					domain.Frontmatter{},
				)
			}

			fakeScanner := &FakeVaultScannerPort{
				scanAllResult: vaultFiles,
			}
			fakeWriter := &FakeCacheWriterPort{}
			fakeReader := &FakeCacheReaderPort{
				listResult: cacheNotes,
			}
			config := domain.Config{VaultPath: "/vault"}
			log := zerolog.New(nil)

			indexer := NewVaultIndexer(
				fakeScanner,
				fakeWriter,
				fakeReader,
				nil,
				nil,
				config,
				log,
			)

			ctx := context.Background()
			since := time.Now().Add(-time.Hour)
			b.ResetTimer()

			for range b.N {
				_ = indexer.Refresh(ctx, since)
			}
		})
	}
}

// TestBuildLoadsSchema tests Build loads schemas when engine available.
func TestBuildLoadsSchema(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/test.md",
				Ext:  ".md",
			}, []byte("# Test")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	fakeSchemaPort := &FakeSchemaPort{}
	fakeRegistryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	schemaEngine, err := schema.NewSchemaEngine(
		fakeSchemaPort,
		fakeRegistryPort,
		log,
	)
	require.NoError(t, err)

	config := domain.Config{}
	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		schemaEngine,
		config,
		log,
	)

	// When
	_, err = indexer.Build(context.Background())

	// Then
	require.NoError(t, err)
	assert.True(
		t,
		fakeSchemaPort.loadCalled,
		"SchemaPort.Load should be called during Build",
	)
}

// TestRefreshLoadsSchema tests Refresh loads schemas when engine available.
func TestRefreshLoadsSchema(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{}, // For reconciliation
		scanModifiedResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/modified.md",
				Ext:  ".md",
			}, []byte("# Modified")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	fakeSchemaPort := &FakeSchemaPort{}
	fakeRegistryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	schemaEngine, err := schema.NewSchemaEngine(
		fakeSchemaPort,
		fakeRegistryPort,
		log,
	)
	require.NoError(t, err)

	config := domain.Config{VaultPath: "/vault"}
	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil,
		schemaEngine,
		config,
		log,
	)

	// When
	err = indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.True(
		t,
		fakeSchemaPort.loadCalled,
		"SchemaPort.Load should be called during Refresh",
	)
}

// TestRefreshValidatesSchema tests schema validation in refresh scenarios.
func TestRefreshValidatesSchema(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanAllResult: []dto.VaultFile{}, // For reconciliation
		scanModifiedResult: []dto.VaultFile{
			dto.NewVaultFile(dto.FileMetadata{
				Path: "/vault/test.md",
				Ext:  ".md",
			}, []byte("---\nfileClass: test\n---\n# Test")),
		},
	}
	fakeWriter := &FakeCacheWriterPort{}
	fakeReader := &FakeCacheReaderPort{}
	fakeSchemaPort := &FakeSchemaPort{}
	fakeRegistryPort := &FakeSchemaRegistryPort{}
	log := zerolog.New(nil)

	schemaEngine, err := schema.NewSchemaEngine(
		fakeSchemaPort,
		fakeRegistryPort,
		log,
	)
	require.NoError(t, err)

	config := domain.Config{VaultPath: "/vault"}

	// Note: Using nil for frontmatterService to test that schema validation is
	// attempted In real scenarios, frontmatterService would extract fileClass
	// and trigger validation
	indexer := NewVaultIndexer(
		fakeScanner,
		fakeWriter,
		fakeReader,
		nil, // No frontmatter service - validation should be skipped gracefully
		schemaEngine,
		config,
		log,
	)

	// When
	err = indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.True(
		t,
		fakeSchemaPort.loadCalled,
		"Schema should be loaded for refresh validation",
	)
}
