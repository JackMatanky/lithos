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
