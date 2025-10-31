package vault

import (
	"context"
	"testing"
	"time"

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
	return nil
}

// TestVaultIndexer_Build_CallsVaultScannerScanAll tests Build calls ScanAll.
func TestVaultIndexer_Build_CallsVaultScannerScanAll(t *testing.T) {
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
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	_, err := indexer.Build(context.Background())

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1)
}

// TestVaultIndexer_Build_CallsCacheWriterPersist tests Build calls Persist.
func TestVaultIndexer_Build_CallsCacheWriterPersist(t *testing.T) {
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
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	_, err := indexer.Build(context.Background())

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1)
}

// TestVaultIndexer_Build_HandlesCacheWriteFailures tests cache failure
// handling.
func TestVaultIndexer_Build_HandlesCacheWriteFailures(t *testing.T) {
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
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

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

// TestVaultIndexer_Build_HandlesVaultScanFailure tests Build on scan failure.
func TestVaultIndexer_Build_HandlesVaultScanFailure(t *testing.T) {
	// Given
	fakeScanner := &FakeVaultScannerPort{
		scanAllError: assert.AnError, // Simulate scan failure
	}
	fakeWriter := &FakeCacheWriterPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	stats, err := indexer.Build(context.Background())

	// Then
	require.Error(t, err) // Build should fail on scan error
	assert.Equal(t, assert.AnError, err)
	assert.Equal(t, 0, stats.ScannedCount)   // No files scanned
	assert.Empty(t, fakeWriter.persistCalls) // No persistence attempted
}

// TestVaultIndexer_Refresh_Success tests successful incremental refresh.
func TestVaultIndexer_Refresh_Success(t *testing.T) {
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
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.Len(t, fakeWriter.persistCalls, 1) // One file processed
}

// TestVaultIndexer_Refresh_NoModifications tests refresh with no modified
// files.
func TestVaultIndexer_Refresh_NoModifications(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedResult: []dto.VaultFile{}, // No modified files
	}
	fakeWriter := &FakeCacheWriterPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(t, err)
	assert.Empty(t, fakeWriter.persistCalls) // No files processed
}

// TestVaultIndexer_Refresh_CacheFailure tests refresh with cache write failure.
func TestVaultIndexer_Refresh_CacheFailure(t *testing.T) {
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
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.NoError(
		t,
		err,
	) // Refresh should succeed despite cache failure
	assert.Len(t, fakeWriter.persistCalls, 1) // Persist was called
}

// TestVaultIndexer_Refresh_ScanFailure tests refresh on scan failure.
func TestVaultIndexer_Refresh_ScanFailure(t *testing.T) {
	// Given
	since := time.Now().Add(-time.Hour)
	fakeScanner := &FakeVaultScannerPort{
		scanModifiedError: assert.AnError, // Simulate scan failure
	}
	fakeWriter := &FakeCacheWriterPort{}
	config := domain.Config{}
	log := zerolog.New(nil)

	indexer := NewVaultIndexer(fakeScanner, fakeWriter, config, log)

	// When
	err := indexer.Refresh(context.Background(), since)

	// Then
	require.Error(t, err) // Refresh should fail on scan error
	assert.Equal(t, assert.AnError, err)
	assert.Empty(t, fakeWriter.persistCalls) // No persistence attempted
}
