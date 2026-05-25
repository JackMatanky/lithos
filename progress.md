
# Progress Tracking

- [x] Initialized plan and findings files.
- [x] Analyzed usages: Found several unused variants in `SchemaStorageError` and `SchemaRepositoryError`.
- [x] Refactored `SchemaRepositoryError` to wrap `DbError` directly.
- [x] Removed `SchemaStorageError`.
- [x] Refined `SchemaRepositoryError` with typed `NotFound` variants and `EmptyVersionHistory`.
- [x] Updated all callers in `lithos-core`.
- [x] Verified with tests: All schema storage and loader tests passed.
- [x] Task completed successfully.
