package dto

import (
	"io/fs"
	"testing"
	"time"
)

// mockFileInfoForDates implements fs.FileInfo for testing.
type mockFileInfoForDates struct {
	name    string
	size    int64
	mode    fs.FileMode
	modTime time.Time
	isDir   bool
}

func (m mockFileInfoForDates) Name() string       { return m.name }
func (m mockFileInfoForDates) Size() int64        { return m.size }
func (m mockFileInfoForDates) Mode() fs.FileMode  { return m.mode }
func (m mockFileInfoForDates) ModTime() time.Time { return m.modTime }
func (m mockFileInfoForDates) IsDir() bool        { return m.isDir }
func (m mockFileInfoForDates) Sys() any           { return nil }

func TestNewFileDatesDTO(t *testing.T) {
	modifiedAt := time.Date(2025, 11, 20, 10, 0, 0, 0, time.UTC)

	// Since NewFileDatesDTO sets IndexedAt to time.Now(), we can't check exact
	// equality easily
	// but we can check it's recent.
	before := time.Now()
	dto := NewFileDatesDTO(modifiedAt)
	after := time.Now()

	if !dto.ModifiedAt.Equal(modifiedAt) {
		t.Errorf("ModifiedAt = %v, want %v", dto.ModifiedAt, modifiedAt)
	}

	if dto.IndexedAt.Before(before) || dto.IndexedAt.After(after) {
		t.Errorf(
			"IndexedAt = %v, want between %v and %v",
			dto.IndexedAt,
			before,
			after,
		)
	}
}

func TestFromVaultFile(t *testing.T) {
	modifiedAt := time.Date(2025, 11, 20, 10, 0, 0, 0, time.UTC)
	info := mockFileInfoForDates{modTime: modifiedAt}
	vf := VaultFile{Info: info}

	before := time.Now()
	dto := FromVaultFile(vf)
	after := time.Now()

	if !dto.ModifiedAt.Equal(modifiedAt) {
		t.Errorf("ModifiedAt = %v, want %v", dto.ModifiedAt, modifiedAt)
	}

	if dto.IndexedAt.Before(before) || dto.IndexedAt.After(after) {
		t.Errorf(
			"IndexedAt = %v, want between %v and %v",
			dto.IndexedAt,
			before,
			after,
		)
	}
}

func TestFileDatesDTO_IsStale(t *testing.T) {
	baseTime := time.Date(2025, 11, 20, 10, 0, 0, 0, time.UTC)

	tests := []struct {
		name       string
		modifiedAt time.Time
		indexedAt  time.Time
		wantStale  bool
	}{
		{
			name:       "fresh: modified before indexed",
			modifiedAt: baseTime,
			indexedAt:  baseTime.Add(1 * time.Minute),
			wantStale:  false,
		},
		{
			name:       "stale: modified after indexed",
			modifiedAt: baseTime.Add(1 * time.Minute),
			indexedAt:  baseTime,
			wantStale:  true,
		},
		{
			name:       "fresh: modified equals indexed (edge case)",
			modifiedAt: baseTime,
			indexedAt:  baseTime,
			wantStale:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dto := FileDatesDTO{
				ModifiedAt: tt.modifiedAt,
				IndexedAt:  tt.indexedAt,
			}
			if got := dto.IsStale(); got != tt.wantStale {
				t.Errorf("IsStale() = %v, want %v", got, tt.wantStale)
			}
		})
	}
}
