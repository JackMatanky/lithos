package performance

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/JackMatanky/lithos/internal/adapters/spi/vault"
	"github.com/JackMatanky/lithos/internal/domain"
	"github.com/JackMatanky/lithos/internal/shared/logger"
)

// setupLargeTestVault creates a test vault with mixed file types for
// performance testing.
// Includes markdown files, binary files, and cache directories.
func setupLargeTestVault(
	t *testing.T,
	vaultPath string,
	numMarkdown, numBinaries int,
) {
	t.Helper()

	// Create vault structure
	requireNoError(t, os.MkdirAll(filepath.Join(vaultPath, "notes"), 0o750))
	requireNoError(
		t,
		os.MkdirAll(filepath.Join(vaultPath, "attachments"), 0o750),
	)
	requireNoError(
		t,
		os.MkdirAll(filepath.Join(vaultPath, ".lithos", "cache"), 0o750),
	)
	requireNoError(
		t,
		os.MkdirAll(
			filepath.Join(vaultPath, "projects", ".lithos", "temp"),
			0o750,
		),
	)

	// Create markdown files
	markdownContent := `# Test Note

This is a test markdown file for performance testing.

## Section 1

Some content here.

## Section 2

More content.

### Subsection

Even more content to make the file larger.

` + string(make([]byte, 1024)) + `

End of file.
`

	for i := range numMarkdown {
		filename := filepath.Join(
			vaultPath,
			"notes",
			fmt.Sprintf("note-%03d.md", i),
		)
		requireNoError(
			t,
			os.WriteFile(
				filename,
				[]byte(fmt.Sprintf("# Note %d\n\n%s", i, markdownContent)),
				0o600,
			),
		)
	}

	// Create binary files of various sizes
	for i := range numBinaries {
		var content []byte
		switch i % 3 {
		case 0:
			// Small binary (1KB)
			content = make([]byte, 1024)
			copy(content, "fake png content small")
			filename := filepath.Join(
				vaultPath,
				"attachments",
				fmt.Sprintf("image-small-%03d.png", i),
			)
			requireNoError(t, os.WriteFile(filename, content, 0o600))
		case 1:
			// Medium binary (100KB)
			content = make([]byte, 100*1024)
			copy(content, "fake pdf content medium")
			filename := filepath.Join(
				vaultPath,
				"attachments",
				fmt.Sprintf("doc-medium-%03d.pdf", i),
			)
			requireNoError(t, os.WriteFile(filename, content, 0o600))
		case 2:
			// Large binary (1MB)
			content = make([]byte, 1024*1024)
			copy(content, "fake video content large")
			filename := filepath.Join(
				vaultPath,
				"attachments",
				fmt.Sprintf("video-large-%03d.mp4", i),
			)
			requireNoError(t, os.WriteFile(filename, content, 0o600))
		}
	}

	// Create cache files
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, ".lithos", "cache", "cached-index.json"),
		[]byte(`{"cached": true, "timestamp": "2025-01-01T00:00:00Z"}`),
		0o600,
	))
	requireNoError(t, os.WriteFile(
		filepath.Join(
			vaultPath,
			"projects",
			".lithos",
			"temp",
			"temp-data.dat",
		),
		[]byte("temporary cache data"),
		0o600,
	))
}

// requireNoError is a helper for tests.
func requireNoError(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
}

// BenchmarkVaultScanning_SmallVault benchmarks scanning a small vault with 10
// markdown + 10 binaries.
func BenchmarkVaultScanning_SmallVault(b *testing.B) {
	vaultPath := b.TempDir()
	setupLargeTestVault(&testing.T{}, vaultPath, 10, 10)

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	b.ResetTimer()
	b.ReportAllocs()

	for range b.N {
		files, err := adapter.ScanAll(context.Background())
		if err != nil {
			b.Fatal(err)
		}
		// Verify we only get markdown files
		if len(files) != 10 {
			b.Fatalf("Expected 10 markdown files, got %d", len(files))
		}
	}
}

// BenchmarkVaultScanning_MediumVault benchmarks scanning a medium vault with
// 100 markdown + 50 binaries.
func BenchmarkVaultScanning_MediumVault(b *testing.B) {
	vaultPath := b.TempDir()
	setupLargeTestVault(&testing.T{}, vaultPath, 100, 50)

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	b.ResetTimer()
	b.ReportAllocs()

	for range b.N {
		files, err := adapter.ScanAll(context.Background())
		if err != nil {
			b.Fatal(err)
		}
		// Verify we only get markdown files
		if len(files) != 100 {
			b.Fatalf("Expected 100 markdown files, got %d", len(files))
		}
	}
}

// BenchmarkVaultScanning_LargeVault benchmarks scanning a large vault with 1000
// markdown + 200 binaries.
func BenchmarkVaultScanning_LargeVault(b *testing.B) {
	vaultPath := b.TempDir()
	setupLargeTestVault(&testing.T{}, vaultPath, 1000, 200)

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	b.ResetTimer()
	b.ReportAllocs()

	for range b.N {
		files, err := adapter.ScanAll(context.Background())
		if err != nil {
			b.Fatal(err)
		}
		// Verify we only get markdown files
		if len(files) != 1000 {
			b.Fatalf("Expected 1000 markdown files, got %d", len(files))
		}
	}
}

// BenchmarkVaultScanning_CacheExclusion benchmarks cache directory exclusion
// performance.
func BenchmarkVaultScanning_CacheExclusion(b *testing.B) {
	vaultPath := b.TempDir()
	setupLargeTestVault(&testing.T{}, vaultPath, 100, 50)

	// Add many cache files
	for i := range 100 {
		if err := os.WriteFile(
			filepath.Join(vaultPath, ".lithos", "cache", fmt.Sprintf("cache-%03d.json", i)),
			[]byte(fmt.Sprintf(`{"cache": true, "id": %d}`, i)),
			0o600,
		); err != nil {
			b.Fatal(err)
		}
	}

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	b.ResetTimer()
	b.ReportAllocs()

	for range b.N {
		files, err := adapter.ScanAll(context.Background())
		if err != nil {
			b.Fatal(err)
		}
		// Should still only get 100 markdown files, cache files excluded
		if len(files) != 100 {
			b.Fatalf("Expected 100 markdown files, got %d", len(files))
		}
	}
}

// TestVaultScanning_MemoryUsage validates that large binary files don't consume
// memory.
func TestVaultScanning_MemoryUsage(t *testing.T) {
	vaultPath := t.TempDir()

	// Create vault with large binary files that should be excluded
	requireNoError(t, os.MkdirAll(filepath.Join(vaultPath, "binaries"), 0o750))
	requireNoError(t, os.MkdirAll(filepath.Join(vaultPath, "notes"), 0o750))

	// Create a 50MB binary file that should NOT be loaded
	largeBinary := make([]byte, 50*1024*1024) // 50MB
	for i := range largeBinary {
		largeBinary[i] = byte(i % 256)
	}
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "binaries", "large-binary.dat"),
		largeBinary,
		0o600,
	))

	// Create a markdown file
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "notes", "test.md"),
		[]byte("# Test\n\nThis is a test."),
		0o600,
	))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())
	requireNoError(t, err)

	// Should only get the markdown file
	if len(files) != 1 {
		t.Fatalf("Expected 1 markdown file, got %d", len(files))
	}

	// Verify the file is the markdown one
	if files[0].Basename != "test" {
		t.Errorf("Expected basename 'test', got '%s'", files[0].Basename)
	}
}

// TestVaultScanning_SizeLimits validates that files over the size limit are
// skipped.
func TestVaultScanning_SizeLimits(t *testing.T) {
	vaultPath := t.TempDir()

	// Create a markdown file that's too large (15MB > 10MB limit)
	largeMarkdown := make([]byte, 15*1024*1024) // 15MB
	copy(largeMarkdown, "# Large Markdown\n\n")
	for i := 20; i < len(largeMarkdown); i++ {
		largeMarkdown[i] = byte(i % 256)
	}
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "large.md"),
		largeMarkdown,
		0o600,
	))

	// Create a normal markdown file
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "normal.md"),
		[]byte("# Normal\n\nThis is normal."),
		0o600,
	))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())
	requireNoError(t, err)

	// Should only get the normal file, large one should be skipped
	if len(files) != 1 {
		t.Fatalf("Expected 1 file (normal.md), got %d", len(files))
	}

	if files[0].Basename != "normal" {
		t.Errorf("Expected basename 'normal', got '%s'", files[0].Basename)
	}
}

// TestVaultScanning_CacheDirectoryExclusion validates cache directory
// exclusion.
func TestVaultScanning_CacheDirectoryExclusion(t *testing.T) {
	vaultPath := t.TempDir()

	// Create vault structure with nested cache directories
	requireNoError(t, os.MkdirAll(filepath.Join(vaultPath, "notes"), 0o750))
	requireNoError(
		t,
		os.MkdirAll(filepath.Join(vaultPath, ".lithos", "cache"), 0o750),
	)
	requireNoError(
		t,
		os.MkdirAll(
			filepath.Join(vaultPath, "projects", ".lithos", "temp"),
			0o750,
		),
	)

	// Create markdown files in different locations
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "notes", "note1.md"),
		[]byte("# Note 1\n\nContent"),
		0o600,
	))
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "projects", "note2.md"),
		[]byte("# Note 2\n\nContent"),
		0o600,
	))

	// Create files in cache directories (should be excluded)
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, ".lithos", "cache", "cached.md"),
		[]byte("# Cached\n\nShould be excluded"),
		0o600,
	))
	requireNoError(t, os.WriteFile(
		filepath.Join(vaultPath, "projects", ".lithos", "temp", "temp.md"),
		[]byte("# Temp\n\nShould be excluded"),
		0o600,
	))

	config := domain.NewConfig(vaultPath, "", "", "", "", "")
	adapter := vault.NewVaultReaderAdapter(config, logger.NewTest())

	files, err := adapter.ScanAll(context.Background())
	requireNoError(t, err)

	// Should only get the 2 legitimate markdown files
	if len(files) != 2 {
		t.Fatalf("Expected 2 files, got %d", len(files))
	}

	// Verify basenames
	basenames := make(map[string]bool)
	for _, file := range files {
		basenames[file.Basename] = true
	}

	if !basenames["note1"] || !basenames["note2"] {
		t.Errorf("Expected basenames 'note1' and 'note2', got %v", basenames)
	}

	if basenames["cached"] || basenames["temp"] {
		t.Error("Cache files should have been excluded")
	}
}
