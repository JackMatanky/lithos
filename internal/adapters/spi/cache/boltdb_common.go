package cache

// boltDBFileMode represents the POSIX file permissions used when creating the
// BoltDB database. `0o600` ensures only the current user can read or write the
// cache contents.
const boltDBFileMode = 0o600
