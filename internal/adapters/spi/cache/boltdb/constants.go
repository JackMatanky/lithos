package boltdb

// BoltDB bucket names.
const (
	// BucketNotes is the primary bucket for storing Note data.
	// Key: Vault-relative path (e.g. "notes/meeting.md")
	// Value: CachedNote JSON (including FileDatesDTO).
	BucketNotes = "notes"

	// BucketIndices is the parent bucket for secondary indices.
	BucketIndices = "indices"

	// BucketIndexByBasename is the secondary index: Basename -> []Path.
	BucketIndexByBasename = "byBasename"

	// BucketIndexByAlias is the secondary index: Alias -> []Path.
	BucketIndexByAlias = "byAlias"

	// BucketIndexByFileClass is the secondary index: FileClass -> []Path.
	BucketIndexByFileClass = "byFileClass"

	// BucketIndexByFolder is the secondary index: Folder -> []Path (for
	// folder-scoped PathQuery).
	BucketIndexByFolder = "byFolder"
)

// boltDBFileMode represents the POSIX file permissions used when creating
// BoltDB database files. Uses restrictive permissions (0600) for security.
const boltDBFileMode = 0o600
