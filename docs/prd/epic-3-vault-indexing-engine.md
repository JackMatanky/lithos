# Epic 3: Vault Indexing Engine

This epic focuses on building the core data layer of Lithos. It will scan the user's vault, parse frontmatter, and build a persistent cache that will power all future dynamic and lookup-based features.

## Story 3.1: Define the Storage Interface and Note Struct

As a developer, I want to define a `Storage` interface and a `Note` struct, so that I have a clear, typed contract for index data.

### Acceptance Criteria

- 3.1.1: A `storage/storage.go` file contains the `Storage` interface with `Get` and `Set` methods.
- 3.1.2: It also contains the `Note` struct with `FilePath`, `FileBasename`, and `Frontmatter` fields.
- 3.1.3: The interface and struct are clearly documented.

## Story 3.2: Implement a File-Based Cache

As a developer, I want to create a file-based implementation of the `Storage` interface using JSON, so that I have a simple persistent cache.

### Acceptance Criteria

- 3.2.1: A `FileCache` struct is created that implements the `Storage` interface.
- 3.2.2: The `Set` method serializes the `Note` struct to a JSON file in a `.lithos/cache` directory.
- 3.2.3: The `Get` method deserializes the JSON file back into a `Note` struct.
- 3.2.4: File I/O errors are handled gracefully.

## Story 3.3: Implement the Vault Scanner Utility

As a developer, I want a utility to find all Markdown files in the vault, so that I have a list of files to index.

### Acceptance Criteria

- 3.3.1: An `indexer/scanner.go` file contains a `ScanVault(vaultPath string)` function.
- 3.3.2: The function returns a slice of absolute paths to all `.md` files.
- 3.3.3: It correctly ignores the `.lithos` cache directory.

## Story 3.4: Implement the Frontmatter Parser Utility

As a developer, I want a utility to read a Markdown file and extract its YAML frontmatter, so that I can get the metadata for each note.

### Acceptance Criteria

- 3.4.1: An `indexer/parser.go` file contains a `ParseFrontmatter(filePath string)` function.
- 3.4.2: It uses `adrg/frontmatter` to extract the frontmatter.
- 3.4.3: It returns the frontmatter as a `map[string]interface{}`.
- 3.4.4: It handles files with no frontmatter or invalid YAML gracefully.

## Story 3.5: Implement the `index` Command Logic

As a user, I want to run `lithos index`, so that I can build or update the vault index.

### Acceptance Criteria

- 3.5.1: The `index` subcommand is added to Cobra.
- 3.5.2: It uses the `ScanVault` and `ParseFrontmatter` utilities to process each note.
- 3.5.3: It saves each parsed note to the `FileCache`.
- 3.5.4: It logs indexing progress at an INFO level.

## Story 3.6: Version and Migrate Cache Artifacts

As a developer, I want the `.lithos` cache to advertise a schema version and handle migrations, so that index changes do not break existing installs.

### Acceptance Criteria

- 3.6.1: Cached metadata written to `.lithos/` includes a semantic version field for the cache schema.
- 3.6.2: At startup, the indexing workflow detects mismatched cache versions and runs a migration or falls back to a clean rebuild with helpful logs.
- 3.6.3: Migration steps and rollback guidance are documented in the architecture or README so operators know how to recover from failures.
