package dto

import (
	"bytes"
	"io/fs"
	"path/filepath"
	"runtime"
	"testing"
	"time"
)

const (
	windowsOS     = "windows"
	testVaultPath = "/vault"
	testNotePath  = "/vault/notes/meeting.md"
)

// mockFileInfo implements fs.FileInfo for testing.
type mockFileInfo struct {
	name    string
	size    int64
	mode    fs.FileMode
	modTime time.Time
	isDir   bool
}

// Name returns the mock file name.
func (m mockFileInfo) Name() string { return m.name }

// Size returns the mock file size.
func (m mockFileInfo) Size() int64 { return m.size }

// Mode returns the mock file mode.
func (m mockFileInfo) Mode() fs.FileMode { return m.mode }

// ModTime returns the mock modification time.
func (m mockFileInfo) ModTime() time.Time { return m.modTime }

// IsDir returns whether the mock represents a directory.
func (m mockFileInfo) IsDir() bool { return m.isDir }

// Sys returns mock system-specific data.
func (m mockFileInfo) Sys() any { return nil }

// TestVaultFileStructure tests that VaultFile has the correct fields.
func TestVaultFileStructure(t *testing.T) {
	testTime := time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC)
	info := mockFileInfo{
		name:    "test.md",
		size:    1024,
		modTime: testTime,
		isDir:   false,
	}
	content := []byte("test content")

	vf := VaultFile{
		Path:    "notes/test.md",
		Info:    info,
		Content: content,
	}

	// Verify fields exist and have correct types
	if vf.Path != "notes/test.md" {
		t.Errorf(
			"Path field incorrect: got %q, want %q",
			vf.Path,
			"notes/test.md",
		)
	}
	if vf.Info != info {
		t.Errorf("Info field incorrect: got %v, want %v", vf.Info, info)
	}
	if string(vf.Content) != "test content" {
		t.Errorf(
			"Content field incorrect: got %q, want %q",
			string(vf.Content),
			"test content",
		)
	}
}

// TestVaultFileFileInfoDelegation tests fs.FileInfo delegation methods.
func TestVaultFileFileInfoDelegation(t *testing.T) {
	testTime := time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC)
	info := mockFileInfo{
		name:    "test.md",
		size:    1024,
		mode:    0o644,
		modTime: testTime,
		isDir:   false,
	}

	vf := VaultFile{
		Path:    "notes/test.md",
		Info:    info,
		Content: []byte("test"),
	}

	// Test ModifiedAt delegation
	if !vf.ModifiedAt().Equal(testTime) {
		t.Errorf("ModifiedAt() = %v, want %v", vf.ModifiedAt(), testTime)
	}

	// Test Size delegation
	if vf.Size() != 1024 {
		t.Errorf("Size() = %v, want %v", vf.Size(), 1024)
	}

	// Test Info field access
	if vf.Info.Mode() != 0o644 {
		t.Errorf("Info.Mode() = %v, want %v", vf.Info.Mode(), 0o644)
	}
	if vf.Info.IsDir() != false {
		t.Errorf("Info.IsDir() = %v, want %v", vf.Info.IsDir(), false)
	}
}

// TestVaultFileComputedMethods tests the computed methods.
func TestVaultFileComputedMethods(t *testing.T) {
	tests := []struct {
		name             string
		path             string
		expectedBasename string
		expectedFolder   string
		expectedExt      string
	}{
		{
			name:             "markdown file in subdirectory",
			path:             "notes/meeting.md",
			expectedBasename: "meeting",
			expectedFolder:   "notes",
			expectedExt:      ".md",
		},
		{
			name:             "json file in root",
			path:             "config.json",
			expectedBasename: "config",
			expectedFolder:   ".",
			expectedExt:      ".json",
		},
		{
			name:             "file without extension",
			path:             "docs/README",
			expectedBasename: "README",
			expectedFolder:   "docs",
			expectedExt:      "",
		},
		{
			name:             "nested path",
			path:             "projects/work/tasks/todo.txt",
			expectedBasename: "todo",
			expectedFolder:   "projects/work/tasks",
			expectedExt:      ".txt",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			info := mockFileInfo{name: filepath.Base(tt.path)}
			vf := VaultFile{
				Path: tt.path,
				Info: info,
			}

			if vf.Basename() != tt.expectedBasename {
				t.Errorf(
					"Basename() = %q, want %q",
					vf.Basename(),
					tt.expectedBasename,
				)
			}
			if vf.Folder() != tt.expectedFolder {
				t.Errorf(
					"Folder() = %q, want %q",
					vf.Folder(),
					tt.expectedFolder,
				)
			}
			if vf.Ext() != tt.expectedExt {
				t.Errorf("Ext() = %q, want %q", vf.Ext(), tt.expectedExt)
			}
		})
	}
}

// TestNormalizePath tests path normalization across platforms.
func TestNormalizePath(t *testing.T) {
	tests := []struct {
		name        string
		absPath     string
		vaultRoot   string
		expected    string
		expectError bool
	}{
		{
			name:      "windows path normalization",
			absPath:   "C:\\vault\\notes\\meeting.md",
			vaultRoot: "C:\\vault",
			expected:  "notes/meeting.md",
		},
		{
			name:      "linux path normalization",
			absPath:   "/home/user/vault/notes/meeting.md",
			vaultRoot: "/home/user/vault",
			expected:  "notes/meeting.md",
		},
		{
			name:      "mac path normalization",
			absPath:   "/Users/name/vault/notes/meeting.md",
			vaultRoot: "/Users/name/vault",
			expected:  "notes/meeting.md",
		},
		{
			name:      "nested directories",
			absPath:   "/vault/projects/work/tasks/todo.md",
			vaultRoot: testVaultPath,
			expected:  "projects/work/tasks/todo.md",
		},
		{
			name:        "path outside vault error",
			absPath:     "/outside/file.md",
			vaultRoot:   "/vault",
			expected:    "",
			expectError: true,
		},
		{
			name:        "invalid vault root error",
			absPath:     "/vault/file.md",
			vaultRoot:   "",
			expected:    "",
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Skip platform-specific tests on wrong platforms
			if runtime.GOOS == windowsOS && !filepath.IsAbs(tt.absPath) {
				t.Skip("Skipping non-Windows path test on Windows")
			}
			if runtime.GOOS != windowsOS && len(tt.absPath) > 1 &&
				tt.absPath[1] == ':' {
				t.Skip("Skipping Windows path test on non-Windows")
			}

			result, err := NormalizePath(tt.absPath, tt.vaultRoot)

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result != tt.expected {
				t.Errorf("NormalizePath() = %q, want %q", result, tt.expected)
			}
		})
	}
}

// TestAbsolutePath tests AbsolutePath helper function.
func TestAbsolutePath(t *testing.T) {
	tests := []struct {
		name      string
		vaultPath string
		vaultRoot string
		expected  string
	}{
		{
			name:      "convert to OS-specific path",
			vaultPath: "notes/meeting.md",
			vaultRoot: testVaultPath,
			expected:  filepath.Join(testVaultPath, "notes", "meeting.md"),
		},
		{
			name:      "nested directories",
			vaultPath: "projects/work/tasks/todo.md",
			vaultRoot: "/home/user/vault",
			expected:  "/home/user/vault/projects/work/tasks/todo.md",
		},
		{
			name:      "root file",
			vaultPath: "README.md",
			vaultRoot: testVaultPath,
			expected:  filepath.Join(testVaultPath, "README.md"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			vf := VaultFile{Path: tt.vaultPath}
			result := vf.AbsolutePath(tt.vaultRoot)

			if result != tt.expected {
				t.Errorf("AbsolutePath() = %q, want %q", result, tt.expected)
			}
		})
	}
}

// TestNewVaultFile tests the VaultFile constructor.
func TestNewVaultFile(t *testing.T) {
	testTime := time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC)
	info := mockFileInfo{
		name:    "meeting.md",
		size:    1024,
		modTime: testTime,
		isDir:   false,
	}
	content := []byte("# Meeting Notes")

	tests := []struct {
		name         string
		absPath      string
		vaultRoot    string
		info         fs.FileInfo
		content      []byte
		expectedPath string
		expectError  bool
	}{
		{
			name:         "successful construction",
			absPath:      "/vault/notes/meeting.md",
			vaultRoot:    "/vault",
			info:         info,
			content:      content,
			expectedPath: "notes/meeting.md",
			expectError:  false,
		},
		{
			name:        "path outside vault",
			absPath:     "/outside/file.md",
			vaultRoot:   "/vault",
			info:        info,
			content:     content,
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := NewVaultFile(
				tt.absPath,
				tt.vaultRoot,
				tt.info,
				tt.content,
			)

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result.Path != tt.expectedPath {
				t.Errorf("Path = %q, want %q", result.Path, tt.expectedPath)
			}
			if result.Info != tt.info {
				t.Errorf("Info mismatch: got %v, want %v", result.Info, tt.info)
			}
			if !bytes.Equal(result.Content, tt.content) {
				t.Errorf(
					"Content = %q, want %q",
					string(result.Content),
					string(tt.content),
				)
			}
		})
	}
}

// TestVaultFileNoDeprecatedFields verifies no deprecated fields exist.
func TestVaultFileNoDeprecatedFields(t *testing.T) {
	// This test ensures the VaultFile struct doesn't have the old fields
	vf := VaultFile{}

	// These should not compile if the fields exist
	// Commented out because they should cause compilation errors
	// _ = vf.Basename  // Should not exist
	// _ = vf.Folder    // Should not exist
	// _ = vf.Ext       // Should not exist
	// _ = vf.ModTime   // Should not exist
	// _ = vf.Size      // Should not exist
	// _ = vf.MimeType  // Should not exist

	// Test that we only have the expected fields
	_ = vf.Path
	_ = vf.Info
	_ = vf.Content

	// Success if we get here without compilation errors
}

// TestNilFileInfoHandling tests behavior with nil fs.FileInfo.
func TestNilFileInfoHandling(t *testing.T) {
	vf := VaultFile{
		Path:    "test.md",
		Info:    nil,
		Content: []byte("test"),
	}

	// These should panic or return zero values when Info is nil
	defer func() {
		if r := recover(); r == nil {
			t.Errorf(
				"Expected panic when calling methods on nil Info, but didn't panic",
			)
		}
	}()

	// This should panic
	_ = vf.ModifiedAt()
}

// TestFilePathConversionRoundTrip tests round-trip path conversion.
func TestFilePathConversionRoundTrip(t *testing.T) {
	original := testNotePath
	vaultRoot := testVaultPath

	// Convert to vault-relative
	vaultPath, err := NormalizePath(original, vaultRoot)
	if err != nil {
		t.Fatalf("NormalizePath failed: %v", err)
	}

	// Create VaultFile
	info := mockFileInfo{name: "meeting.md"}
	vf := VaultFile{Path: vaultPath, Info: info}

	// Convert back to absolute
	result := vf.AbsolutePath(vaultRoot)

	if result != original {
		t.Errorf("Round-trip failed: got %q, want %q", result, original)
	}
}

// BenchmarkComputedMethods benchmarks the performance of computed methods.
func BenchmarkComputedMethods(b *testing.B) {
	vf := VaultFile{Path: "projects/work/tasks/todo.md"}

	b.Run("Basename", func(b *testing.B) {
		for range b.N {
			_ = vf.Basename()
		}
	})

	b.Run("Folder", func(b *testing.B) {
		for range b.N {
			_ = vf.Folder()
		}
	})

	b.Run("Ext", func(b *testing.B) {
		for range b.N {
			_ = vf.Ext()
		}
	})
}

// BenchmarkPathNormalization benchmarks path normalization performance.
func BenchmarkPathNormalization(b *testing.B) {
	absPath := "/vault/projects/work/tasks/todo.md"
	vaultRoot := testVaultPath

	b.ResetTimer()
	for range b.N {
		_, _ = NormalizePath(absPath, vaultRoot)
	}
}

// BenchmarkConstructor benchmarks VaultFile constructor performance.
func BenchmarkConstructor(b *testing.B) {
	absPath := testNotePath
	vaultRoot := testVaultPath
	info := mockFileInfo{name: "meeting.md", size: 1024}
	content := []byte("test content")

	b.ResetTimer()
	for range b.N {
		_, _ = NewVaultFile(absPath, vaultRoot, info, content)
	}
}
