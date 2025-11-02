# Reference Documentation

This directory contains reference materials and external data used for Lithos development and testing.

## Contents

### Production Test Data

**`obsidian-vault-guide.md`** - Guide to Jack's real Obsidian vault data
- **Location**: `docs/refs/obsidian/` (gitignored, 70+ MB)
- **Purpose**: Production-scale test data for Epic 3 vault indexing performance validation
- **Usage**: Extract subsets for testing hybrid BoltDB + SQLite architecture
- **Access**: `ls docs/refs/obsidian/` and `find docs/refs/obsidian/ -name "*.md"`

### Quick Access Commands

```bash
# Explore vault structure
ls docs/refs/obsidian/

# Count available notes
find docs/refs/obsidian/ -name "*.md" | wc -l

# Create large test vault (500+ notes)
mkdir -p testdata/vault-large/
find docs/refs/obsidian/ -name "*.md" | head -500 | xargs -I {} cp {} testdata/vault-large/

# Create diverse test vault (representative sample)
mkdir -p testdata/vault-diverse/
find docs/refs/obsidian/00_system/07_templates/ -name "*.md" | head -10 | xargs -I {} cp {} testdata/vault-diverse/
find docs/refs/obsidian/44_work/ -name "*.md" | head -10 | xargs -I {} cp {} testdata/vault-diverse/
find docs/refs/obsidian/70_pkm/ -name "*.md" | head -10 | xargs -I {} cp {} testdata/vault-diverse/
```

## Integration with Epic 3

This test data is critical for validating:
- **Performance targets**: Sub-100ms template queries at 500+ note scale
- **Real frontmatter patterns**: Actual Templater and Metadata Menu usage
- **Configuration variations**: Testing `file_class_key` with snake_case vs camelCase
- **Staleness detection**: Real file modification patterns for incremental indexing

See `docs/refs/obsidian-vault-guide.md` for detailed usage instructions and Epic 3 integration details.
