package cache

// BoltDB bucket names for structured data storage.
const (
	bucketPaths       = "paths"
	bucketBasenames   = "basenames"
	bucketAliases     = "aliases"
	bucketFileClasses = "file_classes"
	bucketDirectories = "directories"
	bucketStaleness   = "staleness"
)

// boltDBFileMode represents the POSIX file permissions used when creating
// BoltDB database files. Uses restrictive permissions (0600) for security.
const boltDBFileMode = 0o600
