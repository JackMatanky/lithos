# Production Test Data Guide

## Overview

This document describes the production-scale test data available for Epic 3 vault indexing engine validation and performance testing.

**Quick Reference**: See `docs/refs/obsidian-vault-guide.md` for vault structure and usage overview.

## Primary Data Source

### Jack's Obsidian Vault
**Location**: `docs/refs/obsidian/`
**Size**: 70+ MB (gitignored due to size)
**Content**: Real-world Obsidian vault with production frontmatter patterns

#### Accessing the Vault
```bash
# Explore structure
ls docs/refs/obsidian/

# Count total notes
find docs/refs/obsidian/ -name "*.md" | wc -l

# Sample file names
find docs/refs/obsidian/ -name "*.md" | head -20

# Check file sizes
du -sh docs/refs/obsidian/

# Explore specific folders
ls docs/refs/obsidian/00_system/07_templates/
ls docs/refs/obsidian/44_work/
ls docs/refs/obsidian/70_pkm/
```

#### Vault Structure
- **00_system/**: Obsidian configuration, templates, and automation scripts
  - **07_templates/**: 50+ Templater templates for various note types
  - **scripts/**: Dataview queries and Templater JavaScript functions
- **44_work/**: Work-related projects and documentation
- **70_pkm/**: Personal Knowledge Management system
  - **00_tools_and_skills/**: Technical documentation and learning materials
  - **python/**: Programming notes, scripts, and code examples

## Epic 3 Testing Usage

### Performance Validation Data

**Extract Large Test Vault (500+ notes)**:
```bash
mkdir -p testdata/vault-large/
find docs/refs/obsidian/ -name "*.md" | head -500 | while read file; do
  mkdir -p "testdata/vault-large/$(dirname "${file#docs/refs/obsidian/}")"
  cp "$file" "testdata/vault-large/$(dirname "${file#docs/refs/obsidian/}")/"
done
```

**Extract Diverse Test Vault (representative sample)**:
```bash
mkdir -p testdata/vault-diverse/

# Templates (diverse note types)
find docs/refs/obsidian/00_system/07_templates/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done

# Work projects
find docs/refs/obsidian/44_work/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done

# PKM notes
find docs/refs/obsidian/70_pkm/ -name "*.md" | head -15 | while read file; do
  cp "$file" testdata/vault-diverse/
done
```

### Expected File Classes

Based on vault exploration, the following file classes are commonly used:
- **Templates**: Various note type templates
- **Projects**: Work and personal projects
- **Tools**: Technical documentation
- **Knowledge**: Learning and PKM materials
- **Scripts**: Automation and utility documentation
- **Contacts**: People and organization records
- **Tasks**: Action items and workflows

### Configuration Testing

The vault uses various frontmatter key naming patterns:
- **Snake case**: `file_class`, `created_date`, `due_date`
- **Camel case**: `fileClass`, `createdDate`, `dueDate`
- **Mixed patterns**: Different templates use different conventions

This diversity is perfect for testing the configurable `file_class_key` feature.

## Performance Benchmarks

Use this data to validate Epic 3 performance targets:

### BoltDB Hot Cache (Path Lookups)
- **Target**: <1ms average
- **Test**: Path-based queries on 500+ notes
- **Command**: Time individual `ByPath()` calls

### SQLite Deep Storage (Complex Queries)
- **Target**: <50ms average
- **Test**: Frontmatter property searches
- **Command**: Time `ByFrontmatter()` calls with various criteria

### Template Rendering (End-to-End)
- **Target**: <100ms total
- **Test**: Full template rendering pipeline
- **Command**: Time complete template execution with vault queries

### Full Vault Indexing
- **Target**: <5s for 1000+ notes
- **Test**: Complete vault indexing from scratch
- **Command**: Time full `RefreshFromCache()` operation

### Incremental Updates
- **Target**: <1s for typical change sets
- **Test**: Staleness detection and incremental indexing
- **Command**: Time incremental updates after file modifications

## Development Workflow

### Daily Development Testing
```bash
# Quick test with small subset (fast iteration)
find docs/refs/obsidian/ -name "*.md" | head -10 | xargs -I {} cp {} testdata/vault-quick/

# Run tests
go test ./tests/e2e/ -v
```

### Performance Validation Testing
```bash
# Create large test vault
find docs/refs/obsidian/ -name "*.md" | head -500 | xargs -I {} cp {} testdata/vault-large/

# Run performance tests
go test ./tests/e2e/ -v -run TestPerformance
```

### Configuration Variation Testing
```bash
# Test different file_class_key settings
echo '{"file_class_key": "fileClass"}' > testdata/lithos-camel.json
echo '{"file_class_key": "type"}' > testdata/lithos-type.json

# Run with different configs
go test ./tests/e2e/ -v -run TestFileClassKey
```

## Data Privacy and Usage

- **Content**: Personal/work vault data is included for realistic testing
- **Anonymization**: Sensitive content should be reviewed before sharing test results
- **Scope**: Use only for Lithos development and testing purposes
- **Access**: Data is gitignored and only available in local development environment

## Integration with Epic 3 Stories

### Story 3.2 (Multi-Storage Cache Adapters)
- Use vault data to test BoltDB bucket structures
- Validate SQLite schema with real frontmatter patterns
- Test configurable file_class_key with actual note variations

### Story 3.6 (Hybrid Query Service)
- Performance test query routing with realistic note distribution
- Validate smart routing decisions with real query patterns
- Test staleness detection with actual file modification patterns

### Story 3.17 (Production-Scale E2E Testing)
- Create 500+ note test vault from real data
- Validate performance targets with realistic content
- Test configuration variations with actual frontmatter patterns

### Story 3.18 (Documentation)
- Use real performance data for benchmark documentation
- Include actual file class examples in configuration guide
- Reference real-world usage patterns in performance optimization guide

## Troubleshooting

### Missing Vault Data
If `docs/refs/obsidian/` is empty or missing:
```bash
# Check if directory exists
ls -la docs/refs/

# The vault may not be synced or may be gitignored
# Contact Jack for vault data or use alternative test data
```

### Performance Issues
If tests are slow with large vault:
```bash
# Use smaller subsets for development
find docs/refs/obsidian/ -name "*.md" | head -50 | xargs -I {} cp {} testdata/vault-small/

# Or focus on specific note types
find docs/refs/obsidian/00_system/07_templates/ -name "*.md" | head -10 | xargs -I {} cp {} testdata/vault-templates/
```

This production test data ensures Epic 3 validation with realistic, production-scale content that represents actual Obsidian vault usage patterns.
