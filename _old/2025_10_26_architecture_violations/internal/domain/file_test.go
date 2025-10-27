package domain

import (
	"testing"
	"time"
)

const testFileBasename = "file"

func TestNewFile(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		modTime  time.Time
		expected File
	}{
		{
			name:    "basic file",
			path:    "/vault/notes/contact.md",
			modTime: time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC),
			expected: File{
				Path:     "/vault/notes/contact.md",
				Basename: "contact",
				Folder:   "/vault/notes",
				ModTime:  time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC),
			},
		},
		{
			name:    "nested directory",
			path:    "/home/user/docs/project/readme.txt",
			modTime: time.Date(2024, 2, 15, 10, 30, 0, 0, time.UTC),
			expected: File{
				Path:     "/home/user/docs/project/readme.txt",
				Basename: "readme",
				Folder:   "/home/user/docs/project",
				ModTime:  time.Date(2024, 2, 15, 10, 30, 0, 0, time.UTC),
			},
		},
		{
			name:    "no extension",
			path:    "/vault/notes/draft",
			modTime: time.Date(2024, 3, 10, 14, 20, 0, 0, time.UTC),
			expected: File{
				Path:     "/vault/notes/draft",
				Basename: "draft",
				Folder:   "/vault/notes",
				ModTime:  time.Date(2024, 3, 10, 14, 20, 0, 0, time.UTC),
			},
		},
		{
			name:    "multiple extensions",
			path:    "/vault/notes/archive.tar.gz",
			modTime: time.Date(2024, 4, 5, 16, 45, 0, 0, time.UTC),
			expected: File{
				Path:     "/vault/notes/archive.tar.gz",
				Basename: "archive.tar",
				Folder:   "/vault/notes",
				ModTime:  time.Date(2024, 4, 5, 16, 45, 0, 0, time.UTC),
			},
		},
		{
			name:    "root directory",
			path:    "/readme.md",
			modTime: time.Date(2024, 5, 20, 9, 15, 0, 0, time.UTC),
			expected: File{
				Path:     "/readme.md",
				Basename: "readme",
				Folder:   "/",
				ModTime:  time.Date(2024, 5, 20, 9, 15, 0, 0, time.UTC),
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewFile(tt.path, tt.modTime)
			if result != tt.expected {
				t.Errorf("NewFile() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func TestComputeBasename(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected string
	}{
		{
			name:     "simple file",
			path:     "/vault/notes/contact.md",
			expected: "contact",
		},
		{
			name:     "no extension",
			path:     "/vault/notes/draft",
			expected: "draft",
		},
		{
			name:     "multiple extensions",
			path:     "/vault/notes/archive.tar.gz",
			expected: "archive.tar",
		},
		{
			name:     "hidden file",
			path:     "/vault/.obsidian/config",
			expected: "config",
		},
		{
			name:     "just filename",
			path:     "readme.md",
			expected: "readme",
		},
		{
			name:     "empty path",
			path:     "",
			expected: "",
		},
		{
			name:     "just extension",
			path:     ".md",
			expected: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := computeBasename(tt.path)
			if result != tt.expected {
				t.Errorf(
					"computeBasename(%q) = %q, want %q",
					tt.path,
					result,
					tt.expected,
				)
			}
		})
	}
}

func TestComputeFolder(t *testing.T) {
	tests := []struct {
		name     string
		path     string
		expected string
	}{
		{
			name:     "nested path",
			path:     "/vault/notes/contact.md",
			expected: "/vault/notes",
		},
		{
			name:     "root file",
			path:     "/readme.md",
			expected: "/",
		},
		{
			name:     "deep nesting",
			path:     "/home/user/docs/projects/active/project.md",
			expected: "/home/user/docs/projects/active",
		},
		{
			name:     "just filename",
			path:     "readme.md",
			expected: ".",
		},
		{
			name:     "empty path",
			path:     "",
			expected: ".",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := computeFolder(tt.path)
			if result != tt.expected {
				t.Errorf(
					"computeFolder(%q) = %q, want %q",
					tt.path,
					result,
					tt.expected,
				)
			}
		})
	}
}

func TestFileStruct(t *testing.T) {
	// Test that File struct can be created and accessed
	modTime := time.Now()
	file := File{
		Path:     "/test/path/file.md",
		Basename: testFileBasename,
		Folder:   "/test/path",
		ModTime:  modTime,
	}

	if file.Path != "/test/path/file.md" {
		t.Errorf("Path = %q, want %q", file.Path, "/test/path/file.md")
	}
	if file.Basename != testFileBasename {
		t.Errorf("Basename = %q, want %q", file.Basename, testFileBasename)
	}
	if file.Folder != "/test/path" {
		t.Errorf("Folder = %q, want %q", file.Folder, "/test/path")
	}
	if !file.ModTime.Equal(modTime) {
		t.Errorf("ModTime = %v, want %v", file.ModTime, modTime)
	}
}
