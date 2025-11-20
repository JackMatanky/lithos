package dto

import (
	"time"
)

// FileDatesDTO handles timestamp tracking and staleness detection for cache
// adapters.
// It separates filesystem modification times from cache indexing times.
type FileDatesDTO struct {
	ModifiedAt time.Time // File's last modification time from fs.FileInfo
	IndexedAt  time.Time // When the note was cached/indexed
}

// NewFileDatesDTO creates a new FileDatesDTO from a modification time.
// It sets IndexedAt to the current time.
func NewFileDatesDTO(modifiedAt time.Time) FileDatesDTO {
	return FileDatesDTO{
		ModifiedAt: modifiedAt,
		IndexedAt:  time.Now(),
	}
}

// FromVaultFile creates a new FileDatesDTO from a VaultFile.
func FromVaultFile(vf VaultFile) FileDatesDTO {
	return NewFileDatesDTO(vf.ModifiedAt())
}

// IsStale checks if the file has been modified since it was last indexed.
// Returns true if ModifiedAt is after IndexedAt.
func (d FileDatesDTO) IsStale() bool {
	return d.ModifiedAt.After(d.IndexedAt)
}
