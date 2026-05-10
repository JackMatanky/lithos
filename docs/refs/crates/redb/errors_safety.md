# redb Error Handling & Safety

Source: https://docs.rs/redb/latest/redb/enum.DatabaseError.html

## Error Hierarchy
- `DatabaseError`: General failures opening or creating the database.
- `TableError`: Failures accessing tables (e.g., `TableDoesNotExist` when opening a missing table).
- `StorageError`: Low-level I/O or corruption errors.
