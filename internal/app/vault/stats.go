package vault

import "time"

// IndexStats tracks metrics for vault indexing operations.
// Used for performance monitoring and NFR3 compliance.
//
// Fields:
// - ScannedCount: Total files scanned from vault
// - IndexedCount: Notes successfully persisted to cache
// - CacheFailures: Cache write errors (logged as warnings)
// - ValidationSuccesses: Frontmatter validations that passed
// - ValidationFailures: Frontmatter validations that failed (logged but don't
// abort)
// - Duration: Total indexing time for performance tracking.
type IndexStats struct {
	ScannedCount        int
	IndexedCount        int
	CacheFailures       int
	ValidationSuccesses int
	ValidationFailures  int
	Duration            time.Duration
}

// CacheValidationResult contains detailed results of cache state validation.
// Used to verify cache-vault consistency and identify synchronization issues.
//
// Fields:
// - TotalVaultFiles: Number of .md files found in vault
// - TotalCacheEntries: Number of entries found in cache
// - OrphanedCacheFiles: Cache entries without corresponding vault files
// - MissingCacheFiles: Vault files without corresponding cache entries
// - OrphanedDetails: List of orphaned cache entry NoteIDs
// - MissingDetails: List of missing cache entry NoteIDs
// - IsConsistent: True if cache perfectly matches vault state.
type CacheValidationResult struct {
	TotalVaultFiles    int
	TotalCacheEntries  int
	OrphanedCacheFiles int
	MissingCacheFiles  int
	OrphanedDetails    []string
	MissingDetails     []string
	IsConsistent       bool
}
