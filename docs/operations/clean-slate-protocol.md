# Clean-Slate Reindex Protocol

## Purpose

When rkyv format changes or corruption is detected, the database must be rebuilt from source markdown files.

## Prerequisites

- Vault markdown files are the **source of truth**
- `.lithos/` directory contains only **derived data** (can be safely deleted)
- All notes, schemas, and templates exist as files in the vault

## Procedure

### Manual Steps

1. **Backup current state**:
   ```bash
   cp -r vault/.lithos "vault/.lithos.backup.$(date +%Y%m%d_%H%M%S)"
   ```

2. **Clear database**:
   ```bash
   rm -rf vault/.lithos/*.redb
   ```

3. **Reindex**:
   ```bash
   lithos index --vault vault/
   ```

4. **Verify**:
   ```bash
   lithos verify --vault vault/
   ```

### When to Use

**Required**:
- After upgrading Lithos version with rkyv format changes
- When `DbError::CorruptData` is encountered during operations
- After restoring vault from backup (filesystem-level restore)

**Optional (Repair)**:
- When query results seem inconsistent with file contents
- After bulk file operations outside Lithos (git merge, mass rename)
- When troubleshooting indexing issues

## What Gets Rebuilt

The reindex process recreates:

- **Note index**: All markdown files parsed and indexed
- **Schema index**: Schema definitions compiled and validated
- **Template index**: Templates parsed and validated
- **Task index**: Tasks extracted from notes with metadata
- **Tag index**: All tags extracted and mapped
- **Link index**: All wiki-links and references mapped

## Safety Guarantees

- **Non-Destructive**: Original markdown files are never modified
- **Idempotent**: Running reindex multiple times produces same result
- **Atomic**: Partial reindex is not visible (uses temp database)

## Performance

Typical reindex times (commodity hardware):

| Vault Size | Note Count | Time     |
|------------|------------|----------|
| Small      | < 100      | < 5s     |
| Medium     | 100-1000   | 5-30s    |
| Large      | 1000-10000 | 30-300s  |
| Very Large | > 10000    | 5-30min  |

## Future Automation (TBD)

Planned improvements:

- `lithos migrate --clean-slate`: Automated reindex with progress tracking
- **Format version detection**: Automatic prompt when version mismatch detected
- **Incremental repair**: Reindex only affected files (not full rebuild)
- **Parallel indexing**: Multi-threaded parsing for large vaults

## Troubleshooting

### Reindex Fails with Parse Errors

**Symptom**: Reindex stops with "InvalidMarkdown" or "SyntaxError"

**Solution**:
1. Note which file failed (shown in error message)
2. Fix syntax in that file
3. Re-run reindex

### Reindex Succeeds But Data Missing

**Symptom**: Reindex completes but some notes/tasks not found

**Likely Causes**:
- Files not matching configured glob patterns
- Files excluded by `.lithosignore`
- Promotion tags misconfigured (for tasks)

**Solution**:
1. Check config: `lithos config show`
2. Verify file patterns: `lithos files list --vault vault/`
3. Check exclusions: `cat vault/.lithosignore`

### Reindex Very Slow

**Symptom**: Reindex takes much longer than expected

**Likely Causes**:
- Very large files (> 1MB markdown)
- Complex frontmatter (deeply nested objects)
- Many regex patterns in schema validation

**Solution**:
1. Profile: `lithos index --vault vault/ --profile`
2. Check for outliers: Files > 100KB or > 10000 lines
3. Consider splitting large files

## Related Documentation

- [ADR 003: Domain Serialization](../adr/003-domain-serialization.md) - rkyv format decisions
- [Epic 10: Indexing Pipeline](../../_bmad-output/planning-artifacts/epics/epic-10-core-indexing-pipeline.md) - Indexing architecture
- [Epic 11: Query Service](../../_bmad-output/planning-artifacts/epics/epic-11-query-service-knowledge-graph-mvp-core.md) - Query system
