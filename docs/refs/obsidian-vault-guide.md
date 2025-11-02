# Obsidian Vault Reference Data

## Overview

This directory contains real-world Obsidian vault data from Jack's personal vault, serving as production-scale test data for the Lithos vault indexing engine.

## Location

**Path**: `docs/refs/obsidian/`

**Note**: This folder contains 70+ MB of data and is added to `.gitignore`. For development access:
```bash
ls docs/refs/obsidian/
find docs/refs/obsidian/ -name "*.md" | head -20
```

## Structure

- **00_system/**: Obsidian configuration, templates, and scripts
  - **07_templates/**: Templater templates for various note types
  - **scripts/**: Dataview and Templater JavaScript scripts
- **44_work/**: Work-related projects and documentation
- **70_pkm/**: Personal Knowledge Management notes
  - **00_tools_and_skills/**: Technical documentation and learning materials
  - **python/**: Programming notes and scripts

## Usage for Epic 3 Testing

This vault provides **production-scale test data** for:
- **Performance validation**: 500+ notes for scalability testing
- **Real frontmatter patterns**: Actual Metadata Menu and Templater usage
- **Diverse file classes**: Templates, projects, contacts, journals, code snippets
- **Edge cases**: Complex frontmatter, nested folders, various note types
- **Incremental indexing**: Mix of recently modified and older files

## Test Data Generation

To create test vaults for Epic 3:

```bash
# Count total notes
find docs/refs/obsidian/ -name "*.md" | wc -l

# Create subset for testing (first 100 notes)
mkdir -p testdata/vault-large/
find docs/refs/obsidian/ -name "*.md" | head -100 | xargs -I {} cp {} testdata/vault-large/

# Sample by folder for diversity
mkdir -p testdata/vault-diverse/
find docs/refs/obsidian/00_system/07_templates/ -name "*.md" | head -20 | xargs -I {} cp {} testdata/vault-diverse/
find docs/refs/obsidian/44_work/ -name "*.md" | head -20 | xargs -I {} cp {} testdata/vault-diverse/
find docs/refs/obsidian/70_pkm/ -name "*.md" | head -20 | xargs -I {} cp {} testdata/vault-diverse/
```

## File Class Examples

Based on exploration of the vault structure, expected file classes include:
- **Templates**: Various note type templates from 00_system/07_templates/
- **Projects**: Work and personal projects
- **Tools**: Technical documentation and code snippets
- **Knowledge**: PKM and learning materials
- **Scripts**: Automation and utility files

## Performance Targets

This vault will be used to validate Epic 3 performance requirements:
- **Path lookups (BoltDB)**: <1ms average
- **Complex queries (SQLite)**: <50ms average
- **Template rendering**: <100ms total
- **Full indexing**: <5s for 1000+ notes
- **Incremental updates**: <1s for typical change sets

## Configuration Testing

The vault will test configurable `file_class_key` support:
- **Default**: `file_class` (snake_case)
- **Alternative**: `fileClass` (camelCase)
- **Custom**: `type` or other user preferences

This ensures the hybrid BoltDB + SQLite architecture works with real-world frontmatter patterns and user configuration preferences.
