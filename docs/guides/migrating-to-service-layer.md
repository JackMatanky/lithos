# Migrating to Service Layer Pattern

This guide explains how to implement file ingestion for new domain contexts in Lithos, following the architectural pattern established in ADR 010.

## Overview

The Service Layer pattern separates the concerns of file I/O, parsing, validation, and database persistence. This ensures that domain logic remains pure and storage-agnostic, while making the entire pipeline highly testable and performant.

### Ingestion Pipeline

```text
File System → FileSource trait → Parsers → Raw* → Domain (TryFrom) → CQRS Ports → Database
            (abstraction)      (fs/)     (serde) (validation)   (db-only)
```

## Step-by-Step Implementation

When adding a new domain context (e.g., "workspace"), follow these steps:

### 1. Create Raw Input Model

Define a `Raw*` struct in `<context>/raw.rs` that matches the file format (JSON/TOML/YAML). Use `serde` for deserialization.

```rust
#[derive(Debug, serde::Deserialize)]
pub struct RawWorkspace {
    pub id: Uuid,
    pub name: String,
    // ... other fields
}
```

### 2. Implement Domain Conversion

Implement `TryFrom<Raw*>` for your domain aggregate. This is the **validation boundary** where you enforce business invariants.

```rust
impl TryFrom<RawWorkspace> for Workspace {
    type Error = WorkspaceError;

    fn try_from(raw: RawWorkspace) -> Result<Self, Self::Error> {
        // Enforce rules (e.g., non-empty name, valid UUID)
        Self::new(raw.id, &raw.name)
    }
}
```

### 3. Create Ingestion Service

Create a specialized service in `lithos-core/src/application/services/` that orchestrates the pipeline.

```rust
pub struct WorkspaceIngestionService<'svc, Q, C> {
    query: &'svc Query<Q>,
    command: &'svc Command<C>,
}

impl<'svc, Q, C> WorkspaceIngestionService<'svc, Q, C>
where
    Q: ports::Query,
    C: ports::Command,
{
    pub fn ingest_file<S>(&self, source: &S, path: &Path) -> Result<Uuid, IngestionError>
    where S: FileSource<Error = io::Error>
    {
        // 1. Parse (uses generic helper)
        let raw: RawWorkspace = fs::parsers::parse_file(source, path)?;

        // 2. Validate
        let workspace = Workspace::try_from(raw)?;

        // 3. Persist
        self.command.save(&workspace)?;

        Ok(workspace.id())
    }
}
```

### 4. Keep Ports Pure

Ensure your Query and Command ports **never** import `std::fs` or have methods like `load_from_file`. They should only deal with database operations using domain types.

## Anti-Patterns to Avoid

- ❌ **File I/O in Ports**: Do not add `load_from_file()` to CQRS ports.
- ❌ **Direct std::fs**: Do not call `std::fs` directly in domain or adapter code. Use `FileSource`.
- ❌ **Bypassing Raw Types**: Do not deserialize directly into domain aggregates. Use the `Raw*` → `TryFrom` pattern.
- ❌ **Context Cross-Imports**: Ingestion services for one context must not import another context's domain logic.

## Testing

Always test your ingestion services using `InMemoryFileSource` and fake CQRS ports to ensure the logic works without requiring a real filesystem or database.

```rust
#[test]
fn test_workspace_ingestion() {
    let mut source = InMemoryFileSource::new();
    source.insert(Path::new("w.json"), r#"{"id": "...", "name": "test"}"#.to_owned());

    let service = WorkspaceIngestionService::new(&query, &command);
    let id = service.ingest_file(&source, Path::new("w.json")).unwrap();

    assert!(query.find_by_id(id).unwrap().is_some());
}
```
