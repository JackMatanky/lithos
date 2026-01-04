# Configuration Guide

## Overview

Lithos supports comprehensive configuration through `lithos.json` files, environment variables, and command-line flags. Configuration follows a hierarchical precedence: CLI flags > environment variables > config file > defaults.

## Configuration File Location

Lithos searches for `lithos.json` starting from the current working directory and moving upward until found or reaching the root directory.

```
./lithos.json          (highest priority)
../lithos.json
../../lithos.json
... (until root)
defaults               (lowest priority)
```

## Basic Configuration

### Core Settings

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "fileClassKey": "type",
  "logLevel": "info"
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `vaultPath` | string | `"."` | Root directory of your Obsidian vault |
| `templatesDir` | string | `"templates/"` | Directory containing template files |
| `schemasDir` | string | `"schemas/"` | Directory containing schema definitions |
| `propertyBankFile` | string | `"property_bank.json"` | Property bank filename |
| `cacheDir` | string | `".lithos/cache/"` | Index cache storage location |
| `fileClassKey` | string | `"type"` | Frontmatter property for file classification |
| `logLevel` | string | `"info"` | Logging verbosity (debug, info, warn, error) |

### Environment Variables

All configuration fields can be overridden using `LITHOS_` prefixed environment variables:

```bash
export LITHOS_VAULT_PATH="/path/to/vault"
export LITHOS_TEMPLATES_DIR="my-templates/"
export LITHOS_SCHEMAS_DIR="my-schemas/"
export LITHOS_CACHE_DIR=".lithos/cache/"
export LITHOS_FILE_CLASS_KEY="category"
export LITHOS_LOG_LEVEL="debug"
```

## Vault Indexing Configuration

### Indexing Settings

```json
{
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 4,
    "batchSize": 100
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `indexing.enableValidation` | boolean | `true` | Enable schema validation during indexing |
| `indexing.maxConcurrency` | number | `4` | Maximum concurrent file processing workers |
| `indexing.batchSize` | number | `100` | Files to process in each batch |

### Environment Variables for Indexing

```bash
export LITHOS_INDEXING_ENABLE_VALIDATION="true"
export LITHOS_INDEXING_MAX_CONCURRENCY="8"
export LITHOS_INDEXING_BATCH_SIZE="200"
```

## Query Configuration

### Query Routing Settings

```json
{
  "query": {
    "hotFileClasses": ["contact", "project", "daily-note", "meeting-note"],
    "adaptiveLearning": true
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `query.hotFileClasses` | array | `["contact", "project", "daily-note", "meeting-note"]` | File classes served from BoltDB hot cache |
| `query.adaptiveLearning` | boolean | `true` | Enable automatic hot set optimization |

### Environment Variables for Query

```bash
export LITHOS_QUERY_HOT_FILE_CLASSES="contact,project,daily-note"
export LITHOS_QUERY_ADAPTIVE_LEARNING="true"
```

## Complete Configuration Examples

### Minimal Configuration

For basic template generation without schema validation:

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/"
}
```

### Standard Configuration

For typical Obsidian vault with schema validation:

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "fileClassKey": "type",
  "logLevel": "info"
}
```

### Performance-Optimized Configuration

For large vaults requiring high performance:

```json
{
  "vaultPath": "/large/vault",
  "cacheDir": "/fast/ssd/.lithos/cache/",
  "fileClassKey": "category",
  "logLevel": "warn",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 8,
    "batchSize": 200
  },
  "query": {
    "hotFileClasses": ["contact", "project", "meeting", "daily"],
    "adaptiveLearning": true
  }
}
```

### Development Configuration

For development with detailed logging:

```json
{
  "vaultPath": "./test-vault",
  "logLevel": "debug",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 2,
    "batchSize": 10
  }
}
```

## File Classification Configuration

### fileClassKey Setting

The `fileClassKey` determines which frontmatter property Lithos uses to classify notes for indexing and querying:

```json
{
  "fileClassKey": "type"
}
```

With this configuration, Lithos will use the `type` field from note frontmatter:

```yaml
---
type: contact
name: John Doe
email: john@example.com
---

# John Doe

Contact information...
```

### Alternative Classification Keys

```json
{
  "fileClassKey": "category"
}
```

```yaml
---
category: project
title: My Project
status: active
---

# My Project

Project details...
```

### Multiple Classification

You can use multiple classification properties by configuring different keys:

```json
{
  "fileClassKey": "type",
  "query": {
    "hotFileClasses": ["contact", "project", "meeting-note"]
  }
}
```

## Indexing Performance Tuning

### Concurrency Configuration

#### For CPU-Bound Systems
```json
{
  "indexing": {
    "maxConcurrency": 8,
    "batchSize": 200
  }
}
```

#### For Memory-Constrained Systems
```json
{
  "indexing": {
    "maxConcurrency": 2,
    "batchSize": 50
  }
}
```

#### For I/O-Bound Systems
```json
{
  "indexing": {
    "maxConcurrency": 4,
    "batchSize": 100
  }
}
```

### Validation Configuration

#### Enable Full Validation (Recommended)
```json
{
  "indexing": {
    "enableValidation": true
  }
}
```

#### Disable Validation for Speed
```json
{
  "indexing": {
    "enableValidation": false
}
```

*Note: Disabling validation improves indexing speed but may allow invalid notes into the index.*

## Query Performance Tuning

### Hot Set Optimization

#### Default Hot Classes
```json
{
  "query": {
    "hotFileClasses": ["contact", "project", "daily-note", "meeting-note"]
  }
}
```

#### Custom Hot Classes
```json
{
  "query": {
    "hotFileClasses": ["person", "task", "idea", "article"]
  }
}
```

#### Adaptive Learning
```json
{
  "query": {
    "adaptiveLearning": true
  }
}
```

*Note: Adaptive learning automatically adjusts hot sets based on query patterns.*

## Storage Configuration

### Cache Directory

#### Default Location
```json
{
  "cacheDir": ".lithos/cache/"
}
```

#### Custom Location
```json
{
  "cacheDir": "/var/cache/lithos/"
}
```

#### Fast Storage
```json
{
  "cacheDir": "/mnt/fast-ssd/.lithos/cache/"
}
```

*Note: Cache directory should have sufficient space for index data (typically 1.5x vault size).*

## Logging Configuration

### Log Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| `debug` | Detailed diagnostic information | Development, troubleshooting |
| `info` | General operational messages | Normal operation |
| `warn` | Warning conditions | Monitor for issues |
| `error` | Error conditions | Production monitoring |

#### Debug Logging
```json
{
  "logLevel": "debug"
}
```

#### Production Logging
```json
{
  "logLevel": "warn"
}
```

## Environment-Specific Configurations

### Development Environment

```json
{
  "vaultPath": "./dev-vault",
  "logLevel": "debug",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 2,
    "batchSize": 10
  }
}
```

### Testing Environment

```json
{
  "vaultPath": "./test-vault",
  "cacheDir": "./test-cache/",
  "logLevel": "info",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 1,
    "batchSize": 5
  }
}
```

### Production Environment

```json
{
  "vaultPath": "/data/vault",
  "cacheDir": "/data/cache/",
  "logLevel": "warn",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 8,
    "batchSize": 200
  },
  "query": {
    "hotFileClasses": ["contact", "project", "meeting"],
    "adaptiveLearning": true
  }
}
```

## Configuration Validation

### Validating Configuration

Lithos validates configuration on startup and reports errors:

```
Error: invalid configuration: indexing.maxConcurrency must be between 1 and 16
```

### Common Configuration Issues

#### Invalid Vault Path
```
Error: vault path does not exist: /nonexistent/path
Solution: Ensure vaultPath points to a valid directory
```

#### Invalid Cache Directory
```
Error: cannot create cache directory: permission denied
Solution: Ensure write permissions for cacheDir
```

#### Invalid Concurrency
```
Error: indexing.maxConcurrency must be between 1 and 16
Solution: Set maxConcurrency to a valid value
```

#### Invalid File Class Key
```
Error: fileClassKey cannot be empty
Solution: Set fileClassKey to a valid frontmatter property name
```

## Configuration File Examples

### Template for New Projects

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "fileClassKey": "type",
  "logLevel": "info",
  "indexing": {
    "enableValidation": true,
    "maxConcurrency": 4,
    "batchSize": 100
  },
  "query": {
    "hotFileClasses": ["contact", "project", "daily-note", "meeting-note"],
    "adaptiveLearning": true
  }
}
```

### Migration from Previous Versions

If upgrading from a version without indexing configuration:

```json
{
  "vaultPath": ".",
  "templatesDir": "templates/",
  "schemasDir": "schemas/",
  "propertyBankFile": "property_bank.json",
  "cacheDir": ".lithos/cache/",
  "fileClassKey": "type",
  "logLevel": "info"
}
```

The indexing configuration will use defaults, which are suitable for most use cases.
