# Query Optimization Guide

## Overview

This guide explains how to optimize queries in Lithos using the hybrid BoltDB+SQLite storage system. Understanding query routing and optimization techniques can significantly improve performance for large vaults.

## Query Routing Architecture

### Storage Tiers

Lithos uses a two-tier storage system with automatic query routing:

#### BoltDB (Hot Cache)
- **Response Time**: <1ms
- **Use Case**: Frequently accessed data (80% of queries)
- **Storage**: Key-value with secondary indices
- **Query Types**: Path, basename, alias, common file class queries

#### SQLite (Deep Storage)
- **Response Time**: <50ms
- **Use Case**: Complex queries and analytics (20% of queries)
- **Storage**: Relational with JSON columns and full-text search
- **Query Types**: Frontmatter field queries, link analysis, complex filters

### Smart Routing Logic

Queries are automatically routed based on complexity and access patterns:

```go
func routeQuery(query Query) StorageBackend {
    switch query.Type {
    case PathQuery, BasenameQuery, AliasQuery:
        return BoltDB // Always hot path

    case FileClassQuery:
        if isHotFileClass(query.FileClass) {
            return BoltDB
        }
        return SQLite // Cold file classes

    case FrontmatterQuery, LinkQuery:
        return SQLite // Complex queries

    default:
        return SQLite // Default to deep storage
    }
}
```

## Hot Set Optimization

### Configuring Hot File Classes

The most important optimization is configuring the correct hot file classes:

```json
{
  "query": {
    "hotFileClasses": ["contact", "project", "daily-note", "meeting-note"]
  }
}
```

#### Identifying Hot File Classes

1. **Analyze Query Patterns**: Monitor which file classes are queried most frequently
2. **Check Vault Structure**: Identify the most common note types in your vault
3. **Start with Defaults**: Use the default hot classes as a starting point
4. **Tune Gradually**: Add or remove classes based on performance metrics

#### Common Hot File Classes by Use Case

| Use Case | Recommended Hot Classes |
|----------|------------------------|
| Personal Knowledge Base | `["contact", "project", "article", "idea"]` |
| Project Management | `["project", "task", "meeting", "decision"]` |
| Research Notes | `["paper", "concept", "experiment", "finding"]` |
| Daily Journaling | `["daily-note", "weekly-review", "monthly-review"]` |

### Adaptive Learning

Enable adaptive learning to automatically optimize hot sets:

```json
{
  "query": {
    "adaptiveLearning": true
  }
}
```

*Note: Adaptive learning analyzes query patterns over time and adjusts hot sets automatically.*

## Query Performance Optimization

### Fast Query Patterns

#### 1. Path-Based Queries (Fastest)
```go
// Direct path lookup - always BoltDB
PathQuery("vault/notes/contact.md")
```

#### 2. Basename Queries
```go
// Index lookup - BoltDB for hot paths
BasenameQuery("john-doe")
```

#### 3. Hot File Class Queries
```go
// Index lookup - BoltDB for configured hot classes
FileClassQuery("contact")
```

### Slower Query Patterns

#### 1. Cold File Class Queries
```go
// Full scan - SQLite with JSON extraction
FileClassQuery("rare-note-type")
```

#### 2. Frontmatter Field Queries
```go
// JSON extraction - SQLite only
FrontmatterQuery("status", "active")
```

#### 3. Link Analysis Queries
```go
// Complex joins - SQLite only
LinkQuery("target-note.md")
```

## Indexing Strategies

### File Classification

#### Choosing a File Class Key

Select a frontmatter property that effectively categorizes your notes:

```yaml
# Good: Clear categorization
---
type: contact
---

# Good: Domain-specific classification
---
category: research-paper
---

# Avoid: Too generic
---
tags: [work, personal]
---

# Avoid: Inconsistent values
---
class: Contact
class: contact
class: CONTACT
---
```

#### Consistent Classification

1. **Standardize Values**: Use consistent casing and naming
2. **Limit Cardinality**: Don't create too many unique file classes
3. **Plan for Growth**: Choose a classification that scales with your vault

### Schema-Driven Optimization

#### Property Indexing

Design schemas to leverage indexed queries:

```json
{
  "name": "contact",
  "properties": [
    {
      "id": "name",
      "type": "string",
      "required": true
    },
    {
      "id": "email",
      "type": "string",
      "spec": {
        "regex": "^[^@]+@[^@]+\\.[^@]+$"
      }
    },
    {
      "id": "status",
      "type": "string",
      "enum": ["active", "inactive", "prospect"]
    }
  ]
}
```

#### Indexed Properties

Properties used in frequent queries should be indexed:

- **High Cardinality**: Properties with many unique values (names, emails)
- **Frequent Filters**: Properties used in WHERE clauses
- **Join Keys**: Properties used for linking notes

## Performance Monitoring

### Key Metrics to Monitor

#### Query Performance Metrics
- `query_duration_seconds`: Response time by query type
- `query_routing_decisions_total`: Hot vs cold path routing
- `cache_hit_ratio`: Percentage of queries served from hot cache

#### Index Health Metrics
- `indexing_duration_seconds`: Time to rebuild index
- `index_size_bytes`: Storage used by index
- `index_staleness_seconds`: Age of index data

### Monitoring Queries

#### Check Routing Efficiency
```bash
# Monitor routing decisions
grep "query_routing" logs/*.log

# Check cache hit ratios
grep "cache_hit_ratio" metrics/*.json
```

#### Identify Slow Queries
```bash
# Find queries taking >100ms
grep "query_duration_seconds.*[0-9]\.[1-9]" logs/*.log
```

## Optimization Techniques

### 1. Query Pattern Analysis

#### Analyze Query Logs
```bash
# Find most frequent query types
grep "query_type" logs/*.log | sort | uniq -c | sort -nr

# Identify slow queries
grep "query_duration_seconds.*[0-9]\.[1-9]" logs/*.log
```

#### Optimize Based on Patterns
- **Frequent File Classes**: Add to hot set
- **Common Filters**: Ensure properties are indexed
- **Slow Queries**: Consider query restructuring

### 2. Index Maintenance

#### Regular Rebuilding
```bash
# Rebuild index after schema changes
lithos index

# Schedule regular rebuilds for large vaults
0 2 * * * lithos index  # Daily at 2 AM
```

#### Incremental Updates
- Lithos automatically detects changed files
- Only modified notes are re-indexed
- Full rebuilds only needed for schema changes

### 3. Configuration Tuning

#### Memory Optimization
```json
{
  "indexing": {
    "batchSize": 200,
    "maxConcurrency": 8
  }
}
```

#### Storage Optimization
```json
{
  "cacheDir": "/fast/ssd/.lithos/cache/"
}
```

### 4. Schema Optimization

#### Flatten Deep Structures
```yaml
# Avoid: Deep nesting
---
metadata:
  contact:
    personal:
      name: John

# Prefer: Flat structure
---
contact_name: John
contact_type: personal
```

#### Use Appropriate Types
```json
{
  "properties": [
    {
      "id": "priority",
      "type": "string",
      "enum": ["low", "medium", "high"]  // Indexed enum
    },
    {
      "id": "created_date",
      "type": "date"  // Indexed date
    }
  ]
}
```

## Troubleshooting Performance Issues

### Slow Query Diagnosis

#### Step 1: Identify the Query Type
```bash
# Check query logs
grep "slow_query" logs/*.log
```

#### Step 2: Determine Routing
```bash
# Check if query went to correct storage
grep "routing_decision" logs/*.log
```

#### Step 3: Analyze Storage Performance
```bash
# BoltDB performance
time lithos query path "vault/notes/example.md"

# SQLite performance
time lithos query frontmatter status active
```

### Common Performance Issues

#### Issue: Slow File Class Queries
**Symptom**: `FileClassQuery("uncommon-type")` takes 500ms+
**Cause**: Query routed to SQLite for cold file class
**Solution**: Add file class to hot set or accept slower performance

#### Issue: Inefficient Frontmatter Queries
**Symptom**: `FrontmatterQuery("custom_field", "value")` slow
**Cause**: JSON extraction on unindexed properties
**Solution**: Restructure data or accept SQLite performance

#### Issue: Large Result Sets
**Symptom**: Queries returning 1000+ results are slow
**Cause**: Memory allocation and processing overhead
**Solution**: Add LIMIT clauses or paginate results

#### Issue: Index Staleness
**Symptom**: Queries return outdated results
**Cause**: Index not rebuilt after vault changes
**Solution**: Run `lithos index` or enable auto-rebuild

### Advanced Troubleshooting

#### Query Plan Analysis
```sql
-- Analyze SQLite query plans
EXPLAIN QUERY PLAN
SELECT * FROM notes
WHERE json_extract(frontmatter, '$.status') = 'active';
```

#### Index Effectiveness
```sql
-- Check index usage
SELECT * FROM sqlite_master WHERE type = 'index';
ANALYZE; -- Update statistics
```

## Best Practices

### Query Design

#### 1. Prefer Indexed Queries
```go
// Good: Uses basename index
BasenameQuery("john-doe")

// Avoid: Scans all notes
FrontmatterQuery("name", "John Doe")
```

#### 2. Use Appropriate Granularity
```go
// Good: Specific path
PathQuery("vault/contacts/john-doe.md")

// Avoid: Broad search
FileClassQuery("note") // If "note" is too generic
```

#### 3. Leverage Hot Sets
```go
// Good: Query hot file classes
FileClassQuery("contact") // If "contact" is in hot set

// Avoid: Query cold file classes frequently
FileClassQuery("rare-type")
```

### Schema Design

#### 1. Plan for Query Patterns
```json
{
  "name": "contact",
  "properties": [
    {"id": "name", "type": "string"},     // Frequently searched
    {"id": "email", "type": "string"},    // Unique identifier
    {"id": "status", "type": "string", "enum": ["active", "inactive"]}
  ]
}
```

#### 2. Use Consistent Naming
```yaml
# Consistent property names across schemas
---
contact_name: John
project_name: Website
task_name: Design Review
---

# Avoid inconsistent naming
---
name: John
title: Website
subject: Design Review
---
```

### Maintenance

#### 1. Regular Index Rebuilding
- Rebuild after schema changes
- Schedule for large vaults
- Monitor index staleness

#### 2. Configuration Review
- Review hot sets quarterly
- Adjust concurrency based on hardware
- Monitor storage usage

#### 3. Performance Monitoring
- Set up alerts for slow queries
- Monitor cache hit ratios
- Track index rebuild times

## Migration Guide

### Upgrading from Single Storage

#### Before (JSON Files Only)
- All queries scanned filesystem
- No indexing or caching
- Poor performance on large vaults

#### After (Hybrid Storage)
1. **Configure Hot Sets**: Identify frequently queried file classes
2. **Tune Indexing**: Adjust concurrency and batch sizes
3. **Monitor Performance**: Track query patterns and adjust
4. **Optimize Schemas**: Design for indexed queries

### Expected Performance Gains

| Vault Size | Indexing Time | Query Speed | Memory Usage |
|------------|---------------|-------------|--------------|
| 100 notes | -50% | 10x faster | +20MB |
| 1,000 notes | -30% | 50x faster | +50MB |
| 10,000 notes | -20% | 100x faster | +200MB |

*Note: Actual performance gains depend on query patterns and configuration.*

## Future Optimizations

### Planned Features

#### Query Result Caching
- Cache frequent query results in memory
- Reduce database load for popular queries
- Automatic cache invalidation

#### Adaptive Indexing
- Automatically create indices for frequently queried properties
- Learn from query patterns
- Dynamic index maintenance

#### Query Planning
- Cost-based query optimization
- Automatic query rewriting
- Parallel query execution

This guide provides the foundation for optimizing Lithos performance. Regular monitoring and tuning based on your specific vault structure and query patterns will yield the best results.
