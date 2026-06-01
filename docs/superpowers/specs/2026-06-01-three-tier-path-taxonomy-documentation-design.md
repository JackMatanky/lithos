# Design Spec: Three-Tier Path Taxonomy Documentation

Capture the completed path taxonomy as an ADR and in module-level doc comments so the design is durable and navigable without architecture tests.

## 1. ADR Creation
- **File**: `docs/adr/020-three-tier-path-taxonomy.md`
- **Template**: `docs/adr/template.md`
- **Content**:
    - **Context**: The migration from `RelativePath` (struct) → `RelativePath` (enum), config types tightened to `RelativeDirPath`/`RelativeFilePath`, `PathKey` established as sole repository boundary type, `NormalizedPath` removed.
    - **Decision**: Formalize the three-tier path taxonomy with clear boundary rules and conversion seams.
    - **Taxonomy Table**:
        | Tier | Types | Where | Example |
        |------|-------|-------|---------|
        | **Filesystem I/O** | `FsPath`, `FilePath`, `DirPath` | Scanner, reader, writer, vault processor | `DirPath::append_file(&rel_file)` |
        | **Display / Config** | `RelativePath` enum, `Relative*Path` | CLI display, config values, serialization | `RelativeDirPath::try_new("schemas")` |
        | **Storage Keys** | `PathKey` | Repository traits, DB tables | `fn find_file_view_by_path(&PathKey)` |
    - **Conversion Seams Table**:
        | Source → Target | Method | Fallible? |
        |----------------|--------|-----------|
        | Config value → FS path | `DirPath::append_dir(&RelativeDirPath)` | Yes |
        | Config value → FS path | `DirPath::append_file(&RelativeFilePath)` | Yes |
        | FS path → Storage key | `file_path.as_key(root)` | Yes |
        | FS path → Display | `file_path.as_relative(base) → RelativePath::File(...)` | Yes |
        | FS path → Display | `dir_path.as_relative(base) → RelativePath::Dir(...)` | Yes |
    - **Consequences**:
        - **Positive**: Type system enforces correct usage; developers consult the taxonomy table rather than CI failures. Improved security via explicit scoping.
        - **Negative**: Increased number of types; requires explicit conversion at boundaries.

## 2. Module Doc Updates

### `lithos-core/src/fs/path.rs`
- **Goal**: Provide a comprehensive guide to the path system.
- **Content**:
    - taxonomy table (as above).
    - Conversion seams reference.
    - Guidance on when to use each tier.
    - Specific documentation for `RelativePath` enum emphasizing its use for display and serialization.

### `lithos-core/src/config/paths.rs`
- **Goal**: Clarify the role of declarative paths in config.
- **Content**:
    - Note that config stores declarative types (`RelativeDirPath`/`RelativeFilePath`) rather than filesystem-validated or storage-key types.
    - Explain that these types are used for serialization and as input to the FS tier.

## 3. Context Alignment

### `lithos-core/src/fs/CONTEXT.md`
- **Update**: "Normalized Path" entry.
- **Refinement**: Reference the taxonomy and clarify the role of `PathKey` vs `FsPath`.

### `lithos-core/src/config/CONTEXT.md`
- **Update**: Add note about config using `RelativeDirPath`/`RelativeFilePath` for declarative path values.

## 4. Verification Plan
- **Doc generation**: `cargo doc --no-deps` (must have 0 warnings).
- **Doc tests**: `cargo test --doc` (verify all examples).
- **Sanity check**: `mise run test:unit`.
